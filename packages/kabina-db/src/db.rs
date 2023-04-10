use std::path::PathBuf;
use std::sync::Arc;

#[salsa::jar(db = Db)]
pub struct Jar(
	Project,
	crate::fileset::FileGroup,
	Diagnostic,
	Schema,
	crate::collection::Collection,
	crate::collection::collection_files,
	crate::server::Server,
	crate::transform::Transform,
	crate::transform::transform_inputs,
	crate::transform::transform_files,
	crate::transform::transform_result_for_file,
	crate::transform::transform_dependencies,
	crate::fileset::RuntimeTask,
	crate::fileset::File,
	crate::fileset::roots,
	crate::fileset::file_group_root,
	crate::fileset::file_group_matcher,
	crate::fileset::root_files,
	crate::fileset::root_file_groups,
	crate::fileset::file_group_resolved_paths,
	crate::fileset::file_group_files,
	crate::toolchain::Toolchain,
	// crate::toolchain::ToolchainObject,
	crate::toolchain::toolchain_resolve,
);

pub trait Db: salsa::DbWithJar<Jar> {}

use dashmap::DashMap;
use parking_lot::Mutex;
use rusqlite::Connection;
pub use salsa::debug::DebugWithDb;
pub use salsa::AsId;
use url::Url;

use crate::sqlite::sqlite_schema_add;
use crate::Schema;

#[salsa::db(Jar)]
pub struct Database {
	sqlite: Arc<Mutex<Connection>>,
	schemas: DashMap<Url, Schema>,
	storage: salsa::Storage<Self>,
}

impl Database {
	pub fn new() -> Self {
		let sqlite = crate::sqlite::sqlite_setup().unwrap();
		let storage = Default::default();
		Self {
			storage,
			schemas: DashMap::default(),
			sqlite: Arc::new(Mutex::new(sqlite)),
		}
	}

	pub fn schema_add(&self, url: Url, schema: Schema) -> Result<(), anyhow::Error> {
		let c = self.sqlite.lock();
		sqlite_schema_add(&c, &url)?;
		self.schemas.insert(url, schema);
		Ok(())
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
			sqlite: self.sqlite.clone(),
			storage: self.storage.snapshot(),
			schemas: DashMap::new(),
		})
	}
}

#[salsa::accumulator]
struct Diagnostic(String);

impl Diagnostic {}

pub type SharedDatabase = Arc<Mutex<Database>>;
