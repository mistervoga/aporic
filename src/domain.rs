use anyhow::{bail, Context, Result};
use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension, Row};
use std::collections::{HashSet, VecDeque};
use std::fmt::{Display, Formatter};
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryKind {
    Observation,
    Claim,
    Assumption,
    Question,
    Implication,
    Action,
    Outcome,
    Learning,
}

impl EntryKind {
    pub const ALL: [EntryKind; 8] = [
        Self::Observation,
        Self::Claim,
        Self::Assumption,
        Self::Question,
        Self::Implication,
        Self::Action,
        Self::Outcome,
        Self::Learning,
    ];

    pub fn initial_state(self) -> &'static str {
        match self {
            Self::Observation | Self::Outcome => "recorded",
            Self::Question => "open",
            Self::Action => "open",
            Self::Assumption | Self::Claim | Self::Implication | Self::Learning => "active",
        }
    }
}

impl Display for EntryKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            Self::Observation => "observation",
            Self::Claim => "claim",
            Self::Assumption => "assumption",
            Self::Question => "question",
            Self::Implication => "implication",
            Self::Action => "action",
            Self::Outcome => "outcome",
            Self::Learning => "learning",
        };
        f.write_str(value)
    }
}

impl FromStr for EntryKind {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> Result<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "observation" | "observe" => Ok(Self::Observation),
            "claim" => Ok(Self::Claim),
            "assumption" | "assume" => Ok(Self::Assumption),
            "question" | "ask" => Ok(Self::Question),
            "implication" | "imply" => Ok(Self::Implication),
            "action" | "act" => Ok(Self::Action),
            "outcome" => Ok(Self::Outcome),
            "learning" | "learn" => Ok(Self::Learning),
            _ => bail!("unknown entry kind: {value}"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MathKind {
    Definition,
    Conjecture,
    Lemma,
    Theorem,
    Proof,
    Counterexample,
    Example,
    Calculation,
}

impl MathKind {
    pub const ALL: [MathKind; 8] = [
        Self::Definition,
        Self::Conjecture,
        Self::Lemma,
        Self::Theorem,
        Self::Proof,
        Self::Counterexample,
        Self::Example,
        Self::Calculation,
    ];

    pub fn entry_kind(self) -> EntryKind {
        match self {
            Self::Example => EntryKind::Observation,
            Self::Calculation => EntryKind::Outcome,
            Self::Definition
            | Self::Conjecture
            | Self::Lemma
            | Self::Theorem
            | Self::Proof
            | Self::Counterexample => EntryKind::Claim,
        }
    }
}

impl Display for MathKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Definition => "definition",
            Self::Conjecture => "conjecture",
            Self::Lemma => "lemma",
            Self::Theorem => "theorem",
            Self::Proof => "proof",
            Self::Counterexample => "counterexample",
            Self::Example => "example",
            Self::Calculation => "calculation",
        })
    }
}

impl FromStr for MathKind {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> Result<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "definition" | "define" => Ok(Self::Definition),
            "conjecture" => Ok(Self::Conjecture),
            "lemma" => Ok(Self::Lemma),
            "theorem" => Ok(Self::Theorem),
            "proof" | "prove" => Ok(Self::Proof),
            "counterexample" => Ok(Self::Counterexample),
            "example" => Ok(Self::Example),
            "calculation" | "calculate" => Ok(Self::Calculation),
            _ => bail!("unknown mathematical kind: {value}"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelationKind {
    Supports,
    Challenges,
    DerivedFrom,
    FollowsFrom,
    Answers,
    Tests,
    Motivates,
    ResultOf,
    Contradicts,
    Supersedes,
    RelatesTo,
    Defines,
    Proves,
    Disproves,
    Uses,
    DependsOn,
    Generalizes,
    Specializes,
    EquivalentTo,
    ExampleOf,
    CounterexampleTo,
}

impl Display for RelationKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            Self::Supports => "supports",
            Self::Challenges => "challenges",
            Self::DerivedFrom => "derived_from",
            Self::FollowsFrom => "follows_from",
            Self::Answers => "answers",
            Self::Tests => "tests",
            Self::Motivates => "motivates",
            Self::ResultOf => "result_of",
            Self::Contradicts => "contradicts",
            Self::Supersedes => "supersedes",
            Self::RelatesTo => "relates_to",
            Self::Defines => "defines",
            Self::Proves => "proves",
            Self::Disproves => "disproves",
            Self::Uses => "uses",
            Self::DependsOn => "depends_on",
            Self::Generalizes => "generalizes",
            Self::Specializes => "specializes",
            Self::EquivalentTo => "equivalent_to",
            Self::ExampleOf => "example_of",
            Self::CounterexampleTo => "counterexample_to",
        };
        f.write_str(value)
    }
}

