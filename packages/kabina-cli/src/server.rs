use std::sync::Arc;

use kabina_db::{AsId, ServiceRuntime, SharedDatabase};
use kabina_rpc::Kabina;
use parking_lot::Mutex;
use tarpc::context::Context;
use tokio::sync::oneshot;
use url::Url;

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

		let channel = {
			let mut rtm = self.state.rtm.lock();
			rtm.spawn(self.state.database.clone(), url)
		};

		let schema = {
			let (tx, rx) = oneshot::channel();
			channel.send(RuntimeMessage::Schema(tx)).await.unwrap();
			rx.await.unwrap()
		};

		let db = self.state.database.lock();
		let services = schema.services(&*db);

		let mut proc = self.state.process.lock();
		for service in services.iter() {
			let runtime = service.runtime(&*db);
			let ServiceRuntime::Binary(b) = runtime;
			proc.spawn(
				service.as_id().into(),
				ProcessConfig {
					executable: b.executable,
				},
			);
		}
	}
}
