use std::{collections::BTreeMap, path::PathBuf};

use crate::{file_group_files, transform_files, Cause, Db, DependencyKind, File, Outcome, Schema};

#[derive(Debug, Clone)]
pub struct BundleItem {
    pub prefix: PathBuf,
    pub content: DependencyKind,
}

#[salsa::input]
#[derive(Debug, Clone)]
pub struct Bundle {
    pub name: String,
    pub items: Vec<BundleItem>,
}

#[salsa::tracked]
pub fn bundle_files(
    db: &dyn Db,
    schema: Schema,
    bundle: Bundle,
) -> Outcome<BTreeMap<PathBuf, File>> {
    let mut pending = false;
    let mut buffer = BTreeMap::new();

    for item in bundle.items(db) {
        match item.content {
            DependencyKind::FileGroup(g) => match file_group_files(db, schema, g) {
                Ok(files) => {
                    for file in files {
                        let path = item.prefix.join(file.path(db));
                        buffer.insert(path, file);
                    }
                }
                Err(Cause::Pending) => pending = true,
                Err(e) => return Err(e),
            },
            DependencyKind::Transform(t) => match transform_files(db, schema, t) {
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
