use salsa::AsId;
use serde_json::{value::Map, Value};

use crate::{file_group_files, Db, File, FileGroup, Schema};

#[derive(Debug, Clone)]
pub enum RunnerKind {
    JsFunction(u64),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DependencyKind {
    FileGroup(FileGroup),
    Transform(Transform),
}

#[salsa::input]
#[derive(Debug, Clone)]
pub struct Transform {
    name: String,
    runner: RunnerKind,
    input: Value,
    dependencies: Value,
}

pub fn walk_object(o: &Map<String, Value>, buffer: &mut Vec<DependencyKind>) {
    match (o.get("kind"), o.get("id")) {
        (Some(kind), Some(id)) if kind.as_str() == Some("FileGroup") => buffer.push(
            DependencyKind::FileGroup(FileGroup::from_id((id.as_u64().unwrap() as usize).into())),
        ),
        (Some(kind), Some(id)) if kind.as_str() == Some("Transform") => buffer.push(
            DependencyKind::Transform(Transform::from_id((id.as_u64().unwrap() as usize).into())),
        ),
        _ => o.values().for_each(|v| walk_value(v, buffer)),
    }
}

pub fn walk_value(v: &Value, buffer: &mut Vec<DependencyKind>) {
    match v {
        Value::Object(o) => walk_object(o, buffer),
        Value::Array(a) => a.iter().for_each(|o| walk_value(o, buffer)),
        _ => {}
    }
}

#[salsa::tracked]
pub fn transform_inputs(db: &dyn Db, transform: Transform) -> Vec<DependencyKind> {
    let input = transform.input(db);
    let mut buffer: Vec<DependencyKind> = Vec::new();
    walk_value(&input, &mut buffer);
    buffer
}

#[derive(Clone)]
pub struct TransformJob {}

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct TransformState {
    missing: Vec<()>,
    ready: Vec<()>,
}

#[salsa::tracked]
pub fn transform_output(db: &dyn Db, schema: Schema, transform: Transform) -> TransformState {
    let inputs = transform_inputs(db, transform);

    for input in inputs {
        match input {
            DependencyKind::FileGroup(g) => {
                let files = file_group_files(db, schema, g);
            }
            DependencyKind::Transform(t) => {
                panic!("Not supported")
            }
        }
    }

    TransformState::default()
}

#[salsa::tracked]
pub fn transform_result_for_file(db: &dyn Db, transform: Transform, file: File) -> Option<()> {
    None
}
