use mal_rust::env::{evaluate, get_repl_env};
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;

use mal_rust::tokenizer::Tokenizer;
use mal_rust::types::Sexp;

const HIST_PATH: &str = ".mal-history";

fn rep(input: String) -> String {
    // TODO: Make this... better
    match Sexp::read_from(&mut Tokenizer::new(input)) {
        Ok(ast) => match evaluate(ast, &get_repl_env()) {
            Ok(s) => s.to_string(),
            Err(e) => format!("[ERROR] {}", e),
        },
        Err(e) => format!("[ERROR] {}", e),
    }
}

fn main() -> Result<(), ReadlineError> {
    let mut rl = DefaultEditor::new()?;
    if rl.load_history(HIST_PATH).is_err() {
        eprintln!("History file '{}' not found", HIST_PATH);
    }
    loop {
        match rl.readline("user> ") {
            Ok(buf) => {
                if buf.is_empty() {
                    break;
                }
                rl.add_history_entry(buf.as_str())?;
                rl.save_history(HIST_PATH)?;
                println!("{}", rep(buf));
            }
            Err(ReadlineError::Interrupted) => continue,
            Err(ReadlineError::Eof) => break,
            Err(e) => return Err(e),
        }
    }
    Ok(())
}
