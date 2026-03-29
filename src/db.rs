use rusqlite::{Connection, Result, params};
use crate::model::Entry;

const DB_PATH: &str = "time-tracker.db";

const DEFAULT_TASKS: &[&str] = &["Development", "Meetings", "Review", "Admin"];

pub fn open() -> Result<Connection> {
    let conn = Connection::open(DB_PATH)?;
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS entries (
            id          INTEGER PRIMARY KEY AUTOINCREMENT,
            task        TEXT NOT NULL,
            description TEXT NOT NULL DEFAULT '',
            started_at  INTEGER NOT NULL,
            ended_at    INTEGER
        );
        CREATE TABLE IF NOT EXISTS tasks (
            id   INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL UNIQUE
        );",
    )?;
    // Seed defaults on first run
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM tasks", [], |r| r.get(0))?;
    if count == 0 {
        for name in DEFAULT_TASKS {
            conn.execute("INSERT OR IGNORE INTO tasks (name) VALUES (?1)", params![name])?;
        }
    }
    Ok(conn)
}

pub fn load_tasks(conn: &Connection) -> Result<Vec<String>> {
    let mut stmt = conn.prepare("SELECT name FROM tasks ORDER BY id")?;
    let rows = stmt.query_map([], |r| r.get(0))?;
    rows.collect()
}

pub fn insert_task(conn: &Connection, name: &str) -> Result<()> {
    conn.execute("INSERT INTO tasks (name) VALUES (?1)", params![name])?;
    Ok(())
}

pub fn rename_task(conn: &Connection, old_name: &str, new_name: &str) -> Result<()> {
    conn.execute(
        "UPDATE tasks SET name = ?1 WHERE name = ?2",
        params![new_name, old_name],
    )?;
    conn.execute(
        "UPDATE entries SET task = ?1 WHERE task = ?2",
        params![new_name, old_name],
    )?;
    Ok(())
}

pub fn delete_task(conn: &Connection, name: &str) -> Result<()> {
    conn.execute("DELETE FROM tasks WHERE name = ?1", params![name])?;
    Ok(())
}

pub fn load_week(conn: &Connection) -> Result<Vec<Entry>> {
    use chrono::{Utc, TimeZone, Datelike};
    let now = Utc::now();
    let days_since_monday = now.weekday().num_days_from_monday() as i64;
    let monday = now.date_naive() - chrono::Duration::days(days_since_monday);
    let week_start = Utc
        .with_ymd_and_hms(monday.year(), monday.month(), monday.day(), 0, 0, 0)
        .unwrap()
        .timestamp();

    let mut stmt = conn.prepare(
        "SELECT id, task, description, started_at, ended_at FROM entries WHERE started_at >= ? ORDER BY started_at",
    )?;
    let rows = stmt.query_map(params![week_start], |row| {
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

pub fn update_entry(
    conn: &Connection,
    id: i64,
    task: &str,
    description: &str,
    started_at: i64,
    ended_at: Option<i64>,
) -> Result<()> {
    conn.execute(
        "UPDATE entries SET task = ?1, description = ?2, started_at = ?3, ended_at = ?4 WHERE id = ?5",
        params![task, description, started_at, ended_at, id],
    )?;
    Ok(())
}

pub fn delete_entry(conn: &Connection, id: i64) -> Result<()> {
    conn.execute("DELETE FROM entries WHERE id = ?1", params![id])?;
    Ok(())
}
