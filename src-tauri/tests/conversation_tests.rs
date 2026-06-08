use helios_chat_lib::db::{
    append_message, create_conversation, delete_conversation, list_conversations, list_messages,
    migrate, update_conversation, update_message,
};
use std::thread;
use std::time::Duration;

#[test]
fn conversations_are_listed_newest_first_and_can_be_renamed() {
    let dir = tempfile::tempdir().expect("tempdir");
    let db_path = dir.path().join("helios.sqlite3");
    migrate(&db_path).expect("migrate");

    let first = create_conversation(&db_path, "First", "eie-local", "qwen3-4b-q4-k-m")
        .expect("first conversation");
    let second = create_conversation(&db_path, "Second", "openai", "gpt-4.1")
        .expect("second conversation");
    update_conversation(&db_path, &first.id, "Renamed", "eie-local", "qwen3-4b-q4-k-m")
        .expect("rename");

    let conversations = list_conversations(&db_path, None).expect("list");
    assert_eq!(conversations[0].id, first.id);
    assert_eq!(conversations[0].title, "Renamed");
    assert_eq!(conversations[0].provider_id, "eie-local");
    assert_eq!(conversations[1].id, second.id);
}

#[test]
fn messages_persist_with_status_and_parent_links() {
    let dir = tempfile::tempdir().expect("tempdir");
    let db_path = dir.path().join("helios.sqlite3");
    migrate(&db_path).expect("migrate");
    let conversation = create_conversation(&db_path, "Chat", "eie-local", "qwen3-4b-q4-k-m")
        .expect("conversation");

    let user = append_message(&db_path, &conversation.id, "user", "hello", "complete", None)
        .expect("user message");
    let assistant = append_message(
        &db_path,
        &conversation.id,
        "assistant",
        "draft",
        "streaming",
        Some(&user.id),
    )
    .expect("assistant message");
    update_message(&db_path, &assistant.id, "final", "complete").expect("update message");

    let messages = list_messages(&db_path, &conversation.id).expect("messages");
    assert_eq!(messages.len(), 2);
    assert_eq!(messages[1].content, "final");
    assert_eq!(messages[1].status, "complete");
    assert_eq!(messages[1].parent_id.as_deref(), Some(user.id.as_str()));
}

#[test]
fn deleting_a_conversation_cascades_messages() {
    let dir = tempfile::tempdir().expect("tempdir");
    let db_path = dir.path().join("helios.sqlite3");
    migrate(&db_path).expect("migrate");
    let conversation = create_conversation(&db_path, "Chat", "eie-local", "qwen3-4b-q4-k-m")
        .expect("conversation");
    append_message(&db_path, &conversation.id, "user", "hello", "complete", None)
        .expect("message");

    delete_conversation(&db_path, &conversation.id).expect("delete conversation");

    assert!(list_messages(&db_path, &conversation.id)
        .expect("messages")
        .is_empty());
}

#[test]
fn editing_a_user_message_prunes_later_branch_messages() {
    let dir = tempfile::tempdir().expect("tempdir");
    let db_path = dir.path().join("helios.sqlite3");
    migrate(&db_path).expect("migrate");
    let conversation = create_conversation(&db_path, "Chat", "eie-local", "qwen3-4b-q4-k-m")
        .expect("conversation");
    let user = append_message(&db_path, &conversation.id, "user", "hello", "complete", None)
        .expect("user message");
    append_message(
        &db_path,
        &conversation.id,
        "assistant",
        "old answer",
        "complete",
        Some(&user.id),
    )
    .expect("assistant message");

    update_message(&db_path, &user.id, "edited hello", "complete").expect("edit user");

    let messages = list_messages(&db_path, &conversation.id).expect("messages");
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0].content, "edited hello");
}

#[test]
fn regenerating_an_assistant_message_prunes_later_branch_messages() {
    let dir = tempfile::tempdir().expect("tempdir");
    let db_path = dir.path().join("helios.sqlite3");
    migrate(&db_path).expect("migrate");
    let conversation = create_conversation(&db_path, "Chat", "eie-local", "qwen3-4b-q4-k-m")
        .expect("conversation");
    let first_user = append_message(&db_path, &conversation.id, "user", "hello", "complete", None)
        .expect("first user");
    let assistant = append_message(
        &db_path,
        &conversation.id,
        "assistant",
        "old answer",
        "complete",
        Some(&first_user.id),
    )
    .expect("assistant");
    append_message(
        &db_path,
        &conversation.id,
        "user",
        "follow up",
        "complete",
        None,
    )
    .expect("follow-up user");

    update_message(&db_path, &assistant.id, "", "streaming").expect("regenerate assistant");

    let messages = list_messages(&db_path, &conversation.id).expect("messages");
    assert_eq!(messages.len(), 2);
    assert_eq!(messages[1].id, assistant.id);
    assert_eq!(messages[1].status, "streaming");
}

#[test]
fn empty_title_defaults_to_new_chat() {
    let dir = tempfile::tempdir().expect("tempdir");
    let db_path = dir.path().join("helios.sqlite3");
    migrate(&db_path).expect("migrate");

    let conversation = create_conversation(&db_path, "", "eie-local", "qwen3-4b-q4-k-m")
        .expect("create conversation");

    assert_eq!(conversation.title, "New chat");
}

