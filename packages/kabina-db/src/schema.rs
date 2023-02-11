use dashmap::DashSet;

use crate::FileGroup;

#[salsa::input]
pub struct Schema {
    #[return_ref]
    pub file_groups: DashSet<FileGroup>,
}

#[derive(Default)]
pub struct SchemaBuilder {
    pub file_groups: DashSet<FileGroup>,
}

impl SchemaBuilder {
    pub fn register_file_group(&self, file_group: FileGroup) {
        self.file_groups.insert(file_group);
    }
}