impl FromStr for RelationKind {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> Result<Self> {
        match value.trim().to_ascii_lowercase().replace('-', "_").as_str() {
            "supports" | "supported_by" => Ok(Self::Supports),
            "challenges" | "challenged_by" => Ok(Self::Challenges),
            "derived_from" => Ok(Self::DerivedFrom),
            "follows_from" => Ok(Self::FollowsFrom),
            "answers" => Ok(Self::Answers),
            "tests" => Ok(Self::Tests),
            "motivates" => Ok(Self::Motivates),
            "result_of" => Ok(Self::ResultOf),
            "contradicts" => Ok(Self::Contradicts),
            "supersedes" => Ok(Self::Supersedes),
            "relates_to" => Ok(Self::RelatesTo),
            "defines" => Ok(Self::Defines),
            "proves" => Ok(Self::Proves),
            "disproves" => Ok(Self::Disproves),
            "uses" => Ok(Self::Uses),
            "depends_on" => Ok(Self::DependsOn),
            "generalizes" => Ok(Self::Generalizes),
            "specializes" => Ok(Self::Specializes),
            "equivalent_to" => Ok(Self::EquivalentTo),
            "example_of" => Ok(Self::ExampleOf),
            "counterexample_to" => Ok(Self::CounterexampleTo),
            _ => bail!("unknown relation kind: {value}"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Entry {
    pub id: String,
    pub legacy_task_id: Option<i64>,
    pub kind: EntryKind,
    pub body: String,
    pub details: Option<String>,
    pub state: String,
    pub project: Option<String>,
    pub author: String,
    pub origin: String,
    pub source_uri: Option<String>,
    pub repository: Option<String>,
    pub commit: Option<String>,
    pub file: Option<String>,
    pub line: Option<i64>,
    pub occurred_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub revision: i64,
    pub math_kind: Option<MathKind>,
    pub formal_system: Option<String>,
    pub verification: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Relation {
    pub id: String,
    pub from_id: String,
    pub kind: RelationKind,
    pub to_id: String,
    pub rationale: Option<String>,
    pub author: String,
    pub origin: String,
    pub created_at: String,
    pub revision: i64,
}

#[derive(Debug)]
pub struct Trace {
    pub root: String,
    pub entries: Vec<Entry>,
    pub relations: Vec<Relation>,
}

#[derive(Debug, Default)]
pub struct NewEntry<'a> {
    pub project: Option<&'a str>,
    pub author: &'a str,
    pub origin: &'a str,
    pub source_uri: Option<&'a str>,
    pub repository: Option<&'a str>,
    pub commit: Option<&'a str>,
    pub file: Option<&'a str>,
    pub line: Option<i64>,
    pub occurred_at: Option<&'a str>,
    pub math_kind: Option<MathKind>,
    pub formal_system: Option<&'a str>,
    pub verification: Option<&'a str>,
}

#[derive(Debug, Default)]
pub struct EntryFilter<'a> {
    pub project: Option<&'a str>,
    pub kind: Option<EntryKind>,
    pub state: Option<&'a str>,
    pub math_kind: Option<MathKind>,
}

pub fn create_entry(
    conn: &mut Connection,
    kind: EntryKind,
    body: &str,
    options: NewEntry<'_>,
) -> Result<Entry> {
    let body = normalize_body(body)?;
    let project_id = ensure_project(conn, options.project)?;
    let id = crate::db::new_id(conn)?;
    let now = Utc::now().to_rfc3339();
    let actor = if options.author.trim().is_empty() {
        "human"
    } else {
        options.author.trim()
    };
    let origin = if options.origin.trim().is_empty() {
        "human"
    } else {
        options.origin.trim()
    };

    let tx = conn.transaction()?;
    tx.execute(
        "INSERT INTO entries(
            id, kind, body, state, project_id, author, origin, source_uri,
            repository, commit_hash, file_path, line_number, occurred_at,
            created_at, updated_at, revision, math_kind, formal_system, verification
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12,
                   ?13, ?14, ?14, 1, ?15, ?16, ?17)",
        params![
            id,
            kind.to_string(),
            body,
            kind.initial_state(),
            project_id,
            actor,
            origin,
            options.source_uri,
            options.repository,
            options.commit,
            options.file,
            options.line,
            options.occurred_at,
            now,
            options.math_kind.map(|kind| kind.to_string()),
            options.formal_system,
            options.verification
        ],
    )?;
    let payload = format!(
        "{{\"kind\":{},\"body\":{}}}",
        crate::json::quote(&kind.to_string()),
        crate::json::quote(&body)
    );
    crate::db::insert_audit_event(&tx, actor, "create_entry", &id, &payload, &id, &now)?;
    tx.commit()?;
    get_entry(conn, &id)
}

pub fn create_relation(
    conn: &mut Connection,
    from: &str,
    kind: RelationKind,
    to: &str,
    rationale: Option<&str>,
    actor: &str,
) -> Result<Relation> {
    let from = resolve_id(conn, from)?;
    let to = resolve_id(conn, to)?;
    if from == to {
        bail!("an entry cannot be related to itself");
    }
    let id = crate::db::new_id(conn)?;
    let now = Utc::now().to_rfc3339();
    let actor = if actor.trim().is_empty() {
        "human"
    } else {
        actor.trim()
    };
    let tx = conn.transaction()?;
    tx.execute(
        "INSERT INTO relations(
            id, from_id, kind, to_id, rationale, author, origin, created_at, revision
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'human', ?7, 1)",
        params![id, from, kind.to_string(), to, rationale, actor, now],
    )?;
    let payload = format!(
        "{{\"from\":{},\"kind\":{},\"to\":{},\"rationale\":{}}}",
        crate::json::quote(&from),
        crate::json::quote(&kind.to_string()),
        crate::json::quote(&to),
        crate::json::option(rationale)
    );
    crate::db::insert_audit_event(&tx, actor, "create_relation", &id, &payload, &id, &now)?;
    tx.commit()?;
    get_relation(conn, &id)
}

pub fn get_entry(conn: &Connection, identifier: &str) -> Result<Entry> {
    let id = resolve_id(conn, identifier)?;
    query_entry_by_id(conn, &id)?.context("entry disappeared while reading")
}

pub fn list_entries(conn: &Connection, filter: EntryFilter<'_>) -> Result<Vec<Entry>> {
    let mut sql = String::from(
        "SELECT e.id, e.legacy_task_id, e.kind, e.body, e.details, e.state,
                p.name, e.author, e.origin, e.source_uri, e.repository,
                e.commit_hash, e.file_path, e.line_number, e.occurred_at,
                e.created_at, e.updated_at, e.revision, e.math_kind,
                e.formal_system, e.verification
         FROM entries e LEFT JOIN projects p ON p.id=e.project_id WHERE 1=1",
    );
    let mut values = Vec::new();
    if let Some(project) = filter.project {
        if project.eq_ignore_ascii_case("global") {
            sql.push_str(" AND e.project_id IS NULL");
        } else {
            sql.push_str(" AND p.name=?");
            values.push(project.to_string());
        }
    }
    if let Some(kind) = filter.kind {
        sql.push_str(" AND e.kind=?");
        values.push(kind.to_string());
    }
    if let Some(state) = filter.state {
        sql.push_str(" AND e.state=?");
        values.push(state.to_string());
    }
    if let Some(math_kind) = filter.math_kind {
        sql.push_str(" AND e.math_kind=?");
        values.push(math_kind.to_string());
    }
    sql.push_str(" ORDER BY e.created_at, e.id");

    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(rusqlite::params_from_iter(values), map_entry)?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

pub fn trace_entry(conn: &Connection, identifier: &str, depth: usize) -> Result<Trace> {
    let root = resolve_id(conn, identifier)?;
    let mut seen = HashSet::from([root.clone()]);
    let mut queue = VecDeque::from([(root.clone(), 0_usize)]);
    let mut relations = Vec::new();

    while let Some((current, level)) = queue.pop_front() {
        if level >= depth {
            continue;
        }
        for relation in relations_for(conn, &current)? {
            let other = if relation.from_id == current {
                relation.to_id.clone()
            } else {
                relation.from_id.clone()
            };
            if seen.insert(other.clone()) {
                queue.push_back((other, level + 1));
            }
            if !relations
                .iter()
                .any(|existing: &Relation| existing.id == relation.id)
            {
                relations.push(relation);
            }
        }
    }

    let mut entries = seen
        .iter()
        .map(|id| query_entry_by_id(conn, id)?.context("trace contains a missing entry"))
        .collect::<Result<Vec<_>>>()?;
    entries.sort_by(|a, b| a.created_at.cmp(&b.created_at).then(a.id.cmp(&b.id)));
    relations.sort_by(|a, b| a.created_at.cmp(&b.created_at).then(a.id.cmp(&b.id)));
    Ok(Trace {
        root,
        entries,
        relations,
    })
}

pub fn complete_action(conn: &mut Connection, identifier: &str) -> Result<Entry> {
    let id = resolve_id(conn, identifier)?;
    let entry = query_entry_by_id(conn, &id)?.context("no such entry")?;
    if entry.kind != EntryKind::Action {
        bail!("only actions can be completed");
    }
    if entry.state == "done" {
        return Ok(entry);
    }
    let now = Utc::now().to_rfc3339();
    let tx = conn.transaction()?;
    tx.execute(
        "UPDATE entries SET state='done', completed_at=?1, updated_at=?1,
                            revision=revision+1
         WHERE id=?2 AND revision=?3",
        params![now, id, entry.revision],
    )?;
    crate::db::insert_audit_event(
        &tx,
        "human",
        "complete_action",
        &id,
        r#"{"state":"done"}"#,
        &id,
        &now,
    )?;
    tx.commit()?;
    get_entry(conn, &id)
}

pub fn list_projects(conn: &Connection) -> Result<Vec<String>> {
    let mut projects = vec!["global".to_string()];
    let mut stmt = conn.prepare("SELECT name FROM projects ORDER BY name")?;
    let names = stmt.query_map([], |row| row.get::<_, String>(0))?;
    projects.extend(names.collect::<rusqlite::Result<Vec<_>>>()?);
    Ok(projects)
}

fn ensure_project(conn: &Connection, project: Option<&str>) -> Result<Option<i64>> {
    let project = match project.map(str::trim) {
        Some("") | None => return Ok(None),
        Some(project) if project.eq_ignore_ascii_case("global") => return Ok(None),
        Some(project) => project,
    };
    conn.execute(
        "INSERT OR IGNORE INTO projects(name) VALUES (?1)",
        params![project],
    )?;
    Ok(Some(conn.query_row(
        "SELECT id FROM projects WHERE name=?1",
        params![project],
        |row| row.get(0),
    )?))
}

fn normalize_body(body: &str) -> Result<String> {
    let body = body.trim();
    if body.is_empty() {
        bail!("entry body cannot be empty");
    }
    if body.chars().count() > 10_000 {
        bail!("entry body exceeds 10,000 characters");
    }
    Ok(body.to_string())
}

fn resolve_id(conn: &Connection, identifier: &str) -> Result<String> {
    let identifier = identifier.trim();
    if identifier.is_empty() {
        bail!("entry id cannot be empty");
    }
    if let Ok(legacy) = identifier.parse::<i64>() {
        if let Some(id) = conn
            .query_row(
                "SELECT id FROM entries WHERE legacy_task_id=?1",
                params![legacy],
                |row| row.get(0),
            )
            .optional()?
        {
            return Ok(id);
        }
    }

    let pattern = format!("{identifier}%");
    let mut stmt = conn.prepare("SELECT id FROM entries WHERE id LIKE ?1 ORDER BY id LIMIT 2")?;
    let ids = stmt
        .query_map(params![pattern], |row| row.get::<_, String>(0))?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    match ids.as_slice() {
        [id] => Ok(id.clone()),
        [] => bail!("no entry matches id: {identifier}"),
        _ => bail!("ambiguous entry id prefix: {identifier}"),
    }
}

fn query_entry_by_id(conn: &Connection, id: &str) -> Result<Option<Entry>> {
    Ok(conn
        .query_row(
            "SELECT e.id, e.legacy_task_id, e.kind, e.body, e.details, e.state,
                    p.name, e.author, e.origin, e.source_uri, e.repository,
                    e.commit_hash, e.file_path, e.line_number, e.occurred_at,
                    e.created_at, e.updated_at, e.revision, e.math_kind,
                    e.formal_system, e.verification
             FROM entries e LEFT JOIN projects p ON p.id=e.project_id
             WHERE e.id=?1",
            params![id],
            map_entry,
        )
        .optional()?)
}

fn map_entry(row: &Row<'_>) -> rusqlite::Result<Entry> {
    let kind_text: String = row.get(2)?;
    let kind = EntryKind::from_str(&kind_text).map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(2, rusqlite::types::Type::Text, error.into())
    })?;
    Ok(Entry {
        id: row.get(0)?,
        legacy_task_id: row.get(1)?,
        kind,
        body: row.get(3)?,
        details: row.get(4)?,
        state: row.get(5)?,
        project: row.get(6)?,
        author: row.get(7)?,
        origin: row.get(8)?,
        source_uri: row.get(9)?,
        repository: row.get(10)?,
        commit: row.get(11)?,
        file: row.get(12)?,
        line: row.get(13)?,
        occurred_at: row.get(14)?,
        created_at: row.get(15)?,
        updated_at: row.get(16)?,
        revision: row.get(17)?,
        math_kind: row
            .get::<_, Option<String>>(18)?
            .map(|value| MathKind::from_str(&value))
            .transpose()
            .map_err(|error| {
                rusqlite::Error::FromSqlConversionFailure(
                    18,
                    rusqlite::types::Type::Text,
                    error.into(),
                )
            })?,
        formal_system: row.get(19)?,
        verification: row.get(20)?,
    })
}

