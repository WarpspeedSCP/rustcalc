extern crate fst;
extern crate fst_regex;
extern crate scribe;
extern crate std;
extern crate termion;

use termion::raw::RawTerminal;
use termion::raw::IntoRawMode;
use termion::input::TermRead;
use termion::event::Key;
use termion::cursor::Goto;

use std::collections::BTreeMap;
use std::io::{Stdin, Stdout, Write};

pub struct Point {
    pub x: u16,
    pub y: u16,
}

impl Point {
    pub fn new(x: u16, y: u16) -> Point {
        Point { x: x, y: y }
    }
}

#[derive(Clone, Debug)]
struct Trie {
    val: Option<String>,
    children: BTreeMap<char, Trie>,
    end: bool,
}

impl Trie {
    pub fn new() -> Trie {
        Trie {
            val: None,
            children: BTreeMap::new(),
            end: false,
        }
    }

    fn val(mut self, v: &String) -> Trie {
        self.val = Some(v.clone());
        self
    }

    // This fn travels to the various nodes that are
    // valid (partial) completions for the given string and then
    // finds all possible full completions for the string through the descend fn.
    pub fn complete(&self, string: &[u8]) -> Vec<String> {
        match string.len() {
            0 => self.descend(),
            _ => match self.children.get(&(string[0] as char)) {
                Some(s) => s.complete(&string[1..]),
                None => Vec::new(),
            },
        }
    }

    // This fn is called when we have reached a partial completion for a particular string.
    // It collects all possible full completions for that string by traversing the sub-trie
    // of the current node in order and appending the value of each leaf node (of this sub-trie) to a vector.
    pub fn descend(&self) -> Vec<String> {
        let mut vd: Vec<String> = Vec::new();
        match self.children.len() {
            0 => match self.val.clone() {
                Some(v) => vd.push(v),
                None => return Vec::new(),
            },
            _ => for i in &self.children {
                match self.val.clone() {
                    Some(v) => if self.end {
                        vd.push(v)
                    },
                    None => return Vec::new(),
                }
                vd.append(&mut i.1.descend());
            },
        }
        vd
    }

    // This fn inserts a new completion into the specified trie
    // with the wikipedia implementation of the insert operation.
    pub fn insert(mut trie: Trie, string: &String) -> Trie {
        // I am using raw pointers to traverse the trie because
        // it seems to be the only sane way of doing so;
        // I couldn't think of a way to do this with mutable references,
        // and since it hasn't ever paniced since I wrote it, I'm assuming it's safe.
        //
        // Feel free to replace this with a safer/rustier implementation if you can.
        unsafe {
            let mut m: *mut Trie = &mut trie as *mut Trie;
            let d = string.as_bytes();
            for i in 0..string.len() {
                m = match (*m).children.get_mut(&(d[i] as char)) {
                    Some(x) => x as *mut Trie,
                    None => {
                        (*m).children.insert(
                            (d[i] as char),
                            Trie::new().val(&String::from(string.get(..=i).unwrap())),
                        );
                        (*m).children.get_mut(&(d[i] as char)).unwrap()
                    }
                }
            }

            (*m).val = Some(string.clone());
            (*m).end = true;
            trie
        }
    }
}

// This struct can be used anywhere to provide a very basic
// readline interface; There are no internal dependencies.
pub struct InputManager {
    //key_comp_tree: fst::Map,
    user_comp_tree: Trie,
    terminal: RawTerminal<Stdout>,
    history: Vec<String>,
    insert: bool,
    pos: Point,
}

impl InputManager {
    pub fn new() -> InputManager {
        InputManager {
            user_comp_tree: Trie::new(),
            terminal: std::io::stdout().into_raw_mode().unwrap(),
            history: Vec::new(),
            insert: false,
            pos: Point::new(1, 1),
        }
    }

    // Very bad output function; I plan on redoing this later
    pub fn put_line(&mut self, output: &String) {
        write!(
            self.terminal,
            "\r{}{}{}{}\n",
            termion::cursor::Down(2),
            termion::clear::CurrentLine,
            output,
            termion::cursor::Up(2)
        ).unwrap();
    }

