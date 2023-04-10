use std::sync::Arc;

use kabina_db::SharedDatabase;
use kabina_rpc::Kabina;
use parking_lot::Mutex;
use tarpc::context::Context;
use url::Url;

use crate::runtime::RuntimeManager;

pub const VERSION: u32 = const_random::const_random!(u32);

#[derive(Clone)]
pub struct KabinaState {
	pub database: SharedDatabase,
	pub rtm: Arc<Mutex<RuntimeManager>>,
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

		let mut rtm = self.state.rtm.lock();
		let _ = rtm.spawn(self.state.database.clone(), url);
	}
}
