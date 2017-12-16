extern crate termion;

use termion::raw::RawTerminal;
use termion::raw::IntoRawMode;
use termion::input::TermRead;
use termion::event::Key;

use std::io::{stdin, Read, Stdin, Stdout, Write};

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
    history: Vec<&String>,
) -> String {
    write!(
        terminal,
        "{}{}{}",
        termion::clear::All,
        termion::cursor::Goto(1, 1),
        prompt
    ).unwrap();

    terminal.flush().unwrap();

    let mut out: String = String::new();
    let mut pos = point::new(prompt.len() as u16, 1);

    for c in input.keys() {
        pos.x += 1;

        match c.unwrap() {
            // Exit.
            Key::Char('\n') => break,
            Key::Char(c) => {
                out.push(c);
                write!(terminal, "{}{}", termion::cursor::Goto(pos.x, pos.y), c).unwrap();
                terminal.flush().unwrap();
            }
            Key::Alt(c) => handle_alt(c),
            Key::Ctrl(c) => handle_ctrl(c),
            Key::Left => handle_left(terminal),
            Key::Right => handle_right(terminal),
            Key::Up => handle_up(terminal, history),
            Key::Down => handle_down(terminal, history),
            _ => continue,
        }
    }

    write!(terminal, "\n");
    write!(terminal, "{}", termion::clear::CurrentLine);
    terminal.flush().unwrap();

    out
}

fn handle_alt(c: char) {}
fn handle_ctrl(c: char) {}
fn handle_up(terminal: &mut RawTerminal<Stdout>, history: Vec<&String>) {}
fn handle_down(terminal: &mut RawTerminal<Stdout>, history: Vec<&String>) {}
fn handle_left(terminal: &mut RawTerminal<Stdout>) {}
fn handle_right(terminal: &mut RawTerminal<Stdout>) {}
