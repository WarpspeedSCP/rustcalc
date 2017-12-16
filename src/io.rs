extern crate termion;

use termion::raw::RawTerminal;
use termion::raw::IntoRawMode;
use termion::input::TermRead;
use termion::event::Key;

use std::io::{Read, Stdin, Stdout, Write};

pub fn get_line(prompt: &String, terminal: &mut RawTerminal<Stdout>) -> String {
    write!(
        terminal,
        "{}{}q to exit. Type stuff, use alt, and so on.{}",
        termion::clear::All,
        termion::cursor::Goto(1, 1),
        termion::cursor::Hide
    ).unwrap();

    terminal.flush().unwrap();

    String::from("derp\n")
}
