# Artifacts

Onyx Brain v0.0.2 creates local markdown artifacts inside the sandbox. It does not create binary PPTX files, browse the web, or use an LLM by default.

## Artifact Packs

Multi-artifact work is grouped into an artifact pack. A pack contains:

- generated files
- dependency relationships
- validation scores
- assumptions and limitations
- final report references

Pack metadata is stored at:

```text
sandbox/workspaces/{session_id}/artifacts/artifact_pack.json
```

The lightweight index is:

```text
data/indexes/artifact_pack_index.json
```

## Commands

```bash
cargo run -- artifact-packs
cargo run -- artifact-pack-inspect latest
cargo run -- review-artifacts latest
cargo run -- export-package latest
cargo run -- exports
```

## Learning Pack Outputs

A learning-pack prompt can create:

- `presentation.md`
- `speaker_notes.md`
- `design_guide.md`
- `study_guide.md`
- `quiz.md`
- `glossary.md`
- `assumptions.md`
- `limitations.md`
- `artifact_manifest.json`
- `artifact_pack.json`
- `final_report.md`

All writes stay under the sandbox/workspace layout.

## Advanced Packs

Advanced v0.0.2 packs can include launch/startup kits, technical report packs, product spec packs, documentation packs, and learning packs. Export packages include `export_manifest.json` with file sizes and hashes, plus reports such as work contract, done definition, execution trace, quality review, final audit, and report card.
