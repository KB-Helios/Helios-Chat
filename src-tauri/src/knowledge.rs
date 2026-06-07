use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fs::File;
use std::hash::{Hash, Hasher};
use std::io::Read;
use std::path::{Path, PathBuf};
use uuid::Uuid;

const EMBEDDING_DIMS: usize = 128;
const DEFAULT_CHUNK_TOKENS: usize = 220;
const DEFAULT_CHUNK_OVERLAP: usize = 36;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct KnowledgeStack {
    pub id: String,
    pub name: String,
    pub description: String,
    pub created_at: String,
    pub updated_at: String,
    pub source_count: u32,
    pub indexed_source_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct KnowledgeSource {
    pub id: String,
    pub stack_id: String,
    pub path: String,
    pub title: String,
    pub format: String,
    pub status: String,
    pub content_hash: Option<String>,
    pub indexed_at: Option<String>,
    pub error: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TextChunk {
    pub text: String,
    pub token_count: usize,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct RetrievalOptions {
    pub top_k: usize,
    pub semantic_weight: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct KnowledgeSearchResult {
    pub stack_id: String,
    pub source_id: String,
    pub source_title: String,
    pub chunk_id: String,
    pub content: String,
    pub score: f32,
    pub lexical_score: f32,
    pub semantic_score: f32,
}

pub fn create_stack(
    conn: &Connection,
    name: &str,
    description: &str,
) -> anyhow::Result<KnowledgeStack> {
    let now = timestamp();
    let id = Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO knowledge_stacks (id, name, description, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?4)",
        params![id, name.trim(), description.trim(), now],
    )?;
    get_stack(conn, &id)
}

pub fn update_stack(
    conn: &Connection,
    id: &str,
    name: &str,
    description: &str,
) -> anyhow::Result<KnowledgeStack> {
    conn.execute(
        "UPDATE knowledge_stacks SET name = ?1, description = ?2, updated_at = ?3 WHERE id = ?4",
        params![name.trim(), description.trim(), timestamp(), id],
    )?;
    get_stack(conn, id)
}

pub fn delete_stack(conn: &Connection, id: &str) -> anyhow::Result<()> {
    conn.execute("DELETE FROM knowledge_stacks WHERE id = ?1", params![id])?;
    conn.execute(
        "DELETE FROM knowledge_chunks_fts WHERE stack_id = ?1",
        params![id],
    )?;
    Ok(())
}

pub fn list_stacks(conn: &Connection) -> anyhow::Result<Vec<KnowledgeStack>> {
    let mut stmt = conn.prepare(
        "SELECT
            s.id,
            s.name,
            s.description,
            s.created_at,
            s.updated_at,
            COUNT(src.id) AS source_count,
            SUM(CASE WHEN src.status = 'indexed' THEN 1 ELSE 0 END) AS indexed_source_count
        FROM knowledge_stacks s
        LEFT JOIN knowledge_sources src ON src.stack_id = s.id
        GROUP BY s.id
        ORDER BY s.updated_at DESC",
    )?;
    let rows = stmt.query_map([], read_stack)?;
    collect_rows(rows)
}

pub fn list_sources(conn: &Connection, stack_id: &str) -> anyhow::Result<Vec<KnowledgeSource>> {
    let mut stmt = conn.prepare(
        "SELECT id, stack_id, path, title, format, status, content_hash, indexed_at, error, created_at, updated_at
         FROM knowledge_sources
         WHERE stack_id = ?1
         ORDER BY updated_at DESC",
    )?;
    let rows = stmt.query_map(params![stack_id], read_source)?;
    collect_rows(rows)
}

pub fn remove_source(conn: &Connection, source_id: &str) -> anyhow::Result<()> {
    conn.execute(
        "DELETE FROM knowledge_chunks_fts WHERE source_id = ?1",
        params![source_id],
    )?;
    conn.execute(
        "DELETE FROM knowledge_sources WHERE id = ?1",
        params![source_id],
    )?;
    Ok(())
}

pub fn index_file(
    conn: &Connection,
    stack_id: &str,
    path: &Path,
) -> anyhow::Result<KnowledgeSource> {
    let previous = find_source_by_path(conn, stack_id, path)?;
    let (status, previous_hash, previous_error) = previous
        .as_ref()
        .map(|source| {
            (
                source.status.as_str(),
                source.content_hash.clone(),
                source.error.clone(),
            )
        })
        .unwrap_or(("extracting", None, None));
    let source = upsert_source(conn, stack_id, path, status, previous_hash, previous_error)?;

    if !is_supported_path(path) {
        replace_chunks(conn, stack_id, &source.id, "")?;
        let error = "Unsupported file format".to_string();
        return update_source_status(conn, &source.id, "failed", None, Some(error));
    }

    match extract_text(path) {
        Ok(text) => {
            let hash = content_hash(&text);
            replace_chunks_atomically(conn, stack_id, &source.id, &text, hash).map_err(|error| {
                if previous.is_none() {
                    let _ = update_source_status(
                        conn,
                        &source.id,
                        "failed",
                        None,
                        Some(error.to_string()),
                    );
                }
                error
            })
        }
        Err(error) => {
            replace_chunks(conn, stack_id, &source.id, "")?;
            update_source_status(conn, &source.id, "failed", None, Some(error.to_string()))
        }
    }
}

pub fn index_folder(
    conn: &Connection,
    stack_id: &str,
    folder: &Path,
) -> anyhow::Result<Vec<KnowledgeSource>> {
    let mut indexed = Vec::new();
    for path in discover_files(folder)? {
        indexed.push(index_file(conn, stack_id, &path)?);
    }
    Ok(indexed)
}

pub fn reindex_stack(conn: &Connection, stack_id: &str) -> anyhow::Result<Vec<KnowledgeSource>> {
    let sources = list_sources(conn, stack_id)?;
    let mut reindexed = Vec::new();
    for source in sources {
        reindexed.push(index_file(conn, stack_id, Path::new(&source.path))?);
    }
    Ok(reindexed)
}

pub fn search(
    conn: &Connection,
    stack_ids: &[String],
    query: &str,
    options: RetrievalOptions,
) -> anyhow::Result<Vec<KnowledgeSearchResult>> {
    if stack_ids.is_empty() || query.trim().is_empty() {
        return Ok(Vec::new());
    }

    let top_k = options.top_k.max(1);
    let semantic_weight = options.semantic_weight.clamp(0.0, 1.0);
    let lexical_scores = lexical_search(conn, stack_ids, query)?;
    let query_embedding = embed_text(query);
    let mut candidates = all_chunks(conn, stack_ids)?;

    for candidate in &mut candidates {
        candidate.lexical_score = *lexical_scores.get(&candidate.chunk_id).unwrap_or(&0.0);
        candidate.semantic_score = cosine_similarity(&query_embedding, &candidate.embedding);
        candidate.score = (semantic_weight * candidate.semantic_score)
            + ((1.0 - semantic_weight) * candidate.lexical_score);
    }

    candidates.sort_by(|left, right| {
        right
            .score
            .partial_cmp(&left.score)
            .unwrap_or(Ordering::Equal)
            .then_with(|| {
                right
                    .lexical_score
                    .partial_cmp(&left.lexical_score)
                    .unwrap_or(Ordering::Equal)
            })
    });
    candidates.truncate(top_k);

    Ok(candidates
        .into_iter()
        .filter(|candidate| candidate.score > 0.0)
        .map(|candidate| KnowledgeSearchResult {
            stack_id: candidate.stack_id,
            source_id: candidate.source_id,
            source_title: candidate.source_title,
            chunk_id: candidate.chunk_id,
            content: candidate.content,
            score: candidate.score,
            lexical_score: candidate.lexical_score,
            semantic_score: candidate.semantic_score,
        })
        .collect())
}

pub fn build_grounding_context(results: &[KnowledgeSearchResult]) -> String {
    if results.is_empty() {
        return String::new();
    }

    let mut context = String::from(
        "Use the following local Knowledge Hub sources to answer. If the sources do not contain the answer, say so. Always cite sources with their bracketed numbers.\n\n",
    );
    for (index, result) in results.iter().enumerate() {
        context.push_str(&format!(
            "[{}] {}\n{}\n\n",
            index + 1,
            result.source_title,
            result.content
        ));
    }
    context
}

pub fn is_supported_file_name(name: &str) -> bool {
    Path::new(name)
        .extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| {
            matches!(
                extension.to_ascii_lowercase().as_str(),
                "txt" | "md" | "csv" | "json" | "jsonl" | "docx" | "rtf" | "epub"
            )
        })
        .unwrap_or(false)
}

pub fn extract_text(path: &Path) -> anyhow::Result<String> {
    let cleaned = match extension(path).as_deref() {
        Some("docx") => extract_docx_text(path)?,
        Some("epub") => extract_epub_text(path)?,
        Some("pdf") => anyhow::bail!("PDF extraction is not supported yet"),
        Some("rtf") => {
            let bytes = std::fs::read(path)?;
            strip_rtf(&String::from_utf8_lossy(&bytes))
        }
        _ => {
            let bytes = std::fs::read(path)?;
            String::from_utf8_lossy(&bytes).to_string()
        }
    };
    let normalized = normalize_text(&cleaned);
    if normalized.chars().filter(|ch| !ch.is_control()).count() < 8 {
        anyhow::bail!("No extractable text found in {}", path.display());
    }
    Ok(normalized)
}

pub fn chunk_text(text: &str, max_tokens: usize, overlap_tokens: usize) -> Vec<TextChunk> {
    let tokens = text.split_whitespace().collect::<Vec<_>>();
    if tokens.is_empty() {
        return Vec::new();
    }

    let max_tokens = max_tokens.max(1);
    let overlap_tokens = overlap_tokens.min(max_tokens.saturating_sub(1));
    let step = max_tokens.saturating_sub(overlap_tokens).max(1);
    let mut chunks = Vec::new();
    let mut start = 0;

    while start < tokens.len() {
        let end = (start + max_tokens).min(tokens.len());
        let text = tokens[start..end].join(" ");
        chunks.push(TextChunk {
            token_count: end - start,
            text,
        });
        if end == tokens.len() {
            break;
        }
        start += step;
    }

    chunks
}

pub fn embed_text(text: &str) -> Vec<f32> {
    let mut vector = vec![0.0; EMBEDDING_DIMS];
    for token in tokenize(text) {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        token.hash(&mut hasher);
        let hash = hasher.finish();
        let index = (hash as usize) % EMBEDDING_DIMS;
        let sign = if (hash >> 63) == 0 { 1.0 } else { -1.0 };
        vector[index] += sign;
    }
    normalize_vector(&mut vector);
    vector
}

pub fn vector_norm(vector: &[f32]) -> f32 {
    vector.iter().map(|value| value * value).sum::<f32>().sqrt()
}

pub fn cosine_similarity(left: &[f32], right: &[f32]) -> f32 {
    if left.len() != right.len() || left.is_empty() {
        return 0.0;
    }
    left.iter()
        .zip(right.iter())
        .map(|(a, b)| a * b)
        .sum::<f32>()
}

fn get_stack(conn: &Connection, id: &str) -> anyhow::Result<KnowledgeStack> {
    conn.query_row(
        "SELECT
            s.id,
            s.name,
            s.description,
            s.created_at,
            s.updated_at,
            COUNT(src.id) AS source_count,
            SUM(CASE WHEN src.status = 'indexed' THEN 1 ELSE 0 END) AS indexed_source_count
        FROM knowledge_stacks s
        LEFT JOIN knowledge_sources src ON src.stack_id = s.id
        WHERE s.id = ?1
        GROUP BY s.id",
        params![id],
        read_stack,
    )
    .map_err(Into::into)
}

fn upsert_source(
    conn: &Connection,
    stack_id: &str,
    path: &Path,
    status: &str,
    content_hash: Option<String>,
    error: Option<String>,
) -> anyhow::Result<KnowledgeSource> {
    let path_text = path.display().to_string();
    let existing_id = conn
        .query_row(
            "SELECT id FROM knowledge_sources WHERE stack_id = ?1 AND path = ?2",
            params![stack_id, path_text],
            |row| row.get::<_, String>(0),
        )
        .optional()?;
    let id = existing_id.unwrap_or_else(|| Uuid::new_v4().to_string());
    let now = timestamp();
    let title = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("Untitled")
        .to_string();
    let format = extension(path).unwrap_or_else(|| "unknown".to_string());

    conn.execute(
        "INSERT INTO knowledge_sources (id, stack_id, path, title, format, status, content_hash, indexed_at, error, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, NULL, ?8, ?9, ?9)
         ON CONFLICT(id) DO UPDATE SET
            title = excluded.title,
            format = excluded.format,
            status = excluded.status,
            content_hash = excluded.content_hash,
            error = excluded.error,
            updated_at = excluded.updated_at",
        params![id, stack_id, path_text, title, format, status, content_hash, error, now],
    )?;
    get_source(conn, &id)
}

fn update_source_status(
    conn: &Connection,
    source_id: &str,
    status: &str,
    content_hash: Option<String>,
    error: Option<String>,
) -> anyhow::Result<KnowledgeSource> {
    let indexed_at = if status == "indexed" {
        Some(timestamp())
    } else {
        None
    };
    conn.execute(
        "UPDATE knowledge_sources
         SET status = ?1, content_hash = ?2, indexed_at = ?3, error = ?4, updated_at = ?5
         WHERE id = ?6",
        params![
            status,
            content_hash,
            indexed_at,
            error,
            timestamp(),
            source_id
        ],
    )?;
    get_source(conn, source_id)
}

fn get_source(conn: &Connection, id: &str) -> anyhow::Result<KnowledgeSource> {
    conn.query_row(
        "SELECT id, stack_id, path, title, format, status, content_hash, indexed_at, error, created_at, updated_at
         FROM knowledge_sources
         WHERE id = ?1",
        params![id],
        read_source,
    )
    .map_err(Into::into)
}

fn find_source_by_path(
    conn: &Connection,
    stack_id: &str,
    path: &Path,
) -> anyhow::Result<Option<KnowledgeSource>> {
    conn.query_row(
        "SELECT id, stack_id, path, title, format, status, content_hash, indexed_at, error, created_at, updated_at
         FROM knowledge_sources
         WHERE stack_id = ?1 AND path = ?2",
        params![stack_id, path.display().to_string()],
        read_source,
    )
    .optional()
    .map_err(Into::into)
}

fn replace_chunks_atomically(
    conn: &Connection,
    stack_id: &str,
    source_id: &str,
    text: &str,
    hash: String,
) -> anyhow::Result<KnowledgeSource> {
    let savepoint = format!("knowledge_reindex_{}", Uuid::new_v4().simple());
    conn.execute_batch(&format!("SAVEPOINT {savepoint}"))?;
    let result = (|| {
        replace_chunks(conn, stack_id, source_id, text)?;
        update_source_status(conn, source_id, "indexed", Some(hash), None)
    })();

    match result {
        Ok(source) => {
            conn.execute_batch(&format!("RELEASE SAVEPOINT {savepoint}"))?;
            Ok(source)
        }
        Err(error) => {
            let _ = conn.execute_batch(&format!("ROLLBACK TO SAVEPOINT {savepoint}"));
            let _ = conn.execute_batch(&format!("RELEASE SAVEPOINT {savepoint}"));
            Err(error)
        }
    }
}

fn replace_chunks(
    conn: &Connection,
    stack_id: &str,
    source_id: &str,
    text: &str,
) -> anyhow::Result<()> {
    conn.execute(
        "DELETE FROM knowledge_chunks_fts WHERE source_id = ?1",
        params![source_id],
    )?;
    conn.execute(
        "DELETE FROM knowledge_chunks WHERE source_id = ?1",
        params![source_id],
    )?;

    for (index, chunk) in chunk_text(text, DEFAULT_CHUNK_TOKENS, DEFAULT_CHUNK_OVERLAP)
        .into_iter()
        .enumerate()
    {
        let chunk_id = Uuid::new_v4().to_string();
        let vector = embed_text(&chunk.text);
        conn.execute(
            "INSERT INTO knowledge_chunks (id, stack_id, source_id, chunk_index, content, token_count, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                chunk_id,
                stack_id,
                source_id,
                index as i64,
                chunk.text,
                chunk.token_count as i64,
                timestamp()
            ],
        )?;
        conn.execute(
            "INSERT INTO knowledge_embeddings (chunk_id, dimensions, vector, created_at) VALUES (?1, ?2, ?3, ?4)",
            params![chunk_id, EMBEDDING_DIMS as i64, encode_vector(&vector), timestamp()],
        )?;
        conn.execute(
            "INSERT INTO knowledge_chunks_fts (chunk_id, stack_id, source_id, content) VALUES (?1, ?2, ?3, ?4)",
            params![chunk_id, stack_id, source_id, chunk.text],
        )?;
    }

    Ok(())
}

fn lexical_search(
    conn: &Connection,
    stack_ids: &[String],
    query: &str,
) -> anyhow::Result<HashMap<String, f32>> {
    let match_query = fts_query(query);
    if match_query.is_empty() {
        return Ok(HashMap::new());
    }

    let placeholders = stack_ids.iter().map(|_| "?").collect::<Vec<_>>().join(", ");
    let sql = format!(
        "SELECT chunk_id, bm25(knowledge_chunks_fts) AS rank
         FROM knowledge_chunks_fts
         WHERE content MATCH ?1 AND stack_id IN ({})
         ORDER BY rank ASC
         LIMIT 64",
        placeholders
    );

    let mut stmt = conn.prepare(&sql)?;
    let mut params = vec![&match_query as &dyn rusqlite::ToSql];
    for stack_id in stack_ids {
        params.push(stack_id as &dyn rusqlite::ToSql);
    }
    let rows = stmt.query_map(params.as_slice(), |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, f32>(1)?))
    })?;

    let mut scored = Vec::new();
    for row in rows {
        let (chunk_id, rank) = row?;
        scored.push((chunk_id, rank));
    }
    let best = scored
        .iter()
        .map(|(_, rank)| *rank)
        .fold(f32::INFINITY, f32::min);
    let worst = scored
        .iter()
        .map(|(_, rank)| *rank)
        .fold(f32::NEG_INFINITY, f32::max);

    Ok(scored
        .into_iter()
        .map(|(chunk_id, rank)| {
            let score = if (worst - best).abs() < f32::EPSILON {
                1.0
            } else {
                1.0 - ((rank - best) / (worst - best))
            };
            (chunk_id, score.clamp(0.0, 1.0))
        })
        .collect())
}

