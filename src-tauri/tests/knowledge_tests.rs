use helios_chat_lib::{db, knowledge};
use rusqlite::Connection;
use std::fs;

fn migrated_connection() -> (tempfile::TempDir, Connection) {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("helios.sqlite3");
    db::migrate(&path).expect("migrate");
    let conn = Connection::open(path).expect("open db");
    (dir, conn)
}

#[test]
fn migration_creates_knowledge_tables_and_fts_index() {
    let (_dir, conn) = migrated_connection();

    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE name IN (
                'knowledge_stacks',
                'knowledge_sources',
                'knowledge_chunks',
                'knowledge_embeddings',
                'knowledge_chunks_fts'
            )",
            [],
            |row| row.get(0),
        )
        .expect("table count");

    assert_eq!(count, 5);
}

#[test]
fn stack_crud_round_trips_through_sqlite() {
    let (_dir, conn) = migrated_connection();

    let created = knowledge::create_stack(&conn, "Research", "Local docs").expect("create stack");
    let updated =
        knowledge::update_stack(&conn, &created.id, "Research Vault", "Private docs").expect("update");
    let listed = knowledge::list_stacks(&conn).expect("list");

    assert_eq!(updated.name, "Research Vault");
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].description, "Private docs");

    knowledge::delete_stack(&conn, &created.id).expect("delete");
    assert!(knowledge::list_stacks(&conn).expect("list empty").is_empty());
}

#[test]
fn supported_file_filter_accepts_v1_formats_only() {
    for extension in ["txt", "md", "csv", "json", "jsonl", "pdf", "docx", "rtf", "epub"] {
        assert!(knowledge::is_supported_file_name(&format!("notes.{extension}")));
    }

    assert!(!knowledge::is_supported_file_name("image.png"));
    assert!(!knowledge::is_supported_file_name("model.gguf"));
}

#[test]
fn text_extraction_handles_plain_text_and_rejects_binary_without_text() {
    let dir = tempfile::tempdir().expect("tempdir");
    let text_path = dir.path().join("notes.md");
    let binary_path = dir.path().join("empty.pdf");
    fs::write(&text_path, "# Helios\nKnowledge hub notes").expect("write text");
    fs::write(&binary_path, [0_u8, 159, 146, 150]).expect("write binary");

    let text = knowledge::extract_text(&text_path).expect("extract text");
    let err = knowledge::extract_text(&binary_path).expect_err("binary extraction fails");

    assert!(text.contains("Knowledge hub"));
    assert!(err.to_string().contains("No extractable text"));
}

#[test]
fn chunking_creates_overlapping_bounded_chunks() {
    let text = (0..140)
        .map(|index| format!("token{index}"))
        .collect::<Vec<_>>()
        .join(" ");

    let chunks = knowledge::chunk_text(&text, 40, 8);

    assert!(chunks.len() > 1);
    assert!(chunks[0].text.contains("token0"));
    assert!(chunks[1].text.contains("token32"));
    assert!(chunks.iter().all(|chunk| chunk.token_count <= 40));
}

#[test]
fn embeddings_are_normalized_and_rank_related_text_higher() {
    let query = knowledge::embed_text("local private knowledge search");
    let related = knowledge::embed_text("private documents with local search");
    let unrelated = knowledge::embed_text("oranges weather bicycle");

    assert!((knowledge::vector_norm(&query) - 1.0).abs() < 0.0001);
    assert!(knowledge::cosine_similarity(&query, &related) > knowledge::cosine_similarity(&query, &unrelated));
}

#[test]
fn indexing_and_hybrid_search_returns_cited_chunks() {
    let (dir, conn) = migrated_connection();
    let stack = knowledge::create_stack(&conn, "Research", "").expect("create stack");
    let local = dir.path().join("local.md");
    let unrelated = dir.path().join("cooking.txt");
    fs::write(&local, "Helios keeps private knowledge indexed locally for grounded answers.").expect("write local");
    fs::write(&unrelated, "Soup recipes use carrots and onions.").expect("write soup");

    knowledge::index_file(&conn, &stack.id, &local).expect("index local");
    knowledge::index_file(&conn, &stack.id, &unrelated).expect("index soup");

    let results = knowledge::search(
        &conn,
        &[stack.id],
        "private local knowledge",
        knowledge::RetrievalOptions {
            top_k: 3,
            semantic_weight: 0.65,
        },
    )
    .expect("search");

    assert!(!results.is_empty());
    assert!(results[0].content.contains("private knowledge"));
    assert_eq!(results[0].source_title, "local.md");
}

#[test]
fn grounding_context_formats_retrieved_sources_for_chat() {
    let results = vec![knowledge::KnowledgeSearchResult {
        stack_id: "stack-1".to_string(),
        source_id: "source-1".to_string(),
        source_title: "local.md".to_string(),
        chunk_id: "chunk-1".to_string(),
        content: "Helios keeps private knowledge local.".to_string(),
        score: 0.91,
        lexical_score: 0.8,
        semantic_score: 0.95,
    }];

    let context = knowledge::build_grounding_context(&results);

    assert!(context.contains("Use the following local Knowledge Hub sources"));
    assert!(context.contains("[1] local.md"));
    assert!(context.contains("Helios keeps private knowledge local."));
    assert!(context.contains("cite sources"));
}
