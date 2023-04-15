use std::sync::Arc;

use kabina_db::{binary_resolve, AsId, BinaryRuntimeResolved, SharedDatabase};
use kabina_rpc::Kabina;
use parking_lot::Mutex;
use tarpc::context::Context;
use tokio::sync::oneshot;
use url::Url;

use crate::drive::drive;
use crate::process::{ProcessConfig, ProcessMananger};
use crate::runtime::{RuntimeManager, RuntimeMessage};

pub const VERSION: u32 = const_random::const_random!(u32);

#[derive(Clone)]
pub struct KabinaState {
	pub database: SharedDatabase,
	pub rtm: Arc<Mutex<RuntimeManager>>,
	pub process: Arc<Mutex<ProcessMananger>>,
}

#[derive(Clone)]
pub struct KabinaServer {
	pub state: KabinaState,
}

#[tarpc::server]
impl Kabina for KabinaServer {
	async fn hello(self, _: Context, name: String) -> String {
		format!("Hello, {name}! You are connected")
	}

	async fn version(self, _: Context) -> String {
		VERSION.to_string()
	}

	async fn terminate(self, _: Context) {
		let _ = std::fs::remove_file("/tmp/kabina.sock");
		std::process::exit(0)
	}

	async fn schema_run(self, _: Context, url: Url) {
		tracing::info!("[Method] Kabina::schema_run");

		let mut channel = {
			let mut rtm = self.state.rtm.lock();
			rtm.spawn(self.state.database.clone(), url)
		};

		let schema = {
			let (tx, rx) = oneshot::channel();
			channel.send(RuntimeMessage::Schema(tx)).await.unwrap();
			rx.await.unwrap()
		};

		let db = &self.state.database;
		let services = schema.services(&*db.lock()).clone();

		for service in services.iter() {
			tracing::info!("Running service: {}", service.name(&*db.lock()));

			let binary = service.binary(&*db.lock());

			tracing::info!("Resolving binary");

			{
				assert!(db.try_lock().is_some());
			}

			let binary = drive!(channel, binary_resolve(self.state.database, binary)).await;

			match binary {
				BinaryRuntimeResolved::Native { executable } => {
					tracing::info!("Spawning executable: {:?}", executable);
					self.state.process.lock().spawn(
						service.as_id().into(),
						ProcessConfig {
							executable: executable,
						},
					);
				}
			}
		}
	}
}
