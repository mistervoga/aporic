# Tutor Onboarding Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Give a new user a five-minute, visually explicit path from an observation to a trace while making tutor and TUI help complete and discoverable.

**Architecture:** Keep the CLI's existing `dispatch` function as the only command implementation. Add a small stateful quickstart around it in `src/tutor.rs`, route the tutor before the persistent database is opened in `src/main.rs`, and add a modal help flag to the existing TUI `App` state.

**Tech Stack:** Rust 2021, Clap 4, rusqlite 0.31, Ratatui 0.30, Crossterm 0.29, Rust standard-library tests.

## Global Constraints

- Add no dependency and no persistent configuration.
- Keep output plain-text and understandable without color or Unicode glyphs.
- Keep domain behavior in the existing domain and dispatch paths.
- Do not redesign the domain model, command syntax, database schema, or Obsidian format.
- Tutor help and menu navigation must never mutate entries.
- The tutor must touch only its in-memory sandbox.
- Document only behavior implemented in the current binary; do not present roadmap features as available.

---

## File map

- Create `tests/cli_startup.rs`: black-box checks for tutor isolation and the public CLI help contract.
- Modify `src/main.rs`: correct stale help copy and start the tutor before opening the persistent database.
- Modify `src/tutor.rs`: tutor controls, task-oriented menu, resumable quickstart state, dynamic ID prompts, and focused unit tests.
- Modify `src/tui.rs`: modal key-reference overlay, centralized key handling, and focused unit tests.
- Modify `README.md`: describe the new default quickstart and both help surfaces.

### Task 1: Restore the tutor sandbox boundary

**Files:**
- Create: `tests/cli_startup.rs`
- Modify: `src/main.rs:133-134,196-201`

**Interfaces:**
- Consumes: `Command::Tutor`, `tutor::run()`, and the existing `db::connect_and_init()` path.
- Produces: an early `Command::Tutor` route that returns before persistent database initialization.

- [ ] **Step 1: Write failing black-box tests**

Create `tests/cli_startup.rs`:

```rust
use std::io::Write;
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn tutor_does_not_create_the_normal_database() {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let data_home = std::env::temp_dir().join(format!(
        "aporic-tutor-test-{}-{nonce}",
        std::process::id()
    ));

    let mut child = Command::new(env!("CARGO_BIN_EXE_aporic"))
        .arg("tutor")
        .env("XDG_DATA_HOME", &data_home)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(b"quit\n").unwrap();

    assert!(child.wait().unwrap().success());
    let database_created = data_home.join("aporic/aporic.db").exists();
    if data_home.exists() {
        std::fs::remove_dir_all(&data_home).unwrap();
    }
    assert!(!database_created);
}

#[test]
fn help_does_not_advertise_legacy_ids() {
    let output = Command::new(env!("CARGO_BIN_EXE_aporic"))
        .arg("--help")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert!(output.status.success());
    assert!(stdout.contains("UUID or unique UUID prefix"));
    assert!(!stdout.contains("legacy task ID"));
}
```

- [ ] **Step 2: Run the tests and verify the isolation test fails**

Run:

```bash
cargo test --test cli_startup
```

Expected: `tutor_does_not_create_the_normal_database` fails because
`db::connect_and_init()` creates `aporic/aporic.db`; the help assertion also
fails because the old phrase is still present.

- [ ] **Step 3: Add the early route and correct the help description**

Change the `Show` variant documentation in `src/main.rs` to:

```rust
    /// Show one entry by UUID or unique UUID prefix
    Show { id: String },
```

Replace `main` with:

```rust
fn main() -> Result<()> {
    let cli = Cli::parse();
    if matches!(&cli.command, Command::Tutor) {
        return tutor::run();
    }

    let mut conn = db::connect_and_init()?;
    let project = effective_project(cli.project.as_deref());
    dispatch(&mut conn, &cli, project.as_deref(), &cli.command)
}
```