#[test]
fn whitespace_only_title_defaults_to_new_chat() {
    let dir = tempfile::tempdir().expect("tempdir");
    let db_path = dir.path().join("helios.sqlite3");
    migrate(&db_path).expect("migrate");

    let conversation = create_conversation(&db_path, "   \t  ", "eie-local", "qwen3-4b-q4-k-m")
        .expect("create conversation");

    assert_eq!(conversation.title, "New chat");
}

#[test]
fn search_filters_conversations_by_title_substring() {
    let dir = tempfile::tempdir().expect("tempdir");
    let db_path = dir.path().join("helios.sqlite3");
    migrate(&db_path).expect("migrate");

    create_conversation(&db_path, "Alpha project", "eie-local", "qwen3-4b-q4-k-m")
        .expect("alpha");
    create_conversation(&db_path, "Beta analysis", "openai", "gpt-4.1").expect("beta");
    create_conversation(&db_path, "Gamma report", "anthropic", "claude-4-sonnet").expect("gamma");

    let results = list_conversations(&db_path, Some("lpha")).expect("search");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].title, "Alpha project");
}

#[test]
fn empty_string_search_returns_all_conversations() {
    let dir = tempfile::tempdir().expect("tempdir");
    let db_path = dir.path().join("helios.sqlite3");
    migrate(&db_path).expect("migrate");

    create_conversation(&db_path, "First", "eie-local", "qwen3-4b-q4-k-m").expect("first");
    create_conversation(&db_path, "Second", "openai", "gpt-4.1").expect("second");

    let with_none = list_conversations(&db_path, None).expect("none search");
    let with_empty = list_conversations(&db_path, Some("")).expect("empty search");
    let with_whitespace = list_conversations(&db_path, Some("   ")).expect("whitespace search");

    assert_eq!(with_none.len(), 2);
    assert_eq!(with_empty.len(), 2);
    assert_eq!(with_whitespace.len(), 2);
}

#[test]
fn conversation_stores_provider_id_and_model() {
    let dir = tempfile::tempdir().expect("tempdir");
    let db_path = dir.path().join("helios.sqlite3");
    migrate(&db_path).expect("migrate");

    let conversation = create_conversation(&db_path, "My chat", "anthropic", "claude-4-opus")
        .expect("conversation");

    assert_eq!(conversation.provider_id, "anthropic");
    assert_eq!(conversation.model, "claude-4-opus");

    let listed = list_conversations(&db_path, None).expect("list");
    assert_eq!(listed[0].provider_id, "anthropic");
    assert_eq!(listed[0].model, "claude-4-opus");
}

#[test]
fn updating_assistant_message_to_complete_does_not_prune_later_messages() {
    let dir = tempfile::tempdir().expect("tempdir");
    let db_path = dir.path().join("helios.sqlite3");
    migrate(&db_path).expect("migrate");
    let conversation = create_conversation(&db_path, "Chat", "eie-local", "qwen3-4b-q4-k-m")
        .expect("conversation");

    let user = append_message(&db_path, &conversation.id, "user", "question", "complete", None)
        .expect("user");
    let assistant = append_message(
        &db_path,
        &conversation.id,
        "assistant",
        "draft answer",
        "streaming",
        Some(&user.id),
    )
    .expect("assistant draft");
    let follow_up = append_message(
        &db_path,
        &conversation.id,
        "user",
        "follow-up question",
        "complete",
        None,
    )
    .expect("follow-up");

    // Finishing an assistant message (streaming -> complete) should NOT prune later messages
    update_message(&db_path, &assistant.id, "final answer", "complete")
        .expect("finish assistant");

    let messages = list_messages(&db_path, &conversation.id).expect("messages");
    assert_eq!(messages.len(), 3);
    assert_eq!(messages[1].content, "final answer");
    assert_eq!(messages[1].status, "complete");
    assert_eq!(messages[2].id, follow_up.id);
}

#[test]
fn appending_a_message_updates_conversation_timestamp() {
    let dir = tempfile::tempdir().expect("tempdir");
    let db_path = dir.path().join("helios.sqlite3");
    migrate(&db_path).expect("migrate");

    let conversation = create_conversation(&db_path, "Chat", "eie-local", "qwen3-4b-q4-k-m")
        .expect("conversation");
    let before_updated_at = conversation.updated_at.clone();

    // Sleep briefly to ensure timestamp differs
    thread::sleep(Duration::from_millis(2));

    append_message(&db_path, &conversation.id, "user", "hello", "complete", None)
        .expect("message");

    let updated = list_conversations(&db_path, None)
        .expect("list")
        .into_iter()
        .find(|c| c.id == conversation.id)
        .expect("find conversation");

    assert!(
        updated.updated_at >= before_updated_at,
        "updated_at should advance after appending a message"
    );
}

#[test]
fn search_is_case_insensitive_via_sqlite_like() {
    let dir = tempfile::tempdir().expect("tempdir");
    let db_path = dir.path().join("helios.sqlite3");
    migrate(&db_path).expect("migrate");

    create_conversation(&db_path, "Rust Programming", "eie-local", "qwen3-4b-q4-k-m")
        .expect("conversation");

    // SQLite LIKE is case-insensitive for ASCII by default
    let results = list_conversations(&db_path, Some("rust")).expect("lowercase search");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].title, "Rust Programming");
}
