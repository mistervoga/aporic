mod db;
mod domain;
mod json;
mod obsidian;
mod tui;
mod tutor;

use anyhow::Result;
use clap::{Parser, Subcommand};
use domain::{Entry, EntryFilter, EntryKind, MathKind, NewEntry, RelationKind, Trace};
use rusqlite::Connection;
use std::path::PathBuf;
use std::str::FromStr;

#[derive(Parser)]
#[command(name = "aporic")]
#[command(version)]
#[command(about = "From uncertainty to justified action.")]
struct Cli {
    /// Restrict the command to a project; use 'global' for unscoped entries
    #[arg(long, global = true)]
    project: Option<String>,

    /// Emit one machine-readable JSON value
    #[arg(long, global = true)]
    json: bool,

    /// Actor recorded for new entries and relations
    #[arg(long, global = true, default_value = "human")]
    actor: String,

    /// Attach repository context to a newly captured entry
    #[arg(long, global = true)]
    repo: Option<String>,

    /// Attach an immutable commit hash to a newly captured entry
    #[arg(long, global = true)]
    commit: Option<String>,

    /// Attach a source file to a newly captured entry
    #[arg(long, global = true)]
    file: Option<String>,

    /// Attach a source line to a newly captured entry
    #[arg(long, global = true, requires = "file")]
    line: Option<i64>,

    /// Attach a source URI to a newly captured entry
    #[arg(long, global = true)]
    source: Option<String>,

    /// Mathematical setting, such as "Banach spaces" or "Lean 4"
    #[arg(long, global = true)]
    formal_system: Option<String>,

    /// Verification: unverified, checked, peer_reviewed, formally_verified
    #[arg(long, global = true)]
    verification: Option<String>,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Ensure the database is initialized and show its schema version
    Init,

    /// Record something directly noticed or measured
    #[command(alias = ".")]
    Observe { text: String },

    /// Record a statement that can be supported or challenged
    Claim { text: String },

    /// Record a working assumption
    #[command(alias = "~")]
    Assume { text: String },

    /// Record a meaningful unresolved question
    #[command(alias = "?")]
    Ask { text: String },

    /// Record what follows from existing understanding
    #[command(alias = "!")]
    Imply { text: String },

    /// Record a concrete next action
    #[command(alias = ">")]
    Act { text: String },

    /// Record what happened after an action or event
    #[command(alias = "=")]
    Outcome { text: String },

    /// Record a durable update to understanding
    Learn { text: String },

    /// Record a mathematical definition
    Define { text: String },

    /// Record an unresolved mathematical claim
    Conjecture { text: String },

    /// Record a supporting mathematical claim
    Lemma { text: String },

    /// Record a main mathematical claim
    Theorem { text: String },

    /// Record proof evidence
    Proof { text: String },

    /// Record a counterexample or boundary case
    Counterexample { text: String },

    /// Record a clarifying mathematical example
    Example { text: String },

    /// Record a symbolic or numerical derivation
    Calculate { text: String },

    /// List entries
    List {
        #[arg(long, value_name = "KIND")]
        r#type: Option<String>,
        #[arg(long)]
        state: Option<String>,
        #[arg(long, value_name = "MATH_KIND")]
        math_type: Option<String>,
    },

    /// Show one entry by UUID or unique UUID prefix
    Show { id: String },

    /// Connect two entries with a typed relation
    Link {
        from: String,
        relation: String,
        to: String,
        #[arg(long)]
        rationale: Option<String>,
    },

    /// Show the local reasoning graph around an entry
    Trace {
        id: String,
        #[arg(long, default_value_t = 2, value_parser = clap::value_parser!(u8).range(1..=8))]
        depth: u8,
    },

    /// Mark an action as done
    Complete { id: String },

    /// List ready actions
    Actions {
        #[arg(long, default_value_t = true)]
        ready: bool,
    },

    /// List projects
    Projects,

    /// Show database and entry status
    Status,

    /// Open the interactive terminal view
    Tui,

