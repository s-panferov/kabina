#![feature(decl_macro)]

use std::path::PathBuf;
use std::sync::Arc;

use clap::Parser;
use daemon::{daemon_client, daemon_start, tokio_current};
use kabina_db::runtime::Runtime;
use kabina_db::{
	collection_files, Cause, Executable, ResolveRootFiles, RuntimeTask, SharedDatabase,
	ToolchainObject, ToolchainResolve, TransformApply,
};
use kabina_rt::DenoRuntime;
use parking_lot::Mutex;
use tarpc::context::current;
use url::Url;

mod daemon;
mod process;
mod runtime;
mod server;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
enum Command {
	Build {
		#[arg(long)]
		schema: PathBuf,
		#[arg(long)]
		collection: String,
	},
	Run {
		#[arg(index = 1)]
		schema: String,
	},
	#[clap(subcommand)]
	Daemon(Daemon),
}

#[derive(clap::Parser, Debug)]
enum Daemon {
	Start {},
	Stop {},
	Restart {},
}

fn main() -> Result<(), anyhow::Error> {
	let args = Command::parse();

	tracing_subscriber::fmt::init();

	match args {
		Command::Build {
			mut schema,
			collection: bundle,
		} => {
			let rt = tokio::runtime::Builder::new_multi_thread()
				.enable_all()
				.build()?;

			if schema.is_relative() {
				schema = std::env::current_dir()?.join(schema)
			}

			let db = Arc::new(Mutex::new(kabina_db::Database::new()));

			let mut deno_rt = Box::new(rt.block_on(DenoRuntime::new(db.clone())));

			let file_url = Url::from_file_path(schema).unwrap();

			// Populating the schema from TS
			let schema = rt.block_on(deno_rt.load_schema(file_url));

			let collection = {
				let db = db.lock();
				let collections = schema.collections(&*db);
				let collection = collections
					.iter()
					.find(|b| (*b).name(&*db) == bundle)
					.unwrap()
					.clone();

				collection
			};

			rt.block_on(drive!(deno_rt, collection_files(db, schema, collection)));

			Ok(())
		}
		Command::Run { schema } => {
			daemon_start()?;
			let rt = tokio_current();
			rt.block_on(async {
				let client = daemon_client().await?;
				let url = url::Url::parse(&schema).unwrap_or_else(|_| {
					let mut path = PathBuf::from(schema);
					if !path.is_absolute() {
						path = path.canonicalize().unwrap()
					}

					url::Url::from_file_path(path).unwrap()
				});

				client.schema_run(current(), url).await?;
				Ok(())
			})
		}
		Command::Daemon(daemon) => match daemon {
			Daemon::Start {} => daemon::daemon_start(),
			Daemon::Stop {} => daemon::daemon_stop(),
			Daemon::Restart {} => daemon::daemon_restart(),
		},
	}
}

pub macro drive($rt:expr, $func:ident($db:expr, $($arg:expr),+)) {
    async { loop {
        #[allow(unused_assignments)]
        let mut tasks = Vec::new();
        match $func(&*$db.lock(), $($arg),+) {
            Ok(result) => {
                break result;
            }
            Err(Cause::Pending) => {
                tasks = $func::accumulated::<RuntimeTask>(
                    &*$db.lock(), $($arg),+
                );
            }
            Err(e) => panic!("{:?}", e),
        }

        for task in tasks {
            // TODO: parallel
            drive_task(&*task, &$db, &mut $rt).await
        }
    } }
}

pub async fn drive_task(task: &dyn Executable, db: &SharedDatabase, rt: &mut Box<impl Runtime>) {
	if let Some(task) = task.downcast_ref::<ResolveRootFiles>() {
		task.resolve(&mut db.lock())
	} else if let Some(task) = task.downcast_ref::<TransformApply>() {
		rt.transform(task).await;
	} else if let Some(task) = task.downcast_ref::<ToolchainResolve>() {
		let binary = task.toolchain.binary(&*db.lock());
		match which::which(&binary) {
			Ok(path) => {
				tracing::info!("[ToolchainResolve] Resolved {:?} to {:?}", binary, path);
				task.resolve(&mut *db.lock(), Ok(ToolchainObject { binary: path }))
			}
			Err(e) => task.resolve(&mut *db.lock(), Err(Cause::from_err(e))),
		}
	}
}
