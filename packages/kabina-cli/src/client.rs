use kabina_rpc::KabinaObserver;
use tarpc::context::Context;

#[derive(Clone)]
pub struct KabinaObserverImpl {}

#[tarpc::server]
impl KabinaObserver for KabinaObserverImpl {
	async fn log(self, _: Context, name: String) {
		tracing::info!("Hello, {name}! You are connected")
	}
}
