extern crate termion;

use termion::raw::IntoRawMode;
use std::io::Write;

mod io;

use io::get_line;

fn main() {
    let mut history: Vec<String> = Vec::new();
    let mut a: String = String::new();
    let mut insert = false;

    {
        let mut input = std::io::stdin();
        let mut terminal = std::io::stdout().into_raw_mode().unwrap();
        let mut p: io::point = io::point::new(1, 1);

        write!(terminal, "{}", termion::clear::All);

        while a != String::from("exit") {
            a = get_line(
                &String::from("herp >>"),
                &mut terminal,
                &mut input,
                &mut history,
                &mut insert,
                &mut p,
            );

            for i in 0..history.len() - 1 {
                if history[i] == "" {
                    history.remove(i);
                }
            }
        }
    }


    println!("\n");

    for x in history.iter() {
        println!("{}", x);
    }
}
