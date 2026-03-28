use rusqlite::{Connection, Result, params};
use crate::model::Entry;

const DB_PATH: &str = "time-tracker.db";

pub fn open() -> Result<Connection> {
    let conn = Connection::open(DB_PATH)?;
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS entries (
            id          INTEGER PRIMARY KEY AUTOINCREMENT,
            task        TEXT NOT NULL,
            description TEXT NOT NULL DEFAULT '',
            started_at  INTEGER NOT NULL,
            ended_at    INTEGER
        );",
    )?;
    Ok(conn)
}

pub fn load_today(conn: &Connection) -> Result<Vec<Entry>> {
    // midnight unix timestamp for today (UTC)
    let today_start = {
        use chrono::{Utc, TimeZone, Datelike};
        let now = Utc::now();
        Utc.with_ymd_and_hms(now.year(), now.month(), now.day(), 0, 0, 0)
            .unwrap()
            .timestamp()
    };

    let mut stmt = conn.prepare(
        "SELECT id, task, description, started_at, ended_at FROM entries WHERE started_at >= ? ORDER BY started_at",
    )?;
    let rows = stmt.query_map(params![today_start], |row| {
        Ok(Entry {
            id: row.get(0)?,
            task: row.get(1)?,
            description: row.get(2)?,
            started_at: row.get(3)?,
            ended_at: row.get(4)?,
        })
    })?;
    rows.collect()
}

pub fn insert_entry(
    conn: &Connection,
    task: &str,
    description: &str,
    started_at: i64,
    ended_at: Option<i64>,
) -> Result<i64> {
    conn.execute(
        "INSERT INTO entries (task, description, started_at, ended_at) VALUES (?1, ?2, ?3, ?4)",
        params![task, description, started_at, ended_at],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn update_description(conn: &Connection, id: i64, description: &str) -> Result<()> {
    conn.execute(
        "UPDATE entries SET description = ?1 WHERE id = ?2",
        params![description, id],
    )?;
    Ok(())
}

pub fn delete_entry(conn: &Connection, id: i64) -> Result<()> {
    conn.execute("DELETE FROM entries WHERE id = ?1", params![id])?;
    Ok(())
}