Leave the existing `Command::Tutor => tutor::run()?` dispatch arm in place;
the shared dispatcher still needs to remain exhaustive and usable in tests.

- [ ] **Step 4: Run the focused tests**

Run:

```bash
cargo test --test cli_startup
```

Expected: `2 passed; 0 failed`.

- [ ] **Step 5: Commit the boundary fix**

```bash
git add src/main.rs tests/cli_startup.rs
git commit -m "fix: keep tutor outside the user database"
```

### Task 2: Add tutor-wide help controls

**Files:**
- Modify: `src/tutor.rs:7-134`

**Interfaces:**
- Consumes: `Cli::try_parse_from`, `Cli::command()`, `dispatch`, and the existing lesson functions.
- Produces: `TutorControl`, `StepOutcome`, `command_help() -> String`, and a `step(...) -> Result<StepOutcome>` input path used by the quickstart in Task 3.

- [ ] **Step 1: Write failing parser and help tests**

Append this test module to `src/tutor.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognizes_controls_without_treating_cli_commands_as_controls() {
        assert_eq!(tutor_control("help"), Some(TutorControl::Help));
        assert_eq!(tutor_control("commands"), Some(TutorControl::Commands));
        assert_eq!(tutor_control("menu"), Some(TutorControl::Menu));
        assert_eq!(tutor_control("q"), Some(TutorControl::Quit));
        assert_eq!(tutor_control("quit"), Some(TutorControl::Quit));
        assert_eq!(tutor_control("exit"), Some(TutorControl::Quit));
        assert_eq!(tutor_control("observe q"), None);
    }

    #[test]
    fn command_reference_comes_from_clap() {
        let help = command_help();
        assert!(help.contains("Commands:"));
        assert!(help.contains("observe"));
        assert!(help.contains("tutor"));
        assert!(!help.contains("ai examine"));
    }
}
```

- [ ] **Step 2: Run the focused tests and verify they fail to compile**

Run:

```bash
cargo test tutor::tests
```

Expected: compilation fails because `TutorControl`, `tutor_control`, and
`command_help` do not exist.

- [ ] **Step 3: Add the minimal control types and generated command help**

Change the Clap import and add the following immediately below `ExitLesson`:

```rust
use clap::{CommandFactory, Parser};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TutorControl {
    Help,
    Commands,
    Menu,
    Quit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StepOutcome {
    Complete,
    Menu,
    Quit,
}

fn tutor_control(input: &str) -> Option<TutorControl> {
    match input.trim() {
        "help" => Some(TutorControl::Help),
        "commands" => Some(TutorControl::Commands),
        "menu" => Some(TutorControl::Menu),
        "q" | "quit" | "exit" => Some(TutorControl::Quit),
        _ => None,
    }
}

fn command_help() -> String {
    let mut bytes = Vec::new();
    Cli::command().write_long_help(&mut bytes).unwrap();
    String::from_utf8(bytes).unwrap()
}
```

Do not maintain a second command list in the tutor.

- [ ] **Step 4: Make `step` handle controls before parsing CLI input**

Change its signature to:

```rust
fn step(
    conn: &mut Connection,
    intro: &str,
    hint: &str,
    expect: impl Fn(&Cli) -> bool,
) -> Result<StepOutcome>
```

Change end-of-file handling inside its input loop to:

```rust
        let Some(line) = read_line("aporic tutor> ")? else {
            return Ok(StepOutcome::Quit);
        };
```

Replace the current special-case exit block with:

```rust
        match tutor_control(trimmed) {
            Some(TutorControl::Help) => {
                println!("Current step: {hint}");
                continue;
            }
            Some(TutorControl::Commands) => {
                println!("{}", command_help());
                continue;
            }
            Some(TutorControl::Menu) => return Ok(StepOutcome::Menu),
            Some(TutorControl::Quit) => return Ok(StepOutcome::Quit),
            None => {}
        }
```

