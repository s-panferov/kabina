use dashmap::DashSet;

use crate::{FileGroup, Transform};

#[salsa::input]
pub struct Schema {
    #[return_ref]
    pub file_groups: DashSet<FileGroup>,
    #[return_ref]
    pub transforms: DashSet<Transform>,
}

#[derive(Default)]
pub struct SchemaBuilder {
    pub file_groups: DashSet<FileGroup>,
    pub transforms: DashSet<Transform>,
}

impl SchemaBuilder {
    pub fn register_file_group(&self, file_group: FileGroup) {
        self.file_groups.insert(file_group);
    }

    pub fn register_transform(&self, transform: Transform) {
        self.transforms.insert(transform);
    }
}
