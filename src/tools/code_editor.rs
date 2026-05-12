use anyhow::{Context, Result};
use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::{
    tools::{transactional_write_project_file, FilesystemTool},
    utils::errors::OnyxError,
};

#[derive(Debug, Clone)]
pub struct CodeEditorTool {
    fs: FilesystemTool,
}

impl CodeEditorTool {
    pub fn new(sandbox: impl AsRef<Path>) -> Result<Self> {
        Ok(Self {
            fs: FilesystemTool::new(sandbox)?,
        })
    }

    pub fn read_project_file(&self, project: &str, path: &str) -> Result<String> {
        self.fs.read_file(&project_file(project, path)?)
    }

    pub fn write_project_file(&self, project: &str, path: &str, content: &str) -> Result<PathBuf> {
        validate_utf8(content)?;
        let written = transactional_write_project_file(self.fs.sandbox(), project, path, content)?;
        self.log_edit(project, &format!("write {path}"))?;
        Ok(written)
    }

    pub fn append_project_file(&self, project: &str, path: &str, content: &str) -> Result<PathBuf> {
        validate_utf8(content)?;
        let previous = self.read_project_file(project, path).unwrap_or_default();
        self.write_project_file(project, path, &format!("{previous}{content}"))
    }

    pub fn replace_in_project_file(
        &self,
        project: &str,
        path: &str,
        old: &str,
        new: &str,
    ) -> Result<bool> {
        validate_utf8(new)?;
        let previous = self.read_project_file(project, path)?;
        if !previous.contains(old) {
            return Ok(false);
        }
        let next = previous.replace(old, new);
        self.write_project_file(project, path, &next)?;
        self.log_edit(project, &format!("replace {path}"))?;
        Ok(true)
    }

    pub fn insert_function_in_lib_rs(&self, project: &str, function: &str) -> Result<bool> {
        let mut content = self
            .read_project_file(project, "src/lib.rs")
            .unwrap_or_default();
        let name = function
            .split_once("fn ")
            .and_then(|(_, rest)| rest.split('(').next())
            .unwrap_or_default();
        if !name.is_empty() && content.contains(&format!("fn {name}(")) {
            return Ok(false);
        }
        if !content.ends_with('\n') {
            content.push('\n');
        }
        content.push('\n');
        content.push_str(function);
        if !content.ends_with('\n') {
            content.push('\n');
        }
        self.write_project_file(project, "src/lib.rs", &content)?;
        Ok(true)
    }

    pub fn insert_test_in_tests_file(
        &self,
        project: &str,
        file_name: &str,
        test: &str,
    ) -> Result<bool> {
        let path = format!("tests/{file_name}");
        let mut content = self.read_project_file(project, &path).unwrap_or_default();
        let test_name = test
            .split_once("fn ")
            .and_then(|(_, rest)| rest.split('(').next())
            .unwrap_or_default();
        if !test_name.is_empty() && content.contains(&format!("fn {test_name}(")) {
            return Ok(false);
        }
        if !content.ends_with('\n') {
            content.push('\n');
        }
        content.push('\n');
        content.push_str(test);
        if !content.ends_with('\n') {
            content.push('\n');
        }
        self.write_project_file(project, &path, &content)?;
        Ok(true)
    }

    pub fn update_readme_section(&self, project: &str, heading: &str, content: &str) -> Result<()> {
        let marker_start = format!("<!-- ONYX:{heading}:START -->");
        let marker_end = format!("<!-- ONYX:{heading}:END -->");
        let section = format!("{marker_start}\n{content}\n{marker_end}");
        let current = self
            .read_project_file(project, "README.md")
            .unwrap_or_default();
        let updated = if current.contains(&marker_start) && current.contains(&marker_end) {
            replace_between(&current, &marker_start, &marker_end, &section)
        } else if current.trim().is_empty() {
            format!("# {project}\n\n{section}\n")
        } else {
            format!("{}\n\n{}\n", current.trim_end(), section)
        };
        self.write_project_file(project, "README.md", &updated)?;
        Ok(())
    }

    pub fn ensure_module_exists(&self, project: &str, module: &str) -> Result<bool> {
        let module_file = format!("src/{module}.rs");
        if self.read_project_file(project, &module_file).is_ok() {
            return Ok(false);
        }
        self.write_project_file(project, &module_file, "")?;
        let lib = self
            .read_project_file(project, "src/lib.rs")
            .unwrap_or_default();
        if !lib.contains(&format!("pub mod {module};")) {
            self.write_project_file(project, "src/lib.rs", &format!("pub mod {module};\n{lib}"))?;
        }
        Ok(true)
    }

    pub fn replace_between_markers(
        &self,
        project: &str,
        path: &str,
        start_marker: &str,
        end_marker: &str,
        replacement: &str,
    ) -> Result<bool> {
        let current = self.read_project_file(project, path)?;
        if !(current.contains(start_marker) && current.contains(end_marker)) {
            return Ok(false);
        }
        let next = replace_between(current.as_str(), start_marker, end_marker, replacement);
        self.write_project_file(project, path, &next)?;
        Ok(true)
    }

    fn log_edit(&self, project: &str, message: &str) -> Result<()> {
        let log_path = project_file(project, ".onyx_edits.log")?;
        let existing = self.fs.read_file(&log_path).unwrap_or_default();
        self.fs
            .write_file(&log_path, &format!("{existing}{message}\n"))
            .with_context(|| format!("logging edit for {project}"))?;
        Ok(())
    }
}

fn project_file(project: &str, path: &str) -> Result<String> {
    if project.contains("..") || path.contains("..") || Path::new(project).is_absolute() {
        return Err(OnyxError::SandboxEscape.into());
    }
    if Path::new(path).is_absolute() {
        return Err(OnyxError::SandboxEscape.into());
    }
    Ok(format!("projects/{project}/{path}"))
}

fn validate_utf8(content: &str) -> Result<()> {
    let _ = fs::metadata(".").ok();
    if content.contains('\0') {
        return Err(anyhow::anyhow!("refusing to write non-text content"));
    }
    Ok(())
}

fn replace_between(input: &str, start_marker: &str, end_marker: &str, replacement: &str) -> String {
    let Some(start) = input.find(start_marker) else {
        return input.to_string();
    };
    let Some(end_relative) = input[start..].find(end_marker) else {
        return input.to_string();
    };
    let end = start + end_relative + end_marker.len();
    format!("{}{}{}", &input[..start], replacement, &input[end..])
}