fn all_chunks(conn: &Connection, stack_ids: &[String]) -> anyhow::Result<Vec<SearchCandidate>> {
    if stack_ids.is_empty() {
        return Ok(Vec::new());
    }

    let placeholders = stack_ids.iter().map(|_| "?").collect::<Vec<_>>().join(", ");
    let sql = format!(
        "SELECT c.id, c.stack_id, c.source_id, s.title, c.content, e.vector
         FROM knowledge_chunks c
         JOIN knowledge_sources s ON s.id = c.source_id
         JOIN knowledge_embeddings e ON e.chunk_id = c.id
         WHERE c.stack_id IN ({})",
        placeholders
    );
    let mut stmt = conn.prepare(&sql)?;
    let params = stack_ids
        .iter()
        .map(|stack_id| stack_id as &dyn rusqlite::ToSql)
        .collect::<Vec<_>>();
    let rows = stmt.query_map(params.as_slice(), |row| {
        Ok(SearchCandidate {
            chunk_id: row.get(0)?,
            stack_id: row.get(1)?,
            source_id: row.get(2)?,
            source_title: row.get(3)?,
            content: row.get(4)?,
            embedding: decode_vector(&row.get::<_, Vec<u8>>(5)?),
            lexical_score: 0.0,
            semantic_score: 0.0,
            score: 0.0,
        })
    })?;

    let mut chunks = Vec::new();
    for row in rows {
        chunks.push(row?);
    }
    Ok(chunks)
}

