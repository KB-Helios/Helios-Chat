use helios_chat_lib::setup::{choose_build_backend, detect_prereq_from_path, BuildBackend, ToolStatus};

#[test]
fn chooses_cuda_when_compiler_and_cuda_are_available() {
    let tools = vec![
        ToolStatus::present("git", "git.exe"),
        ToolStatus::present("cmake", "cmake.exe"),
        ToolStatus::present("cl", "cl.exe"),
        ToolStatus::present("nvcc", "nvcc.exe"),
    ];

    assert_eq!(choose_build_backend(&tools), BuildBackend::Cuda);
}

#[test]
fn falls_back_to_cpu_when_cuda_is_missing_but_cxx_tools_exist() {
    let tools = vec![
        ToolStatus::present("git", "git.exe"),
        ToolStatus::present("cmake", "cmake.exe"),
        ToolStatus::present("cl", "cl.exe"),
        ToolStatus::missing("nvcc", "Install CUDA Toolkit for GPU acceleration"),
    ];

    assert_eq!(choose_build_backend(&tools), BuildBackend::Cpu);
}

#[test]
fn marks_missing_tool_with_guided_install_url() {
    let status = detect_prereq_from_path("definitely-not-a-helios-tool", None);

    assert!(!status.present);
    assert!(status.install_url.starts_with("https://"));
}
