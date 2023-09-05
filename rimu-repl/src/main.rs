use rustyline::error::ReadlineError;
use rustyline::{DefaultEditor, Result};

use rimu::{evaluate_expression, parse_expression, Environment, ReportError, SourceId};

fn main() -> Result<()> {
    // `()` can be used when no completer is required
    let mut rl = DefaultEditor::new()?;
    if rl.load_history(".repl-history.txt").is_err() {
        println!("No previous history.");
    }
    let env = Environment::new();
    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str())?;
                let (expr, errors) = parse_expression(line.as_str(), SourceId::repl());
                if !errors.is_empty() {
                    for error in errors {
                        error.display(line.as_str(), SourceId::repl());
                    }
                    continue;
                }
                let Some(expr) = expr else {
                        println!("No expression.");
                        continue;
                    };
                // println!("Expression: {}", expr);
                match evaluate_expression(&expr, &env) {
                    Ok(value) => println!("{}", value),
                    Err(error) => error.display(line.as_str(), SourceId::repl()),
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
    rl.save_history(".repl-history.txt")?;
    Ok(())
}
