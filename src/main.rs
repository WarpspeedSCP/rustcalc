extern crate itertools;
extern crate termion;

use termion::raw::IntoRawMode;
use std::io::Write;

mod io;
mod parser;

use io::get_line;

macro_rules! scanline {
    ($x:expr) => {
       std::io::stdin().read_line(&mut $x).unwrap();
    };
}

fn main() {
    let mut history: Vec<String> = Vec::new();
    let mut a: String = String::new();
    let mut insert = false;
    let mut parser = parser::Parser::new();
    let mut out: u32 = 0xFFFFFFFF;

    /*
    let m = parser::Token::UnaryFn(|a| if a > 10 { a * 4 } else { a * 2 }); // example of fn from enum
    let n = parser::Token::BinaryFn(|a, b| if a > b { a } else { b }); // another example


    println!(
        "before:{}\nafter{}\n a = {}, b = {}\n now, result1 = {}\n",
        3,
        match m {
            parser::Token::UnaryFn(m) => m(3),
            _ => panic!("Darp\n!"),
        },
        16,
        45,
        match n {
            parser::Token::BinaryFn(n) => n(16, 45),
            _ => panic!("Derp!"),
        }
    );
*/
    //    /*
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

            if a != "exit" {
                out = parser.expr(&a.as_str());
                write!(
                    terminal,
                    "{}{}{}\n",
                    termion::cursor::Goto(1, p.y + 1),
                    termion::clear::CurrentLine,
                    out
                );
            }

            if history.len() > 0 {
                for i in 0..history.len() - 1 {
                    if history[i] == "" {
                        history.remove(i);
                    }
                }
            }
        }
    }

    for x in history.iter() {
        println!("{}", x);
    }
    //        */
/*
    let res = parser.expr(&"3 - 2"[..]);
    println!("Res: {}\n", res);
*/}
