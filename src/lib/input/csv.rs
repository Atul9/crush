use crate::lang::execution_context::ExecutionContext;
use crate::{
    lang::errors::{argument_error, CrushError},
    lang::{table::Row, value::Value},
};
use std::{io::prelude::*, io::BufReader};

use crate::lang::errors::{error, to_crush_error, CrushResult};
use crate::lang::{binary::BinaryReader, table::ColumnType};

use crate::lang::argument::ArgumentHandler;
use crate::lang::ordered_string_map::OrderedStringMap;
use crate::lang::value::ValueType;
use signature::signature;
use std::path::PathBuf;

#[signature]
#[derive(Debug)]
struct Signature {
    #[unnamed()]
    files: Vec<PathBuf>,
    #[named()]
    columns: OrderedStringMap<ValueType>,
    #[default(',')]
    separator: char,
    #[default(0)]
    head: i128,
    trim: Option<char>,
}

pub fn csv(context: ExecutionContext) -> CrushResult<()> {
    let cfg: Signature = Signature::parse(context.arguments, &context.printer)?;
    let columns = cfg
        .columns
        .iter()
        .map(|(k, v)| ColumnType::new(k, v.clone()))
        .collect::<Vec<_>>();
    let output = context.output.initialize(columns.clone())?;

    let mut reader = BufReader::new(match cfg.files.len() {
        0 => match context.input.recv()? {
            Value::BinaryStream(b) => Ok(b),
            Value::Binary(b) => Ok(BinaryReader::vec(&b)),
            _ => argument_error("Expected either a file to read or binary pipe input"),
        },
        _ => BinaryReader::paths(cfg.files),
    }?);

    let separator = cfg.separator;
    let trim = cfg.trim;
    let skip = cfg.head as usize;

    let mut line = String::new();
    let mut skipped = 0usize;
    loop {
        line.clear();
        to_crush_error(reader.read_line(&mut line))?;
        if line.is_empty() {
            break;
        }
        if skipped < skip {
            skipped += 1;
            continue;
        }
        let line_without_newline = &line[0..line.len() - 1];
        let mut split: Vec<&str> = line_without_newline
            .split(separator)
            .map(|s| trim.map(|c| s.trim_matches(c)).unwrap_or(s))
            .collect();

        if split.len() != columns.len() {
            return error("csv: Wrong number of columns in CSV file");
        }

        if let Some(trim) = trim {
            split = split.iter().map(|s| s.trim_matches(trim)).collect();
        }

        match split
            .iter()
            .zip(columns.iter())
            .map({ |(s, t)| t.cell_type.parse(*s) })
            .collect::<Result<Vec<Value>, CrushError>>()
        {
            Ok(cells) => {
                let _ = output.send(Row::new(cells));
            }
            Err(err) => {
                return Err(err);
            }
        }
    }
    Ok(())
}
