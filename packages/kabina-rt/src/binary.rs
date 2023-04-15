use deno_core::{op, OpState};
use kabina_db::{Binary, BinaryNative, BinaryRuntime, SharedDatabase};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct JsBinary {
	pub name: String,
	pub runtime: JsBinaryRuntime,
}

#[derive(Deserialize)]
#[serde(tag = "kind")]
#[serde(rename_all = "lowercase")]
pub enum JsBinaryRuntime {
	Native(JsBinaryNative),
}

#[derive(Deserialize)]
pub struct JsBinaryNative {
	pub executable: String,
}

#[op]
pub fn binary(state: &mut OpState, b: JsBinary) -> Result<f64, deno_core::error::AnyError> {
	tracing::info!("Binary {:?} created", b.name);

	let db = state.borrow::<SharedDatabase>();
	// let schema = state.borrow::<Arc<SchemaBuilder>>();

	let handle = Binary::new(
		&*db.lock(),
		b.name,
		match b.runtime {
			JsBinaryRuntime::Native(b) => BinaryRuntime::Native(BinaryNative {
				executable: b.executable,
			}),
		},
	);

	// schema.register_transform(handle);

	Ok(usize::from(kabina_db::AsId::as_id(handle)) as f64)
}
