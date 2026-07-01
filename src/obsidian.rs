use crate::domain::{list_entries, Entry, EntryFilter, EntryKind, MathKind};
use anyhow::{bail, Context, Result};
use rusqlite::Connection;
use std::path::{Path, PathBuf};

const START: &str = "<!-- aporic:start version=1 -->";
const END: &str = "<!-- aporic:end -->";

pub fn export(conn: &Connection, project: Option<&str>, path: &Path) -> Result<usize> {
    let entries = list_entries(
        conn,
        EntryFilter {
            project,
            ..EntryFilter::default()
        },
    )?;
    let generated = render(&entries);
    let existing = if path.exists() {
        std::fs::read_to_string(path).with_context(|| format!("cannot read {}", path.display()))?
    } else {
        String::new()
    };
    let output = replace_owned_section(&existing, &generated)?;
    atomic_write(path, output.as_bytes())?;
    Ok(entries.len())
}

pub fn render(entries: &[Entry]) -> String {
    let mut output = format!("{START}\n");
    for math_kind in MathKind::ALL {
        let matching = entries
            .iter()
            .filter(|entry| entry.math_kind == Some(math_kind))
            .collect::<Vec<_>>();
        if matching.is_empty() {
            continue;
        }
        output.push_str(&format!("\n## {}\n\n", math_heading(math_kind)));
        for entry in matching {
            render_entry(&mut output, entry);
        }
    }
    for kind in EntryKind::ALL {
        let matching = entries
            .iter()
            .filter(|entry| entry.kind == kind && entry.math_kind.is_none())
            .collect::<Vec<_>>();
        if matching.is_empty() {
            continue;
        }
        output.push_str(&format!("\n## {}\n\n", heading(kind)));
        for entry in matching {
            render_entry(&mut output, entry);
        }
    }
    output.push_str(&format!("\n{END}"));
    output
}

fn render_entry(output: &mut String, entry: &Entry) {
    let body = entry.body.replace(['\r', '\n'], " ");
    if entry.kind == EntryKind::Action {
        let mark = if entry.state == "done" { "x" } else { " " };
        output.push_str(&format!("- [{mark}] {body}"));
    } else {
        output.push_str(&format!("- {body}"));
    }
    output.push_str(&format!(
        " <!-- aporic:id={} kind={} math_kind={} revision={} verification={} -->\n",
        entry.id,
        entry.kind,
        entry
            .math_kind
            .map(|kind| kind.to_string())
            .unwrap_or_else(|| "none".to_string()),
        entry.revision,
        entry.verification.as_deref().unwrap_or("none")
    ));
}

fn math_heading(kind: MathKind) -> &'static str {
    match kind {
        MathKind::Definition => "Definitions",
        MathKind::Conjecture => "Conjectures",
        MathKind::Lemma => "Lemmas",
        MathKind::Theorem => "Theorems",
        MathKind::Proof => "Proofs",
        MathKind::Counterexample => "Counterexamples",
        MathKind::Example => "Examples",
        MathKind::Calculation => "Calculations",
    }
}

fn heading(kind: EntryKind) -> &'static str {
    match kind {
        EntryKind::Observation => "Observations",
        EntryKind::Claim => "Claims",
        EntryKind::Assumption => "Assumptions",
        EntryKind::Question => "Open questions",
        EntryKind::Implication => "Implications",
        EntryKind::Action => "Actions",
        EntryKind::Outcome => "Outcomes",
        EntryKind::Learning => "Learnings",
    }
}

fn replace_owned_section(existing: &str, generated: &str) -> Result<String> {
    match marker_range(existing, START, END)? {
        Some((start, end)) => Ok(format!(
            "{}{}{}",
            &existing[..start],
            generated,
            &existing[end..]
        )),
        None if existing.is_empty() => Ok(format!("{generated}\n")),
        None => Ok(format!("{}\n\n{generated}\n", existing.trim_end())),
    }
}

fn marker_range(
    existing: &str,
    start_marker: &str,
    end_marker: &str,
) -> Result<Option<(usize, usize)>> {
    let start = existing.find(start_marker);
    let end = existing.find(end_marker);
    match (start, end) {
        (None, None) => Ok(None),
        (Some(start), Some(end)) if start < end => {
            if existing[start + start_marker.len()..].contains(start_marker)
                || existing[end + end_marker.len()..].contains(end_marker)
            {
                bail!("target contains duplicate generated-section markers");
            }
            Ok(Some((start, end + end_marker.len())))
        }
        _ => bail!("target contains invalid generated-section markers"),
    }
}

fn atomic_write(path: &Path, contents: &[u8]) -> Result<()> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    std::fs::create_dir_all(parent)?;
    let file_name = path
        .file_name()
        .context("Obsidian export path must name a file")?
        .to_string_lossy();
    let temporary: PathBuf = parent.join(format!(".{file_name}.aporic.tmp"));
    std::fs::write(&temporary, contents)?;
    std::fs::rename(&temporary, path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preserves_handwritten_content_around_the_owned_section() {
        let existing = "# Investigation\n\nBefore\n\n<!-- aporic:start version=1 -->\nold\n<!-- aporic:end -->\n\nAfter\n";
        let generated = "<!-- aporic:start version=1 -->\nnew\n<!-- aporic:end -->";
        let output = replace_owned_section(existing, generated).expect("replace");
        assert_eq!(
            output,
            "# Investigation\n\nBefore\n\n<!-- aporic:start version=1 -->\nnew\n<!-- aporic:end -->\n\nAfter\n"
        );
    }

    #[test]
    fn rejects_unbalanced_markers() {
        assert!(replace_owned_section("<!-- aporic:start version=1 -->", "new").is_err());
    }
}
