use crate::domain::{Entry, Relation, Trace};

pub fn quote(value: &str) -> String {
    let mut output = String::with_capacity(value.len() + 2);
    output.push('"');
    for character in value.chars() {
        match character {
            '"' => output.push_str("\\\""),
            '\\' => output.push_str("\\\\"),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            '\u{08}' => output.push_str("\\b"),
            '\u{0c}' => output.push_str("\\f"),
            value if value <= '\u{1f}' => {
                output.push_str(&format!("\\u{:04x}", value as u32));
            }
            value => output.push(value),
        }
    }
    output.push('"');
    output
}

pub fn option(value: Option<&str>) -> String {
    value.map(quote).unwrap_or_else(|| "null".to_string())
}

pub fn entry(value: &Entry) -> String {
    format!(
        concat!(
            "{{\"id\":{},\"kind\":{},\"body\":{},",
            "\"details\":{},\"state\":{},\"project\":{},\"author\":{},",
            "\"origin\":{},\"source_uri\":{},\"repository\":{},\"commit\":{},",
            "\"file\":{},\"line\":{},\"occurred_at\":{},\"created_at\":{},",
            "\"updated_at\":{},\"revision\":{},\"math_kind\":{},",
            "\"formal_system\":{},\"verification\":{}}}"
        ),
        quote(&value.id),
        quote(&value.kind.to_string()),
        quote(&value.body),
        option(value.details.as_deref()),
        quote(&value.state),
        option(value.project.as_deref()),
        quote(&value.author),
        quote(&value.origin),
        option(value.source_uri.as_deref()),
        option(value.repository.as_deref()),
        option(value.commit.as_deref()),
        option(value.file.as_deref()),
        value
            .line
            .map(|line| line.to_string())
            .unwrap_or_else(|| "null".to_string()),
        option(value.occurred_at.as_deref()),
        quote(&value.created_at),
        quote(&value.updated_at),
        value.revision,
        value
            .math_kind
            .map(|kind| quote(&kind.to_string()))
            .unwrap_or_else(|| "null".to_string()),
        option(value.formal_system.as_deref()),
        option(value.verification.as_deref())
    )
}

pub fn relation(value: &Relation) -> String {
    format!(
        concat!(
            "{{\"id\":{},\"from_id\":{},\"kind\":{},\"to_id\":{},",
            "\"rationale\":{},\"author\":{},\"origin\":{},",
            "\"created_at\":{},\"revision\":{}}}"
        ),
        quote(&value.id),
        quote(&value.from_id),
        quote(&value.kind.to_string()),
        quote(&value.to_id),
        option(value.rationale.as_deref()),
        quote(&value.author),
        quote(&value.origin),
        quote(&value.created_at),
        value.revision
    )
}

pub fn entries(values: &[Entry]) -> String {
    array(values.iter().map(entry))
}

pub fn strings(values: &[String]) -> String {
    array(values.iter().map(|value| quote(value)))
}

pub fn trace(value: &Trace) -> String {
    format!(
        "{{\"root\":{},\"entries\":{},\"relations\":{}}}",
        quote(&value.root),
        entries(&value.entries),
        array(value.relations.iter().map(relation))
    )
}

pub fn array(values: impl Iterator<Item = String>) -> String {
    format!("[{}]", values.collect::<Vec<_>>().join(","))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escapes_json_control_characters() {
        assert_eq!(quote("a\n\"b\\c\u{01}"), "\"a\\n\\\"b\\\\c\\u0001\"");
    }
}
