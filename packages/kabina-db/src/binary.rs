use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Arc;

use serde::Serialize;

use crate::{Cause, Db, Executable, Outcome, RuntimeTask, Schema};

#[salsa::input]
#[derive(Debug, Clone)]
pub struct Binary {
	pub name: String,
	pub runtime: BinaryRuntime,
}

#[derive(Debug, Clone)]
pub enum BinaryRuntime {
	Native(BinaryNative),
}

#[derive(Debug, Clone)]
pub struct BinaryNative {
	pub executable: String,
	pub env: BTreeMap<String, String>,
	pub args: Vec<String>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub enum BinaryRuntimeResolved {
	Native {
		executable: PathBuf,
		env: BTreeMap<String, String>,
		args: Vec<String>,
	},
}

#[salsa::tracked]
pub fn binary_resolve(
	db: &dyn Db,
	schema: Schema,
	binary: Binary,
) -> Outcome<BinaryRuntimeResolved> {
	RuntimeTask::push(db, Arc::new(BinaryResolve { schema, binary }));
	Outcome::Err(Cause::Pending)
}

pub struct BinaryResolve {
	pub schema: Schema,
	pub binary: Binary,
}

impl Executable for BinaryResolve {}

impl BinaryResolve {
	pub fn resolve(&self, db: &mut dyn Db, object: Outcome<BinaryRuntimeResolved>) {
		binary_resolve::set(db, self.schema, self.binary, object)
	}
}