fn get_relation(conn: &Connection, id: &str) -> Result<Relation> {
    Ok(conn.query_row(
        "SELECT id, from_id, kind, to_id, rationale, author, origin,
                created_at, revision FROM relations WHERE id=?1",
        params![id],
        map_relation,
    )?)
}

fn relations_for(conn: &Connection, id: &str) -> Result<Vec<Relation>> {
    let mut stmt = conn.prepare(
        "SELECT id, from_id, kind, to_id, rationale, author, origin,
                created_at, revision
         FROM relations WHERE from_id=?1 OR to_id=?1 ORDER BY created_at, id",
    )?;
    let rows = stmt.query_map(params![id], map_relation)?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

fn map_relation(row: &Row<'_>) -> rusqlite::Result<Relation> {
    let kind_text: String = row.get(2)?;
    let kind = RelationKind::from_str(&kind_text).map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(2, rusqlite::types::Type::Text, error.into())
    })?;
    Ok(Relation {
        id: row.get(0)?,
        from_id: row.get(1)?,
        kind,
        to_id: row.get(3)?,
        rationale: row.get(4)?,
        author: row.get(5)?,
        origin: row.get(6)?,
        created_at: row.get(7)?,
        revision: row.get(8)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn connection() -> Connection {
        let mut conn = Connection::open_in_memory().expect("open database");
        crate::db::ensure_schema(&mut conn).expect("create schema");
        conn
    }

    fn options<'a>() -> NewEntry<'a> {
        NewEntry {
            author: "tester",
            origin: "human",
            ..NewEntry::default()
        }
    }

    #[test]
    fn creates_and_traces_reasoning_chain() {
        let mut conn = connection();
        let observation = create_entry(
            &mut conn,
            EntryKind::Observation,
            "latency increased",
            options(),
        )
        .expect("observation");
        let assumption = create_entry(
            &mut conn,
            EntryKind::Assumption,
            "the index caused it",
            options(),
        )
        .expect("assumption");
        create_relation(
            &mut conn,
            &observation.id,
            RelationKind::Supports,
            &assumption.id,
            None,
            "tester",
        )
        .expect("relation");

        let trace = trace_entry(&conn, &assumption.id[..18], 2).expect("trace");
        assert_eq!(trace.entries.len(), 2);
        assert_eq!(trace.relations.len(), 1);
    }

    #[test]
    fn only_actions_can_complete() {
        let mut conn = connection();
        let question = create_entry(&mut conn, EntryKind::Question, "what changed?", options())
            .expect("question");
        assert!(complete_action(&mut conn, &question.id).is_err());
    }
}
