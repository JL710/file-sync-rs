use rusqlite::{self, Connection, OptionalExtension, Result};
use std::path::PathBuf;

trait DBManager {
    fn get_path(&self) -> String;
    fn create_tables(&self, connection: &Connection) -> Result<(), rusqlite::Error>;
    fn connect(&self) -> Result<Connection, rusqlite::Error> {
        Connection::open(self.get_path())
    }
}

pub struct AppSettings {
    path: PathBuf,
}

impl AppSettings {
    pub fn new(path: PathBuf) -> Result<Self, rusqlite::Error> {
        let new_self = AppSettings { path };

        let connection = new_self.connect()?;
        new_self.create_tables(&connection)?;

        Ok(new_self)
    }

    pub fn set_setting(&self, key: &str, value: &str) -> Result<(), rusqlite::Error> {
        let connection = self.connect()?;
        connection.execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
            (key, value),
        )?;
        Ok(())
    }

    pub fn del_setting(&self, key: &str) -> Result<(), rusqlite::Error> {
        let connection = self.connect()?;
        connection.execute("DELETE FROM settings WHERE key=?1", (key,))?;
        Ok(())
    }

    pub fn get_setting(&self, key: &str) -> Result<Option<String>, rusqlite::Error> {
        let connection = self.connect()?;
        let mut smtp = connection.prepare("SELECT key, value FROM settings WHERE key=?1")?;
        let result = smtp
            .query_row((key,), |x| Ok(x.get::<usize, String>(1).unwrap()))
            .optional()
            .unwrap();
        Ok(result)
    }

    pub fn add_source(&self, path: PathBuf) -> Result<(), rusqlite::Error> {
        let connection = self.connect()?;
        connection.execute(
            "
            INSERT INTO sources (path) VALUES (?1);
            ",
            (path.to_str().unwrap(),),
        )?;
        Ok(())
    }

    pub fn remove_source(&self, path: PathBuf) -> Result<(), rusqlite::Error> {
        let connection = self.connect()?;
        connection.execute(
            "DELETE FROM sources WHERE path = ?1",
            (path.to_str().unwrap(),),
        )?;
        Ok(())
    }

    pub fn get_sources(&self) -> Result<Vec<PathBuf>, rusqlite::Error> {
        let connection = self.connect()?;
        let mut smtp = connection.prepare("SELECT path FROM sources;")?;
        let result = smtp
            .query_map([], |row| row.get::<usize, String>(0))
            .unwrap()
            .map(|row| row.unwrap().into())
            .collect::<Vec<PathBuf>>();

        Ok(result)
    }
}

impl DBManager for AppSettings {
    fn get_path(&self) -> String {
        self.path.to_str().unwrap().to_owned()
    }

    fn create_tables(&self, connection: &Connection) -> Result<(), rusqlite::Error> {
        connection.execute(
            "
            CREATE TABLE IF NOT EXISTS sources (
                path TEXT NOT NULL
            );
            ",
            (),
        )?;

        connection.execute(
            "
            CREATE TABLE IF NOT EXISTS settings (
                key TEXT NOT NULL UNIQUE PRIMARY KEY,
                value TEXT NOT NULL
            )
            ",
            (),
        )?;

        Ok(())
    }
}
