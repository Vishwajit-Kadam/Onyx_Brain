use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::io::{self, Write};

use crate::{
    conversation::{
        append_turn, apply_personality, check_conversation_safety, extract_topic,
        generate_response_plan, load_personality, mode_name, render_response,
        repair_unsafe_response, save_conversation_memory, score_response, start_conversation,
        ConversationMode, ConversationState, ResponseQualityReport,
    },
    storage::DiskStore,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationEngine {
    pub session_id: String,
    pub mode: ConversationMode,
    pub state: ConversationState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationTurnOutput {
    pub session_id: String,
    pub mode: String,
    pub response: String,
    pub quality: ResponseQualityReport,
    pub memory_saved: Option<String>,
}

pub fn run_conversation_turn(
    store: &DiskStore,
    mode: ConversationMode,
    input: &str,
    show_quality: bool,
) -> Result<ConversationTurnOutput> {
    let mut state = start_conversation(store, mode.clone())?;
    state.current_topic = extract_topic(input);
    let personality = load_personality(store)?;
    let input_safety = check_conversation_safety(input);
    let mut response = if input_safety.allowed {
        let plan = generate_response_plan(mode.clone(), input);
        render_response(&plan, input)
    } else {
        input_safety.safe_response.unwrap_or_else(|| {
            "I cannot help with that unsafe request. I can suggest a bounded, local alternative."
                .to_string()
        })
    };
    response = apply_personality(&response, &personality);
    response = repair_unsafe_response(&response);
    let mut quality = score_response(input, &response, &mode);
    if quality.overall < 0.72 || !quality.issues.is_empty() {
        response = deterministic_repair(input, &response, &mode);
        quality = score_response(input, &response, &mode);
    }
    if show_quality {
        response.push_str(&format!(
            "\n\n## Response Quality\n- overall: {:.2}\n- safety: {:.2}\n- honesty: {:.2}\n",
            quality.overall, quality.safety, quality.honesty
        ));
    }
    append_turn(store, &mut state, input, &response, &json!(quality))?;
    let memory = save_conversation_memory(store, &state).ok();
    Ok(ConversationTurnOutput {
        session_id: state.session_id,
        mode: mode_name(&mode).to_string(),
        response,
        quality,
        memory_saved: memory.map(|row| row.id),
    })
}

pub fn chat_loop(store: &DiskStore) -> Result<()> {
    println!("Onyx Brain {} chat", crate::ONYX_VERSION);
    println!("Mode: standard");
    println!("Type /help for commands. Type /exit to quit.");
    let mut mode = ConversationMode::Standard;
    loop {
        print!("> ");
        io::stdout().flush()?;
        let mut input = String::new();
        if io::stdin().read_line(&mut input)? == 0 {
            break;
        }
        let input = input.trim();
        if input == "/exit" {
            break;
        }
        if input == "/help" {
            println!("/mode <name>, /summary, /save, /exit");
            continue;
        }
        if let Some(next) = input.strip_prefix("/mode ") {
            if let Some(parsed) = crate::conversation::parse_mode(next) {
                mode = parsed;
                println!("Mode: {}", mode_name(&mode));
            } else {
                println!("Unknown mode.");
            }
            continue;
        }
        if input == "/summary" || input == "/save" {
            println!("Conversation summaries are saved after each turn.");
            continue;
        }
        if input.is_empty() {
            continue;
        }
        let output = run_conversation_turn(store, mode.clone(), input, false)?;
        println!("{}", output.response);
    }
    Ok(())
}

fn deterministic_repair(input: &str, response: &str, mode: &ConversationMode) -> String {
    if response.trim().is_empty() {
        return render_response(&generate_response_plan(mode.clone(), input), input);
    }
    let mut repaired = response.to_string();
    if !repaired.contains("##") {
        repaired = format!("# {}\n\n{}", mode_name(mode), repaired);
    }
    if !repaired.to_lowercase().contains("no web")
        && matches!(mode, ConversationMode::ResearchOutline)
    {
        repaired.push_str("\n\n## Verification Notes\nNo web access was used. Add citation placeholders and verify externally.\n");
    }
    repair_unsafe_response(&repaired)
}
