#[salsa::input]
#[derive(Debug, Clone)]
pub struct Service {
	pub name: String,
	pub runtime: ServiceRuntime,
}

#[derive(Debug, Clone)]
pub enum ServiceRuntime {
	Binary(ServiceRuntimeBinary),
}

#[derive(Debug, Clone)]
pub struct ServiceRuntimeBinary {
	pub executable: String,
}
