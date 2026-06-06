use rusqlite::Connection;
use std::path::Path;

pub fn migrate(path: &Path) -> anyhow::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let connection = Connection::open(path)?;
    connection.execute_batch(
        r#"
        PRAGMA journal_mode = WAL;
        PRAGMA foreign_keys = ON;

        CREATE TABLE IF NOT EXISTS conversations (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS messages (
            id TEXT PRIMARY KEY,
            conversation_id TEXT NOT NULL,
            role TEXT NOT NULL,
            content TEXT NOT NULL,
            created_at TEXT NOT NULL,
            FOREIGN KEY(conversation_id) REFERENCES conversations(id)
        );

        CREATE TABLE IF NOT EXISTS knowledge_stacks (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            description TEXT NOT NULL DEFAULT '',
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS knowledge_sources (
            id TEXT PRIMARY KEY,
            stack_id TEXT NOT NULL,
            path TEXT NOT NULL,
            title TEXT NOT NULL,
            format TEXT NOT NULL,
            status TEXT NOT NULL,
            content_hash TEXT,
            indexed_at TEXT,
            error TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            FOREIGN KEY(stack_id) REFERENCES knowledge_stacks(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS knowledge_chunks (
            id TEXT PRIMARY KEY,
            stack_id TEXT NOT NULL,
            source_id TEXT NOT NULL,
            chunk_index INTEGER NOT NULL,
            content TEXT NOT NULL,
            token_count INTEGER NOT NULL,
            created_at TEXT NOT NULL,
            FOREIGN KEY(stack_id) REFERENCES knowledge_stacks(id) ON DELETE CASCADE,
            FOREIGN KEY(source_id) REFERENCES knowledge_sources(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS knowledge_embeddings (
            chunk_id TEXT PRIMARY KEY,
            dimensions INTEGER NOT NULL,
            vector BLOB NOT NULL,
            created_at TEXT NOT NULL,
            FOREIGN KEY(chunk_id) REFERENCES knowledge_chunks(id) ON DELETE CASCADE
        );

        CREATE VIRTUAL TABLE IF NOT EXISTS knowledge_chunks_fts USING fts5(
            chunk_id UNINDEXED,
            stack_id UNINDEXED,
            source_id UNINDEXED,
            content
        );
        "#,
    )?;
    Ok(())
}
