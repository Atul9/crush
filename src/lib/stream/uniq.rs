use crate::lang::argument::Argument;
use crate::lang::errors::{error, CrushResult};
use crate::lang::execution_context::{ArgumentVector, ExecutionContext};
use crate::lang::printer::Printer;
use crate::lang::stream::{OutputStream, Readable};
use crate::lang::table::ColumnVec;
use crate::lang::table::Row;
use crate::lang::{table::ColumnType, value::Value};
use std::collections::HashSet;

pub fn parse(
    input_type: &[ColumnType],
    mut arguments: Vec<Argument>,
) -> CrushResult<Option<usize>> {
    arguments.check_len_range(0, 1)?;
    if let Some(f) = arguments.optional_field(0)? {
        Ok(Some(input_type.find(&f)?))
    } else {
        Ok(None)
    }
}

pub fn run(
    idx: Option<usize>,
    input: &mut dyn Readable,
    output: OutputStream,
    printer: &Printer,
) -> CrushResult<()> {
    match idx {
        None => {
            let mut seen: HashSet<Row> = HashSet::new();
            while let Ok(row) = input.read() {
                if !seen.contains(&row) {
                    seen.insert(row.clone());
                    printer.handle_error(output.send(row));
                }
            }
        }
        Some(idx) => {
            let mut seen: HashSet<Value> = HashSet::new();
            while let Ok(row) = input.read() {
                if !seen.contains(&row.cells()[idx]) {
                    seen.insert(row.cells()[idx].clone());
                    printer.handle_error(output.send(row));
                }
            }
        }
    }
    Ok(())
}

pub fn uniq(context: ExecutionContext) -> CrushResult<()> {
    match context.input.recv()?.readable() {
        Some(mut input) => {
            let idx = parse(input.types(), context.arguments)?;
            let output = context.output.initialize(input.types().to_vec())?;
            run(idx, input.as_mut(), output, &context.printer)
        }
        _ => error("Expected input to be a stream"),
    }
}
