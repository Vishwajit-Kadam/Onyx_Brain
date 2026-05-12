use anyhow::Result;

use crate::tools::{CodeEditorTool, DiagnosticKind, DiagnosticReport};

pub fn retry_allowed(attempts: u32, max_attempts: u32) -> bool {
    attempts < max_attempts
}

pub fn apply_simple_rust_fix(
    editor: &CodeEditorTool,
    project_name: &str,
    diagnostic: &DiagnosticReport,
) -> Result<Option<String>> {
    match diagnostic.kind {
        DiagnosticKind::MissingFunction => {
            let excerpt = diagnostic.raw_output_excerpt.to_lowercase();
            for (name, body) in [
                (
                    "multiply",
                    "pub fn multiply(left: i32, right: i32) -> i32 {\n    left * right\n}\n",
                ),
                (
                    "divide",
                    "pub fn divide(left: i32, right: i32) -> Option<i32> {\n    if right == 0 {\n        None\n    } else {\n        Some(left / right)\n    }\n}\n",
                ),
                (
                    "modulo",
                    "pub fn modulo(left: i32, right: i32) -> Option<i32> {\n    if right == 0 {\n        None\n    } else {\n        Some(left % right)\n    }\n}\n",
                ),
                (
                    "power",
                    "pub fn power(left: i32, right: u32) -> i32 {\n    left.pow(right)\n}\n",
                ),
            ] {
                if excerpt.contains(name) {
                    editor.insert_function_in_lib_rs(project_name, body)?;
                    return Ok(Some(format!("Added missing {name} function.")));
                }
            }
            Ok(None)
        }
        DiagnosticKind::SyntaxError | DiagnosticKind::MissingModule => {
            let main_rs = editor.read_project_file(project_name, "src/main.rs")?;
            if !main_rs.contains("fn main()") {
                editor.write_project_file(
                    project_name,
                    "src/main.rs",
                    "fn main() {\n    println!(\"Project repaired by Onyx Brain.\");\n}\n",
                )?;
                return Ok(Some(
                    "Replaced invalid main.rs with a minimal main function.".to_string(),
                ));
            }
            Ok(None)
        }
        DiagnosticKind::TypeMismatch | DiagnosticKind::TestFailure => Ok(None),
        _ => Ok(None),
    }
}
