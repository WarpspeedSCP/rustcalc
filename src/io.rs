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
) -> String {
    write!(terminal, "{}{}{}", termion::clear::All, Goto(1, 1), prompt).unwrap();

    terminal.flush().unwrap();

    let mut out: scribe::buffer::GapBuffer = scribe::buffer::GapBuffer::new(String::from(""));
    let mut pos = point::new(prompt.len() as u16, 1);

    for c in input.keys() {
        match c.unwrap() {
            // Exit.
            Key::Char('\n') => {
                pos.x = prompt.len() as u16 + 1;
                break;
            }
            Key::Char(c) => {
                pos.x += 1;
                out.insert(
                    &c.to_string(),
                    &scribe::buffer::Position {
                        line: 0,
                        offset: (pos.x - prompt.len() as u16 - 1) as usize,
                    },
                );
                write!(terminal, "{}{}", Goto(pos.x, pos.y), c).unwrap();
                terminal.flush().unwrap();
            }
            Key::Alt(c) => handle_alt(c),
            Key::Ctrl(c) => handle_ctrl(c),
            Key::Left => handle_left(
                terminal,
                &mut out.to_string(),
                &mut pos,
                prompt.len() as u16,
            ),
            Key::Right => handle_right(terminal),
            Key::Up => handle_up(terminal, history, &mut out.to_string()),
            Key::Down => handle_down(terminal, history, &mut out.to_string()),
            //           Key::Backspace => handle_bkspc(),
            //          Key::Delete => handle_del(),
            //          Key::Insert => handle_ins(),
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

fn handle_left(
    terminal: &mut RawTerminal<Stdout>,
    line: &mut String,
    pos: &mut point,
    promptLength: u16,
) {
    if pos.x - promptLength > 0 {
        pos.x -= 1;
    } else {
    }

    Goto(pos.x, pos.y);
}

fn handle_right(terminal: &mut RawTerminal<Stdout>) {}
