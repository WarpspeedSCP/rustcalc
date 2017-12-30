extern crate fst;
extern crate scribe;
extern crate termion;

use termion::raw::RawTerminal;
use termion::input::TermRead;
use termion::event::Key;
use termion::cursor::Goto;

use std::io::{Stdin, Stdout, Write};

pub struct point {
    pub x: u16,
    pub y: u16,
}

impl point {
    pub fn new(x: u16, y: u16) -> point {
        point { x: x, y: y }
    }
}

pub fn get_line(
    prompt: &String,
    terminal: &mut RawTerminal<Stdout>,
    input: &mut Stdin,
    history: &mut Vec<String>,
    insert: &mut bool,
    pos: &mut point,
) -> String {
    write!(
        terminal,
        "{}{}{}",
        termion::clear::CurrentLine,
        Goto(1, pos.y),
        prompt
    ).unwrap();

    terminal.flush().unwrap();

    let mut out: scribe::buffer::GapBuffer = scribe::buffer::GapBuffer::new(String::from(""));
    pos.x = prompt.len() as u16;
    let pl = prompt.len() as u16;
    let mut h_index = history.len();

    for c in input.keys() {
        if pos.x <= pl {
            pos.x = pl;
        }
        if pos.x > pl + out.to_string().len() as u16 {
            pos.x = pl + out.to_string().len() as u16;
        }
        match c.unwrap() {
            // Exit.
            Key::Char('\n') => {
                history.push(out.to_string());
                pos.y += 1;
                break;
            }
            Key::Char('\t') => handle_tab(),
            Key::Char(c) => {
                if pos.x - pl == out.to_string().len() as u16 {
                    // End of the line

                    pos.x += 1;

                    out.insert(
                        &c.to_string(),
                        &scribe::buffer::Position {
                            line: 0,
                            offset: (pos.x - pl as u16 - 1) as usize,
                        },
                    );

                    write!(terminal, "{}{}", Goto(pos.x, pos.y), c).unwrap();
                } else if !*insert {
                    let tmp = &out.to_string()[(pos.x - pl) as usize..];

                    pos.x += 1;

                    out.insert(
                        &c.to_string(),
                        &scribe::buffer::Position {
                            line: 0,
                            offset: (pos.x - pl - 1) as usize,
                        },
                    );

                    write!(
                        terminal,
                        "{}{}{}{}",
                        Goto(pos.x, pos.y),
                        c,
                        tmp,
                        Goto(pos.x + 1, pos.y)
                    ).unwrap();
                } else if *insert {
                    // Insert mode off, anywhere other than the end

                    out.delete(&scribe::buffer::Range::new(
                        scribe::buffer::Position {
                            line: 0,
                            offset: (pos.x - pl) as usize,
                        },
                        scribe::buffer::Position {
                            line: 0,
                            offset: (pos.x + 1 - pl) as usize,
                        },
                    ));

                    pos.x += 1;

                    out.insert(
                        &c.to_string(),
                        &scribe::buffer::Position {
                            line: 0,
                            offset: (pos.x - pl as u16 - 1) as usize,
                        },
                    );

                    write!(
                        terminal,
                        "{}{}{}",
                        Goto(pos.x, pos.y),
                        c,
                        Goto(pos.x, pos.y)
                    ).unwrap();
                }
            }
            Key::Alt(c) => handle_alt(c),
            Key::Ctrl(c) => {
                if c == 'c' {
                    return String::from("exit");
                }
                handle_ctrl(c, terminal)
            }
            Key::Left => handle_left(terminal, pos, &pl),
            Key::Right => handle_right(terminal, &mut out.to_string(), pos, &pl),
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
                    pos.x = pl + out.to_string().len() as u16;
                    write!(
                        terminal,
                        "{}{}{}{}{}{}",
                        termion::clear::CurrentLine,
                        Goto(1, pos.y),
                        prompt,
                        Goto(pl + 1, pos.y),
                        out.to_string(),
                        Goto(pos.x + 1, pos.y)
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
                    pos.x = pl + out.to_string().len() as u16;
                    write!(
                        terminal,
                        "{}{}{}{}{}{}",
                        termion::clear::CurrentLine,
                        Goto(1, pos.y),
                        prompt,
                        Goto(pl + 1, pos.y),
                        out.to_string(),
                        Goto(pos.x + 1, pos.y)
                    ).unwrap();
                }
            }
            Key::Backspace => {
                if !out.to_string().is_empty() {
                    if pos.x - pl == 0 {
;
                    } else {
                        out.delete(&scribe::buffer::Range::new(
                            scribe::buffer::Position {
                                line: 0,
                                offset: (pos.x - pl - 1) as usize,
                            },
                            scribe::buffer::Position {
                                line: 0,
                                offset: (pos.x - pl) as usize,
                            },
                        ));
                    } //Erase preceding character

                    if pos.x - pl > 0 {
                        write!(
                            terminal,
                            "{}{}{}{}{}{}",
                            termion::clear::CurrentLine,
                            Goto(1, pos.y),
                            prompt,
                            Goto(pl + 1, pos.y),
                            out.to_string(),
                            Goto(pos.x, pos.y)
                        ).unwrap();
                    }
                    pos.x -= 1;
                }
            }

            Key::Delete => handle_del(),
            Key::Insert => if *insert {
                *insert = false;
            } else {
                *insert = true;
            },
            _ => continue,
        }
        terminal.flush().unwrap();
    }

    write!(terminal, "\n{}", termion::clear::CurrentLine).unwrap();
    terminal.flush().unwrap();
    out.to_string()
}

fn handle_alt(c: char) {}
fn handle_ctrl(c: char, terminal: &mut RawTerminal<Stdout>) {}

fn handle_left(terminal: &mut RawTerminal<Stdout>, pos: &mut point, promptLength: &u16) {
    if pos.x - promptLength > 0 {
        pos.x -= 1;
        write!(terminal, "{}", Goto(pos.x + 1, pos.y)).unwrap();
    }
}

fn handle_right(
    terminal: &mut RawTerminal<Stdout>,
    line: &mut String,
    pos: &mut point,
    promptLength: &u16,
) {
    if pos.x - promptLength < line.len() as u16 {
        pos.x += 1;
        write!(terminal, "{}", Goto(pos.x + 1, pos.y)).unwrap();
    }
}

fn handle_bkspc() {}
fn handle_del() {}

fn handle_tab() {}
