// Interactive, vimtutor-style teacher. Every step parses and runs a real
// command through the same `dispatch` used by the CLI and the TUI, against
// an in-memory sandbox database - nothing here is simulated, and nothing
// here ever touches the user's real Aporic data.
use crate::domain::{self, EntryFilter, EntryKind, NewEntry};
use crate::{dispatch, effective_project, Cli, Command, ObsidianCommand};
use anyhow::{anyhow, Result};
use clap::{CommandFactory, Parser};
use rusqlite::Connection;
use std::io::{self, Write};

#[derive(Debug)]
struct ExitLesson;

impl std::fmt::Display for ExitLesson {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "exit lesson")
    }
}
impl std::error::Error for ExitLesson {}

#[derive(Debug)]
struct ExitTutor;

impl std::fmt::Display for ExitTutor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "exit tutor")
    }
}
impl std::error::Error for ExitTutor {}

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
    format!(
        "Aporic quickstart                 step {} of 5\n\n{flow}",
        current + 1
    )
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

pub fn run() -> Result<()> {
    let mut conn = Connection::open_in_memory()?;
    crate::db::ensure_schema(&mut conn)?;

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
}

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

fn read_line(prompt: &str) -> Result<Option<String>> {
    print!("{prompt}");
    io::stdout().flush()?;
    let mut line = String::new();
    let bytes = io::stdin().read_line(&mut line)?;
    if bytes == 0 {
        return Ok(None);
    }
    Ok(Some(line))
}

/// Prints `intro`, then loops reading a line, parsing it as a real Aporic
/// command line, and running it through the shared `dispatch` once it
/// matches `expect`. Wrong input re-prompts with `hint` instead of advancing.
fn step(
    conn: &mut Connection,
    intro: &str,
    hint: &str,
    expect: impl Fn(&Cli) -> bool,
) -> Result<StepOutcome> {
    println!("{intro}");
    loop {
        let Some(line) = read_line("aporic tutor> ")? else {
            return Ok(StepOutcome::Quit);
        };
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
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
        let tokens = match shell_words::split(trimmed) {
            Ok(tokens) => tokens,
            Err(_) => {
                println!(
                    "could not parse quoting in that line. {hint} Type help to repeat the current instructions."
                );
                continue;
            }
        };
        let argv = std::iter::once("aporic".to_string()).chain(tokens);
        let cli = match Cli::try_parse_from(argv) {
            Ok(cli) => cli,
            Err(err) => {
                println!("{err}");
                println!("{hint} Type help to repeat the current instructions.");
                continue;
            }
        };
        if !expect(&cli) {
            println!("Not quite what this step needs. {hint}");
            continue;
        }
        let project = effective_project(cli.project.as_deref());
        if let Err(err) = dispatch(conn, &cli, project.as_deref(), &cli.command) {
            println!("error: {err}");
            println!("{hint}");
            continue;
        }
        println!();
        return Ok(StepOutcome::Complete);
    }
}

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

fn lesson_capture_types(conn: &mut Connection) -> Result<()> {
    println!(
        "\nCapture types\n=============\n\
         Aporic records reasoning as small typed entries instead of one flat\n\
         task list:\n\n  \
         observation -> claim -> question -> implication -> action -> outcome -> learning\n\n\
         observation   something you directly noticed or measured\n\
         claim         a statement evidence can support or challenge\n\
         assumption    a premise you're currently relying on\n\
         question      a meaningful unresolved unknown\n\
         implication   what follows if a premise holds\n\
         action        a concrete next step\n\
         outcome       what happened after an action\n\
         learning      a durable update to how you'll work next time\n\n\
         Capture these with observe, claim, assume, ask, imply, act, outcome,\n\
         and learn. Each command prints an ID you can use with link.\n\n\
         Mathematics uses the same model with define, conjecture, lemma,\n\
         theorem, proof, counterexample, example, and calculate.\n"
    );
    topic_step(
        conn,
        "Try it: check the sandbox's status.",
        "type: status",
        |cli| matches!(cli.command, Command::Status),
    )?;
    Ok(())
}

