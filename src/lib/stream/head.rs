use crate::lang::errors::{error, CrushResult};
use crate::lang::execution_context::{ArgumentVector, ExecutionContext};
use crate::lang::stream::{Readable, ValueSender};

pub fn run(lines: i128, input: &mut dyn Readable, sender: ValueSender) -> CrushResult<()> {
    let output = sender.initialize(input.types().to_vec())?;
    let mut count = 0;
    while let Ok(row) = input.read() {
        if count >= lines {
            break;
        }
        output.send(row)?;
        count += 1;
    }
    Ok(())
}

pub fn perform(mut context: ExecutionContext) -> CrushResult<()> {
    context.arguments.check_len_range(0, 1)?;
    let lines = context.arguments.optional_integer(0)?.unwrap_or(10);
    match context.input.recv()?.readable() {
        Some(mut r) => run(lines, r.as_mut(), context.output),
        None => error("Expected a stream"),
    }
}
