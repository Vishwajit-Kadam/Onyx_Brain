pub fn markdown_report(title: &str, sections: &[(&str, String)]) -> String {
    let mut out = format!("# {title}\n\n");
    for (heading, body) in sections {
        out.push_str(&format!("## {heading}\n{body}\n\n"));
    }
    out
}
