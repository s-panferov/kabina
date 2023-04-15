use std::path::PathBuf;

use kabina_db::{
	self, FileGroup, FileGroupItem, FileGroupStategy, ResolveRootFiles, RuntimeTask, Schema,
};

#[test]
fn test() {
	let mut db = kabina_db::Database::new();

	let files = FileGroup::new(
		&db,
		String::from("Test"),
		PathBuf::new(),
		vec![FileGroupItem {
			strategy: FileGroupStategy::Time,
			pattern: String::from("**/*"),
		}],
	);

	let schema = Schema::new(
		&db,
		[files].into_iter().collect(),
		Default::default(),
		Default::default(),
		Default::default(),
		Default::default(),
		Default::default(),
	);

	let _ = kabina_db::file_group_files(&db, schema, files);
	let tasks = kabina_db::file_group_files::accumulated::<RuntimeTask>(&db, schema, files);
	assert_eq!(tasks.len(), 1);

	let tasks = kabina_db::file_group_files::accumulated::<RuntimeTask>(&db, schema, files);
	assert_eq!(tasks.len(), 1);

	for task in tasks {
		if let Some(task) = task.downcast_ref::<ResolveRootFiles>() {
			kabina_db::root_files::set(
				&mut db,
				task.schema,
				task.root.clone(),
				Ok(task.matchers.keys().map(|k| (*k, Vec::new())).collect()),
			)
		}
	}

	let result = kabina_db::file_group_files(&db, schema, files);
	assert!(result.is_ok());

	let tasks = kabina_db::file_group_files::accumulated::<RuntimeTask>(&db, schema, files);
	assert_eq!(tasks.len(), 0);

	files.set_root(&mut db).to(PathBuf::from("abc"));

	let result = kabina_db::file_group_files(&db, schema, files);
	assert!(result.is_err());

	let tasks = kabina_db::file_group_files::accumulated::<RuntimeTask>(&db, schema, files);
	assert_eq!(tasks.len(), 1);
}
