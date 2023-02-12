use dashmap::DashSet;

use crate::{Bundle, FileGroup, Server, Transform};

#[salsa::input]
pub struct Schema {
    #[return_ref]
    pub file_groups: DashSet<FileGroup>,

    #[return_ref]
    pub transforms: DashSet<Transform>,

    #[return_ref]
    pub bundles: DashSet<Bundle>,

    #[return_ref]
    pub servers: DashSet<Server>,
}

#[derive(Default)]
pub struct SchemaBuilder {
    pub file_groups: DashSet<FileGroup>,
    pub transforms: DashSet<Transform>,
    pub bundles: DashSet<Bundle>,
    pub servers: DashSet<Server>,
}

impl SchemaBuilder {
    pub fn register_file_group(&self, file_group: FileGroup) {
        self.file_groups.insert(file_group);
    }

    pub fn register_transform(&self, transform: Transform) {
        self.transforms.insert(transform);
    }

    pub fn register_bundle(&self, bundle: Bundle) {
        self.bundles.insert(bundle);
    }

    pub fn register_server(&self, server: Server) {
        self.servers.insert(server);
    }
}