Change the successful return at the bottom of `step` to:

```rust
        println!();
        return Ok(StepOutcome::Complete);
```

Add a compatibility wrapper for the existing topic lessons:

```rust
fn topic_step(
    conn: &mut Connection,
    intro: &str,
    hint: &str,
    expect: impl Fn(&Cli) -> bool,
) -> Result<()> {
    match step(conn, intro, hint, expect)? {
        StepOutcome::Complete => Ok(()),
        StepOutcome::Menu => Err(anyhow!(ExitLesson)),
        StepOutcome::Quit => Err(anyhow!(ExitTutor)),
    }
}
```

Add `ExitTutor` beside `ExitLesson`, with the same `Display` and `Error`
behavior:

```rust
#[derive(Debug)]
struct ExitTutor;

impl std::fmt::Display for ExitTutor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "exit tutor")
    }
}
impl std::error::Error for ExitTutor {}
```

Mechanically rename every existing lesson call from `step(` to `topic_step(`;
do not change its lesson predicate. In both CLI parse-error branches, print
the existing hint followed by `Type help to repeat the current instructions.`
so invalid input never advances and always points back to contextual help.

- [ ] **Step 5: Update the menu loop to distinguish menu and quit exits**

In the existing lesson-result error handling, replace the block with:

```rust
        if let Err(err) = outcome {
            if err.downcast_ref::<ExitTutor>().is_some() {
                break;
            }
            if err.downcast_ref::<ExitLesson>().is_none() {
                return Err(err);
            }
        }
```

At the menu prompt, keep contextual help and command help distinct:

```rust
            "help" => {
                println!("Choose a topic number, or use commands, menu, or quit.");
                continue;
            }
            "commands" => {
                println!("{}", command_help());
                continue;
            }
            "menu" => continue,
```

- [ ] **Step 6: Run formatter and focused tests**

Run:

```bash
cargo fmt -- --check
cargo test tutor::tests
```

Expected: formatting succeeds and both tutor tests pass.

- [ ] **Step 7: Commit tutor controls**

```bash
git add src/tutor.rs
git commit -m "feat: add contextual tutor help"
```

### Task 3: Make the five-section quickstart the default

**Files:**
- Modify: `src/tutor.rs:22-353`

**Interfaces:**
- Consumes: `StepOutcome` and `step` from Task 2; `domain::list_entries` and the existing in-memory connection.
- Produces: `Quickstart::run_next(&mut self, &mut Connection) -> Result<StepOutcome>`, a resumable six-command state machine rendered as five user-facing sections.

- [ ] **Step 1: Write failing progress and generated-ID tests**

Add these tests inside `tutor::tests`:

```rust
    #[test]
    fn quickstart_progress_groups_both_links_into_connect() {
        assert!(quickstart_progress(0).contains("[>] Observe"));
        assert!(quickstart_progress(3).contains("[>] Connect"));
        assert!(quickstart_progress(4).contains("[>] Connect"));
        assert!(quickstart_progress(5).contains("[>] Trace"));
    }

    #[test]
    fn connect_commands_use_generated_id_prefixes() {
        let quickstart = Quickstart {
            stage: 3,
            observation: Some("01911111-observation".into()),
            question: Some("01922222-question".into()),
            action: Some("01933333-action".into()),
        };

        assert_eq!(
            quickstart.connect_commands(),
            (
                "link 01922222 derived-from 01911111".to_string(),
                "link 01922222 motivates 01933333".to_string(),
            )
        );
    }

    #[test]
    fn non_completed_outcomes_do_not_advance_quickstart() {
        let mut quickstart = Quickstart::default();
        quickstart.apply_outcome(StepOutcome::Menu);
        assert_eq!(quickstart.stage, 0);
        quickstart.apply_outcome(StepOutcome::Quit);
        assert_eq!(quickstart.stage, 0);
    }
```

