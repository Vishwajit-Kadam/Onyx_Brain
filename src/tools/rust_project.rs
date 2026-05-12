use anyhow::Result;
use std::path::PathBuf;

use crate::tools::FilesystemTool;

#[derive(Debug, Clone)]
pub struct RustProjectTool {
    fs: FilesystemTool,
}

#[derive(Debug, Clone)]
pub struct CreatedRustProject {
    pub path: PathBuf,
    pub files: Vec<PathBuf>,
}

impl RustProjectTool {
    pub fn new(fs: FilesystemTool) -> Self {
        Self { fs }
    }

    pub fn create_hello_world(&self, project_name: &str) -> Result<CreatedRustProject> {
        let safe_name = sanitize_project_name(project_name);
        let root = format!("projects/{safe_name}");
        let mut files = Vec::new();
        self.fs.create_dir(&format!("{root}/src"))?;
        self.fs.create_dir(&format!("{root}/tests"))?;
        files.push(self.fs.write_file(
            &format!("{root}/Cargo.toml"),
            &format!(
                "[package]\nname = \"{safe_name}\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\n"
            ),
        )?);
        files.push(self.fs.write_file(
            &format!("{root}/src/main.rs"),
            "fn main() {\n    println!(\"Hello from Onyx Brain sandbox!\");\n}\n",
        )?);
        files.push(self.fs.write_file(
            &format!("{root}/src/lib.rs"),
            "pub fn greeting() -> &'static str {\n    \"Hello from Onyx Brain sandbox!\"\n}\n\n#[cfg(test)]\nmod tests {\n    use super::*;\n\n    #[test]\n    fn greeting_mentions_onyx() {\n        assert!(greeting().contains(\"Onyx Brain\"));\n    }\n}\n",
        )?);
        files.push(self.fs.write_file(
            &format!("{root}/tests/basic.rs"),
            &format!(
                "#[test]\nfn crate_builds() {{\n    assert_eq!({safe_name}::greeting(), \"Hello from Onyx Brain sandbox!\");\n}}\n"
            ),
        )?);
        let path = self.fs.safe_path(&root)?;
        Ok(CreatedRustProject { path, files })
    }
}

pub fn sanitize_project_name(input: &str) -> String {
    let cleaned = input
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '_' || c == '-' {
                c
            } else {
                '_'
            }
        })
        .collect::<String>()
        .trim_matches('_')
        .to_lowercase();
    if cleaned.is_empty() {
        "mini_hello".to_string()
    } else {
        cleaned.replace('-', "_")
    }
}
