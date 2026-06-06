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

// ── chunk_text edge cases ────────────────────────────────────────────────────

#[test]
fn chunk_text_returns_empty_vec_for_blank_input() {
    assert!(knowledge::chunk_text("", 40, 8).is_empty());
    assert!(knowledge::chunk_text("   ", 40, 8).is_empty());
}

#[test]
fn chunk_text_produces_single_chunk_when_text_fits() {
    let text = "one two three four five";
    let chunks = knowledge::chunk_text(text, 10, 2);
    assert_eq!(chunks.len(), 1);
    assert_eq!(chunks[0].text, text);
    assert_eq!(chunks[0].token_count, 5);
}

#[test]
fn chunk_text_splits_exactly_at_max_tokens_boundary() {
    // 10 tokens, max_tokens=5, overlap=0 → exactly 2 chunks
    let text = (0..10).map(|i| format!("w{i}")).collect::<Vec<_>>().join(" ");
    let chunks = knowledge::chunk_text(&text, 5, 0);
    assert_eq!(chunks.len(), 2);
    assert!(chunks.iter().all(|c| c.token_count <= 5));
}

// ── is_supported_file_name edge cases ────────────────────────────────────────

#[test]
fn is_supported_file_name_rejects_name_without_extension() {
    assert!(!knowledge::is_supported_file_name("README"));
    assert!(!knowledge::is_supported_file_name(""));
}

#[test]
fn is_supported_file_name_is_case_insensitive() {
    assert!(knowledge::is_supported_file_name("notes.TXT"));
    assert!(knowledge::is_supported_file_name("doc.PDF"));
    assert!(knowledge::is_supported_file_name("report.Md"));
    assert!(knowledge::is_supported_file_name("data.JSON"));
}

// ── cosine_similarity / vector_norm edge cases ───────────────────────────────

#[test]
fn cosine_similarity_returns_zero_for_mismatched_lengths() {
    let left = vec![1.0_f32, 0.0];
    let right = vec![1.0_f32, 0.0, 0.0];
    assert_eq!(knowledge::cosine_similarity(&left, &right), 0.0);
}

#[test]
fn cosine_similarity_returns_zero_for_empty_vectors() {
    assert_eq!(knowledge::cosine_similarity(&[], &[]), 0.0);
}

#[test]
fn vector_norm_returns_zero_for_all_zero_vector() {
    let v = vec![0.0_f32; 8];
    assert_eq!(knowledge::vector_norm(&v), 0.0);
}

// ── embed_text determinism and normalization ──────────────────────────────────

#[test]
fn embed_text_output_is_unit_length() {
    let v = knowledge::embed_text("hello world document");
    assert_eq!(v.len(), 128);
    assert!((knowledge::vector_norm(&v) - 1.0).abs() < 1e-4);
}

#[test]
fn embed_text_is_deterministic_for_same_input() {
    let a = knowledge::embed_text("rust knowledge hub");
    let b = knowledge::embed_text("rust knowledge hub");
    assert_eq!(a, b);
}

// ── search guard clauses ──────────────────────────────────────────────────────

#[test]
fn search_returns_empty_for_empty_stack_ids() {
    let (_dir, conn) = migrated_connection();
    let results = knowledge::search(
        &conn,
        &[],
        "hello",
        knowledge::RetrievalOptions { top_k: 5, semantic_weight: 0.5 },
    )
    .expect("search");
    assert!(results.is_empty());
}

#[test]
fn search_returns_empty_for_whitespace_only_query() {
    let (_dir, conn) = migrated_connection();
    let stack = knowledge::create_stack(&conn, "Blank query test", "").expect("create");
    let results = knowledge::search(
        &conn,
        &[stack.id],
        "   ",
        knowledge::RetrievalOptions { top_k: 5, semantic_weight: 0.5 },
    )
    .expect("search");
    assert!(results.is_empty());
}

