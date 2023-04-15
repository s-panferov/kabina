use std::path::PathBuf;
use std::sync::Arc;

use serde::Serialize;

use crate::{Cause, Db, Executable, Outcome, RuntimeTask};

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
}

pub struct BinaryResolve {
	pub binary: Binary,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub enum BinaryRuntimeResolved {
	Native { executable: PathBuf },
}

#[salsa::tracked]
pub fn binary_resolve(db: &dyn Db, binary: Binary) -> Outcome<BinaryRuntimeResolved> {
	RuntimeTask::push(db, Arc::new(BinaryResolve { binary }));
	Outcome::Err(Cause::Pending)
}

impl Executable for BinaryResolve {}

impl BinaryResolve {
	pub fn resolve(&self, db: &mut dyn Db, object: Outcome<BinaryRuntimeResolved>) {
		binary_resolve::set(db, self.binary, object)
	}
}
