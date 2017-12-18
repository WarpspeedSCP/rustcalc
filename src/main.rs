extern crate termion;

use termion::raw::IntoRawMode;

mod io;

use io::get_line;

fn main() {
    let mut history: Vec<String> = Vec::new();
    let mut a: String = String::new();
    let mut insert = false;

    {
        let mut input = std::io::stdin();
        let mut terminal = std::io::stdout().into_raw_mode().unwrap();

        while a != String::from("exit") {
            a = get_line(
                &String::from("herp >>"),
                &mut terminal,
                &mut input,
                &mut history,
                &mut insert,
            );
        }
    }


    println!("\n");

    for x in history.iter() {
        println!("{}", x);
    }
}
