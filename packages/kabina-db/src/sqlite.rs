use rusqlite::{params, Connection};
use rusqlite_migration::{Migrations, M};
use url::Url;

fn migrations() -> Migrations<'static> {
	Migrations::new(vec![M::up(
		r#"
        CREATE TABLE schema_files (url TEXT NOT NULL);
        "#,
	)])
}

pub fn sqlite_setup() -> Result<Connection, anyhow::Error> {
	let migrations = migrations();
	let mut conn = Connection::open_in_memory()?;
	conn.pragma_update(None, "journal_mode", &"WAL")?;
	migrations.to_latest(&mut conn)?;

	Ok(conn)
}

pub struct SqliteSchema {
	pub url: Url,
}

pub fn sqlite_schema_add(c: &Connection, url: url::Url) -> anyhow::Result<()> {
	c.execute("INSERT INTO schema_files (url) VALUES (?1);", params![url])?;
	Ok(())
}

pub fn sqlite_schema_all(c: &Connection) -> anyhow::Result<Vec<SqliteSchema>> {
	let mut stmt = c.prepare("SELECT * FROM schema_files;")?;
	let schemas = stmt
		.query_map([], |r| Ok(SqliteSchema { url: r.get("url")? }))?
		.filter_map(|v| v.ok());

	Ok(schemas.collect())
}

pub fn sqlite_schema_remove(c: &Connection, url: url::Url) -> anyhow::Result<()> {
	let res = c.execute("DELETE FROM schema_files WHERE url = ?1;", params![url])?;
	assert!(res > 0);
	Ok(())
}

#[cfg(test)]
mod tests {
	use std::path::PathBuf;

	use url::Url;

	use super::*;

	#[test]
	fn migrations_test() {
		assert!(migrations().validate().is_ok());
	}

	#[test]
	fn test_sqlite_schema_add() -> Result<(), anyhow::Error> {
		let c = sqlite_setup().unwrap();
		let url = Url::from_file_path(PathBuf::from("/test/a.ts")).unwrap();
		sqlite_schema_add(&c, url.clone())?;

		let schemas = sqlite_schema_all(&c)?;
		assert_eq!(schemas[0].url, url);

		sqlite_schema_remove(&c, url.clone())?;

		Ok(())
	}
}
