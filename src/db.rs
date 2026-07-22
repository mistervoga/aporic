use anyhow::{Context, Result};
use rusqlite::{params, Connection, Transaction};
use std::path::PathBuf;

const SCHEMA_VERSION: i64 = 3;

fn database_path() -> Result<PathBuf> {
    let base = dirs::data_dir().context("cannot resolve user data directory")?;
    Ok(base.join("aporic").join("aporic.db"))
}

pub fn connect_and_init() -> Result<Connection> {
    let path = database_path()?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut conn = Connection::open(&path)
        .with_context(|| format!("cannot open database at {}", path.display()))?;
    ensure_schema(&mut conn)?;
    Ok(conn)
}

pub(crate) fn ensure_schema(conn: &mut Connection) -> Result<()> {
    conn.execute_batch(
        r#"
        PRAGMA foreign_keys = ON;

        CREATE TABLE IF NOT EXISTS schema_migrations (
            version INTEGER PRIMARY KEY,
            applied_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS projects (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL UNIQUE
        );
        "#,
    )?;

    let mut current = conn
        .query_row("SELECT MAX(version) FROM schema_migrations", [], |row| {
            row.get::<_, Option<i64>>(0)
        })?
        .unwrap_or(0);

    if current < 1 {
        migrate_v1(conn)?;
        current = 1;
    }
    if current < 2 {
        migrate_v2(conn)?;
        current = 2;
    }
    if current < 3 {
        migrate_v3(conn)?;
        current = 3;
    }
    anyhow::ensure!(
        current <= SCHEMA_VERSION,
        "database schema is newer than this binary"
    );
    Ok(())
}

fn migrate_v2(conn: &mut Connection) -> Result<()> {
    let tx = conn.transaction()?;
    tx.execute_batch(
        r#"
        ALTER TABLE entries ADD COLUMN math_kind TEXT;
        ALTER TABLE entries ADD COLUMN formal_system TEXT;
        ALTER TABLE entries ADD COLUMN verification TEXT;
        CREATE INDEX entries_math_kind ON entries(math_kind);
        "#,
    )?;
    tx.execute(
        "INSERT INTO schema_migrations(version, applied_at) VALUES (2, ?1)",
        params![chrono::Utc::now().to_rfc3339()],
    )?;
    tx.commit()?;
    Ok(())
}

fn migrate_v1(conn: &mut Connection) -> Result<()> {
    let tx = conn.transaction()?;
    create_v1_tables(&tx)?;

    let now = chrono::Utc::now().to_rfc3339();
    tx.execute(
        "INSERT INTO schema_migrations(version, applied_at) VALUES (1, ?1)",
        params![now],
    )?;
    tx.commit()?;
    Ok(())
}

fn migrate_v3(conn: &mut Connection) -> Result<()> {
    let tx = conn.transaction()?;
    tx.execute_batch(
        r#"
        DROP TABLE IF EXISTS tasks;
        DROP TABLE IF EXISTS meta;
        "#,
    )?;
    tx.execute(
        "INSERT INTO schema_migrations(version, applied_at) VALUES (3, ?1)",
        params![chrono::Utc::now().to_rfc3339()],
    )?;
    tx.commit()?;
    Ok(())
}

// ponytail: databases created before this release still carry the unused
// `legacy_task_id` column; SQLite cannot drop a UNIQUE column, and no code
// reads it any more. Rebuild `entries` only if the dead column ever costs
// something.
fn create_v1_tables(tx: &Transaction<'_>) -> Result<()> {
    tx.execute_batch(
        r#"
        CREATE TABLE entries (
            id TEXT PRIMARY KEY,
            kind TEXT NOT NULL CHECK(kind IN (
                'observation', 'claim', 'assumption', 'question',
                'implication', 'action', 'outcome', 'learning'
            )),
            body TEXT NOT NULL,
            details TEXT,
            state TEXT NOT NULL,
            project_id INTEGER,
            author TEXT NOT NULL,
            origin TEXT NOT NULL,
            source_uri TEXT,
            repository TEXT,
            commit_hash TEXT,
            file_path TEXT,
            line_number INTEGER,
            occurred_at TEXT,
            due_at TEXT,
            completed_at TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            revision INTEGER NOT NULL DEFAULT 1 CHECK(revision > 0),
            FOREIGN KEY(project_id) REFERENCES projects(id) ON DELETE SET NULL
        );

        CREATE INDEX entries_project_kind_state
            ON entries(project_id, kind, state);

        CREATE TABLE relations (
            id TEXT PRIMARY KEY,
            from_id TEXT NOT NULL,
            kind TEXT NOT NULL,
            to_id TEXT NOT NULL,
            rationale TEXT,
            author TEXT NOT NULL,
            origin TEXT NOT NULL,
            created_at TEXT NOT NULL,
            revision INTEGER NOT NULL DEFAULT 1 CHECK(revision > 0),
            UNIQUE(from_id, kind, to_id),
            FOREIGN KEY(from_id) REFERENCES entries(id) ON DELETE RESTRICT,
            FOREIGN KEY(to_id) REFERENCES entries(id) ON DELETE RESTRICT
        );

        CREATE INDEX relations_from ON relations(from_id);
        CREATE INDEX relations_to ON relations(to_id);

        CREATE TABLE sources (
            id TEXT PRIMARY KEY,
            entry_id TEXT NOT NULL,
            kind TEXT NOT NULL,
            uri TEXT NOT NULL,
            content_hash TEXT,
            captured_at TEXT NOT NULL,
            metadata_json TEXT NOT NULL DEFAULT '{}',
            FOREIGN KEY(entry_id) REFERENCES entries(id) ON DELETE RESTRICT
        );

        CREATE TABLE audit_events (
            id TEXT PRIMARY KEY,
            occurred_at TEXT NOT NULL,
            actor TEXT NOT NULL,
            operation TEXT NOT NULL,
            entity_id TEXT NOT NULL,
            payload_json TEXT NOT NULL,
            correlation_id TEXT NOT NULL
        );
        "#,
    )?;
    Ok(())
}

pub(crate) fn insert_audit_event(
    tx: &Transaction<'_>,
    actor: &str,
    operation: &str,
    entity_id: &str,
    payload_json: &str,
    correlation_id: &str,
    occurred_at: &str,
) -> Result<()> {
    tx.execute(
        "INSERT INTO audit_events(
            id, occurred_at, actor, operation, entity_id, payload_json, correlation_id
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            new_id(tx)?,
            occurred_at,
            actor,
            operation,
            entity_id,
            payload_json,
            correlation_id
        ],
    )?;
    Ok(())
}

