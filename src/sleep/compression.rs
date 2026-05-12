pub fn summarize_text(input: &str, max_chars: usize) -> String {
    input.chars().take(max_chars).collect()
}
