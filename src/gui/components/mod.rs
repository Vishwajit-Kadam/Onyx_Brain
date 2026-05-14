pub fn status_badge(label: &str, ok: bool) -> String {
    format!("{}: {}", label, if ok { "ok" } else { "review" })
}
