use std::sync::Arc;

use deno_core::{op, OpState};
use kabina_db::{SchemaBuilder, Service, SharedDatabase};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct JsService {
	pub name: String,
	pub binary: usize,
}

#[op]
pub fn service(state: &mut OpState, s: JsService) -> Result<f64, deno_core::error::AnyError> {
	tracing::info!("Service {:?} created", s.name);

	let db = state.borrow::<SharedDatabase>();
	let schema = state.borrow::<Arc<SchemaBuilder>>();

	let binary = kabina_db::AsId::from_id(s.binary.into());
	let handle = Service::new(&*db.lock(), s.name, binary);

	schema.register_service(handle);

	Ok(usize::from(kabina_db::AsId::as_id(handle)) as f64)
}
