use std::{collections::BTreeMap, path::PathBuf};

use crate::{deps::Input, file_group_files, transform_files, Cause, Db, File, Outcome, Schema};

#[derive(Debug, Clone)]
pub struct CollectionItem {
    pub prefix: PathBuf,
    pub content: Input,
}

#[salsa::input]
#[derive(Debug, Clone)]
pub struct Collection {
    pub name: String,
    pub items: Vec<CollectionItem>,
}

#[salsa::tracked]
pub fn collection_files(
    db: &dyn Db,
    schema: Schema,
    collection: Collection,
) -> Outcome<BTreeMap<PathBuf, File>> {
    let mut pending = false;
    let mut buffer = BTreeMap::new();

    for item in collection.items(db) {
        match item.content {
            Input::FileGroup(g) => match file_group_files(db, schema, g) {
                Ok(files) => {
                    for file in files {
                        let path = item.prefix.join(file.path(db));
                        buffer.insert(path, file);
                    }
                }
                Err(Cause::Pending) => pending = true,
                Err(e) => return Err(e),
            },
            Input::Transform(t) => match transform_files(db, schema, t) {
                Ok(files) => {
                    for file in files {
                        let path = item.prefix.join(file.path(db));
                        buffer.insert(path, file);
                    }
                }
                Err(Cause::Pending) => pending = true,
                Err(e) => return Err(e),
            },
        }
    }

    if pending {
        return Outcome::Err(Cause::Pending);
    }

    Result::Ok(buffer)
}
