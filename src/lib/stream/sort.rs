use crate::lang::errors::{error, CrushResult};
use crate::lang::execution_context::{ArgumentVector, ExecutionContext};
use crate::lang::stream::Readable;
use crate::lang::table::{ColumnType, ColumnVec};
use crate::lang::{argument::Argument, table::Row};
use crate::{lang::errors::argument_error, lang::stream::OutputStream};

fn parse(mut arguments: Vec<Argument>, types: &[ColumnType]) -> CrushResult<usize> {
    arguments.check_len_range(0, 1)?;
    if let Some(f) = arguments.optional_field(0)? {
        Ok(types.find(&f)?)
    } else if types.len() != 1 {
        argument_error("No sort key specified")
    } else {
        Ok(0)
    }
}

pub fn run(idx: usize, input: &mut dyn Readable, output: OutputStream) -> CrushResult<()> {
    let mut res: Vec<Row> = Vec::new();
    while let Ok(row) = input.read() {
        res.push(row);
    }

    res.sort_by(|a, b| a.cells()[idx].partial_cmp(&b.cells()[idx]).expect("OH NO!"));

    for row in res {
        output.send(row)?;
    }

    Ok(())
}

pub fn perform(context: ExecutionContext) -> CrushResult<()> {
    match context.input.recv()?.readable() {
        Some(mut input) => {
            let output = context.output.initialize(input.types().to_vec())?;
            let idx = parse(context.arguments, input.types())?;
            if input.types()[idx].cell_type.is_comparable() {
                run(idx, input.as_mut(), output)
            } else {
                argument_error("Bad comparison key")
            }
        }
        None => error("Expected a stream"),
    }
}