fn discover_files(root: &Path) -> anyhow::Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    if root.is_file() {
        files.push(root.to_path_buf());
        return Ok(files);
    }
    for entry in std::fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        let metadata = std::fs::symlink_metadata(&path)?;
        if metadata.file_type().is_symlink() {
            continue;
        }
        if metadata.is_dir() {
            files.extend(discover_files(&path)?);
        } else {
            files.push(path);
        }
    }
    Ok(files)
}

fn is_supported_path(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .map(is_supported_file_name)
        .unwrap_or(false)
}

fn extension(path: &Path) -> Option<String> {
    path.extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| extension.to_ascii_lowercase())
}

fn normalize_text(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn strip_rtf(text: &str) -> String {
    let mut output = String::new();
    let mut in_control = false;
    for ch in text.chars() {
        match ch {
            '\\' => in_control = true,
            '{' | '}' => in_control = false,
            ' ' if in_control => in_control = false,
            _ if !in_control && !ch.is_control() => output.push(ch),
            _ => {}
        }
    }
    output
}

fn extract_docx_text(path: &Path) -> anyhow::Result<String> {
    let file = File::open(path)?;
    let mut archive = zip::ZipArchive::new(file)?;
    let mut document = archive.by_name("word/document.xml")?;
    let mut xml = String::new();
    document.read_to_string(&mut xml)?;
    Ok(strip_markup(&xml))
}

fn extract_epub_text(path: &Path) -> anyhow::Result<String> {
    let file = File::open(path)?;
    let mut archive = zip::ZipArchive::new(file)?;
    let mut text = String::new();

    for index in 0..archive.len() {
        let mut entry = archive.by_index(index)?;
        let name = entry.name().to_ascii_lowercase();
        if !(name.ends_with(".xhtml") || name.ends_with(".html") || name.ends_with(".xml")) {
            continue;
        }

        let mut contents = String::new();
        if entry.read_to_string(&mut contents).is_ok() {
            text.push(' ');
            text.push_str(&strip_markup(&contents));
        }
    }

    Ok(text)
}

fn strip_markup(text: &str) -> String {
    let mut output = String::new();
    let mut in_tag = false;
    let mut entity: Option<String> = None;

    for ch in text.chars() {
        match ch {
            '<' => {
                in_tag = true;
                output.push(' ');
            }
            '>' => {
                in_tag = false;
                output.push(' ');
            }
            '&' if !in_tag => entity = Some(String::new()),
            ';' if !in_tag && entity.is_some() => {
                let name = entity.take().unwrap_or_default();
                output.push_str(match name.as_str() {
                    "amp" => "&",
                    "lt" => "<",
                    "gt" => ">",
                    "quot" => "\"",
                    "apos" => "'",
                    _ => " ",
                });
            }
            _ if in_tag => {}
            _ if entity.is_some() => {
                if let Some(name) = entity.as_mut() {
                    name.push(ch);
                }
            }
            _ if !ch.is_control() => output.push(ch),
            _ => {}
        }
    }

    output
}

fn tokenize(text: &str) -> Vec<String> {
    text.split(|ch: char| !ch.is_alphanumeric())
        .filter_map(|token| {
            let normalized = token.trim().to_ascii_lowercase();
            if normalized.len() > 1 {
                Some(normalized)
            } else {
                None
            }
        })
        .collect()
}

fn fts_query(query: &str) -> String {
    tokenize(query)
        .into_iter()
        .take(16)
        .map(|token| format!("\"{token}\""))
        .collect::<Vec<_>>()
        .join(" OR ")
}

fn normalize_vector(vector: &mut [f32]) {
    let norm = vector_norm(vector);
    if norm > 0.0 {
        for value in vector {
            *value /= norm;
        }
    }
}

fn encode_vector(vector: &[f32]) -> Vec<u8> {
    vector
        .iter()
        .flat_map(|value| value.to_le_bytes())
        .collect::<Vec<_>>()
}

fn decode_vector(bytes: &[u8]) -> Vec<f32> {
    bytes
        .chunks_exact(4)
        .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
        .collect()
}

fn content_hash(text: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(text.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn timestamp() -> String {
    let seconds = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    seconds.to_string()
}

fn read_stack(row: &rusqlite::Row<'_>) -> rusqlite::Result<KnowledgeStack> {
    Ok(KnowledgeStack {
        id: row.get(0)?,
        name: row.get(1)?,
        description: row.get(2)?,
        created_at: row.get(3)?,
        updated_at: row.get(4)?,
        source_count: row.get::<_, i64>(5)? as u32,
        indexed_source_count: row.get::<_, Option<i64>>(6)?.unwrap_or(0) as u32,
    })
}

fn read_source(row: &rusqlite::Row<'_>) -> rusqlite::Result<KnowledgeSource> {
    Ok(KnowledgeSource {
        id: row.get(0)?,
        stack_id: row.get(1)?,
        path: row.get(2)?,
        title: row.get(3)?,
        format: row.get(4)?,
        status: row.get(5)?,
        content_hash: row.get(6)?,
        indexed_at: row.get(7)?,
        error: row.get(8)?,
        created_at: row.get(9)?,
        updated_at: row.get(10)?,
    })
}

fn collect_rows<T>(
    rows: rusqlite::MappedRows<'_, impl FnMut(&rusqlite::Row<'_>) -> rusqlite::Result<T>>,
) -> anyhow::Result<Vec<T>> {
    let mut values = Vec::new();
    for row in rows {
        values.push(row?);
    }
    Ok(values)
}

#[derive(Debug)]
struct SearchCandidate {
    chunk_id: String,
    stack_id: String,
    source_id: String,
    source_title: String,
    content: String,
    embedding: Vec<f32>,
    lexical_score: f32,
    semantic_score: f32,
    score: f32,
}
