use std::{path::PathBuf, process, time::Duration};

use futures::{future, StreamExt};
use kabina_rpc::{World, WorldClient};
use tarpc::{
    context::{current, Context},
    server::Channel,
    tokio_serde::formats::Bincode,
};

use const_random::const_random;
use tokio::time::sleep;

const VERSION: u32 = const_random!(u32);

pub fn daemon_start() {
    // let stdout = File::create("/tmp/kabina.out").unwrap();
    // let stderr = File::create("/tmp/kabina.err").unwrap();

    let daemon = daemonize::Daemonize::new()
        .pid_file(PathBuf::from("/tmp/kabina.pid"))
        .stdout(daemonize::Stdio::keep())
        .stderr(daemonize::Stdio::keep());

    match daemon.execute() {
        daemonize::Outcome::Parent(r) => match r {
            Ok(p) => {
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .unwrap();

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
                    daemon_start()
                }
            }
            Err(e) => panic!("Failed to start the daemon {}", e),
        },
        daemonize::Outcome::Child(r) => match r {
            Ok(_) => {
                tracing::info!("Starting server");
                let rt = tokio::runtime::Builder::new_multi_thread()
                    .enable_all()
                    .build()
                    .unwrap();

                rt.block_on(daemon_server()).unwrap();
            }
            Err(e) => panic!("Failed to start the daemon: {}", e),
        },
    }
}

pub async fn daemon_server() -> Result<(), anyhow::Error> {
    let _ = std::fs::remove_file("/tmp/kabina.sock");

    let mut listener =
        tarpc::serde_transport::unix::listen("/tmp/kabina.sock", &Bincode::default).await?;

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
            let server = HelloServer();
            channel.execute(server.serve())
        })
        // Max 10 channels.
        .buffer_unordered(10)
        .for_each(|_| async {})
        .await;

    Ok(())
}

pub async fn daemon_client() -> Result<WorldClient, anyhow::Error> {
    let mut transport = tarpc::serde_transport::unix::connect("/tmp/kabina.sock", Bincode::default);
    transport.config_mut().max_frame_length(usize::MAX);

    // WorldClient is generated by the service attribute. It has a constructor `new` that takes a
    // config and any Transport as input.
    let client = WorldClient::new(tarpc::client::Config::default(), transport.await?).spawn();

    let reponse = client.hello(current(), "CLIENT".into()).await;

    match reponse {
        Ok(hello) => tracing::info!("{hello:?}"),
        Err(e) => tracing::warn!("{:?}", anyhow::Error::from(e)),
    }

    Ok(client)
}

// This is the type that implements the generated World trait. It is the business logic
// and is used to start the server.
#[derive(Clone)]
struct HelloServer();

#[tarpc::server]
impl World for HelloServer {
    async fn hello(self, _: Context, name: String) -> String {
        format!("Hello, {name}! You are connected")
    }

    async fn version(self, _: Context) -> String {
        VERSION.to_string()
    }

    async fn terminate(self, _: Context) {
        let _ = std::fs::remove_file("/tmp/kabina.sock");
        process::exit(0)
    }
}