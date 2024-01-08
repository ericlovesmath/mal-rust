use mal_rust::env::{env_default, evaluate, Env};
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;

use mal_rust::tokenizer::Tokenizer;
use mal_rust::types::Sexp;

const HIST_PATH: &str = ".mal-history";

fn rep(input: String, env: &Env) -> Result<String, String> {
    let ast = Sexp::read_from(&mut Tokenizer::new(input))?;
    let output = evaluate(ast, env.clone())?;
    Ok(output.to_string())
}

fn main() -> Result<(), ReadlineError> {
    let env = env_default();
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
                match rep(buf, &env) {
                    Ok(output) => println!("{}", output),
                    Err(error) => println!("[ERROR] {}", error),
                };
            }
            Err(ReadlineError::Interrupted) => continue,
            Err(ReadlineError::Eof) => break,
            Err(e) => return Err(e),
        }
    }
    Ok(())
}
