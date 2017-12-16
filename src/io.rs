extern crate termion;

use termion::raw::IntoRawMode;
use std::io::{Read, Stdin, Stdout, Write};

mod io {
    pub fn GetLine(prompt: str, terminal: Stdout) -> String {
        let out: String;

        let rawTerm = terminal.into_raw_mode().unwrap();
    }
}
