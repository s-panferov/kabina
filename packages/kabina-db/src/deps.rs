use std::collections::BTreeMap;

use salsa::AsId;
use serde::Serialize;
use serde_json::{Map, Value};

use crate::{Binary, BinaryRuntimeResolved, FileGroup, Transform};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Input {
	FileGroup(FileGroup),
	Transform(Transform),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Dependency {
	FileGroup(FileGroup),
	Transform(Transform),
	Toolchain(Binary),
}

impl Dependency {
	pub fn to_input_kind(self) -> Option<Input> {
		match self {
			Dependency::Toolchain(_) => None,
			Dependency::FileGroup(f) => Some(Input::FileGroup(f)),
			Dependency::Transform(t) => Some(Input::Transform(t)),
		}
	}
}

#[derive(Serialize)]
pub enum ResolvedDependency {
	Binary(BinaryRuntimeResolved),
}

pub fn extract_dependencies_from_object(o: &Map<String, Value>, buffer: &mut Vec<Dependency>) {
	match (o.get("kind"), o.get("id")) {
		(Some(kind), Some(id)) if kind.as_str() == Some("FileGroup") => buffer.push(
			Dependency::FileGroup(FileGroup::from_id((id.as_u64().unwrap() as usize).into())),
		),
		(Some(kind), Some(id)) if kind.as_str() == Some("Transform") => buffer.push(
			Dependency::Transform(Transform::from_id((id.as_u64().unwrap() as usize).into())),
		),
		(Some(kind), Some(id)) if kind.as_str() == Some("Toolchain") => buffer.push(
			Dependency::Toolchain(Binary::from_id((id.as_u64().unwrap() as usize).into())),
		),
		_ => o.values().for_each(|v| extract_dependencies(v, buffer)),
	}
}

pub fn extract_dependencies(v: &Value, buffer: &mut Vec<Dependency>) {
	match v {
		Value::Object(o) => extract_dependencies_from_object(o, buffer),
		Value::Array(a) => a.iter().for_each(|o| extract_dependencies(o, buffer)),
		_ => {}
	}
}

fn to_object(value: Value) -> Map<String, Value> {
	match value {
		Value::Object(o) => o,
		_ => panic!(),
	}
}

pub fn replace_dependencies_in_object(
	o: &mut Map<String, Value>,
	subst: &BTreeMap<Dependency, ResolvedDependency>,
) -> Option<Value> {
	match (o.get("kind"), o.get("id")) {
		(Some(kind), Some(id)) if kind.as_str() == Some("FileGroup") => {
			let f = FileGroup::from_id((id.as_u64().unwrap() as usize).into());
			Some(serde_json::to_value(&subst.get(&Dependency::FileGroup(f)).unwrap()).unwrap())
		}
		(Some(kind), Some(id)) if kind.as_str() == Some("Transform") => {
			let t = Transform::from_id((id.as_u64().unwrap() as usize).into());
			Some(serde_json::to_value(&subst.get(&Dependency::Transform(t)).unwrap()).unwrap())
		}
		(Some(kind), Some(id)) if kind.as_str() == Some("Toolchain") => {
			let t = Binary::from_id((id.as_u64().unwrap() as usize).into());
			Some(serde_json::to_value(&subst.get(&Dependency::Toolchain(t)).unwrap()).unwrap())
		}
		_ => {
			o.values_mut().for_each(|v| replace_dependencies(v, subst));
			None
		}
	}
}

pub fn replace_dependencies(v: &mut Value, subst: &BTreeMap<Dependency, ResolvedDependency>) {
	match v {
		Value::Object(o) => {
			if let Some(new) = replace_dependencies_in_object(o, subst) {
				*v = new
			}
		}
		Value::Array(a) => a.iter_mut().for_each(|o| replace_dependencies(o, subst)),
		_ => {}
	}
}