Also append this black-box check to `tests/cli_startup.rs`:

```rust
#[test]
fn wrong_tutor_input_stays_on_the_current_step() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_aporic"))
        .arg("tutor")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(b"status\nquit\n")
        .unwrap();
    let output = child.wait_with_output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert!(output.status.success());
    assert!(stdout.contains("Not quite what this step needs"));
    assert!(stdout.contains("step 1 of 5"));
    assert!(!stdout.contains("step 2 of 5"));
}
```

- [ ] **Step 2: Run the tests and verify they fail to compile**

Run:

```bash
cargo test tutor::tests
```

Expected: compilation fails because `Quickstart` and `quickstart_progress` do
not exist.

- [ ] **Step 3: Add quickstart state, progress rendering, and ID helpers**

Add above `run`:

```rust
#[derive(Default)]
struct Quickstart {
    stage: usize,
    observation: Option<String>,
    question: Option<String>,
    action: Option<String>,
}

impl Quickstart {
    fn complete(&self) -> bool {
        self.stage == 6
    }

    fn connect_commands(&self) -> (String, String) {
        let observation = id_prefix(self.observation.as_deref().unwrap());
        let question = id_prefix(self.question.as_deref().unwrap());
        let action = id_prefix(self.action.as_deref().unwrap());
        (
            format!("link {question} derived-from {observation}"),
            format!("link {question} motivates {action}"),
        )
    }

    fn apply_outcome(&mut self, outcome: StepOutcome) {
        if outcome == StepOutcome::Complete {
            self.stage += 1;
        }
    }
}

fn id_prefix(id: &str) -> &str {
    &id[..8.min(id.len())]
}

fn quickstart_progress(stage: usize) -> String {
    let current = match stage {
        0 => 0,
        1 => 1,
        2 => 2,
        3 | 4 => 3,
        _ => 4,
    };
    let labels = ["Observe", "Ask", "Act", "Connect", "Trace"];
    let flow = labels
        .iter()
        .enumerate()
        .map(|(index, label)| match index.cmp(&current) {
            std::cmp::Ordering::Less => format!("[x] {label}"),
            std::cmp::Ordering::Equal => format!("[>] {label}"),
            std::cmp::Ordering::Greater => format!("[ ] {label}"),
        })
        .collect::<Vec<_>>()
        .join(" -> ");
    format!("Aporic quickstart                 step {} of 5\n\n{flow}", current + 1)
}

fn latest_id(conn: &Connection, kind: EntryKind) -> Result<String> {
    domain::list_entries(
        conn,
        EntryFilter {
            kind: Some(kind),
            ..EntryFilter::default()
        },
    )?
    .last()
    .map(|entry| entry.id.clone())
    .ok_or_else(|| anyhow!("quickstart command created no {kind} entry"))
}
```

- [ ] **Step 4: Implement one resumable command per state-machine stage**

Add `Quickstart::run_next` using the exact command sequence below. Each match
arm calls `step`, returns `Menu` or `Quit` unchanged, and only stores an ID and
increments `stage` after `StepOutcome::Complete`:

```rust
impl Quickstart {
    fn run_next(&mut self, conn: &mut Connection) -> Result<StepOutcome> {
        let progress = quickstart_progress(self.stage);
        let outcome = match self.stage {
            0 => step(
                conn,
                &format!("{progress}\n\nCapture what you directly noticed.\nTry:\n  observe \"checkout took 3 seconds\""),
                "type: observe \"checkout took 3 seconds\" (or help)",
                |cli| matches!(cli.command, Command::Observe { .. }),
            )?,
            1 => step(
                conn,
                &format!("{progress}\n\nTurn the observation into an open question.\nTry:\n  ask \"why did checkout get slower?\""),
                "type: ask \"why did checkout get slower?\" (or help)",
                |cli| matches!(cli.command, Command::Ask { .. }),
            )?,
            2 => step(
                conn,
                &format!("{progress}\n\nChoose an action that reduces the uncertainty.\nTry:\n  act \"profile the checkout endpoint\""),
                "type: act \"profile the checkout endpoint\" (or help)",
                |cli| matches!(cli.command, Command::Act { .. }),
            )?,
            3 | 4 => {
                let (first, second) = self.connect_commands();
                let expected = if self.stage == 3 { first } else { second };
                step(
                    conn,
                    &format!("{progress}\n\nConnect the reasoning. The IDs are already filled in.\nType:\n  {expected}"),
                    &format!("type: {expected} (or help)"),
                    |cli| match &cli.command {
                        Command::Link {
                            from,
                            relation,
                            to,
                            ..
                        } => format!("link {from} {relation} {to}") == expected,
                        _ => false,
                    },
                )?
            }
            5 => {
                let question = id_prefix(self.question.as_deref().unwrap());
                let expected = format!("trace {question}");
                step(
                    conn,
                    &format!("{progress}\n\nInspect why the action exists.\nType:\n  {expected}"),
                    &format!("type: {expected} (or help)"),
                    |cli| matches!(&cli.command, Command::Trace { id, .. } if id == question),
                )?
            }
            _ => return Ok(StepOutcome::Complete),
        };

        if outcome != StepOutcome::Complete {
            return Ok(outcome);
        }
        match self.stage {
            0 => self.observation = Some(latest_id(conn, EntryKind::Observation)?),
            1 => self.question = Some(latest_id(conn, EntryKind::Question)?),
            2 => self.action = Some(latest_id(conn, EntryKind::Action)?),
            _ => {}
        }
        self.apply_outcome(outcome);
        Ok(StepOutcome::Complete)
    }
}
```

- [ ] **Step 5: Start with quickstart and resume it after opening the menu**

Extract the current numbered lesson loop into `topic_menu`. Use `resumable` to
let an empty line return to an interrupted quickstart while keeping the menu
open after the quickstart:

```rust
fn topic_menu(conn: &mut Connection, resumable: bool) -> Result<StepOutcome> {
    loop {
        println!(
            "\nTopics:\n\
             \x20 1) Capture types - reasoning and mathematics\n\
             \x20 2) Find and finish - list, show, trace, actions, complete\n\
             \x20 3) Context - projects and metadata\n\
             \x20 4) Workbench - aporic tui\n\
             \x20 5) Export - Obsidian\n\
             \x20 q) Quit the tutor"
        );
        if resumable {
            println!("  enter) Return to the quickstart");
        }

        let Some(choice) = read_line("aporic tutor> ")? else {
            return Ok(StepOutcome::Quit);
        };
        let outcome = match choice.trim() {
            "1" => lesson_capture_types(conn),
            "2" => lesson_find_and_finish(conn),
            "3" => lesson_context(conn),
            "4" => lesson_workbench(conn),
            "5" => lesson_export(conn),
            "help" => {
                println!("Choose a topic number, or use commands, menu, or quit.");
                continue;
            }
            "commands" => {
                println!("{}", command_help());
                continue;
            }
            "menu" => continue,
            "q" | "quit" | "exit" => return Ok(StepOutcome::Quit),
            "" if resumable => return Ok(StepOutcome::Complete),
            "" => continue,
            other => {
                println!("unknown option: {other}. Type help for menu help.");
                continue;
            }
        };
        if let Err(err) = outcome {
            if err.downcast_ref::<ExitTutor>().is_some() {
                return Ok(StepOutcome::Quit);
            }
            if err.downcast_ref::<ExitLesson>().is_none() {
                return Err(err);
            }
        }
    }
}
```

The rendered menu is therefore exactly:

```text
Topics:
  1) Capture types - reasoning and mathematics
  2) Find and finish - list, show, trace, actions, complete
  3) Context - projects and metadata
  4) Workbench - aporic tui
  5) Export - Obsidian
  q) Quit the tutor
```