fn lesson_context(conn: &mut Connection) -> Result<()> {
    println!(
        "\nContext\n=======\n\
         Aporic never remembers a default project for you - every command is\n\
         either scoped to one project with --project, or belongs to 'global'\n\
         if you omit it. This is deliberate: no hidden state to lose track of.\n"
    );
    topic_step(conn, "Try: projects", "type: projects", |cli| {
        matches!(cli.command, Command::Projects)
    })?;
    topic_step(
        conn,
        "Now capture project and source context. Try:\n  observe --project demo --source https://example.test/report \"trying project context\"",
        "type: observe --project demo --source https://example.test/report \"trying project context\"",
        |cli| {
            matches!(cli.command, Command::Observe { .. })
                && cli.project.is_some()
                && cli.source.is_some()
        },
    )?;
    topic_step(
        conn,
        "Check again - 'demo' should now be listed too:",
        "type: projects",
        |cli| matches!(cli.command, Command::Projects),
    )?;
    println!("Explicit --project everywhere is more typing but never surprises you later.");
    Ok(())
}

fn lesson_find_and_finish(conn: &mut Connection) -> Result<()> {
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
    println!(
        "\nFind and finish\n===============\n\
         list, show, and trace inspect work without changing it. actions and\n\
         complete help find and finish ready work.\n"
    );
    topic_step(conn, "Try: list", "type: list", |cli| {
        matches!(cli.command, Command::List { .. })
    })?;

    let entries = domain::list_entries(conn, EntryFilter::default())?;
    println!("ids you can use below:");
    for entry in &entries {
        println!("  {} {:<11} {}", &entry.id[..8], entry.kind, entry.body);
    }
    println!();

    topic_step(
        conn,
        "Try: show <id> - use one of the ids above (a unique prefix is enough)",
        "type: show <id>",
        |cli| matches!(cli.command, Command::Show { .. }),
    )?;
    topic_step(
        conn,
        "Try: trace <id> - shows the local reasoning graph around an entry",
        "type: trace <id>",
        |cli| matches!(cli.command, Command::Trace { .. }),
    )?;
    topic_step(
        conn,
        "Filter to open questions. Try: list --type question --state open",
        "type: list --type question --state open",
        |cli| {
            matches!(
                &cli.command,
                Command::List { r#type, state, .. }
                    if r#type.as_deref() == Some("question")
                        && state.as_deref() == Some("open")
            )
        },
    )?;
    topic_step(
        conn,
        "Show ready work. Try: actions",
        "type: actions",
        |cli| matches!(cli.command, Command::Actions { .. }),
    )?;
    let action = latest_id(conn, EntryKind::Action)?;
    let expected = format!("complete {}", id_prefix(&action));
    topic_step(
        conn,
        &format!("Complete the ready action. Try: {expected}"),
        &format!("type: {expected}"),
        |cli| matches!(&cli.command, Command::Complete { id } if id == id_prefix(&action)),
    )?;
    Ok(())
}

fn lesson_workbench(conn: &mut Connection) -> Result<()> {
    println!(
        "\nWorkbench\n=========\n\
         aporic tui gives you a full-screen view of the same data: a\n\
         filterable list, a detail panel, and a trace view. It launches now\n\
         inside this same sandbox, so it shows whatever you've built in\n\
         the quickstart. ? opens key help; q returns here.\n"
    );
    topic_step(
        conn,
        "Type `tui` to open the workbench. Inside it, press ? for key help.",
        "type: tui",
        |cli| matches!(cli.command, Command::Tui),
    )?;
    println!(
        "\nWelcome back. Quick reference: j/k move, Tab switches list filters,\n\
         enter/t toggle detail and trace, c completes the selected action,\n\
         p cycles the project, ? opens key help, q quits.\n"
    );
    Ok(())
}

fn lesson_export(conn: &mut Connection) -> Result<()> {
    let path = std::env::temp_dir().join("aporic-tutor-export.md");
    println!(
        "\nExport\n======\n\
         Export writes entries between markers:\n\n  \
         <!-- aporic:start version=1 -->\n  ...\n  <!-- aporic:end -->\n\n\
         Only that fenced block is ever rewritten. Anything you write outside\n\
         it in a real note is preserved untouched on every re-export.\n"
    );
    let hint = format!("type: obsidian export {}", path.display());
    topic_step(
        conn,
        &format!(
            "Try (this writes a scratch file, not anything real):\n  obsidian export {}",
            path.display()
        ),
        &hint,
        |cli| {
            matches!(
                cli.command,
                Command::Obsidian {
                    command: ObsidianCommand::Export { .. }
                }
            )
        },
    )?;
    if let Ok(contents) = std::fs::read_to_string(&path) {
        println!("--- {} ---\n{contents}\n---", path.display());
    }
    let _ = std::fs::remove_file(&path);
    println!("(scratch file removed)");
    Ok(())
}

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
}