    /// Sandboxed five-minute quickstart with optional learning topics
    Tutor,

    /// Export an Aporic-owned section into an Obsidian note
    Obsidian {
        #[command(subcommand)]
        command: ObsidianCommand,
    },
}

#[derive(Subcommand)]
enum ObsidianCommand {
    /// Export entries without overwriting handwritten note content
    Export { path: PathBuf },
}

struct StatusOutput {
    product: &'static str,
    schema_version: i64,
    project: String,
    entries: usize,
    open_questions: usize,
    ready_actions: usize,
    mathematical_entries: usize,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    if matches!(&cli.command, Command::Tutor) {
        return tutor::run();
    }

    let mut conn = db::connect_and_init()?;
    let project = effective_project(cli.project.as_deref());
    dispatch(&mut conn, &cli, project.as_deref(), &cli.command)
}

/// Runs one parsed command against an open connection. Shared by the CLI
/// entry point and the sandboxed tutor so there is one implementation of
/// command behavior, not two.
pub(crate) fn dispatch(
    conn: &mut Connection,
    cli: &Cli,
    project: Option<&str>,
    command: &Command,
) -> Result<()> {
    match command {
        Command::Init => {
            let status = StatusOutput {
                product: "aporic",
                schema_version: db::schema_version(conn)?,
                project: project_name(project),
                entries: 0,
                open_questions: 0,
                ready_actions: 0,
                mathematical_entries: 0,
            };
            if cli.json {
                print_json_value(&status_json(&status));
            } else {
                println!("aporic ready (schema {})", status.schema_version);
            }
        }
        Command::Observe { text } => {
            capture(conn, cli, project, EntryKind::Observation, None, text)?
        }
        Command::Claim { text } => capture(conn, cli, project, EntryKind::Claim, None, text)?,
        Command::Assume { text } => capture(conn, cli, project, EntryKind::Assumption, None, text)?,
        Command::Ask { text } => capture(conn, cli, project, EntryKind::Question, None, text)?,
        Command::Imply { text } => capture(conn, cli, project, EntryKind::Implication, None, text)?,
        Command::Act { text } => capture(conn, cli, project, EntryKind::Action, None, text)?,
        Command::Outcome { text } => capture(conn, cli, project, EntryKind::Outcome, None, text)?,
        Command::Learn { text } => capture(conn, cli, project, EntryKind::Learning, None, text)?,
        Command::Define { text } => capture_math(conn, cli, project, MathKind::Definition, text)?,
        Command::Conjecture { text } => {
            capture_math(conn, cli, project, MathKind::Conjecture, text)?
        }
        Command::Lemma { text } => capture_math(conn, cli, project, MathKind::Lemma, text)?,
        Command::Theorem { text } => capture_math(conn, cli, project, MathKind::Theorem, text)?,
        Command::Proof { text } => capture_math(conn, cli, project, MathKind::Proof, text)?,
        Command::Counterexample { text } => {
            capture_math(conn, cli, project, MathKind::Counterexample, text)?
        }
        Command::Example { text } => capture_math(conn, cli, project, MathKind::Example, text)?,
        Command::Calculate { text } => {
            capture_math(conn, cli, project, MathKind::Calculation, text)?
        }
        Command::List {
            r#type,
            state,
            math_type,
        } => {
            let kind = r#type.as_deref().map(EntryKind::from_str).transpose()?;
            let math_kind = math_type.as_deref().map(MathKind::from_str).transpose()?;
            let entries = domain::list_entries(
                conn,
                EntryFilter {
                    project,
                    kind,
                    state: state.as_deref(),
                    math_kind,
                },
            )?;
            output_entries(&entries, cli.json)?;
        }
        Command::Show { id } => {
            let entry = domain::get_entry(conn, id)?;
            if cli.json {
                print_json_value(&json::entry(&entry));
            } else {
                print_entry(&entry);
            }
        }
        Command::Link {
            from,
            relation,
            to,
            rationale,
        } => {
            let relation = domain::create_relation(
                conn,
                from,
                RelationKind::from_str(relation)?,
                to,
                rationale.as_deref(),
                &cli.actor,
            )?;
            if cli.json {
                print_json_value(&json::relation(&relation));
            } else {
                println!(
                    "linked {} {} {}",
                    short_id(&relation.from_id),
                    relation.kind,
                    short_id(&relation.to_id)
                );
            }
        }
        Command::Trace { id, depth } => {
            let trace = domain::trace_entry(conn, id, (*depth).into())?;
            output_trace(&trace, cli.json)?;
        }
        Command::Complete { id } => {
            let entry = domain::complete_action(conn, id)?;
            if cli.json {
                print_json_value(&json::entry(&entry));
            } else {
                println!("completed {}", short_id(&entry.id));
            }
        }
        Command::Actions { ready } => {
            let entries = domain::list_entries(
                conn,
                EntryFilter {
                    project,
                    kind: Some(EntryKind::Action),
                    state: ready.then_some("open"),
                    math_kind: None,
                },
            )?;
            output_entries(&entries, cli.json)?;
        }
        Command::Projects => {
            let projects = domain::list_projects(conn)?;
            if cli.json {
                print_json_value(&json::strings(&projects));
            } else {
                for project in projects {
                    println!("{project}");
                }
            }
        }
        Command::Status => output_status(conn, project, cli.json)?,
        Command::Tui => tui::run(conn, project)?,
        Command::Tutor => tutor::run()?,
        Command::Obsidian { command } => match command {
            ObsidianCommand::Export { path } => {
                let count = obsidian::export(conn, project, path)?;
                if cli.json {
                    print_json_value(&format!(
                        "{{\"exported\":{count},\"path\":{}}}",
                        json::quote(&path.to_string_lossy())
                    ));
                } else {
                    println!("exported {count} -> {}", path.display());
                }
            }
        },
    }
    Ok(())
}

