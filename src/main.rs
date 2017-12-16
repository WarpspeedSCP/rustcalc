extern crate termion;

use termion::raw::IntoRawMode;

mod io;

use io::get_line;

fn main() {
    let a = get_line(
        &String::from("herp >>"),
        &mut std::io::stdout().into_raw_mode().unwrap(),
    );

    println!("{}", a)
}
