#![feature(async_fn_in_trait)]

use std::rc::Rc;
use std::sync::Arc;

use deno_core::url::Url;
use deno_core::v8::{HandleScope, Local};
use deno_core::{v8, Extension, JsRuntime, ModuleLoader, RuntimeOptions};
use kabina_db::runtime::Runtime;
use kabina_db::{AsId, Schema, SchemaBuilder, SharedDatabase};
use module::KabinaModuleLoader;
use serde::Serialize;

mod binary;
mod collection;
mod fileset;
mod module;
mod server;
mod service;
mod transform;

pub struct DenoRuntime {
	db: SharedDatabase,
	runtime: JsRuntime,
	std: usize,
}

impl DenoRuntime {
	pub async fn new(db: SharedDatabase) -> Self {
		let ext = Extension::builder("runtime")
			.ops(vec![fileset::file_group::decl()])
			.ops(vec![transform::transform::decl()])
			.ops(vec![collection::collection::decl()])
			.ops(vec![server::server::decl()])
			.ops(vec![service::service::decl()])
			.ops(vec![binary::binary::decl()])
			.build();

		let loader = Rc::new(KabinaModuleLoader);

		// Initialize a runtime instance
		let mut runtime = JsRuntime::new(RuntimeOptions {
			module_loader: Some(loader.clone()),
			extensions: vec![ext],
			..Default::default()
		});

		runtime.op_state().borrow_mut().put(db.clone());

		let std_url = &KabinaModuleLoader::runtime_module_specifier();
		let std = runtime
			.load_side_module(
				&std_url,
				Some(loader.load(&std_url, None, false).await.unwrap().code),
			)
			.await
			.unwrap();

		let receiver = runtime.mod_evaluate(std);
		runtime.run_event_loop(false).await.unwrap();
		let _ = receiver.await;

		DenoRuntime { db, runtime, std }
	}
}

impl Runtime for DenoRuntime {
	async fn load_schema(&mut self, url: Url) -> Schema {
		let schema = Arc::new(SchemaBuilder::default());
		self.runtime.op_state().borrow_mut().put(schema);

		let source = tokio::fs::read_to_string(url.path()).await.unwrap();
		let module = self
			.runtime
			.load_main_module(
				&url,
				Some(deno_core::ModuleCode::Owned(source.into_bytes())),
			)
			.await
			.unwrap();

		let eval = self.runtime.mod_evaluate(module);

		self.runtime.run_event_loop(false).await.unwrap();
		eval.await.unwrap().unwrap();

		let builder = self
			.runtime
			.op_state()
			.borrow_mut()
			.take::<Arc<SchemaBuilder>>();

		let Ok(builder) = Arc::try_unwrap(builder) else {
            panic!("Arc cloned")
        };

		Schema::new(
			&*self.db.lock(),
			url,
			builder.file_groups,
			builder.transforms,
			builder.collections,
			builder.servers,
			builder.services,
			builder.binaries,
		)
	}

	async fn transform(&mut self, task: &kabina_db::TransformApply) -> kabina_db::File {
		let id = AsId::as_id(task.transform).as_u32();

		let ns = self.runtime.get_module_namespace(self.std).unwrap();

		let context = self.runtime.global_context();
		let isolate = self.runtime.v8_isolate();

		let ns = ns.open(isolate);
		let mut scope = HandleScope::with_context(isolate, context);

		let string = v8::String::new(&mut scope, "__transforms").unwrap();

		let value = ns.get(&mut scope, string.into()).unwrap();
		let js_id = v8::Number::new(&mut scope, id as f64);

		let function = value
			.to_object(&mut scope)
			.unwrap()
			.get(&mut scope, js_id.into())
			.unwrap();

		let function = Local::<v8::Function>::try_from(function).unwrap();
		let null = v8::null(&mut scope).into();

		#[derive(Serialize)]
		#[allow(non_snake_case)]
		struct Context {
			filePath: String,
		}

		let db = self.db.lock();

		let deps_v8 = deno_core::serde_v8::to_v8(&mut scope, &*task.dependencies).unwrap();
		let context = deno_core::serde_v8::to_v8(
			&mut scope,
			Context {
				filePath: task.file.path(&*db).to_string_lossy().to_string(),
			},
		)
		.unwrap();

		let value = function
			.call(&mut scope, null, &[context, deps_v8])
			.unwrap();

		println!("{:?}", value.is_boolean());

		todo!();
	}
}
