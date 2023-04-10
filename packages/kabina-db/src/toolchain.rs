use std::path::PathBuf;
use std::sync::Arc;

use serde::Serialize;

use crate::{Cause, Db, Executable, Outcome, RuntimeTask};

#[salsa::input]
#[derive(Debug, Clone)]
pub struct Toolchain {
	pub name: String,
	pub binary: String,
	pub runner: String,
}
pub struct ToolchainResolve {
	pub toolchain: Toolchain,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub struct ToolchainObject {
	pub binary: PathBuf,
}

#[salsa::tracked]
pub fn toolchain_resolve(db: &dyn Db, toolchain: Toolchain) -> Outcome<ToolchainObject> {
	RuntimeTask::push(db, Arc::new(ToolchainResolve { toolchain }));
	Outcome::Err(Cause::Pending)
}

impl Executable for ToolchainResolve {}

impl ToolchainResolve {
	pub fn resolve(&self, db: &mut dyn Db, object: Outcome<ToolchainObject>) {
		toolchain_resolve::set(db, self.toolchain, object)
	}
}
