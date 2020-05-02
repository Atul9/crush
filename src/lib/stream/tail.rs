use std::collections::VecDeque;

use crate::lang::errors::{error, CrushResult};
use crate::lang::execution_context::{ArgumentVector, ExecutionContext};
use crate::lang::stream::{Readable, ValueSender};
use crate::lang::table::Row;

pub fn run(lines: i128, input: &mut dyn Readable, sender: ValueSender) -> CrushResult<()> {
    let output = sender.initialize(input.types().to_vec())?;
    let mut q: VecDeque<Row> = VecDeque::new();
    while let Ok(row) = input.read() {
        if q.len() >= lines as usize {
            q.pop_front();
        }
        q.push_back(row);
    }
    for row in q.drain(..) {
        output.send(row)?;
    }
    Ok(())
}

pub fn perform(mut context: ExecutionContext) -> CrushResult<()> {
    context.arguments.check_len_range(0, 1)?;
    let lines = context.arguments.optional_integer(0)?.unwrap_or(10);
    match context.input.recv()?.readable() {
        Some(mut input) => run(lines, input.as_mut(), context.output),
        None => error("Expected a stream"),
    }
}
