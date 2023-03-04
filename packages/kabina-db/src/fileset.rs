use std::{
    borrow::Cow,
    collections::BTreeMap,
    path::{Path, PathBuf},
    sync::Arc,
    time::UNIX_EPOCH,
};

use by_address::ByAddress;
use downcast_rs::DowncastSync;
use globset::{Candidate, GlobSet};

use crate::{Cause, Database, Outcome, Schema};

use super::db::Db;

#[derive(Debug)]
pub enum FileGroupStategy {
    Hash,
    Time,
}

#[derive(Debug)]
pub struct FileGroupItem {
    pub strategy: FileGroupStategy,
    pub pattern: String,
}

#[salsa::input]
#[derive(Debug)]
pub struct FileGroup {
    #[id]
    pub name: String,
    pub root: PathBuf,
    #[return_ref]
    pub items: Vec<FileGroupItem>,
}

#[derive(Default, Debug, PartialEq, Eq)]
pub struct SchemaRoots {
    roots: BTreeMap<PathBuf, BTreeMap<FileGroup, PathBuf>>,
}

impl SchemaRoots {
    pub fn add(&mut self, db: &dyn Db, file_group: FileGroup) {
        let root = file_group.root(db);

        let mut to_remove = Vec::new();
        let mut should_add = true;
        let mut own_groups = BTreeMap::new();

        for (r, groups) in &mut self.roots {
            if root.starts_with(r) {
                // We already found the root that covers this path
                should_add = false;
                groups.insert(file_group, root.strip_prefix(&r).unwrap().to_owned());
                break;
            }

            if r.starts_with(&root) {
                // Out path is more general, so consume this root
                own_groups.extend(groups.drain_filter(|_, _| true).map(|(g, prefix)| {
                    (g, r.strip_prefix(&root).unwrap().to_owned().join(prefix))
                }));
                to_remove.push(r.clone());
            }
        }

        if should_add {
            own_groups.insert(file_group, PathBuf::new());
            self.roots.insert(root, own_groups);
        }

        to_remove.iter().for_each(|p| {
            self.roots.remove(p);
        });
    }
}

#[salsa::tracked]
pub fn roots(db: &dyn Db, schema: Schema) -> Arc<SchemaRoots> {
    let mut roots = SchemaRoots::default();
    let file_groups = schema.file_groups(db);
    for group in file_groups.iter() {
        roots.add(db, group.clone())
    }

    tracing::info!("Calculate roots: {:?}", roots);

    Arc::new(roots)
}

#[salsa::tracked]
pub fn file_group_root(db: &dyn Db, schema: Schema, group: FileGroup) -> PathBuf {
    let roots = roots(db, schema);
    roots
        .roots
        .iter()
        .find(|r| r.1.contains_key(&group))
        .map(|(k, _)| k.clone())
        .unwrap()
}

#[salsa::tracked]
pub fn file_group_matcher(
    db: &dyn Db,
    group: FileGroup,
    prefix: PathBuf,
) -> ByAddress<Arc<GlobSet>> {
    let mut matcher = globset::GlobSetBuilder::new();

    let items = group.items(db);
    for item in items {
        let mut glob_str = Cow::Borrowed(&item.pattern);
        let prefix = prefix.to_string_lossy();
        if prefix != "" {
            let mut p = prefix.as_ref();
            if prefix.ends_with("/") {
                p = &prefix[0..prefix.len() - 1];
            }
            glob_str = Cow::Owned(format!("{}/{}", p, glob_str))
        }

        tracing::info!("Glob: {:?}", glob_str);

        let glob = globset::Glob::new(&glob_str).unwrap();

        matcher.add(glob);
    }

    ByAddress(Arc::new(matcher.build().unwrap()))
}

#[salsa::tracked]
pub fn root_file_groups(
    db: &dyn Db,
    schema: Schema,
    path: PathBuf,
) -> BTreeMap<FileGroup, PathBuf> {
    let roots = roots(db, schema);

    tracing::info!("Getting groups for root {:?}", path);

    let root = roots.roots.get(&path).unwrap();

    root.clone()
}

pub trait Executable: DowncastSync {}
downcast_rs::impl_downcast!(sync Executable);

#[salsa::accumulator]
pub struct RuntimeTask(Arc<dyn Executable>);

pub struct ResolveRootFiles {
    pub schema: Schema,
    pub root: PathBuf,
    pub matchers: BTreeMap<FileGroup, ByAddress<Arc<GlobSet>>>,
}

impl Executable for ResolveRootFiles {}

impl ResolveRootFiles {
    pub fn resolve(&self, db: &mut Database) {
        let mut results: BTreeMap<FileGroup, Vec<PathBuf>> = BTreeMap::new();

        for group in self.matchers.keys() {
            // Fill with default values for each group
            results.insert(*group, Vec::new());
        }

        for _entry in walkdir::WalkDir::new(&self.root)
            .into_iter()
            .filter_entry(|e| {
                if e.file_type().is_dir() {
                    // TODO: blacklist
                    return true;
                }

                tracing::info!("Visiting {:?}", e.path());

                let candidate = Candidate::new(e.path().strip_prefix(&self.root).unwrap());

                for (group, matcher) in &self.matchers {
                    if !matcher.matches_candidate(&candidate).is_empty() {
                        tracing::info!("Matched group {:?}", group);
                        results.get_mut(group).unwrap().push(e.path().to_owned())
                    }
                }

                true
            })
            .filter_map(|e| Some(e))
        {}

        root_files::set(db, self.schema, self.root.clone(), Result::Ok(results))
    }
}

#[salsa::tracked]
pub fn root_files(
    db: &dyn Db,
    schema: Schema,
    root: PathBuf,
) -> Outcome<BTreeMap<FileGroup, Vec<PathBuf>>> {
    let groups = root_file_groups(db, schema, root.clone());

    let matchers = groups
        .iter()
        .map(|(g, prefix)| (*g, file_group_matcher(db, *g, prefix.clone())))
        .collect::<BTreeMap<_, _>>();

    let task = Arc::new(ResolveRootFiles {
        root,
        schema,
        matchers,
    });

    RuntimeTask::push(db, task.clone());

    Outcome::Err(Cause::Pending)
}

#[salsa::tracked]
pub fn file_group_resolved_paths(
    db: &dyn Db,
    schema: Schema,
    group: FileGroup,
) -> Outcome<Vec<PathBuf>> {
    let root = file_group_root(db, schema, group);

    tracing::info!("Gettings files for {:?}, root: {:?}", group, root);
    let root = root_files(db, schema, root)?;

    tracing::info!("Root files {:?}", root);

    let paths = root
        .get(&group)
        .cloned()
        .expect("Root does not have this group");

    Outcome::Ok(paths)
}

#[salsa::input]
pub struct File {
    #[salsa::id]
    pub path: PathBuf,
    pub revision: u64,
}

pub fn file_modified_time_in_seconds(path: &Path) -> u64 {
    std::fs::metadata(path)
        .unwrap()
        .modified()
        .unwrap()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

#[salsa::tracked]
pub fn file_group_files(db: &dyn Db, schema: Schema, group: FileGroup) -> Outcome<Vec<File>> {
    let files = file_group_resolved_paths(db, schema, group)?
        .into_iter()
        .map(|path| {
            let revision = file_modified_time_in_seconds(&path);
            File::new(db, path, revision)
        })
        .collect();

    Outcome::Ok(files)
}