Replace the top-level body of `run` after schema initialization with:

```rust
    println!(
        "\naporic tutor\n============\n\
         Every command runs against a throw-away in-memory database. Your\n\
         real project data is never touched. At any prompt: help, commands,\n\
         menu, or quit.\n"
    );

    let mut quickstart = Quickstart::default();
    while !quickstart.complete() {
        match quickstart.run_next(&mut conn)? {
            StepOutcome::Complete => {}
            StepOutcome::Menu => {
                if topic_menu(&mut conn, true)? == StepOutcome::Quit {
                    println!("\nGoodbye - the sandbox was discarded.");
                    return Ok(());
                }
            }
            StepOutcome::Quit => {
                println!("\nGoodbye - the sandbox was discarded.");
                return Ok(());
            }
        }
    }

    println!(
        "Quickstart complete. You captured uncertainty, connected it to an\n\
         action, and inspected why that action exists.\n"
    );
    let _ = topic_menu(&mut conn, false)?;
    println!("\nGoodbye - the sandbox was discarded.");
    Ok(())
```

Opening the menu during quickstart must return to the unchanged `stage`; it
must not recreate completed entries.

- [ ] **Step 6: Fill the task-oriented topic gaps without adding new lessons**

Reuse and relabel the five existing lesson functions rather than creating a
second tutorial system:

- Merge the existing vocabulary and first-chain explanatory copy into
  `lesson_capture_types`; keep the vocabulary table and add the eight math
  command names. Delete the now-redundant `lesson_first_chain` function.
- Rename `lesson_inspecting` to `lesson_find_and_finish`. Replace its single
  `is_empty()` seed condition with independent observation and action checks,
  so opening this topic midway through quickstart always supplies both kinds:

```rust
    let entries = domain::list_entries(conn, EntryFilter::default())?;
    if !entries
        .iter()
        .any(|entry| entry.kind == EntryKind::Observation)
    {
        domain::create_entry(
            conn,
            EntryKind::Observation,
            "sample observation seeded for this topic",
            NewEntry {
                author: "tutor",
                origin: "tutor",
                ..NewEntry::default()
            },
        )?;
    }
    if !entries.iter().any(|entry| entry.kind == EntryKind::Action) {
        domain::create_entry(
            conn,
            EntryKind::Action,
            "sample action seeded for this topic",
            NewEntry {
                author: "tutor",
                origin: "tutor",
                ..NewEntry::default()
            },
        )?;
    }
```

  After its existing `list`, `show`, and `trace` steps, add these real steps:

```rust
    topic_step(
        conn,
        "Filter to open questions. Try: list --type question --state open",
        "type: list --type question --state open",
        |cli| matches!(
            &cli.command,
            Command::List { r#type, state, .. }
                if r#type.as_deref() == Some("question")
                    && state.as_deref() == Some("open")
        ),
    )?;
    topic_step(conn, "Show ready work. Try: actions", "type: actions", |cli| {
        matches!(cli.command, Command::Actions { .. })
    })?;
```

  Then require completion of the seeded action with:

```rust
    let action = latest_id(conn, EntryKind::Action)?;
    let expected = format!("complete {}", id_prefix(&action));
    topic_step(
        conn,
        &format!("Complete the ready action. Try: {expected}"),
        &format!("type: {expected}"),
        |cli| matches!(&cli.command, Command::Complete { id } if id == id_prefix(&action)),
    )?;
```
- Rename `lesson_projects` to `lesson_context`; keep explicit project scoping
  and change its capture example to include implemented metadata:

```text
observe --project demo --source https://example.test/report "trying project context"
```

- Rename `lesson_tui` to `lesson_workbench`, include `? opens key help` in its
  introduction, and replace the raw Enter prompt plus direct TUI call with:

```rust
    topic_step(
        conn,
        "Type `tui` to open the workbench. Inside it, press ? for key help.",
        "type: tui",
        |cli| matches!(cli.command, Command::Tui),
    )?;
```

  This ensures tutor controls also work at the Workbench prompt.
- Rename `lesson_obsidian` to `lesson_export` without changing its safe scratch
  file behavior.

Finally, keep the terminal copy ASCII-only by applying these exact textual
replacements throughout `src/tutor.rs`: `—` to `-`, `→` to `->`, and curly
apostrophes to `'`. Verify with:

```bash
LC_ALL=C rg -n '[^ -~[:space:]]' src/tutor.rs
```

Expected: no matches.

- [ ] **Step 7: Run focused and full tests**

Run:

```bash
cargo fmt -- --check
cargo test tutor::tests
cargo test --test cli_startup
cargo test
```

Expected: all commands succeed; the quickstart tests and all three startup tests
pass; the existing seven tests remain green.

- [ ] **Step 8: Commit the quickstart**

```bash
git add src/tutor.rs tests/cli_startup.rs
git commit -m "feat: guide newcomers through a reasoning trace"
```

### Task 4: Replace the TUI help hint with a modal key reference

**Files:**
- Modify: `src/tui.rs:15,67-100,233-311,416-436`

**Interfaces:**
- Consumes: the existing `App`, event loop, and draw functions.
- Produces: `App::show_help: bool`, `handle_key(...) -> Result<()>`, and `draw_help(...)`.

- [ ] **Step 1: Write a failing key-capture test**

Append to `src/tui.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn help_overlay_closes_before_quit_is_handled() {
        let mut conn = Connection::open_in_memory().unwrap();
        crate::db::ensure_schema(&mut conn).unwrap();
        let mut app = App::new(&conn, None).unwrap();

        handle_key(&mut app, &mut conn, KeyCode::Char('?')).unwrap();
        assert!(app.show_help);

        handle_key(&mut app, &mut conn, KeyCode::Char('q')).unwrap();
        assert!(!app.show_help);
        assert!(!app.quit);

        handle_key(&mut app, &mut conn, KeyCode::Char('?')).unwrap();
        handle_key(&mut app, &mut conn, KeyCode::Esc).unwrap();
        assert!(!app.show_help);
        assert!(!app.quit);
    }
}
```

- [ ] **Step 2: Run the focused test and verify it fails to compile**

Run:

```bash
cargo test tui::tests
```

Expected: compilation fails because `show_help` and `handle_key` do not exist.

- [ ] **Step 3: Add modal state and centralize existing key handling**

Add `show_help: bool` to `App` and initialize it to `false`. Move the existing
`pending_complete` block and key `match` from `event_loop` into:

```rust
fn handle_key(app: &mut App, conn: &mut Connection, key: KeyCode) -> Result<()> {
    if app.show_help {
        if matches!(key, KeyCode::Char('?') | KeyCode::Char('q') | KeyCode::Esc) {
            app.show_help = false;
        }
        return Ok(());
    }

    if app.pending_complete {
        app.pending_complete = false;
        if key == KeyCode::Char('y') {
            app.complete_selected(conn)?;
        } else {
            app.status = "cancelled".to_string();
        }
        return Ok(());
    }

    match key {
        KeyCode::Char('q') | KeyCode::Esc => app.quit = true,
        KeyCode::Char('j') | KeyCode::Down => app.next(),
        KeyCode::Char('k') | KeyCode::Up => app.previous(),
        KeyCode::Tab => app.change_tab(),
        KeyCode::Enter => app.right = RightPane::Detail,
        KeyCode::Char('t') => app.show_trace(conn)?,
        KeyCode::Char('p') => app.cycle_project(conn)?,
        KeyCode::Char('c') => {
            if let Some(entry) = app.selected_entry() {
                if entry.kind == EntryKind::Action {
                    app.pending_complete = true;
                    app.status = "complete this action? (y/n)".to_string();
                } else {
                    app.status = "only actions can be completed".to_string();
                }
            }
        }
        KeyCode::Char('?') => app.show_help = true,
        _ => {}
    }
    Ok(())
}
```

