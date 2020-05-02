use crate::lang::dict::Dict;
use crate::lang::errors::{to_crush_error, CrushResult};
use crate::lang::list::List;
use crate::lang::r#struct::Struct;
use crate::lang::scope::Scope;
use crate::lang::value::{Value, ValueType};
use model::Element;
use model::SerializedValue;
use prost::Message;
use std::collections::HashMap;
use std::fs::File;
use std::io::{Cursor, Read, Write};
use std::path::Path;

mod dict_serializer;
mod integer_serializer;
mod list_serializer;
mod scope_serializer;
mod string_serializer;
mod struct_serializer;
mod table_serializer;
mod value_serializer;
mod value_type_serializer;

//pub mod model;
pub mod model {
    include!(concat!(env!("OUT_DIR"), "/model.rs"));
}

pub struct SerializationState {
    pub with_id: HashMap<u64, usize>,
    pub values: HashMap<Value, usize>,
}

pub struct DeserializationState {
    pub env: Scope,
    pub values: HashMap<usize, Value>,
    pub lists: HashMap<usize, List>,
    pub types: HashMap<usize, ValueType>,
    pub dicts: HashMap<usize, Dict>,
    pub structs: HashMap<usize, Struct>,
    pub scopes: HashMap<usize, Scope>,
}

pub fn serialize(value: &Value, destination: &Path) -> CrushResult<()> {
    let mut res = SerializedValue::default();
    let mut state = SerializationState {
        with_id: HashMap::new(),
        values: HashMap::new(),
    };
    res.root = value
        .clone()
        .materialize()
        .serialize(&mut res.elements, &mut state)? as u64;

    let mut buf = Vec::new();
    buf.reserve(res.encoded_len());
    res.encode(&mut buf).unwrap();

    let mut file_buffer = to_crush_error(File::create(destination))?;
    let mut pos = 0;

    while pos < buf.len() {
        let bytes_written = to_crush_error(file_buffer.write(&buf[pos..]))?;
        pos += bytes_written;
    }
    Ok(())
}

pub fn deserialize(source: &Path, env: &Scope) -> CrushResult<Value> {
    let mut buf = Vec::new();
    let mut file_buffer = to_crush_error(File::open(source))?;
    buf.reserve(to_crush_error(source.metadata())?.len() as usize);
    to_crush_error(file_buffer.read_to_end(&mut buf))?;

    let mut state = DeserializationState {
        values: HashMap::new(),
        types: HashMap::new(),
        lists: HashMap::new(),
        dicts: HashMap::new(),
        structs: HashMap::new(),
        scopes: HashMap::new(),
        env: env.clone(),
    };

    let res = SerializedValue::decode(&mut Cursor::new(buf)).unwrap();

    //    println!("AAA {:?}", res);

    Ok(Value::deserialize(
        res.root as usize,
        &res.elements,
        &mut state,
    )?)
}

pub trait Serializable<T> {
    fn deserialize(
        id: usize,
        elements: &[Element],
        state: &mut DeserializationState,
    ) -> CrushResult<T>;
    fn serialize(
        &self,
        elements: &mut Vec<Element>,
        state: &mut SerializationState,
    ) -> CrushResult<usize>;
}
