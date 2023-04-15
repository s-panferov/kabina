use std::collections::BTreeMap;
use std::sync::Arc;

use tokio::process::{Child, Command};

#[derive(Default)]
pub struct ProcessMananger {
	running: BTreeMap<usize, Arc<Process>>,
}

impl ProcessMananger {
	pub fn spawn(&mut self, id: usize, config: ProcessConfig) -> Arc<Process> {
		tracing::info!("Spawning a process: {}", config.executable);
		let process = Arc::new(Process::new(config));
		self.running.insert(id, process.clone());
		process
	}
}

pub struct ProcessConfig {
	pub executable: String,
}

pub struct Process {
	pub child: Child,
	pub config: ProcessConfig,
}

impl Process {
	pub fn new(config: ProcessConfig) -> Process {
		let mut command = Command::new(&config.executable);
		let child = command.spawn().unwrap();
		Process { child, config }
	}
}
