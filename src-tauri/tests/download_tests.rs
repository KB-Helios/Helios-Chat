use helios_chat_lib::download::verify_sha256;

#[test]
fn verifies_expected_sha256_for_downloaded_file() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("model.gguf");
    std::fs::write(&path, b"helios").expect("write file");

    assert!(verify_sha256(
        &path,
        "48582bd628b7c80064780ba9ecce2d435db042b40bd4335a7cea4b4c254e8178"
    )
    .expect("hash"));
    assert!(!verify_sha256(
        &path,
        "0000000000000000000000000000000000000000000000000000000000000000"
    )
    .expect("hash"));
}
