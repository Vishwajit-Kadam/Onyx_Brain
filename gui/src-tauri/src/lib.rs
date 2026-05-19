#[cfg_attr(mobile, tauri::mobile_entry_point)]
#[derive(serde::Serialize)]
struct ChatResponse {
  response: String,
}

#[tauri::command]
fn send_chat_message(input: String) -> Result<ChatResponse, String> {
  let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    .parent()
    .and_then(std::path::Path::parent)
    .map(std::path::Path::to_path_buf)
    .ok_or_else(|| "Unable to resolve Onyx Brain project root".to_string())?;
  let api = onyx_brain::app_api::AppApi::new(root);
  let output = api
    .send_chat_message(&input, onyx_brain::conversation::ConversationMode::Standard)
    .map_err(|error| error.to_string())?;
  Ok(ChatResponse {
    response: output.response,
  })
}

pub fn run() {
  tauri::Builder::default()
    .setup(|app| {
      if cfg!(debug_assertions) {
        app.handle().plugin(
          tauri_plugin_log::Builder::default()
            .level(log::LevelFilter::Info)
            .build(),
        )?;
      }
      Ok(())
    })
    .invoke_handler(tauri::generate_handler![send_chat_message])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
