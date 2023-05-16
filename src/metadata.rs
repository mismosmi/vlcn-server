use std::fs;

use rusqlite::{named_params, OpenFlags};

use crate::error::CRRError;

pub(crate) struct Metadata {
    conn: rusqlite::Connection,
}

impl Metadata {
    pub(crate) fn open() -> Result<Self, CRRError> {
        Ok(Self {
            conn: rusqlite::Connection::open("./data/metadata.sqlite3")?,
        })
    }

    pub(crate) fn open_readonly() -> Result<Self, CRRError> {
        Ok(Self {
            conn: rusqlite::Connection::open_with_flags(
                "./data/metadata.sqlite3",
                OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
            )?,
        })
    }

    #[cfg(test)]
    pub(crate) fn open_for_test(env: &crate::tests::TestEnv) -> Self {
        Self {
            conn: rusqlite::Connection::open(env.folder().join("metadata.sqlite3"))
                .expect("failed to open metadata database"),
        }
    }

    pub(crate) fn apply_migrations(&self) -> Result<(), CRRError> {
        let dir = fs::read_dir("./migrations")?;

        let mut latest_version: Option<i64> = self
            .prepare(
                "
                    SELECT MAX(version)
                    FROM metadata_migrations
                ",
            )
            .and_then(|mut stmt| stmt.query_row([], |row| row.get(0)))
            .unwrap_or_default();

        for file in dir.into_iter() {
            if let Ok(file) = file {
                if let Some(file_name) = file.file_name().to_str() {
                    if file_name.ends_with(".sql") {
                        if let Ok(version) = file_name[0..3].parse::<i64>() {
                            if let Some(latest_version) = latest_version {
                                if version <= latest_version {
                                    continue;
                                }
                            }

                            self.apply_migration(version, fs::read_to_string(file.path())?)?;

                            latest_version = Some(version);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    fn apply_migration(&self, version: i64, sql: String) -> Result<(), CRRError> {
        println!("Applying metadata migration version {}", version);
        self.execute_batch(&sql)?;

        self.prepare("INSERT INTO metadata_migrations (version) VALUES (:version)")?
            .insert(named_params! { ":version": version })?;

        Ok(())
    }
}

impl std::ops::Deref for Metadata {
    type Target = rusqlite::Connection;

    fn deref(&self) -> &Self::Target {
        &self.conn
    }
}

impl std::ops::DerefMut for Metadata {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.conn
    }
}