    // Clears everything on the screen
    pub fn clear_all(&mut self) {
        write!(self.terminal, "{}{}", termion::clear::All, Goto(1, 1));
    }

    // The heart of this library. It reads a line until a
    // newline character is encountered and returns that line
    // after adding all words in it to the trie for autocompletion support,
    // as well as the history buffer.
    //
    // The history buffer does not work in the same way as a normal
    // unix commandline history buffer would work though, since all
    // elements in the buffer are immutable. The only operations
    // that can be performed are insertion, lookup and deletion.
    //
    // This fn also handles a few special keys including ctrl, alt and the arrow keys.
    // I plan to add a multiline mode as well.
    pub fn get_line(&mut self, prompt: &String, input: &mut Stdin) -> String {
        // Clear the current line first.
        write!(
            self.terminal,
            "{}{}{}",
            termion::clear::CurrentLine,
            Goto(1, self.pos.y), // The terminal controlled by termion is, annoyingly, 1 indexed.
            prompt // There's an explaination for why this is so, but it is quite annoying
        ).unwrap(); // to have to shift from 0 to 1 indexed thinking every now and then.

        // This is how we update the screen.
        self.terminal.flush().unwrap();

        // A gap buffer makes for easy insertion and deletion from any position in a string.
        // I was too lazy to do it myself, so now I have this rather large dependency to lug around.
        // I may replace this with my own implementation.
        let mut out: scribe::buffer::GapBuffer = scribe::buffer::GapBuffer::new(String::from(""));

        self.pos.x = prompt.len() as u16;
        let pl = prompt.len() as u16;
        let mut h_index = self.history.len();

        // The main input loop
        for c in input.keys() {
            // Some bounds checks and corrections to keep the index (pos) within bounds.
            if self.pos.x <= pl {
                self.pos.x = pl;
            }
            if self.pos.x > pl + out.to_string().len() as u16 {
                self.pos.x = pl + out.to_string().len() as u16;
            }

            // The part that handles the various keys. Rust pattern matching is exceptionally useful here.
            match c.unwrap() {
                // When we press Return.
                Key::Char('\n') => {
                    self.history.push(out.to_string());
                    self.pos.y += 1;
                    break;
                }

                // Pressing tab causes a list of possible completions of a string to pop up.
                Key::Char('\t') => self.handle_tab(out.to_string()),

                // This is where insertion at any point is handled.
                Key::Char(c) => {
                    // Inserting at the end of the line.
                    if self.pos.x - pl == out.to_string().len() as u16 {
                        // Nothing fancy to do; increment position on the screen and print the new character at the end.
                        self.pos.x += 1;

                        out.insert(
                            &c.to_string(),
                            &scribe::buffer::Position {
                                line: 0,
                                offset: (self.pos.x - pl as u16 - 1) as usize,
                            },
                        );

                        write!(self.terminal, "{}{}", Goto(self.pos.x, self.pos.y), c).unwrap();
                    }
                    // Insert is off, just insert a single character at any position.
                    else if !self.insert {
                        let tmp = &out.to_string()[(self.pos.x - pl) as usize..];

                        self.pos.x += 1;

                        out.insert(
                            &c.to_string(),
                            &scribe::buffer::Position {
                                line: 0,
                                offset: (self.pos.x - pl - 1) as usize,
                            },
                        );

                        write!(
                            self.terminal,
                            "{}{}{}{}",
                            Goto(self.pos.x, self.pos.y),
                            c,
                            tmp,
                            Goto(self.pos.x + 1, self.pos.y)
                        ).unwrap();
                    }
                    // If insert mode is on, we can overwrite parts of a string.
                    // Rust strings make doing this a little annoying, since
                    // they are composed of hard to index UTF-8 characters.
                    else if self.insert {
                        // The way this gap buffer is implemented doesn't help.
                        // I pobably should implement my own.

                        // Delete a single character, move forward one space and then insert the new character.
                        out.delete(&scribe::buffer::Range::new(
                            scribe::buffer::Position {
                                line: 0,
                                offset: (self.pos.x - pl) as usize,
                            },
                            scribe::buffer::Position {
                                line: 0,
                                offset: (self.pos.x + 1 - pl) as usize,
                            },
                        ));

                        self.pos.x += 1;

                        out.insert(
                            &c.to_string(),
                            &scribe::buffer::Position {
                                line: 0,
                                offset: (self.pos.x - pl as u16 - 1) as usize,
                            },
                        );

                        write!(
                            self.terminal,
                            "{}{}{}",
                            Goto(self.pos.x, self.pos.y),
                            c,
                            Goto(self.pos.x, self.pos.y)
                        ).unwrap();
                    }
                }

                // Alt doesn't do anything yet.
                Key::Alt(c) => self.handle_alt(c),

                // Ctrl + 'c' can be used to gracefully exit the app.
                Key::Ctrl(c) => {
                    if c == 'c' {
                        return String::from("exit");
                    }
                    self.handle_ctrl(c)
                }

                // Moves the cursor left.
                Key::Left => self.handle_left(&pl),

                // Moves the cursor right.
                Key::Right => self.handle_right(out.to_string().len() as u16, &pl),

                // Loads up the previous history item.
                Key::Up => {
                    // These checks are necessary since I'm not using an iterator over the buffer.
                    if !self.history.is_empty() {
                        if h_index > 0 {
                            // This first if block stores anything freshly typed in the buffer.
                            if h_index == self.history.len()
                                && (!out.to_string().chars().all(|c: char| {
                                    (c == ' ' || c == '\n' || c == '\r' || c == '\t')
                                })
                                    || !out.to_string().is_empty())
                            {
                                self.history.push(out.to_string());
                            }

                            // Delete the old value and insert the value of the current history item.

                            let d = out.to_string().len();
                            out.delete(&scribe::buffer::Range::new(
                                scribe::buffer::Position { line: 0, offset: 0 },
                                scribe::buffer::Position { line: 0, offset: d },
                            ));

                            out.insert(
                                &self.history[h_index - 1],
                                &scribe::buffer::Position { line: 0, offset: 0 },
                            );

                            h_index -= 1;
                        }

                        // Print the results.
                        self.pos.x = pl + out.to_string().len() as u16;
                        write!(
                            self.terminal,
                            "{}{}{}{}{}{}",
                            termion::clear::CurrentLine,
                            Goto(1, self.pos.y),
                            prompt,
                            Goto(pl + 1, self.pos.y),
                            out.to_string(),
                            Goto(self.pos.x + 1, self.pos.y)
                        ).unwrap();
                    }
                }
                Key::Down => {
                    if !self.history.is_empty() {
                        // This is some very sketchy code that was supposed to handle a case where, 
                        // if you press up while typing, what you just typed should be saved, 
                        // and when you press down again, it should be loaded back, like a normal unix terminal. 
                        //
                        // It doesn't work well.
                        /* if h_index == self.history.len() - 1 {
                            let d = out.to_string().len();

                            out.delete(&scribe::buffer::Range::new(
                                scribe::buffer::Position { line: 0, offset: 0 },
                                scribe::buffer::Position { line: 0, offset: d },
                            ));

                            out.insert(
                                &self.history.pop().unwrap(),
                                &scribe::buffer::Position { line: 0, offset: 0 },
                            );
                        } else */

                        // If we are anywhere but at the end of the history buffer.
                        if h_index < self.history.len() {
                            let d = out.to_string().len();

                            out.delete(&scribe::buffer::Range::new(
                                scribe::buffer::Position { line: 0, offset: 0 },
                                scribe::buffer::Position { line: 0, offset: d },
                            ));

                            if h_index < self.history.len() - 1 {
                                h_index += 1;
                                out.insert(
                                    &self.history[h_index],
                                    &scribe::buffer::Position { line: 0, offset: 0 },
                                );
                            } else {
                                h_index = self.history.len();
                                out.insert(
                                    &String::new(),
                                    &scribe::buffer::Position { line: 0, offset: 0 },
                                );
                            }
                        }

                        self.pos.x = pl + out.to_string().len() as u16;
                        write!(
                            self.terminal,
                            "{}{}{}{}{}{}",
                            termion::clear::CurrentLine,
                            Goto(1, self.pos.y),
                            prompt,
                            Goto(pl + 1, self.pos.y),
                            out.to_string(),
                            Goto(self.pos.x + 1, self.pos.y)
                        ).unwrap();
                    }
                }
                Key::Backspace => {
                    if !out.to_string().is_empty() {
                        if self.pos.x - pl != 0 {
                            out.delete(&scribe::buffer::Range::new(
                                scribe::buffer::Position {
                                    line: 0,
                                    offset: (self.pos.x - pl - 1) as usize,
                                },
                                scribe::buffer::Position {
                                    line: 0,
                                    offset: (self.pos.x - pl) as usize,
                                },
                            ));
                        } //Erase preceding character

                        if self.pos.x - pl > 0 {
                            write!(
                                self.terminal,
                                "{}{}{}{}{}{}",
                                termion::clear::CurrentLine,
                                Goto(1, self.pos.y),
                                prompt,
                                Goto(pl + 1, self.pos.y),
                                out.to_string(),
                                Goto(self.pos.x, self.pos.y)
                            ).unwrap();
                        }
                        self.pos.x -= 1;
                    }
                }

                // Delete is like backwards backspace.
                Key::Delete => {
                    if !out.to_string().is_empty() {
                        if self.pos.x - pl < (out.to_string().len()) as u16 {
                            out.delete(&scribe::buffer::Range::new(
                                scribe::buffer::Position {
                                    line: 0,
                                    offset: (self.pos.x - pl) as usize,
                                },
                                scribe::buffer::Position {
                                    line: 0,
                                    offset: (self.pos.x - pl + 1) as usize,
                                },
                            ));

                            write!(
                                self.terminal,
                                "{}{}{}{}",
                                termion::clear::AfterCursor,
                                Goto(pl + 1, self.pos.y),
                                out.to_string(),
                                Goto(self.pos.x + 1, self.pos.y)
                            ).unwrap();
                        } //Erase current character
                        // TODO: Find out why this exists.
/*                         if self.pos.x - pl > 0 {
                            self.pos.x - pl;
                        } */                    }
                }

                // Just enable or disable insert mode.
                Key::Insert => if self.insert {
                    self.insert = false;
                } else {
                    self.insert = true;
                },
                _ => continue,
            }
            self.terminal.flush().unwrap();
        }

        write!(self.terminal, "\n{}", termion::clear::CurrentLine).unwrap();
        if out.to_string().len() > 0 {
            for i in out.to_string()
                .split(|x: char| x.is_whitespace() || !x.is_alphanumeric())
            {
                if !i.chars().all(|x: char| x.is_numeric()) {
                    self.user_comp_tree = Trie::insert(self.user_comp_tree.clone(), &i.to_owned());
                }
            }
        }
        self.terminal.flush().unwrap();

        // Make sure history buffer doesn't contain any empty or whitespace only strings.
        if self.history.len() > 0 {
            self.history.retain(|t: &String| {
                !(t.chars()
                    .all(|c: char| c == ' ' || c == '\n' || c == '\t' || c == '\r')
                    || t.is_empty())
            });
        }
        out.to_string()
    }

    fn handle_alt(&mut self, c: char) {}
    fn handle_ctrl(&mut self, c: char) {}

    fn handle_left(&mut self, prompt_length: &u16) {
        if self.pos.x - prompt_length > 0 {
            self.pos.x -= 1;
            write!(self.terminal, "{}", Goto(self.pos.x + 1, self.pos.y)).unwrap();
        }
    }

    fn handle_right(&mut self, line: u16, prompt_length: &u16) {
        if self.pos.x - prompt_length < line {
            self.pos.x += 1;
            write!(self.terminal, "{}", Goto(self.pos.x + 1, self.pos.y)).unwrap();
        }
    }

    fn handle_bkspc(&mut self) {}
    fn handle_del(&mut self, out: &mut scribe::buffer::GapBuffer, pl: u16) {}

    fn handle_tab(&mut self, out: String) {
        // Triggers completion of the entered string.
        let mut x = self.user_comp_tree.complete(&out.as_bytes());
        x.dedup();
        let m = format!("{:?}", x,);
        self.put_line(&m);
    }
}