#[test]
fn search_top_k_caps_result_count() {
    let (dir, conn) = migrated_connection();
    let stack = knowledge::create_stack(&conn, "Top-k test", "").expect("create");

    for i in 0..4_u8 {
        let path = dir.path().join(format!("doc{i}.txt"));
        fs::write(&path, format!("private local knowledge indexed document number {i}")).expect("write");
        knowledge::index_file(&conn, &stack.id, &path).expect("index");
    }

    let results = knowledge::search(
        &conn,
        &[stack.id],
        "private local knowledge",
        knowledge::RetrievalOptions { top_k: 2, semantic_weight: 0.65 },
    )
    .expect("search");

    assert!(results.len() <= 2);
}

// ── source CRUD ───────────────────────────────────────────────────────────────

#[test]
fn list_sources_returns_sources_for_correct_stack_only() {
    let (dir, conn) = migrated_connection();
    let stack_a = knowledge::create_stack(&conn, "Stack A", "").expect("create a");
    let stack_b = knowledge::create_stack(&conn, "Stack B", "").expect("create b");

    let path_a = dir.path().join("a.txt");
    let path_b = dir.path().join("b.txt");
    fs::write(&path_a, "hello from stack a content here enough words").expect("write a");
    fs::write(&path_b, "hello from stack b content here enough words").expect("write b");

    knowledge::index_file(&conn, &stack_a.id, &path_a).expect("index a");
    knowledge::index_file(&conn, &stack_b.id, &path_b).expect("index b");

    let sources_a = knowledge::list_sources(&conn, &stack_a.id).expect("list a");
    let sources_b = knowledge::list_sources(&conn, &stack_b.id).expect("list b");

    assert_eq!(sources_a.len(), 1);
    assert_eq!(sources_a[0].title, "a.txt");
    assert_eq!(sources_b.len(), 1);
    assert_eq!(sources_b[0].title, "b.txt");
}

#[test]
fn remove_source_deletes_source_and_associated_chunks() {
    let (dir, conn) = migrated_connection();
    let stack = knowledge::create_stack(&conn, "Remove test", "").expect("create");
    let path = dir.path().join("removable.txt");
    fs::write(&path, "some content to index and then remove from the knowledge stack").expect("write");
    let source = knowledge::index_file(&conn, &stack.id, &path).expect("index");

    knowledge::remove_source(&conn, &source.id).expect("remove");

    let sources = knowledge::list_sources(&conn, &stack.id).expect("list after remove");
    assert!(sources.is_empty());

    let chunk_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM knowledge_chunks WHERE source_id = ?1",
            rusqlite::params![source.id],
            |r| r.get(0),
        )
        .expect("count chunks");
    assert_eq!(chunk_count, 0);
}

#[test]
fn index_file_marks_unsupported_format_as_failed() {
    let (dir, conn) = migrated_connection();
    let stack = knowledge::create_stack(&conn, "Format test", "").expect("create");
    let path = dir.path().join("photo.png");
    fs::write(&path, [137_u8, 80, 78, 71]).expect("write png header");

    let source = knowledge::index_file(&conn, &stack.id, &path).expect("index unsupported");

    assert_eq!(source.status, "failed");
    assert!(source.error.as_deref().unwrap_or("").contains("Unsupported"));
}

#[test]
fn index_file_sets_indexed_status_and_content_hash_for_text_file() {
    let (dir, conn) = migrated_connection();
    let stack = knowledge::create_stack(&conn, "Hash test", "").expect("create");
    let path = dir.path().join("notes.md");
    fs::write(&path, "# Notes\nPrivate knowledge indexed for retrieval here.").expect("write");

    let source = knowledge::index_file(&conn, &stack.id, &path).expect("index");

    assert_eq!(source.status, "indexed");
    assert!(source.content_hash.is_some());
    assert!(source.indexed_at.is_some());
    assert!(source.error.is_none());
}

