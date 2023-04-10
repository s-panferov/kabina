use dashmap::DashSet;

use crate::{Collection, FileGroup, Server, Service, Transform};

#[salsa::input]
pub struct Schema {
	#[return_ref]
	pub file_groups: DashSet<FileGroup>,

	#[return_ref]
	pub transforms: DashSet<Transform>,

	#[return_ref]
	pub collections: DashSet<Collection>,

	#[return_ref]
	pub servers: DashSet<Server>,

	#[return_ref]
	pub services: DashSet<Service>,
}

#[derive(Default)]
pub struct SchemaBuilder {
	pub file_groups: DashSet<FileGroup>,
	pub transforms: DashSet<Transform>,
	pub collections: DashSet<Collection>,
	pub servers: DashSet<Server>,
	pub services: DashSet<Service>,
}

impl SchemaBuilder {
	pub fn register_file_group(&self, file_group: FileGroup) {
		self.file_groups.insert(file_group);
	}

	pub fn register_transform(&self, transform: Transform) {
		self.transforms.insert(transform);
	}

	pub fn register_collection(&self, collection: Collection) {
		self.collections.insert(collection);
	}

	pub fn register_server(&self, server: Server) {
		self.servers.insert(server);
	}

	pub fn register_service(&self, service: Service) {
		self.services.insert(service);
	}
}
