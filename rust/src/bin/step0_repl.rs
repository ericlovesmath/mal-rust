extern crate mal_rust;

use std::io::{stdin, stdout, Write};

fn read(text: &str) -> &str {
    text
}
fn eval(text: &str) -> &str {
    text
}
fn print(text: &str) -> &str {
    text
}

fn rep(input: &str) -> &str {
    print(eval(read(input)))
}

fn main() {
    let stdin = stdin();
    let mut stdout = stdout();
    let mut buf = String::new();
    loop {
        print!("user> ");
        stdout.flush().expect("Failed to flush()");

        buf.clear();
        match stdin.read_line(&mut buf) {
            Ok(0) => break,
            Ok(_) => (),
            Err(err) => panic!("Failed to read_line(), {}", err),
        }
        print!("{}", rep(&buf))
    }
}
