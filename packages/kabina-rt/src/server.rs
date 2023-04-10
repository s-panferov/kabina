use std::sync::Arc;

use deno_core::{op, OpState};
use kabina_db::{SchemaBuilder, Server, SharedDatabase};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct JsServer {
	name: String,
}

#[op]
pub fn server(state: &mut OpState, s: JsServer) -> Result<f64, deno_core::error::AnyError> {
	tracing::info!("Server {:?} created", s.name);

	let db = state.borrow::<SharedDatabase>();
	let schema = state.borrow::<Arc<SchemaBuilder>>();

	let handle = Server::new(&*db.lock(), s.name);

	schema.register_server(handle);

	Ok(usize::from(kabina_db::AsId::as_id(handle)) as f64)
}
