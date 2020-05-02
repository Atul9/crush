use crate::lang::errors::{argument_error, to_crush_error, CrushResult};
use crate::lang::scope::Scope;
use crate::lang::{
    binary::BinaryReader, execution_context::ExecutionContext, list::List, value::Value,
    value::ValueType,
};
use std::env;

mod r#for;
mod r#if;
mod r#loop;
mod r#while;

use std::path::PathBuf;

pub fn r#break(context: ExecutionContext) -> CrushResult<()> {
    context.env.do_break()?;
    Ok(())
}

pub fn r#continue(context: ExecutionContext) -> CrushResult<()> {
    context.env.do_continue()?;
    Ok(())
}

pub fn cmd(mut context: ExecutionContext) -> CrushResult<()> {
    if context.arguments.is_empty() {
        return argument_error("No command given");
    }
    match context.arguments.remove(0).value {
        Value::File(f) => {
            let mut cmd = std::process::Command::new(f.as_os_str());
            for a in context.arguments.drain(..) {
                match a.argument_type {
                    None => {
                        cmd.arg(a.value.to_string());
                    }
                    Some(name) => {
                        if name.len() == 1 {
                            cmd.arg(format!("-{}", name));
                        } else {
                            cmd.arg(format!("--{}", name));
                        }
                        match a.value {
                            Value::Bool(true) => {}
                            _ => {
                                cmd.arg(a.value.to_string());
                            }
                        }
                    }
                }
            }
            let output = to_crush_error(cmd.output())?;
            let errors = String::from_utf8_lossy(&output.stderr);
            for e in errors.split('\n') {
                let err = e.trim();
                if !err.is_empty() {
                    context.printer.error(err);
                }
            }
            context
                .output
                .send(Value::BinaryStream(BinaryReader::vec(&output.stdout)))
        }
        _ => argument_error("Not a valid command"),
    }
}

pub fn declare(root: &Scope) -> CrushResult<()> {
    let e = root.create_lazy_namespace(
        "control",
        Box::new(move |env| {
            let path = List::new(ValueType::File, vec![]);
            to_crush_error(env::var("PATH").map(|v| {
                let mut dirs: Vec<Value> = v
                    .split(':')
                    .map(|s| Value::File(PathBuf::from(s)))
                    .collect();
                let _ = path.append(&mut dirs);
            }))?;
            env.declare("cmd_path", Value::List(path))?;

            env.declare_condition_command(
                "if",
                r#if::r#if,
                "if condition:bool if-clause:command [else-clause:command]",
                "Conditionally execute a command once.",
                Some(r#"    If the condition is true, the if-clause is executed. Otherwise, the else-clause
    (if specified) is executed.

    Example:

    if a > 10 {echo "big"} {echo "small"}"#))?;

            env.declare_condition_command(
                "while",
                r#while::r#while,
                "while condition:command [body:command]",
                "Repeatedly execute the body for as long the condition is met",
                Some(r#"    In every pass of the loop, the condition is executed. If it returns false,
    the loop terminates. If it returns true, the body is executed and the loop
    continues.

    Example:

    while {not (./some_file:stat):is_file} {echo "hello"}

    The loop body is optional. If not specified, the condition is executed until it returns false.
    This effectively means that the condition becomes the body, and the loop break check comes at
    the end of the loop."#))?;

            env.declare_condition_command(
                "loop",
                r#loop::r#loop,
                "loop body:command",
                "Repeatedly execute the body until the break command is called.",
                Some(r#"    Example:
    loop {
        if (i_am_tired) {
            break
        }
        echo "Working"
    }"#))?;

            env.declare_condition_command(
                "for",
                r#for::r#for,
                "for [name=]iterable:(table_stream|table|dict|list) body:command",
                "Execute body once for every element in iterable.",
                Some(r#"    Example:

    for (seq 10) {
        echo ("Lap #{}":format value)
    }"#))?;


            env.declare_command(
                "break", r#break, false,
                "break", "Stop execution of a loop", None)?;
            env.declare_command(
                "continue", r#continue, false,
                "continue",
                "Skip execution of the current iteration of a loop",
                None)?;
            env.declare_command(
                "cmd", cmd, true,
                "cmd external_command:(file|string) @arguments:any",
                "Execute external commands",
                None)?;
            Ok(())
        }))?;
    root.r#use(&e);
    Ok(())
}
