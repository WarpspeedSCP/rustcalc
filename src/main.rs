#[macro_use]
extern crate lazy_static;

extern crate ordered_float;

#[macro_use]
extern crate node_t_derive;

#[macro_use]
extern crate terminal_node_derive;

#[macro_use]
extern crate encapsulating_node_derive;

extern crate termion;
extern crate serde_json;

#[macro_use]
extern crate serde_derive;

mod io;
mod parser;
#[macro_use]
mod ast;
//mod interpreter;

use std::io::Write;

use io::InputManager;
//use ast::SymTable;
use ast::Node;

/*
fn a (x, y, z) { if x == y return z * 2; else return z / 2; } fn b (l, m) { if a(l, m, 2) > 2 { x = 3; y = 16;  m = (l * x) / y; } else m = 2; return m; }
*/
fn main() {
    let mut history: Vec<String> = Vec::new();
    let mut a: String = String::new();
    let mut parser = parser::Parser::new();
    let mut d = ast::Program::default();

    //let mut sym_table = SymTable::new();

    //*
    {
        let mut input = std::io::stdin();
        //let mut p: io::Point = io::Point::new(1, 1);
        //let mut im = InputManager::new();

        //im.clear_all();

        while a != String::from("exit") {
            input.read_line(&mut a).expect("Could not read line from input."); //im.get_line(&"prompt:>".to_owned(), &mut input);

            parser.input(a.clone());

            if a != "exit".to_owned() {
                d = parser.eval();
                break;
            }
        }

        //        im.put_line(&"\r\n".to_owned());
    }

    println!("{:#?}", d);
    //Node::po_(&d);

    /*
    {
        let mut res = parser.eval(&"array a = 2 // 4"[..].as_bytes());
        println!("{:?}", res);

        res.statement_eval(&mut sym_table);
        println!("\n\nGlobal scope: {:?}", sym_table);

        println!("Answer: {}\n", res.eval());
    } */
}
