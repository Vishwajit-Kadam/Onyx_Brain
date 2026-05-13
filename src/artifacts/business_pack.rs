pub fn executive_summary(topic: &str) -> String {
    format!("# Executive Summary: {topic}\n\n## Purpose\nCreate a practical launch-ready package while preserving bounded safety claims.\n\n## Audience\nBuilders, contributors, and early reviewers.\n\n## Positioning\nExperimental bounded autonomous worker runtime. It is not AGI, not conscious, and has no LLM or network access by default.\n\n## Verification Notes\nVerify any public factual or market claims before publishing.\n")
}

pub fn product_spec(topic: &str) -> String {
    format!("# Product Spec: {topic}\n\n## Problem\nUsers need bounded automation for repeatable project and artifact workflows.\n\n## Goals\n- Create artifact packs\n- Validate outputs\n- Repair safe issues\n- Export deliverables\n\n## User Stories\n- As a user, I can run one bounded command and receive a complete local package.\n- As a maintainer, I can inspect reports, traces, and audits.\n\n## Acceptance Criteria\n- All required artifacts exist.\n- Final report references deliverables.\n- Safety boundaries are stated.\n")
}

pub fn competitive_analysis(topic: &str) -> String {
    format!("# Competitive Analysis: {topic}\n\n## Scope\nLocal, deterministic workflow runtime comparison.\n\n## Comparison Table\n| Dimension | Onyx Brain | Notes |\n| --- | --- | --- |\n| Default network | Disabled | Safer local operation |\n| Artifacts | Markdown packs | Export-ready text |\n| Recovery | Journal/snapshot/doctor | Reliability-first |\n\n## Verification Notes\nExternal product comparisons require independent verification.\n")
}

pub fn swot_analysis(topic: &str) -> String {
    format!("# SWOT Analysis: {topic}\n\n## Strengths\n- Disk-backed sparse design\n- Bounded autonomy\n- Recovery tools\n\n## Weaknesses\n- No LLM by default\n- Markdown-only presentation exports\n\n## Opportunities\n- Plugin boundaries\n- Better examples and benchmarks\n\n## Threats\n- Overstated claims\n- Unsafe user expectations\n")
}

pub fn metrics_plan(topic: &str) -> String {
    format!("# Metrics Plan: {topic}\n\n## Quality Metrics\n- Completeness score\n- Quality review score\n- Consistency score\n- Regression status\n\n## Safety Metrics\n- Safety stops\n- Doctor critical issues\n- Rollback readiness\n\n## Adoption Metrics\n- Example runs completed\n- Issues reported\n- Docs improvements merged\n")
}

pub fn launch_checklist(topic: &str) -> String {
    format!("# Launch Checklist: {topic}\n\n- [ ] Review generated artifacts\n- [ ] Verify external claims\n- [ ] Run `cargo fmt`\n- [ ] Run `cargo check`\n- [ ] Run `cargo test -- --nocapture`\n- [ ] Run `cargo run -- doctor`\n- [ ] Run `cargo run -- regression-check`\n- [ ] Inspect export package\n")
}
