extern crate fst;
extern crate fst_regex;
extern crate scribe;
extern crate std;
extern crate termion;

use termion::raw::RawTerminal;
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

    pub fn complete(&self, string: &[u8]) -> Vec<String> {
        match string.len() {
            0 => self.descend(),
            _ => match self.children.get(&(string[0] as char)) {
                Some(s) => s.complete(&string[1..]),
                None => Vec::new(),
            },
        }
    }

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

    pub fn insert(mut trie: Trie, string: &String) -> Trie {
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

pub struct InputManager<'a> {
    //key_comp_tree: fst::Map,
    user_comp_tree: Trie, //Trie<String, String>,
    terminal: &'a mut RawTerminal<Stdout>,
    insert: bool,
    pos: Point,
}

impl<'a> InputManager<'a> {
    pub fn new(t: &mut RawTerminal<Stdout>) -> InputManager {
        InputManager {
            user_comp_tree: Trie::new(),
            terminal: t,
            insert: false,
            pos: Point::new(1, 1),
        }
    }

    pub fn put_line(&mut self, output: &String) {
        write!(
            self.terminal,
            "{}{}{}{}\n",
            Goto(1, self.pos.y + 2),
            termion::clear::CurrentLine,
            output,
            Goto(1, self.pos.y)
        ).unwrap();
    }

    pub fn clear_all(&mut self) {
        write!(self.terminal, "{}{}", termion::clear::All, Goto(1, 1));
    }

    pub fn get_line(
        &mut self,
        prompt: &String,
        input: &mut Stdin,
        history: &mut Vec<String>,
    ) -> String {
        write!(
            self.terminal,
            "{}{}{}",
            termion::clear::CurrentLine,
            Goto(1, self.pos.y),
            prompt
        ).unwrap();

        self.terminal.flush().unwrap();

        let mut out: scribe::buffer::GapBuffer = scribe::buffer::GapBuffer::new(String::from(""));
        self.pos.x = prompt.len() as u16;
        let pl = prompt.len() as u16;
        let mut h_index = history.len();

        for c in input.keys() {
            if self.pos.x <= pl {
                self.pos.x = pl;
            }
            if self.pos.x > pl + out.to_string().len() as u16 {
                self.pos.x = pl + out.to_string().len() as u16;
            }
            match c.unwrap() {
                // Exit.
                Key::Char('\n') => {
                    history.push(out.to_string());
                    self.pos.y += 1;
                    break;
                }
                Key::Char('\t') => match self.handle_tab(out.to_string()) {
                    _ => {}
                },
                Key::Char(c) => {
                    if self.pos.x - pl == out.to_string().len() as u16 {
                        // End of the line

                        self.pos.x += 1;

                        out.insert(
                            &c.to_string(),
                            &scribe::buffer::Position {
                                line: 0,
                                offset: (self.pos.x - pl as u16 - 1) as usize,
                            },
                        );

                        write!(self.terminal, "{}{}", Goto(self.pos.x, self.pos.y), c).unwrap();
                    } else if !self.insert {
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
                    } else if self.insert {
                        // Insert mode off, anywhere other than the end

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
                Key::Alt(c) => self.handle_alt(c),
                Key::Ctrl(c) => {
                    if c == 'c' {
                        return String::from("exit");
                    }
                    self.handle_ctrl(c)
                }
                Key::Left => self.handle_left(&pl),
                Key::Right => self.handle_right(&mut out.to_string(), &pl),
                Key::Up => {
                    if !history.is_empty() {
                        if h_index > 0 {
                            if h_index == history.len() {
                                history.push(out.to_string());
                            } else if history[h_index - 1] == "" || history[h_index - 1] == " " {
;
                            }
                            let d = out.to_string().len();
                            out.delete(&scribe::buffer::Range::new(
                                scribe::buffer::Position { line: 0, offset: 0 },
                                scribe::buffer::Position { line: 0, offset: d },
                            ));

                            out.insert(
                                &history[h_index - 1],
                                &scribe::buffer::Position { line: 0, offset: 0 },
                            );

                            h_index -= 1;
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
                Key::Down => {
                    if !history.is_empty() {
                        if h_index < history.len() {
                            let d = out.to_string().len();
                            out.delete(&scribe::buffer::Range::new(
                                scribe::buffer::Position { line: 0, offset: 0 },
                                scribe::buffer::Position { line: 0, offset: d },
                            ));

                            if h_index != history.len() - 1 {
                                h_index += 1
                            } else {
                                h_index = history.len() - 1;
                            }
                            out.insert(
                                &history[h_index],
                                &scribe::buffer::Position { line: 0, offset: 0 },
                            );
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

                Key::Delete => self.handle_del(&mut out, pl),
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

    fn handle_right(&mut self, line: &mut String, prompt_length: &u16) {
        if self.pos.x - prompt_length < line.len() as u16 {
            self.pos.x += 1;
            write!(self.terminal, "{}", Goto(self.pos.x + 1, self.pos.y)).unwrap();
        }
    }

    fn handle_bkspc(&mut self) {}
    fn handle_del(&mut self, out: &mut scribe::buffer::GapBuffer, pl: u16) {
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

            if self.pos.x - pl > 0 {
                self.pos.x - pl;
            }
        }
    }

    fn handle_tab(&mut self, out: String) {
        let mut x = self.user_comp_tree.complete(&out.as_bytes());
        x.dedup();
        let m = format!("{:?}", x,);
        self.put_line(&m);
    }
}
