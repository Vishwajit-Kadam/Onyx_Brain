# Creative Production Studio

The creative studio creates production planning packages. It does not render actual video.

```bash
cargo run -- creative "Create a cinematic editing plan for a 3-hour original sci-fi feature film with scene breakdown, timeline, sound design, VFX notes, color grading notes, and final production package"
```

Outputs are markdown and JSON files under `sandbox/workspaces/{session_id}/`.

Supported deliverables include creative brief, story outline, scene breakdown, shot list, timeline plan, edit decision list, sound design plan, VFX plan, color grade plan, subtitle script, review checklist, final production report, and manifest.

Prompts that mention protected franchises are treated as broad inspiration only and include originality/IP caution.