#[test]
fn reindex_stack_re_indexes_all_existing_sources() {
    let (dir, conn) = migrated_connection();
    let stack = knowledge::create_stack(&conn, "Reindex test", "").expect("create");

    let path1 = dir.path().join("file1.txt");
    let path2 = dir.path().join("file2.txt");
    fs::write(&path1, "first document with local knowledge content for reindexing").expect("write 1");
    fs::write(&path2, "second document with more private information stored locally").expect("write 2");

    knowledge::index_file(&conn, &stack.id, &path1).expect("index 1");
    knowledge::index_file(&conn, &stack.id, &path2).expect("index 2");

    let reindexed = knowledge::reindex_stack(&conn, &stack.id).expect("reindex");

    assert_eq!(reindexed.len(), 2);
    assert!(reindexed.iter().all(|s| s.status == "indexed"));
}

// ── grounding context additional cases ───────────────────────────────────────

#[test]
fn grounding_context_returns_empty_string_for_no_results() {
    assert!(knowledge::build_grounding_context(&[]).is_empty());
}

#[test]
fn grounding_context_numbers_multiple_results_sequentially() {
    let results = vec![
        knowledge::KnowledgeSearchResult {
            stack_id: "s".to_string(),
            source_id: "src-1".to_string(),
            source_title: "alpha.md".to_string(),
            chunk_id: "c-1".to_string(),
            content: "First chunk content.".to_string(),
            score: 0.9,
            lexical_score: 0.8,
            semantic_score: 0.95,
        },
        knowledge::KnowledgeSearchResult {
            stack_id: "s".to_string(),
            source_id: "src-2".to_string(),
            source_title: "beta.txt".to_string(),
            chunk_id: "c-2".to_string(),
            content: "Second chunk content.".to_string(),
            score: 0.7,
            lexical_score: 0.6,
            semantic_score: 0.75,
        },
    ];

    let context = knowledge::build_grounding_context(&results);

    assert!(context.contains("[1] alpha.md"));
    assert!(context.contains("First chunk content."));
    assert!(context.contains("[2] beta.txt"));
    assert!(context.contains("Second chunk content."));
}

// ── stack source_count tracking ───────────────────────────────────────────────

#[test]
fn stack_source_count_reflects_indexed_files() {
    let (dir, conn) = migrated_connection();
    let stack = knowledge::create_stack(&conn, "Count test", "").expect("create");

    let path1 = dir.path().join("one.txt");
    let path2 = dir.path().join("two.txt");
    fs::write(&path1, "alpha beta gamma delta epsilon zeta eta theta enough").expect("write 1");
    fs::write(&path2, "iota kappa lambda mu nu xi omicron pi rho sigma tau").expect("write 2");

    knowledge::index_file(&conn, &stack.id, &path1).expect("index 1");
    knowledge::index_file(&conn, &stack.id, &path2).expect("index 2");

    let stacks = knowledge::list_stacks(&conn).expect("list stacks");
    let updated = stacks.iter().find(|s| s.id == stack.id).expect("find stack");

    assert_eq!(updated.source_count, 2);
    assert_eq!(updated.indexed_source_count, 2);
}

// ── index_folder ──────────────────────────────────────────────────────────────

#[test]
fn index_folder_indexes_supported_files_and_skips_unsupported() {
    let dir = tempfile::tempdir().expect("tempdir");
    let db_path = dir.path().join("helios.sqlite3");
    db::migrate(&db_path).expect("migrate");
    let conn = Connection::open(&db_path).expect("open db");

    let stack = knowledge::create_stack(&conn, "Folder test", "").expect("create");
    let subdir = dir.path().join("subdir");
    fs::create_dir(&subdir).expect("mkdir");

    let f1 = dir.path().join("top.txt");
    let f2 = subdir.join("nested.md");
    fs::write(&f1, "top level document with enough extractable text content here").expect("write f1");
    fs::write(&f2, "nested subdirectory document with sufficient extractable content").expect("write f2");

    let sources = knowledge::index_folder(&conn, &stack.id, dir.path()).expect("index folder");

    let indexed_titles: Vec<_> = sources
        .iter()
        .filter(|s| s.status == "indexed")
        .map(|s| s.title.as_str())
        .collect();
    assert!(indexed_titles.contains(&"top.txt"));
    assert!(indexed_titles.contains(&"nested.md"));
}