fn capture(
    conn: &mut Connection,
    cli: &Cli,
    project: Option<&str>,
    kind: EntryKind,
    math_kind: Option<MathKind>,
    text: &str,
) -> Result<()> {
    validate_verification(cli.verification.as_deref())?;
    let entry = domain::create_entry(
        conn,
        kind,
        text,
        NewEntry {
            project,
            author: &cli.actor,
            origin: "human",
            source_uri: cli.source.as_deref(),
            repository: cli.repo.as_deref(),
            commit: cli.commit.as_deref(),
            file: cli.file.as_deref(),
            line: cli.line,
            occurred_at: None,
            math_kind,
            formal_system: cli.formal_system.as_deref(),
            verification: cli
                .verification
                .as_deref()
                .or_else(|| math_kind.map(|_| "unverified")),
        },
    )?;
    if cli.json {
        print_json_value(&json::entry(&entry));
    } else {
        let label = entry
            .math_kind
            .map(|kind| kind.to_string())
            .unwrap_or_else(|| entry.kind.to_string());
        println!("{label} {}", short_id(&entry.id));
    }
    Ok(())
}

fn capture_math(
    conn: &mut Connection,
    cli: &Cli,
    project: Option<&str>,
    math_kind: MathKind,
    text: &str,
) -> Result<()> {
    capture(
        conn,
        cli,
        project,
        math_kind.entry_kind(),
        Some(math_kind),
        text,
    )
}

fn validate_verification(value: Option<&str>) -> Result<()> {
    if let Some(value) = value {
        if !matches!(
            value,
            "unverified" | "checked" | "peer_reviewed" | "formally_verified"
        ) {
            anyhow::bail!("invalid verification state: {value}");
        }
    }
    Ok(())
}

/// A project is always explicit: pass `--project`, or omit it for `global`.
/// There is no remembered default.
pub(crate) fn effective_project(explicit: Option<&str>) -> Option<String> {
    explicit
        .filter(|project| !project.eq_ignore_ascii_case("global"))
        .map(str::to_string)
}

