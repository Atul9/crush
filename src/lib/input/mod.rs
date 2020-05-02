use crate::lang::errors::CrushResult;
use crate::lang::list::List;
use crate::lang::pretty_printer::PrettyPrinter;
use crate::lang::scope::Scope;
use crate::lang::value::ValueType;
use crate::lang::{
    binary::BinaryReader, execution_context::ArgumentVector, execution_context::ExecutionContext,
    value::Value,
};

mod csv;
mod http;
mod lines;

pub fn val(mut context: ExecutionContext) -> CrushResult<()> {
    context.arguments.check_len(1)?;
    context.output.send(context.arguments.value(0)?)
}

pub fn dir(mut context: ExecutionContext) -> CrushResult<()> {
    context.arguments.check_len(1)?;
    context.output.send(Value::List(List::new(
        ValueType::String,
        context
            .arguments
            .value(0)?
            .fields()
            .drain(..)
            .map(|n| Value::String(n))
            .collect(),
    )))
}

fn echo(mut context: ExecutionContext) -> CrushResult<()> {
    for arg in context.arguments.drain(..) {
        PrettyPrinter::new(context.printer.clone()).print_value(arg.value);
    }
    Ok(())
}

fn cat(mut context: ExecutionContext) -> CrushResult<()> {
    context.output.send(Value::BinaryStream(BinaryReader::paths(
        context.arguments.files(&context.printer)?,
    )?))
}

pub fn declare(root: &Scope) -> CrushResult<()> {
    let e = root.create_lazy_namespace(
        "input",
        Box::new(move |env| {
            env.declare_command(
                "cat", cat, true,
                "cat @files:(file|glob)", "Read specified files as binary stream", None)?;
            env.declare_command(
                "http", http::perform, true,
                "http url:string [form=formdata:string] [method=method:string] [header=header:string]...",
                "Make a http request",
                Some(r#"    Headers should be on the form "key:value".

    Examples:

    http "https://example.com/" header="Authorization: Bearer {}":format token"#))?;
            env.declare_command(
                "lines", lines::perform, true,
                "lines @files:(file|glob)", "Read specified files as a table with one line of text per row", None)?;
            env.declare_command(
                "csv", csv::csv, true,
                "csv <column_name>=type:type... [head=skip:integer] [separator=separator:string] [trim=trim:string] @files:(file|glob)",
                "Parse specified files as CSV files", Some(r#"    Examples:

    csv separator="," head=1 name=string age=integer nick=string"#))?;

            env.declare_command(
                "echo", echo, false,
                "echo @value:any", "Prints all arguments directly to the screen", None)?;
            env.declare_command(
                "val", val, false,
                "val value:any",
                "Return value",
                Some(r#"    This command is useful if you want to e.g. pass a command as input in
    a pipeline instead of executing it. It is different from the echo command
    in that val returns the value, and echo prints it to screen."#))?;
            env.declare_command(
                "dir", dir, false,
                "dir value:any", "List members of value", None)?;
            Ok(())
        }))?;
    root.r#use(&e);
    Ok(())
}
