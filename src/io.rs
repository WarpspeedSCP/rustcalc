extern crate scribe;
extern crate termion;

use termion::raw::RawTerminal;
use termion::input::TermRead;
use termion::event::Key;
use termion::cursor::Goto;

use std::io::{Stdin, Stdout, Write};

struct point {
    x: u16,
    y: u16,
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
) -> String {
    write!(terminal, "{}{}{}", termion::clear::All, Goto(1, 1), prompt).unwrap();

    terminal.flush().unwrap();

    let mut out: scribe::buffer::GapBuffer = scribe::buffer::GapBuffer::new(String::from(""));
    let mut pos = point::new(prompt.len() as u16, 1);
    let pl = prompt.len() as u16;

    for c in input.keys() {
        match c.unwrap() {
            // Exit.
            Key::Char('\n') => break,
            Key::Char('\t') => handle_tab(),
            Key::Char(c) => {
                if pos.x - pl == out.to_string().len() as u16 {
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
                    write!(terminal, "{}{}{}", Goto(pos.x, pos.y), c, tmp).unwrap();
                } else if *insert {
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
                    write!(terminal, "{}{}", Goto(pos.x, pos.y), c).unwrap();
                }

                terminal.flush().unwrap();
            }
            Key::Alt(c) => handle_alt(c),
            Key::Ctrl(c) => handle_ctrl(c),
            Key::Left => handle_left(terminal, &mut pos, pl),
            Key::Right => handle_right(terminal, &mut out.to_string(), &mut pos, pl),
            Key::Up => handle_up(terminal, history, &mut out.to_string()),
            Key::Down => handle_down(terminal, history, &mut out.to_string()),
            Key::Backspace => handle_bkspc(),
            Key::Delete => handle_del(),
            Key::Insert => if *insert {
                *insert = false;
                write!(terminal, "{}Insert of{}", Goto(2, 1), Goto(pos.x, pos.y)).unwrap();
                terminal.flush().unwrap();
            } else {
                *insert = true;
                write!(terminal, "{}Insert on{}", Goto(2, 1), Goto(pos.x, pos.y)).unwrap();
                terminal.flush().unwrap();
            },
            _ => continue,
        }
    }

    write!(terminal, "\n{}", termion::clear::CurrentLine).unwrap();
    terminal.flush().unwrap();

    history.push(out.to_string());

    out.to_string()
}

fn handle_alt(c: char) {}
fn handle_ctrl(c: char) {}
fn handle_up(terminal: &mut RawTerminal<Stdout>, history: &mut Vec<String>, line: &mut String) {}
fn handle_down(terminal: &mut RawTerminal<Stdout>, history: &mut Vec<String>, line: &mut String) {}

fn handle_left(terminal: &mut RawTerminal<Stdout>, pos: &mut point, promptLength: u16) {
    if pos.x - promptLength > 0 {
        pos.x -= 1;
        write!(terminal, "{}", Goto(pos.x + 1, pos.y)).unwrap();
        terminal.flush().unwrap();
    } else {
        pos.x = promptLength + 1;
    }
}



fn handle_right(
    terminal: &mut RawTerminal<Stdout>,
    line: &mut String,
    pos: &mut point,
    promptLength: u16,
) {
    if pos.x - promptLength < line.len() as u16 {
        pos.x += 1;
        write!(terminal, "{}", Goto(pos.x + 1, pos.y)).unwrap();
        terminal.flush().unwrap();
    } else {
        pos.x = promptLength + line.len() as u16;
    }
}

fn handle_bkspc() {}
fn handle_del() {}

fn handle_tab() {}
