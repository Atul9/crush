use crate::lang::errors::{argument_error, CrushResult};
use crate::lang::execution_context::ExecutionContext;
use crate::lang::stream::Readable;
use crate::lang::value::Value;

fn count_rows(mut s: Box<dyn Readable>) -> Value {
    let mut res: i128 = 0;
    while let Ok(_) = s.read() {
        res += 1;
    }
    Value::Integer(res)
}

pub fn perform(context: ExecutionContext) -> CrushResult<()> {
    match context.input.recv()? {
        Value::Table(r) => context.output.send(Value::Integer(r.rows().len() as i128)),
        Value::List(r) => context.output.send(Value::Integer(r.len() as i128)),
        Value::Dict(r) => context.output.send(Value::Integer(r.len() as i128)),
        v => match v.readable() {
            Some(readable) => context.output.send(count_rows(readable)),
            None => argument_error("Expected a stream"),
        },
    }
}
