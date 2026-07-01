// Interactive, vimtutor-style teacher. Every step parses and runs a real
// command through the same `dispatch` used by the CLI and the TUI, against
// an in-memory sandbox database — nothing here is simulated, and nothing
// here ever touches the user's real Aporic data.
use crate::domain::{self, EntryFilter, EntryKind, NewEntry};
use crate::{dispatch, effective_project, Cli, Command, ObsidianCommand};
use anyhow::{anyhow, Result};
use clap::Parser;
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

pub fn run() -> Result<()> {
    let mut conn = Connection::open_in_memory()?;
    crate::db::ensure_schema(&mut conn)?;

    println!(
        "\naporic tutor\n============\n\
         A vimtutor-style walkthrough. Every command you type here is a real\n\
         Aporic command, run against a throw-away in-memory database — your\n\
         real project data is never touched. Type 'menu' at any prompt to go\n\
         back to this list, or 'q' here to leave.\n"
    );

    loop {
        println!(
            "\nLessons:\n\
             \x20 1) Vocabulary — the reasoning lifecycle\n\
             \x20 2) Capture and connect a first reasoning chain\n\
             \x20 3) Projects — scoping your work with --project\n\
             \x20 4) Inspecting — list, show, trace, status\n\
             \x20 5) The interactive view — aporic tui\n\
             \x20 6) Exporting into Obsidian\n\
             \x20 q) Quit the tutor\n"
        );
        let Some(choice) = read_line("aporic tutor> ")? else {
            break;
        };
        let outcome = match choice.trim() {
            "1" => lesson_vocabulary(&mut conn),
            "2" => lesson_first_chain(&mut conn),
            "3" => lesson_projects(&mut conn),
            "4" => lesson_inspecting(&mut conn),
            "5" => lesson_tui(&mut conn),
            "6" => lesson_obsidian(&mut conn),
            "q" | "quit" | "exit" => break,
            "" => continue,
            other => {
                println!("unknown option: {other}");
                continue;
            }
        };
        if let Err(err) = outcome {
            if err.downcast_ref::<ExitLesson>().is_none() {
                return Err(err);
            }
        }
    }

    println!("\nGoodbye — the sandbox is discarded, your real data was never touched.");
    Ok(())
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
) -> Result<()> {
    println!("{intro}");
    loop {
        let Some(line) = read_line("aporic tutor> ")? else {
            return Err(anyhow!(ExitLesson));
        };
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if matches!(trimmed, "menu" | "q" | "quit" | "exit") {
            return Err(anyhow!(ExitLesson));
        }
        let tokens = match shell_words::split(trimmed) {
            Ok(tokens) => tokens,
            Err(_) => {
                println!("could not parse quoting in that line. {hint}");
                continue;
            }
        };
        let argv = std::iter::once("aporic".to_string()).chain(tokens);
        let cli = match Cli::try_parse_from(argv) {
            Ok(cli) => cli,
            Err(err) => {
                println!("{err}");
                println!("{hint}");
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
        return Ok(());
    }
}

fn lesson_vocabulary(conn: &mut Connection) -> Result<()> {
    println!(
        "\nLesson 1: Vocabulary\n=====================\n\
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
         (Mathematics uses the same model with definition, conjecture, lemma,\n\
         theorem, proof, counterexample, example, and calculation.)\n"
    );
    step(
        conn,
        "Try it: check the sandbox's status.",
        "type: status",
        |cli| matches!(cli.command, Command::Status),
    )?;
    println!("Good — an empty project. On to a first real chain (lesson 2).");
    Ok(())
}

fn lesson_first_chain(conn: &mut Connection) -> Result<()> {
    println!(
        "\nLesson 2: Capture and connect\n==============================\n\
         Each command below prints the id of what it created. You'll use two\n\
         of those ids in the last step, so keep an eye on the output.\n"
    );
    step(
        conn,
        "Step 1 — observe something. Try:\n  observe \"the checkout page took 3s to load\"",
        "start the line with observe, followed by a quoted sentence",
        |cli| matches!(cli.command, Command::Observe { .. }),
    )?;
    step(
        conn,
        "Step 2 — turn it into a question. Try:\n  ask \"why did checkout get slower?\"",
        "start the line with ask",
        |cli| matches!(cli.command, Command::Ask { .. }),
    )?;
    step(
        conn,
        "Step 3 — decide what to do about it. Try:\n  act \"profile the checkout endpoint\"",
        "start the line with act",
        |cli| matches!(cli.command, Command::Act { .. }),
    )?;
    step(
        conn,
        "Step 4 — record what happened. Try:\n  outcome \"the product-image query was the bottleneck\"",
        "start the line with outcome",
        |cli| matches!(cli.command, Command::Outcome { .. }),
    )?;
    step(
        conn,
        "Step 5 — capture the durable lesson. Try:\n  learn \"cache expensive queries before they reach checkout\"",
        "start the line with learn",
        |cli| matches!(cli.command, Command::Learn { .. }),
    )?;
    println!(
        "Now connect two entries. Pick two ids from the output above and try:\n  \
         link <id> answers <id>\n\
         (other relations exist too: supports, challenges, motivates, result-of, derived-from...)"
    );
    step(
        conn,
        "Step 6 — link two entries.",
        "type: link <id> <relation> <id> — use ids printed earlier in this lesson",
        |cli| matches!(cli.command, Command::Link { .. }),
    )?;
    println!("That's a full reasoning chain. Lesson 4 shows how to inspect it later.");
    Ok(())
}

fn lesson_projects(conn: &mut Connection) -> Result<()> {
    println!(
        "\nLesson 3: Projects\n===================\n\
         Aporic never remembers a default project for you — every command is\n\
         either scoped to one project with --project, or belongs to 'global'\n\
         if you omit it. This is deliberate: no hidden state to lose track of.\n"
    );
    step(
        conn,
        "Try: projects",
        "type: projects",
        |cli| matches!(cli.command, Command::Projects),
    )?;
    step(
        conn,
        "Now capture something scoped to a project. Try:\n  observe --project demo \"trying out project scoping\"",
        "add --project demo before or after the quoted text",
        |cli| matches!(cli.command, Command::Observe { .. }) && cli.project.is_some(),
    )?;
    step(
        conn,
        "Check again — 'demo' should now be listed too:",
        "type: projects",
        |cli| matches!(cli.command, Command::Projects),
    )?;
    println!("Explicit --project everywhere is more typing but never surprises you later.");
    Ok(())
}

fn lesson_inspecting(conn: &mut Connection) -> Result<()> {
    let seeded = domain::list_entries(conn, EntryFilter::default())?.is_empty();
    if seeded {
        domain::create_entry(
            conn,
            EntryKind::Observation,
            "sample observation seeded for this lesson",
            NewEntry {
                author: "tutor",
                origin: "tutor",
                ..NewEntry::default()
            },
        )?;
        domain::create_entry(
            conn,
            EntryKind::Action,
            "sample action seeded for this lesson",
            NewEntry {
                author: "tutor",
                origin: "tutor",
                ..NewEntry::default()
            },
        )?;
        println!(
            "\n(This lesson works standalone, so the sandbox was seeded with two\n\
             sample entries since it was otherwise empty.)"
        );
    }
    println!(
        "\nLesson 4: Inspecting\n=====================\n\
         list, show, trace, and status are all read-only.\n"
    );
    step(
        conn,
        "Try: list",
        "type: list",
        |cli| matches!(cli.command, Command::List { .. }),
    )?;

    let entries = domain::list_entries(conn, EntryFilter::default())?;
    println!("ids you can use below:");
    for entry in &entries {
        println!("  {} {:<11} {}", &entry.id[..8], entry.kind, entry.body);
    }
    println!();

    step(
        conn,
        "Try: show <id> — use one of the ids above (a unique prefix is enough)",
        "type: show <id>",
        |cli| matches!(cli.command, Command::Show { .. }),
    )?;
    step(
        conn,
        "Try: trace <id> — shows the local reasoning graph around an entry",
        "type: trace <id>",
        |cli| matches!(cli.command, Command::Trace { .. }),
    )?;
    step(
        conn,
        "Try: status",
        "type: status",
        |cli| matches!(cli.command, Command::Status),
    )?;
    Ok(())
}

fn lesson_tui(conn: &mut Connection) -> Result<()> {
    println!(
        "\nLesson 5: The interactive view\n===============================\n\
         aporic tui gives you a full-screen view of the same data: a\n\
         filterable list, a detail panel, and a trace view. It launches now\n\
         inside this same sandbox, so it shows whatever you've built in\n\
         earlier lessons. Press q inside it to come back here.\n"
    );
    let Some(_) = read_line("Press enter to launch aporic tui> ")? else {
        return Err(anyhow!(ExitLesson));
    };
    crate::tui::run(conn, None)?;
    println!(
        "\nWelcome back. Quick reference: j/k move, Tab switches list filters,\n\
         enter/t toggle detail and trace, c completes the selected action,\n\
         p cycles the project, q quits.\n"
    );
    Ok(())
}

fn lesson_obsidian(conn: &mut Connection) -> Result<()> {
    let path = std::env::temp_dir().join("aporic-tutor-export.md");
    println!(
        "\nLesson 6: Exporting into Obsidian\n===================================\n\
         Export writes entries between markers:\n\n  \
         <!-- aporic:start version=1 -->\n  ...\n  <!-- aporic:end -->\n\n\
         Only that fenced block is ever rewritten. Anything you write outside\n\
         it in a real note is preserved untouched on every re-export.\n"
    );
    let hint = format!("type: obsidian export {}", path.display());
    step(
        conn,
        &format!("Try (this writes a scratch file, not anything real):\n  obsidian export {}", path.display()),
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
