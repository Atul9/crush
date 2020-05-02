#[macro_use]
extern crate lalrpop_util;

mod lang;
mod lib;
mod util;

use rustyline;

use crate::lang::errors::{to_crush_error, CrushResult};
use crate::lang::pretty_printer::create_pretty_printer;
use crate::lang::printer::Printer;
use crate::lang::scope::Scope;
use crate::lang::{execute, printer};
use crate::util::file::home;
use lib::declare;
use rustyline::error::ReadlineError;
use rustyline::Editor;
use std::path::{Path, PathBuf};

fn crush_history_file() -> String {
    home()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join(Path::new(".crush_history"))
        .to_str()
        .unwrap_or(".crush_history")
        .to_string()
}

fn run_interactive(global_env: Scope, printer: Printer) -> CrushResult<()> {
    printer.line("Welcome to Crush");
    printer.line(r#"Type "help" for... help."#);

    let pretty_printer = create_pretty_printer(printer.clone());

    let mut rl = Editor::<()>::new();
    let _ = rl.load_history(&crush_history_file());
    loop {
        let readline = rl.readline("crush> ");

        match readline {
            Ok(cmd) => {
                if !cmd.is_empty() {
                    rl.add_history_entry(cmd.as_str());
                    execute::string(global_env.clone(), &cmd.as_str(), &printer, &pretty_printer);
                }
            }
            Err(ReadlineError::Interrupted) => {
                printer.line("^C");
            }
            Err(ReadlineError::Eof) => {
                printer.line("exit");
                break;
            }
            Err(err) => {
                printer.handle_error::<()>(to_crush_error(Err(err)));
                break;
            }
        }
        match rl.save_history(&crush_history_file()) {
            Ok(_) => {}
            Err(_) => {
                printer.line("Error: Failed to save history.");
            }
        }
    }
    Ok(())
}

fn run() -> CrushResult<()> {
    let global_env = lang::scope::Scope::create_root();
    let (printer, print_handle) = printer::init();
    let pretty_printer = create_pretty_printer(printer.clone());
    declare(&global_env, &printer, &pretty_printer)?;
    let my_scope = global_env.create_child(&global_env, false);

    let args = std::env::args().collect::<Vec<String>>();
    match args.len() {
        1 => run_interactive(my_scope, printer)?,
        2 => execute::file(
            my_scope,
            PathBuf::from(&args[1]).as_path(),
            &printer,
            &pretty_printer,
        )?,
        _ => {}
    }
    drop(pretty_printer);
    let _ = print_handle.join();
    Ok(())
}

fn main() {
    match run() {
        Ok(_) => (),
        Err(e) => println!("Error during initialization: {}", e.message),
    }
}
