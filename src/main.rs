#![feature(inclusive_range_syntax)]
#[macro_use]
extern crate lazy_static;
extern crate radix_trie;
extern crate termion;

mod io;
mod parser;
mod ast;

use termion::raw::IntoRawMode;

use std::io::Write;

use io::InputManager;
//use ast::SymTable;
use ast::Node;

fn main() {
    let mut history: Vec<String> = Vec::new();
    let mut a: String = String::new();
    let mut parser = parser::Parser::new();
    let mut d = Node::new();

    //let mut sym_table = SymTable::new();

    //*
    {
        let mut input = std::io::stdin();
        let mut terminal = std::io::stdout().into_raw_mode().unwrap();
        let mut p: io::Point = io::Point::new(1, 1);
        let mut im = InputManager::new(&mut terminal);

        im.clear_all();

        while a != String::from("exit") {
            a = im.get_line(&"prompt:>".to_owned(), &mut input, &mut history);

            parser.input(a.clone());

            if a != "exit".to_owned() {
                d = parser.eval();
            }

            if history.len() > 0 {
                history.retain(|t| *t != "".to_owned());
            }
        }

        im.put_line(&"\r\n".to_owned());
    }

    Node::po_(&d);

    /*
    {
        let mut res = parser.eval(&"array a = 2 // 4"[..].as_bytes());
        println!("{:?}", res);

        res.statement_eval(&mut sym_table);
        println!("\n\nGlobal scope: {:?}", sym_table);

        println!("Answer: {}\n", res.eval());
    } */
}
