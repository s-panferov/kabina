use std::fs::File;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use futures::{future, StreamExt};
use kabina_db::Database;
use kabina_rpc::{Kabina, KabinaClient};
use parking_lot::Mutex;
use tarpc::context::current;
use tarpc::server::Channel;
use tarpc::tokio_serde::formats::Bincode;
use tokio::runtime::Runtime;
use tokio::time::sleep;

use crate::process::ProcessMananger;
use crate::runtime::RuntimeManager;
use crate::server::{KabinaServer, KabinaState, VERSION};

pub fn tokio_current() -> Runtime {
	tokio::runtime::Builder::new_current_thread()
		.enable_all()
		.build()
		.unwrap()
}

pub fn tokio_multi() -> Runtime {
	tokio::runtime::Builder::new_multi_thread()
		.enable_all()
		.build()
		.unwrap()
}

pub fn daemon_start() -> Result<(), anyhow::Error> {
	let stdout = File::create("/tmp/kabina.out")?;
	let stderr = File::create("/tmp/kabina.err")?;

	let daemon = daemonize::Daemonize::new()
		.pid_file(PathBuf::from("/tmp/kabina.pid"))
		.stdout(stdout)
		.stderr(stderr);

	match daemon.execute() {
		daemonize::Outcome::Parent(r) => match r {
			Ok(_p) => {
				let rt = tokio_current();
				let restart = rt.block_on(async move {
					// FIXME: wait until socket is created
					sleep(Duration::from_millis(300)).await;

					let client = daemon_client().await.unwrap();
					let version = client.version(current()).await.unwrap();

					if version != VERSION.to_string() {
						tracing::info!("Daemon version does not match, restarting...");
						let _ = client.terminate(current()).await;
						return true;
					} else {
						return false;
					}
				});

				if restart {
					std::mem::drop(rt);
					return daemon_start();
				}

				Ok(())
			}
			Err(e) => panic!("Failed to start the daemon {}", e),
		},
		daemonize::Outcome::Child(r) => match r {
			Ok(_) => {
				tracing::info!("Starting server");
				let rt = tokio_multi();
				rt.block_on(daemon_server())
			}
			Err(e) => panic!("Failed to start the daemon: {}", e),
		},
	}
}

pub fn daemon_restart() -> Result<(), anyhow::Error> {
	daemon_stop()?;
	daemon_start()
}

pub fn daemon_stop() -> Result<(), anyhow::Error> {
	let rt = tokio_current();
	rt.block_on(async {
		match daemon_client().await {
			Ok(client) => match client.terminate(current()).await {
				Ok(_) => unreachable!(),
				Err(_) => return Ok(()),
			},
			Err(_) => return Ok(()),
		}
	})
}

pub async fn daemon_server() -> Result<(), anyhow::Error> {
	let _ = std::fs::remove_file("/tmp/kabina.sock");

	let mut listener =
		// tarpc::serde_transport::unix::listen("/tmp/kabina.sock", &Bincode::default).await?;
		tarpc::serde_transport::tcp::listen("0.0.0.0:34123", &Bincode::default).await?;

	let db = Arc::new(Mutex::new(Database::new()));
	let rtm = Arc::new(Mutex::new(RuntimeManager::default()));
	let proc = Arc::new(Mutex::new(ProcessMananger::default()));

	let state = KabinaState {
		database: db.clone(),
		process: proc,
		rtm,
	};

	listener.config_mut().max_frame_length(usize::MAX);
	listener
		// Ignore accept errors.
		.filter_map(|r| future::ready(r.ok()))
		.map(tarpc::server::BaseChannel::with_defaults)
		// Limit channels to 1 per IP.
		// .max_channels_per_key(1, |t| t.transport().peer_addr().unwrap().as_pathname())
		// serve is generated by the service attribute. It takes as input any type implementing
		// the generated World trait.
		.map(|channel| {
			let server = KabinaServer {
				state: state.clone(),
			};
			tracing::info!("New connection");
			channel.execute(server.serve())
		})
		// Max 10 channels.
		.buffer_unordered(10)
		.for_each(|_| async {})
		.await;

	Ok(())
}

pub async fn daemon_client() -> Result<KabinaClient, anyhow::Error> {
	// let mut transport = tarpc::serde_transport::unix::connect("/tmp/kabina.sock", Bincode::default);
	let mut transport = tarpc::serde_transport::tcp::connect("0.0.0.0:34123", Bincode::default);
	transport.config_mut().max_frame_length(usize::MAX);

	let client = KabinaClient::new(tarpc::client::Config::default(), transport.await?).spawn();

	// let reponse = client.hello(current(), "CLIENT".into()).await;
	// match reponse {
	// 	Ok(hello) => tracing::info!("{hello:?}"),
	// 	Err(e) => tracing::warn!("{:?}", anyhow::Error::from(e)),
	// }

	Ok(client)
}
