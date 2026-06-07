use helios_chat_lib::db::{
    append_message, create_conversation, delete_conversation, list_conversations, list_messages,
    migrate, update_conversation, update_message,
};

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