fn output_status(conn: &Connection, project: Option<&str>, json: bool) -> Result<()> {
    let entries = domain::list_entries(
        conn,
        EntryFilter {
            project,
            ..EntryFilter::default()
        },
    )?;
    let status = StatusOutput {
        product: "aporic",
        schema_version: db::schema_version(conn)?,
        project: project_name(project),
        entries: entries.len(),
        open_questions: entries
            .iter()
            .filter(|entry| entry.kind == EntryKind::Question && entry.state == "open")
            .count(),
        ready_actions: entries
            .iter()
            .filter(|entry| entry.kind == EntryKind::Action && entry.state == "open")
            .count(),
        mathematical_entries: entries
            .iter()
            .filter(|entry| entry.math_kind.is_some())
            .count(),
    };
    if json {
        print_json_value(&status_json(&status));
    } else {
        println!("project:        {}", status.project);
        println!("entries:        {}", status.entries);
        println!("open questions: {}", status.open_questions);
        println!("ready actions:  {}", status.ready_actions);
        println!("mathematics:    {}", status.mathematical_entries);
        println!("schema:         {}", status.schema_version);
    }
    Ok(())
}

fn output_entries(entries: &[Entry], json: bool) -> Result<()> {
    if json {
        print_json_value(&json::entries(entries));
    } else if entries.is_empty() {
        println!("no entries");
    } else {
        for entry in entries {
            println!(
                "{} {:<11} {:<11} {}",
                short_id(&entry.id),
                entry
                    .math_kind
                    .map(|kind| kind.to_string())
                    .unwrap_or_else(|| entry.kind.to_string()),
                entry.state,
                one_line(&entry.body)
            );
        }
    }
    Ok(())
}

fn print_entry(entry: &Entry) {
    println!("id:      {}", entry.id);
    println!("kind:    {}", entry.kind);
    if let Some(math_kind) = entry.math_kind {
        println!("math:    {math_kind}");
    }
    println!("state:   {}", entry.state);
    println!("project: {}", entry.project.as_deref().unwrap_or("global"));
    println!("author:  {}", entry.author);
    println!("origin:  {}", entry.origin);
    println!("revision:{}", entry.revision);
    if let Some(system) = &entry.formal_system {
        println!("system:  {system}");
    }
    if let Some(verification) = &entry.verification {
        println!("verify:  {verification}");
    }
    println!();
    println!("{}", entry.body);
}

fn output_trace(trace: &Trace, json: bool) -> Result<()> {
    if json {
        print_json_value(&json::trace(trace));
        return Ok(());
    }
    for entry in &trace.entries {
        let root = if entry.id == trace.root { "*" } else { " " };
        println!(
            "{root} {} {:<11} {}",
            short_id(&entry.id),
            entry.kind,
            one_line(&entry.body)
        );
    }
    if !trace.relations.is_empty() {
        println!();
        for relation in &trace.relations {
            println!(
                "  {} --{}--> {}",
                short_id(&relation.from_id),
                relation.kind,
                short_id(&relation.to_id)
            );
        }
    }
    Ok(())
}

fn print_json_value(value: &str) {
    println!("{value}");
}

fn status_json(status: &StatusOutput) -> String {
    format!(
        concat!(
            "{{\"product\":{},\"schema_version\":{},\"project\":{},",
            "\"entries\":{},\"open_questions\":{},\"ready_actions\":{},",
            "\"mathematical_entries\":{}}}"
        ),
        json::quote(status.product),
        status.schema_version,
        json::quote(&status.project),
        status.entries,
        status.open_questions,
        status.ready_actions,
        status.mathematical_entries
    )
}

fn short_id(id: &str) -> &str {
    id.get(..18).unwrap_or(id)
}

fn one_line(value: &str) -> String {
    value.replace(['\r', '\n'], " ")
}

fn project_name(project: Option<&str>) -> String {
    project.unwrap_or("global").to_string()
}
