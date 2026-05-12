use onyx_brain::tools::{FilesystemTool, TerminalTool};

#[test]
fn filesystem_tool_rejects_path_traversal() {
    let temp = tempfile::tempdir().expect("tempdir");
    let fs = FilesystemTool::new(temp.path()).expect("fs");
    assert!(fs.write_file("../escape.txt", "nope").is_err());
}

#[test]
fn terminal_tool_rejects_dangerous_commands() {
    let temp = tempfile::tempdir().expect("tempdir");
    let terminal = TerminalTool::new(temp.path()).expect("terminal");
    assert!(terminal.run(&["rm", "-rf", "."], temp.path()).is_err());
}