pub(crate) fn new_id(conn: &Connection) -> Result<String> {
    let mut bytes: Vec<u8> = conn.query_row("SELECT randomblob(16)", [], |row| row.get(0))?;
    let milliseconds = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_millis() as u64;
    let timestamp = milliseconds.to_be_bytes();
    bytes[..6].copy_from_slice(&timestamp[2..]);
    bytes[6] = (bytes[6] & 0x0f) | 0x70;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    Ok(format!(
        "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        bytes[8], bytes[9], bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15]
    ))
}

pub fn schema_version(conn: &Connection) -> Result<i64> {
    Ok(conn
        .query_row("SELECT MAX(version) FROM schema_migrations", [], |row| {
            row.get::<_, Option<i64>>(0)
        })?
        .unwrap_or(0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_current_schema_from_scratch() {
        let mut conn = Connection::open_in_memory().expect("open database");
        ensure_schema(&mut conn).expect("create schema");
        ensure_schema(&mut conn).expect("schema creation is idempotent");

        assert_eq!(schema_version(&conn).expect("version"), SCHEMA_VERSION);
        let math_columns: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM pragma_table_info('entries')
                 WHERE name IN ('math_kind', 'formal_system', 'verification')",
                [],
                |row| row.get(0),
            )
            .expect("math columns");
        assert_eq!(math_columns, 3);
        let legacy_tables: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name IN ('tasks', 'meta')",
                [],
                |row| row.get(0),
            )
            .expect("legacy table check");
        assert_eq!(legacy_tables, 0);
        let legacy_columns: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM pragma_table_info('entries') WHERE name='legacy_task_id'",
                [],
                |row| row.get(0),
            )
            .expect("legacy column check");
        assert_eq!(legacy_columns, 0);
    }

    #[test]
    fn drops_legacy_tables_from_an_already_migrated_database() {
        let mut conn = Connection::open_in_memory().expect("open database");
        conn.execute_batch(
            r#"
            CREATE TABLE schema_migrations (version INTEGER PRIMARY KEY, applied_at TEXT NOT NULL);
            CREATE TABLE meta (key TEXT PRIMARY KEY, value TEXT NOT NULL);
            CREATE TABLE tasks (id INTEGER PRIMARY KEY, description TEXT NOT NULL);
            "#,
        )
        .expect("simulate a pre-cleanup database");

        migrate_v3(&mut conn).expect("drop legacy tables");

        let legacy_tables: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name IN ('tasks', 'meta')",
                [],
                |row| row.get(0),
            )
            .expect("legacy table check");
        assert_eq!(legacy_tables, 0);
    }
}
