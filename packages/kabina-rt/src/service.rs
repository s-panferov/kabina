use std::sync::Arc;

use deno_core::{op, OpState};
use kabina_db::{SchemaBuilder, Service, ServiceRuntime, ServiceRuntimeBinary, SharedDatabase};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct JsService {
	pub name: String,
	pub runtime: JsServiceRuntime,
}

#[derive(Deserialize)]
#[serde(tag = "kind")]
#[serde(rename_all = "lowercase")]
pub enum JsServiceRuntime {
	Binary(JsServiceRuntimeBinary),
}

#[derive(Deserialize)]
pub struct JsServiceRuntimeBinary {
	pub executable: String,
}

#[op]
pub fn service(state: &mut OpState, s: JsService) -> Result<f64, deno_core::error::AnyError> {
	tracing::info!("Service {:?} created", s.name);

	let db = state.borrow::<SharedDatabase>();
	let schema = state.borrow::<Arc<SchemaBuilder>>();

	let handle = Service::new(
		&*db.lock(),
		s.name,
		match s.runtime {
			JsServiceRuntime::Binary(b) => ServiceRuntime::Binary(ServiceRuntimeBinary {
				executable: b.executable,
			}),
		},
	);

	schema.register_service(handle);

	Ok(usize::from(kabina_db::AsId::as_id(handle)) as f64)
}
