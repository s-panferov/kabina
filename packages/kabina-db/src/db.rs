use std::path::PathBuf;

#[salsa::jar(db = Db)]
pub struct Jar(
    Project,
    File,
    FileGroup,
    Diagnostic,
    Schema,
    fileset::roots,
    fileset::file_group_root,
    fileset::file_group_matcher,
    fileset::root_files,
    fileset::root_file_groups,
    fileset::file_group_files,
);

#[salsa::input]
pub struct File {
    path: PathBuf,
}

pub trait Db: salsa::DbWithJar<Jar> {}

pub use salsa::AsId;

use crate::{
    fileset::{self, FileGroup},
    Schema,
};

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
