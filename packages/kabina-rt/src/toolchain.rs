use deno_core::{op, OpState};
use kabina_db::{SharedDatabase, Toolchain};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct JsToolchain {
    name: String,
    binary: String,
    runner: String,
}

#[op]
pub fn toolchain(state: &mut OpState, f: JsToolchain) -> Result<f64, deno_core::error::AnyError> {
    tracing::info!("Toolchain {:?} created", f.name);

    let db = state.borrow::<SharedDatabase>();
    // let schema = state.borrow::<Arc<SchemaBuilder>>();

    let handle = Toolchain::new(&*db.read(), f.name, f.binary, f.runner);

    // schema.register_transform(handle);

    Ok(usize::from(kabina_db::AsId::as_id(handle)) as f64)
}
