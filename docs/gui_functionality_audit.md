# GUI Functionality Audit

Onyx Brain v0.0.4 now exposes a Rust-native `eframe/egui` GUI through `cargo run -- gui`. Every visible element is wired to a real AppApi action, a view transition, a persisted setting, a clear disabled state, or demo data marked as demo.

| Screen / view | Element | Current behavior | Expected behavior | Backend command/API needed | Final status |
| --- | --- | --- | --- | --- | --- |
| Home / New task | Composer send | Routes chat, autonomy, or creative prompts | Never silently ignore user input | `send_chat_message`, `run_autonomous_goal`, `run_creative_project` | Working |
| Home / New task | Plus button | Creates a safe local GUI task entry | Task appears in Tasks/search | `create_task` | Working |
| Home / New task | Mode selector | Changes conversation mode | Used by chat calls | `send_chat_message` | Working |
| Home / New task | Personality selector | Changes response personality | Used by chat calls and settings | `send_chat_message`, settings save | Working |
| Home / New task | Autonomy selector | Changes bounded autonomy level | Used by autonomy calls | `run_autonomous_goal` | Working |
| Home / New task | Quick chips | Prefill composer with useful prompts | Help start real actions | GUI state | Working |
| Home / New task | More | Opens command palette | Command palette actions work | GUI state / AppApi | Working |
| Scheduled | Create scheduled task | Opens disabled explanation | No hidden background jobs in v0.0.4 | None | Disabled intentionally |
| Scheduled | Suggestion rows | Preview-only text | No fake scheduling | None | Disabled intentionally |
| Search modal | Search input | Searches local Onyx data only | Filter tasks, artifacts, sessions, memories, projects | `search_all` | Working |
| Search modal | Close | Closes modal | Return to current view | GUI state | Working |
| Search modal | New task row | Opens Home | Start a real prompt | GUI state | Working |
| Search modal | Result rows | Opens related view | Navigate to matching area | `search_all` action field | Working |
| Library | All filter | Shows all loaded library rows | Display artifacts/packs/sessions | `list_artifacts`, `list_artifact_packs`, `list_sessions` | Working |
| Library | My favorites | Shows explanation | Favorites not implemented | None | Disabled intentionally |
| Library | Search files | Filters loaded library items | Local filtering only | GUI state | Working |
| Library | Grid/list toggle | Changes layout | Visual mode toggle | GUI state | Working |
| Library | New task | Opens Home | Start new work | GUI state | Working |
| Projects | New project | Creates safe project task entry | User can explicitly run later | `create_task` | Wired now |
| Projects | Project list | Lists backend projects | Show registry projects | `list_projects` | Working |
| Projects | Create thread in project | Prefills Home prompt | Thread-like project prompt | GUI state | Working |
| Projects | All tasks | Opens Tasks view | Inspect task list | `list_tasks` | Working |
| Settings | General tab | Displays language and theme | Language display-only, theme functional | GUI settings | Working / Disabled intentionally |
| Settings | Light/Dark/Auto | Applies and persists theme | Immediate egui visual update | `data/config/gui_settings.json` | Working |
| Settings | Personalization | Selects/saves personality | Persist default personality | GUI settings | Working |
| Settings | Skills | Shows local memory/skill summary | List available local summaries | `list_memories` | Working |
| Settings | Connectors | Disabled explanation | External connectors disabled | None | Disabled intentionally |
| Settings | Get help | Shows commands/docs | Local help text | None | Working |
| Settings | Close | Returns Home | Close settings view | GUI state | Working |
| Chat | Send message | Adds user message and Onyx response | Working transcript | `send_chat_message` | Working |
| Chat | Mode select | Changes mode | Used by next send | GUI state / AppApi | Working |
| Chat | Personality select | Changes personality | Used by next send | GUI state / AppApi | Working |
| Autonomy | Run goal | Runs bounded autonomous worker | Explicit user-launched run only | `run_autonomous_goal` | Working |
| Autonomy | Level selection | Selects autonomy level | Safe bounded level | GUI state | Working |
| Creative Studio | Generate plan | Creates creative planning package | No rendered video claim | `run_creative_project` | Working |
| Creative Studio | Project type | Selects typed project intent | Passed to API as typed model | `run_creative_project` | Working |
| Tasks | Refresh | Reloads task list | Show goals and GUI tasks | `list_tasks` | Working |
| Tasks | Inspect task rows | Displays task status/subtitle | Clear task state | `list_tasks` | Working |
| Artifacts | Refresh | Reloads artifacts/packs | Show generated files | `list_artifacts`, `list_artifact_packs` | Working |
| Artifacts | Export latest | Attempts backend export | Friendly empty/error state if none | `export_latest_package` | Working |
| Artifacts | Inspect latest | Explains current limitation | No silent dead button | None | Needs future backend |
| Memory | Refresh | Reloads memory summary | Show local memory counts | `list_memories` | Working |
| Safety | Run Doctor | Runs doctor and reports issues | Local state health check | `run_doctor` | Working |
| Safety | Regression Check | Runs regression guard | Structured pass/fail toast | `run_regression_check` | Working |
| Safety | Maintain | Runs bounded maintenance | Refresh status after maintenance | `run_maintain` | Working |
| System | Refresh status | Reloads brain/safety status | Show current runtime state | `get_brain_status`, `get_safety_status` | Working |
| System | Workspace labels | Shows current root/data/sandbox | No unrestricted file browsing | `get_current_workspace` | Working |
| Command palette | Ctrl+K | Opens command palette | Quick command execution | GUI state / AppApi | Working |
| Command palette | Doctor/regression/export | Runs matching actions | Same as visible buttons | AppApi | Working |

## Intentional limits

- Scheduled autonomous jobs are disabled in v0.0.4. Onyx Brain does not run hidden background workers from the GUI.
- External connectors are disabled in v0.0.4.
- Favorites are not implemented yet and clearly report that limitation.
- Inspect-latest artifact deep inspection is not a silent button; the GUI reports the limitation and keeps export/listing functional.