After filtering for `KeyEventKind::Press`, `event_loop` now calls only:

```rust
        handle_key(&mut app, conn, key.code)?;
```

Also change the existing tab title from the Unicode em dash to ASCII:

```rust
.title(format!(" aporic - project: {project_label} "))
```

- [ ] **Step 4: Render a bounded help overlay after the normal screen**

Add `Clear` to the widget imports. At the end of `draw`, add:

```rust
    if app.show_help {
        draw_help(frame, area);
    }
```

Add:

```rust
fn draw_help(frame: &mut Frame, area: Rect) {
    let width = area.width.min(58);
    let height = area.height.min(15);
    let popup = Rect::new(
        area.x + area.width.saturating_sub(width) / 2,
        area.y + area.height.saturating_sub(height) / 2,
        width,
        height,
    );
    let help = Paragraph::new(
        "j / Down    next entry\n\
         k / Up      previous entry\n\
         Tab         next filter\n\
         Enter       show details\n\
         t           show trace\n\
         c           complete action\n\
         p           next project\n\
         ?           close this help\n\
         q / Esc     close help, then quit",
    )
    .block(Block::default().borders(Borders::ALL).title(" keys "))
    .wrap(Wrap { trim: false });
    frame.render_widget(Clear, popup);
    frame.render_widget(help, popup);
}
```

- [ ] **Step 5: Run focused and full tests**

Run:

```bash
cargo fmt -- --check
cargo test tui::tests
cargo test
```

Expected: all commands succeed and the overlay test passes.

- [ ] **Step 6: Commit the TUI help**

```bash
git add src/tui.rs
git commit -m "feat: show TUI key help"
```

### Task 5: Update onboarding documentation and verify the release surface

**Files:**
- Modify: `README.md:43-55,75-79,90-93`

**Interfaces:**
- Consumes: the completed tutor and TUI behavior.
- Produces: accurate onboarding documentation with no promise of roadmap commands.

- [ ] **Step 1: Replace the old tutor description**

Replace the paragraph beginning with `` `aporic tutor` is the primary way ``
with:

```markdown
`aporic tutor` starts with a five-minute quickstart that runs real commands in
a throw-away sandbox. It takes you from an observation through a question and
action to a connected trace, then offers optional topics for capture types,
projects, inspection, the TUI, and Obsidian export. At any tutor prompt, use
`help`, `commands`, `menu`, or `quit`.
```

Change the TUI command-table purpose to:

```markdown
| `tui` | Browse entries and traces; press `?` for the complete key reference |
```

Change the Obsidian tutor reference from a numbered lesson to the named topic:

```markdown
The tutor's Export topic walks through this against a scratch file before you
try it on a real note.
```

- [ ] **Step 2: Check documentation and CLI text for stale claims**

Run:

```bash
rg -n "lesson 6|legacy task ID|full walkthrough|vimtutor-style" README.md src
```

Expected: no matches. Any match is stale copy and must be replaced with the
task-oriented quickstart terms already defined above.

- [ ] **Step 3: Run all required checks**

Run:

```bash
cargo fmt -- --check
cargo test
cargo clippy --all-targets --all-features -- -D warnings
```

Expected: all tests pass and Clippy exits without warnings.

- [ ] **Step 4: Manually smoke-test the public flow**

Run:

```bash
cargo run -- tutor
```

Complete the prompted observation, question, action, two link commands, and
trace command. During one step, run `help`, `commands`, and `menu`; confirm the
same step resumes. From the Workbench topic, press `?`, then `q`, and confirm
the overlay closes before the TUI quits.

- [ ] **Step 5: Commit the documentation**

```bash
git add README.md
git commit -m "docs: update the tutor quickstart"
```
