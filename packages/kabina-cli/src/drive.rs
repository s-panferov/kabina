use std::path::PathBuf;

use kabina_db::{
	BinaryResolve, BinaryRuntime, BinaryRuntimeResolved, Cause, Executable, ResolveRootFiles,
	RuntimeTask, SharedDatabase, TransformApply,
};
use tokio::sync::mpsc::Sender;

use crate::runtime::RuntimeMessage;

pub macro drive($rt:expr, $func:ident($db:expr, $($arg:expr),+)) {
    async { loop {
				tracing::info!("Resolving {}", std::stringify!($func));

        #[allow(unused_assignments)]
        let mut tasks = Vec::new();

				{
					let db_lock = $db.lock();
					match $func(&*db_lock, $($arg),+) {
						Ok(result) => {
							break result;
						}
						Err(Cause::Pending) => {
							tracing::info!("Pending {}", std::stringify!($func));
							tasks = $func::accumulated::<RuntimeTask>(
									&*db_lock, $($arg),+
							);
							tracing::info!("Pending {}", std::stringify!($func));
						}
						Err(e) => panic!("{:?}", e),
					}
				}

				tracing::info!("Resolving {}, tasks: {}", std::stringify!($func), tasks.len());

        for task in tasks {
					// TODO: parallel
					drive_task(&*task, &$db, &mut $rt).await
        }
    } }
}

pub async fn drive_task(
	task: &dyn Executable,
	db: &SharedDatabase,
	_rt: &mut Sender<RuntimeMessage>,
) {
	if let Some(task) = task.downcast_ref::<ResolveRootFiles>() {
		task.resolve(&mut db.lock())
	} else if let Some(_task) = task.downcast_ref::<TransformApply>() {
		unimplemented!()
	} else if let Some(task) = task.downcast_ref::<BinaryResolve>() {
		let runtime = task.binary.runtime(&*db.lock());
		let BinaryRuntime::Native(b) = runtime;
		let url = task.schema.url(&*db.lock());
		let path = PathBuf::from(url.path());
		let path = path.parent().unwrap();

		tracing::info!("Resolving binary {} in {}", b.executable, url.path());

		match which::WhichConfig::new()
			.binary_name(b.executable.clone().into())
			.custom_cwd(path.to_owned())
			.first_result()
		{
			Ok(path) => {
				tracing::info!(
					"[ToolchainResolve] Resolved {:?} to {:?}",
					b.executable,
					path
				);
				task.resolve(
					&mut *db.lock(),
					Ok(BinaryRuntimeResolved::Native {
						executable: path,
						args: b.args,
						env: b.env,
					}),
				)
			}
			Err(e) => task.resolve(&mut *db.lock(), Err(Cause::from_err(e))),
		}
	} else {
		unimplemented!()
	}
}
