use std::collections::HashMap;
use std::thread::JoinHandle;

use kabina_db::runtime::Runtime;
use kabina_db::{Schema, SharedDatabase};
use kabina_rt::DenoRuntime;
use tokio::sync::mpsc::{channel, Sender};
use tokio::sync::oneshot;
use url::Url;

pub struct RuntimeChannel {
	handle: JoinHandle<()>,
	sender: Sender<RuntimeMessage>,
}

#[derive(Debug)]
pub enum RuntimeMessage {
	Schema(oneshot::Sender<Schema>),
}

#[derive(Default)]
pub struct RuntimeManager {
	runtimes: HashMap<Url, RuntimeChannel>,
}

impl RuntimeManager {
	pub fn spawn(&mut self, db: SharedDatabase, url: Url) -> Sender<RuntimeMessage> {
		if let Some(cx) = self.runtimes.get(&url) {
			return cx.sender.clone();
		}

		let (sender, mut rx) = channel::<RuntimeMessage>(10);

		let handle = std::thread::spawn({
			let url = url.clone();
			move || {
				let tokio_rt = tokio::runtime::Builder::new_current_thread()
					.enable_all()
					.build()
					.unwrap();

				let mut deno_rt = tokio_rt.block_on(DenoRuntime::new(db));
				let schema = tokio_rt.block_on(deno_rt.load_schema(url.clone()));

				while let Some(msg) = tokio_rt.block_on(rx.recv()) {
					match msg {
						RuntimeMessage::Schema(rx) => {
							let _ = rx.send(schema);
						}
					}
				}
			}
		});

		let monitor = std::thread::spawn(move || match handle.join() {
			Ok(_) => {
				tracing::info!("Runtime thread completed");
			}
			Err(_) => {
				tracing::error!("Runtime thread failed");
				std::process::abort();
			}
		});

		self.runtimes.insert(
			url,
			RuntimeChannel {
				handle: monitor,
				sender: sender.clone(),
			},
		);

		sender
	}
}
