use std::sync::Arc;

use serde::Serialize;

use crate::{Cause, Db, Executable, Outcome, RuntimeTask};

#[salsa::input]
#[derive(Debug, Clone)]
pub struct Toolchain {
    name: String,
}
pub struct ToolchainResolve {
    pub toolchain: Toolchain,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub struct ToolchainObject {}

#[salsa::tracked]
pub fn toolchain_resolve(db: &dyn Db, toolchain: Toolchain) -> Outcome<ToolchainObject> {
    RuntimeTask::push(db, Arc::new(ToolchainResolve { toolchain }));
    Outcome::Err(Cause::Pending)
}

impl Executable for ToolchainResolve {}
