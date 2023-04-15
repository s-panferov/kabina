use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Arc;

use tokio::process::Command;

#[derive(Default)]
pub struct ProcessMananger {
	running: BTreeMap<usize, Arc<Process>>,
}

impl ProcessMananger {
	pub fn spawn(&mut self, id: usize, config: ProcessConfig) -> Arc<Process> {
		tracing::info!("Spawning a process: {:?}", config.executable);
		let process = Arc::new(Process::new(config));
		self.running.insert(id, process.clone());
		process
	}
}

pub struct ProcessConfig {
	pub executable: PathBuf,
	pub env: BTreeMap<String, String>,
	pub args: Vec<String>,
}

pub struct Process {
	pub config: ProcessConfig,
}

impl Process {
	pub fn new(config: ProcessConfig) -> Process {
		let mut command = Command::new(&config.executable);
		command.args(config.args.iter());
		command.envs(config.env.iter());
		let mut child = command.spawn().unwrap();
		let _stdin = child.stdin.take();
		tokio::spawn(async move {
			let res = child.wait().await;
			tracing::info!("Process completed with exit code {:?}", res)
		});

		Process { config }
	}
}
