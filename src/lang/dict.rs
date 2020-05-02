use crate::lang::errors::{argument_error, error, CrushResult};
use crate::lang::stream::Readable;
use crate::lang::{table::ColumnType, table::Row, value::Value, value::ValueType};
use crate::util::identity_arc::Identity;
use crate::util::replace::Replace;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::hash::Hasher;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct Dict {
    key_type: ValueType,
    value_type: ValueType,
    entries: Arc<Mutex<HashMap<Value, Value>>>,
}

impl Identity for Dict {
    fn id(&self) -> u64 {
        self.entries.id()
    }
}

impl Dict {
    pub fn new(key_type: ValueType, value_type: ValueType) -> Dict {
        if !key_type.is_hashable() {
            panic!("Tried to create dict with unhashable key type");
        }
        Dict {
            key_type,
            value_type,
            entries: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn to_string(&self) -> String {
        let mut res = "dict{".to_string();
        let entries = self.entries.lock().unwrap();
        res += &entries
            .iter()
            .map(|(k, v)| format!("{}: {}", k.to_string(), v.to_string()))
            .collect::<Vec<String>>()
            .join(" ");
        res += "}";
        res
    }

    pub fn len(&self) -> usize {
        let entries = self.entries.lock().unwrap();
        entries.len()
    }

    pub fn clear(&self) {
        let mut entries = self.entries.lock().unwrap();
        entries.clear()
    }

    pub fn copy(&self) -> Dict {
        let entries = self.entries.lock().unwrap();
        Dict {
            key_type: self.key_type.clone(),
            value_type: self.value_type.clone(),
            entries: Arc::new(Mutex::new(entries.clone())),
        }
    }

    pub fn get(&self, key: &Value) -> Option<Value> {
        let entries = self.entries.lock().unwrap();
        entries.get(key).map(|c| c.clone())
    }

    pub fn remove(&self, key: &Value) -> Option<Value> {
        let mut entries = self.entries.lock().unwrap();
        entries.remove(key)
    }

    pub fn insert(&self, key: Value, value: Value) -> CrushResult<()> {
        let mut entries = self.entries.lock().unwrap();
        if !self.key_type.is(&key) {
            return argument_error("Invalid key type");
        }
        if !self.value_type.is(&value) {
            return argument_error("Invalid value type");
        }
        entries.insert(key, value);
        Ok(())
    }

    pub fn key_type(&self) -> ValueType {
        self.key_type.clone()
    }

    pub fn value_type(&self) -> ValueType {
        self.value_type.clone()
    }
    pub fn dict_type(&self) -> ValueType {
        ValueType::Dict(
            Box::from(self.key_type.clone()),
            Box::from(self.value_type.clone()),
        )
    }

    pub fn elements(&self) -> Vec<(Value, Value)> {
        let entries = self.entries.lock().unwrap();
        entries
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }

    pub fn materialize(self) -> Dict {
        let mut entries = self.entries.lock().unwrap();
        let map = entries
            .drain()
            .map(|(k, v)| (k.materialize(), v.materialize()))
            .collect();
        Dict {
            key_type: self.key_type.materialize(),
            value_type: self.value_type.materialize(),
            entries: Arc::new(Mutex::new(map)),
        }
    }
}

impl std::hash::Hash for Dict {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let entries = self.entries.lock().unwrap().clone();
        for (k, v) in entries.iter() {
            k.hash(state);
            v.hash(state);
        }
    }
}

impl std::cmp::PartialEq for Dict {
    fn eq(&self, other: &Dict) -> bool {
        let us = self.entries.lock().unwrap().clone();
        let them = other.entries.lock().unwrap().clone();
        if us.len() != them.len() {
            return false;
        }
        for (k, v) in us.iter() {
            let them_value = them.get(k);
            match them_value {
                None => return false,
                Some(v2) => {
                    if !v.eq(v2) {
                        return false;
                    }
                }
            }
        }
        true
    }
}

impl std::cmp::PartialOrd for Dict {
    fn partial_cmp(&self, _other: &Dict) -> Option<Ordering> {
        None
    }
}

pub struct DictReader {
    list: Vec<(Value, Value)>,
    idx: usize,
    types: Vec<ColumnType>,
}

impl DictReader {
    pub fn new(dict: Dict) -> DictReader {
        DictReader {
            types: vec![
                ColumnType::new("key", dict.key_type.clone()),
                ColumnType::new("value", dict.value_type.clone()),
            ],
            list: dict.elements(),
            idx: 0usize,
        }
    }
}

impl Readable for DictReader {
    fn read(&mut self) -> CrushResult<Row> {
        if self.idx >= self.list.len() {
            return error("End of stream");
        }
        let (a, b) = self
            .list
            .replace(self.idx, (Value::Bool(false), Value::Bool(false)));
        self.idx += 1;
        Ok(Row::new(vec![a, b]))
    }

    fn types(&self) -> &[ColumnType] {
        &self.types
    }
}
