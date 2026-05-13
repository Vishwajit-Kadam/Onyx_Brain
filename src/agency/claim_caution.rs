pub fn add_claim_caution(content: &str, citations_requested: bool) -> String {
    let mut out = content.to_string();
    if !out.to_lowercase().contains("verification notes") {
        out.push_str("\n## Verification Notes\nThis document was generated without network access by default. Verify time-sensitive or factual claims externally before publication.\n");
    }
    if citations_requested && !out.to_lowercase().contains("citation placeholders") {
        out.push_str(
            "\n## Citation Placeholders\n- [Verify source before use]\n- [Add citation manually]\n",
        );
    }
    out
}
