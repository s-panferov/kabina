use std::path::PathBuf;

#[salsa::jar(db = Db)]
pub struct Jar(
    Project,
    crate::fileset::FileGroup,
    Diagnostic,
    Schema,
    crate::bundle::Bundle,
    crate::server::Server,
    crate::transform::Transform,
    crate::transform::transform_inputs,
    crate::transform::transform_output,
    crate::transform::transform_result_for_file,
    crate::fileset::RuntimeTask,
    crate::fileset::File,
    crate::fileset::roots,
    crate::fileset::file_group_root,
    crate::fileset::file_group_matcher,
    crate::fileset::root_files,
    crate::fileset::root_file_groups,
    crate::fileset::file_group_resolved_paths,
    crate::fileset::file_group_files,
);

pub trait Db: salsa::DbWithJar<Jar> {}

pub use salsa::debug::DebugWithDb;
pub use salsa::AsId;

use crate::Schema;

#[salsa::db(Jar)]
pub struct Database {
    storage: salsa::Storage<Self>,
}

impl Database {
    pub fn new() -> Self {
        let storage = Default::default();
        Self { storage }
    }
}

#[salsa::input]
pub struct Project {
    root: PathBuf,
}

impl Db for Database {}
// ANCHOR_END: db

impl salsa::Database for Database {}

impl salsa::ParallelDatabase for Database {
    fn snapshot(&self) -> salsa::Snapshot<Self> {
        salsa::Snapshot::new(Database {
            storage: self.storage.snapshot(),
        })
    }
}

#[salsa::accumulator]
struct Diagnostic(String);

impl Diagnostic {}
