use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Conversation {
    pub id: String,
    pub title: String,
    pub provider_id: String,
    pub model: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    pub id: String,
    pub conversation_id: String,
    pub role: String,
    pub content: String,
    pub status: String,
    pub parent_id: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Preset {
    pub id: String,
    pub name: String,
    pub provider_id: String,
    pub model: String,
    pub system_prompt: String,
    pub temperature: f32,
    pub top_p: f32,
    pub max_tokens: u32,
    pub created_at: String,
    pub updated_at: String,
}

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
            provider_id TEXT NOT NULL DEFAULT 'eie-local',
            model TEXT NOT NULL DEFAULT 'qwen3-4b-q4-k-m',
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS messages (
            id TEXT PRIMARY KEY,
            conversation_id TEXT NOT NULL,
            role TEXT NOT NULL,
            content TEXT NOT NULL,
            status TEXT NOT NULL DEFAULT 'complete',
            parent_id TEXT,
            created_at TEXT NOT NULL,
            FOREIGN KEY(conversation_id) REFERENCES conversations(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS presets (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            provider_id TEXT NOT NULL,
            model TEXT NOT NULL,
            system_prompt TEXT NOT NULL,
            temperature REAL NOT NULL,
            top_p REAL NOT NULL,
            max_tokens INTEGER NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
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
    ensure_column(
        &connection,
        "conversations",
        "provider_id",
        "TEXT NOT NULL DEFAULT 'eie-local'",
    )?;
    ensure_column(
        &connection,
        "conversations",
        "model",
        "TEXT NOT NULL DEFAULT 'qwen3-4b-q4-k-m'",
    )?;
    ensure_column(
        &connection,
        "messages",
        "status",
        "TEXT NOT NULL DEFAULT 'complete'",
    )?;
    ensure_column(&connection, "messages", "parent_id", "TEXT")?;
    Ok(())
}

pub fn create_conversation(
    path: &Path,
    title: &str,
    provider_id: &str,
    model: &str,
) -> anyhow::Result<Conversation> {
    let connection = open_migrated(path)?;
    let now = now_string();
    let conversation = Conversation {
        id: uuid::Uuid::new_v4().to_string(),
        title: non_empty_title(title),
        provider_id: provider_id.to_string(),
        model: model.to_string(),
        created_at: now.clone(),
        updated_at: now,
    };
    connection.execute(
        "INSERT INTO conversations (id, title, provider_id, model, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            conversation.id,
            conversation.title,
            conversation.provider_id,
            conversation.model,
            conversation.created_at,
            conversation.updated_at
        ],
    )?;
    Ok(conversation)
}

pub fn list_conversations(path: &Path, search: Option<&str>) -> anyhow::Result<Vec<Conversation>> {
    let connection = open_migrated(path)?;
    let mut conversations = Vec::new();
    if let Some(search) = search.filter(|value| !value.trim().is_empty()) {
        let pattern = format!("%{}%", search.trim());
        let mut statement = connection.prepare(
            "SELECT id, title, provider_id, model, created_at, updated_at
             FROM conversations
             WHERE title LIKE ?1
             ORDER BY updated_at DESC, rowid DESC",
        )?;
        let rows = statement.query_map([pattern], conversation_from_row)?;
        for row in rows {
            conversations.push(row?);
        }
    } else {
        let mut statement = connection.prepare(
            "SELECT id, title, provider_id, model, created_at, updated_at
             FROM conversations
             ORDER BY updated_at DESC, rowid DESC",
        )?;
        let rows = statement.query_map([], conversation_from_row)?;
        for row in rows {
            conversations.push(row?);
        }
    }
    Ok(conversations)
}

pub fn update_conversation(
    path: &Path,
    id: &str,
    title: &str,
    provider_id: &str,
    model: &str,
) -> anyhow::Result<Conversation> {
    let connection = open_migrated(path)?;
    connection.execute(
        "UPDATE conversations
         SET title = ?1, provider_id = ?2, model = ?3
         WHERE id = ?4",
        params![non_empty_title(title), provider_id, model, id],
    )?;
    get_conversation(&connection, id)
}

pub fn delete_conversation(path: &Path, id: &str) -> anyhow::Result<()> {
    let connection = open_migrated(path)?;
    connection.execute("DELETE FROM conversations WHERE id = ?1", [id])?;
    Ok(())
}

pub fn append_message(
    path: &Path,
    conversation_id: &str,
    role: &str,
    content: &str,
    status: &str,
    parent_id: Option<&str>,
) -> anyhow::Result<Message> {
    let connection = open_migrated(path)?;
    let message = Message {
        id: uuid::Uuid::new_v4().to_string(),
        conversation_id: conversation_id.to_string(),
        role: role.to_string(),
        content: content.to_string(),
        status: status.to_string(),
        parent_id: parent_id.map(ToString::to_string),
        created_at: now_string(),
    };
    connection.execute(
        "INSERT INTO messages (id, conversation_id, role, content, status, parent_id, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            message.id,
            message.conversation_id,
            message.role,
            message.content,
            message.status,
            message.parent_id,
            message.created_at
        ],
    )?;
    touch_conversation(&connection, conversation_id)?;
    Ok(message)
}

pub fn list_messages(path: &Path, conversation_id: &str) -> anyhow::Result<Vec<Message>> {
    let connection = open_migrated(path)?;
    let mut statement = connection.prepare(
        "SELECT id, conversation_id, role, content, status, parent_id, created_at
         FROM messages
         WHERE conversation_id = ?1
         ORDER BY rowid ASC",
    )?;
    let rows = statement.query_map([conversation_id], message_from_row)?;
    let mut messages = Vec::new();
    for row in rows {
        messages.push(row?);
    }
    Ok(messages)
}

pub fn update_message(path: &Path, id: &str, content: &str, status: &str) -> anyhow::Result<Message> {
    let connection = open_migrated(path)?;
    let (conversation_id, role, rowid): (String, String, i64) = connection.query_row(
        "SELECT conversation_id, role, rowid FROM messages WHERE id = ?1",
        [id],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
    )?;
    connection.execute(
        "UPDATE messages SET content = ?1, status = ?2 WHERE id = ?3",
        params![content, status, id],
    )?;
    if role == "user" {
        connection.execute(
            "DELETE FROM messages WHERE conversation_id = ?1 AND rowid > ?2",
            params![conversation_id, rowid],
        )?;
    }
    touch_conversation(&connection, &conversation_id)?;
    get_message(&connection, id)
}

pub fn list_presets(path: &Path) -> anyhow::Result<Vec<Preset>> {
    let connection = open_migrated(path)?;
    let mut statement = connection.prepare(
        "SELECT id, name, provider_id, model, system_prompt, temperature, top_p, max_tokens, created_at, updated_at
         FROM presets
         ORDER BY name ASC",
    )?;
    let rows = statement.query_map([], preset_from_row)?;
    let mut presets = Vec::new();
    for row in rows {
        presets.push(row?);
    }
    Ok(presets)
}

pub fn save_preset(path: &Path, preset: &Preset) -> anyhow::Result<Preset> {
    let connection = open_migrated(path)?;
    let now = now_string();
    let id = if preset.id.trim().is_empty() {
        uuid::Uuid::new_v4().to_string()
    } else {
        preset.id.clone()
    };
    let created_at = preset.created_at.clone();
    connection.execute(
        "INSERT INTO presets
         (id, name, provider_id, model, system_prompt, temperature, top_p, max_tokens, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
         ON CONFLICT(id) DO UPDATE SET
            name = excluded.name,
            provider_id = excluded.provider_id,
            model = excluded.model,
            system_prompt = excluded.system_prompt,
            temperature = excluded.temperature,
            top_p = excluded.top_p,
            max_tokens = excluded.max_tokens,
            updated_at = excluded.updated_at",
        params![
            id,
            preset.name,
            preset.provider_id,
            preset.model,
            preset.system_prompt,
            preset.temperature,
            preset.top_p,
            preset.max_tokens,
            if created_at.is_empty() { now.clone() } else { created_at },
            now
        ],
    )?;
    get_preset(&connection, &id)
}

pub fn delete_preset(path: &Path, id: &str) -> anyhow::Result<()> {
    let connection = open_migrated(path)?;
    connection.execute("DELETE FROM presets WHERE id = ?1", [id])?;
    Ok(())
}

fn ensure_column(
    connection: &Connection,
    table: &str,
    column: &str,
    definition: &str,
) -> anyhow::Result<()> {
    let mut statement = connection.prepare(&format!("PRAGMA table_info({})", table))?;
    let existing = statement
        .query_map([], |row| row.get::<_, String>(1))?
        .collect::<Result<Vec<_>, _>>()?;
    if !existing.iter().any(|name| name == column) {
        connection.execute(
            &format!("ALTER TABLE {} ADD COLUMN {} {}", table, column, definition),
            [],
        )?;
    }
    Ok(())
}

fn open_migrated(path: &Path) -> anyhow::Result<Connection> {
    migrate(path)?;
    let connection = Connection::open(path)?;
    connection.execute_batch("PRAGMA foreign_keys = ON;")?;
    Ok(connection)
}

fn get_conversation(connection: &Connection, id: &str) -> anyhow::Result<Conversation> {
    connection
        .query_row(
            "SELECT id, title, provider_id, model, created_at, updated_at
             FROM conversations WHERE id = ?1",
            [id],
            conversation_from_row,
        )
        .optional()?
        .ok_or_else(|| anyhow::anyhow!("Conversation not found: {}", id))
}

fn get_message(connection: &Connection, id: &str) -> anyhow::Result<Message> {
    connection
        .query_row(
            "SELECT id, conversation_id, role, content, status, parent_id, created_at
             FROM messages WHERE id = ?1",
            [id],
            message_from_row,
        )
        .optional()?
        .ok_or_else(|| anyhow::anyhow!("Message not found: {}", id))
}

fn get_preset(connection: &Connection, id: &str) -> anyhow::Result<Preset> {
    connection
        .query_row(
            "SELECT id, name, provider_id, model, system_prompt, temperature, top_p, max_tokens, created_at, updated_at
             FROM presets WHERE id = ?1",
            [id],
            preset_from_row,
        )
        .optional()?
        .ok_or_else(|| anyhow::anyhow!("Preset not found: {}", id))
}

fn touch_conversation(connection: &Connection, id: &str) -> anyhow::Result<()> {
    connection.execute(
        "UPDATE conversations SET updated_at = ?1 WHERE id = ?2",
        params![now_string(), id],
    )?;
    Ok(())
}

fn conversation_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Conversation> {
    Ok(Conversation {
        id: row.get(0)?,
        title: row.get(1)?,
        provider_id: row.get(2)?,
        model: row.get(3)?,
        created_at: row.get(4)?,
        updated_at: row.get(5)?,
    })
}

fn message_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Message> {
    Ok(Message {
        id: row.get(0)?,
        conversation_id: row.get(1)?,
        role: row.get(2)?,
        content: row.get(3)?,
        status: row.get(4)?,
        parent_id: row.get(5)?,
        created_at: row.get(6)?,
    })
}

fn preset_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Preset> {
    Ok(Preset {
        id: row.get(0)?,
        name: row.get(1)?,
        provider_id: row.get(2)?,
        model: row.get(3)?,
        system_prompt: row.get(4)?,
        temperature: row.get(5)?,
        top_p: row.get(6)?,
        max_tokens: row.get(7)?,
        created_at: row.get(8)?,
        updated_at: row.get(9)?,
    })
}

fn now_string() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .to_string()
}

fn non_empty_title(title: &str) -> String {
    let trimmed = title.trim();
    if trimmed.is_empty() {
        "New chat".to_string()
    } else {
        trimmed.to_string()
    }
}
