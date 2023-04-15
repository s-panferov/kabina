use std::collections::BTreeMap;
use std::sync::Arc;

use serde_json::Value;

use crate::deps::{
	extract_dependencies, replace_dependencies, Dependency, Input, ResolvedDependency,
};
use crate::{
	binary_resolve, file_group_files, Cause, Db, Executable, File, Outcome, RuntimeTask, Schema,
};

#[derive(Debug, Clone)]
pub enum RunnerKind {
	JsFunction(u64),
}

#[salsa::input]
#[derive(Debug, Clone)]
pub struct Transform {
	name: String,
	runner: RunnerKind,
	input: Value,
	dependencies: Value,
}

#[salsa::tracked]
pub fn transform_inputs(db: &dyn Db, transform: Transform) -> Vec<Input> {
	let input = transform.input(db);
	let mut buffer: Vec<Dependency> = Vec::new();
	extract_dependencies(&input, &mut buffer);
	buffer
		.into_iter()
		.filter_map(Dependency::to_input_kind)
		.collect()
}

#[salsa::tracked]
pub fn transform_dependencies(db: &dyn Db, transform: Transform) -> Outcome<Arc<Value>> {
	let mut deps = transform.dependencies(db);
	let mut buffer: Vec<Dependency> = Vec::new();
	extract_dependencies(&deps, &mut buffer);

	let mut error: Result<(), Cause> = Ok(());
	let mut resolved: BTreeMap<Dependency, ResolvedDependency> = Default::default();

	for dep in &buffer {
		match dep {
			Dependency::Toolchain(t) => match binary_resolve(db, *t) {
				Ok(t) => {
					resolved.insert(*dep, ResolvedDependency::Binary(t));
				}
				Err(c) => error = Err(c),
			},
			Dependency::FileGroup(_) => todo!(),
			Dependency::Transform(_) => todo!(),
		}
	}

	error?;

	replace_dependencies(&mut deps, &mut resolved);
	Ok(Arc::new(deps))
}

#[salsa::tracked]
pub fn transform_files(db: &dyn Db, schema: Schema, transform: Transform) -> Outcome<Vec<File>> {
	let inputs = transform_inputs(db, transform);

	let mut pending = false;
	let mut buffer = Vec::new();

	for input in inputs {
		match input {
			Input::FileGroup(g) => match file_group_files(db, schema, g) {
				Ok(files) => {
					for file in files {
						match transform_result_for_file(db, schema, transform, file) {
							Ok(file) => buffer.push(file),
							Err(Cause::Pending) => pending = true,
							Err(e) => return Err(e),
						}
					}
				}
				Err(Cause::Pending) => pending = true,
				Err(e) => return Err(e),
			},
			Input::Transform(t) => match transform_files(db, schema, t) {
				Ok(files) => {
					for file in files {
						match transform_result_for_file(db, schema, transform, file) {
							Ok(file) => buffer.push(file),
							Err(Cause::Pending) => pending = true,
							Err(e) => return Err(e),
						}
					}
				}
				Err(Cause::Pending) => pending = true,
				Err(e) => return Err(e),
			},
		}
	}

	if pending {
		return Err(Cause::Pending);
	}

	return Ok(buffer);
}

#[salsa::tracked]
pub fn transform_result_for_file(
	db: &dyn Db,
	schema: Schema,
	transform: Transform,
	file: File,
) -> Outcome<File> {
	let dependencies = transform_dependencies(db, transform)?;

	RuntimeTask::push(
		db,
		Arc::new(TransformApply {
			schema,
			file,
			transform,
			dependencies,
		}),
	);
	Outcome::Err(Cause::Pending)
}

pub struct TransformApply {
	pub schema: Schema,
	pub file: File,
	pub transform: Transform,
	pub dependencies: Arc<Value>,
}

impl Executable for TransformApply {}
