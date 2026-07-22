# Tutor onboarding design

## Goal

A new user should understand Aporic's core value in five minutes by completing
one real reasoning flow in the sandboxed tutor. The flow must make the
connection from observation to justified action visible without requiring the
user to memorize or copy identifiers.

## Quickstart flow

`aporic tutor` starts the quickstart immediately instead of opening a lesson
menu. Every screen shows the current position in a plain-text progress line:

```text
Aporic quickstart                         step 1 of 5

[>] Observe -> [ ] Ask -> [ ] Act -> [ ] Connect -> [ ] Trace
```

The five sections are:

1. Capture an observation.
2. Formulate a question derived from it.
3. Record a concrete action.
4. Connect the three entries using exact commands containing the IDs created
   in the sandbox.
5. Inspect the resulting trace.

The tutor continues to parse and execute real `aporic` commands through the
existing shared `dispatch` function. It may inspect its in-memory database to
insert the generated ID prefixes into the next prompt, but it must not invent
tutor-only ID syntax or duplicate command behavior.

After the quickstart, the tutor shows a short summary and opens the optional
topic menu.

## Help and topic menu

The following tutor controls work at every tutor prompt and never advance the
current step:

- `help` explains the current step and repeats its valid example.
- `commands` renders the implemented CLI command reference from Clap's command
  definition so the tutor cannot drift from the CLI.
- `menu` opens the optional topic menu; returning resumes the current
  quickstart step.
- `quit` exits and discards the sandbox. The existing `q` and `exit` aliases
  remain accepted.

The topic menu is organized by user task:

- **Capture types:** ordinary and mathematical entry vocabulary.
- **Find and finish:** list filters, `show`, `trace`, `actions`, and `complete`.
- **Context:** projects and capture metadata.
- **Workbench:** the terminal UI.
- **Export:** safe Obsidian export.

The tutor documents only implemented behavior. Roadmap commands such as AI,
MCP, source management, and bidirectional sync are not presented as available.

## TUI help

Pressing `?` in the TUI opens a dismissible help overlay listing every active
key and its effect. Pressing `?`, `Esc`, or `q` closes the overlay without
closing the TUI. While the overlay is open, no entry action is performed.

The normal status line remains concise. The tutor's Workbench topic introduces
the same bindings shown by the overlay, so both surfaces use consistent terms.

## Architecture and scope

- Keep domain behavior in the existing domain and dispatch paths.
- Keep tutor orchestration in `src/tutor.rs` and TUI rendering/input in
  `src/tui.rs`.
- Reuse Clap's generated command definition for `commands`.
- Add no dependency and no persistent configuration.
- Keep output plain-text and understandable without color or Unicode glyphs.
- Do not redesign the domain model, command syntax, database schema, or
  Obsidian format.

The main entry point must dispatch `Command::Tutor` before opening the normal
database. This restores the documented invariant that the tutor touches only
its in-memory sandbox. All other commands keep the existing connection path.

The stale `show` description referring to legacy task IDs is corrected to say
that only a UUID or unique UUID prefix is accepted.

## Failure behavior

- Empty input repeats the current prompt.
- Invalid quoting or invalid CLI syntax leaves the user on the current step and
  prints the step-specific example plus a `help` hint.
- A valid command that does not match the current step does not run and does not
  advance progress.
- End-of-file, `quit`, `q`, and `exit` exit cleanly and discard the sandbox.
- Tutor help and menu navigation never mutate entries.
- The TUI help overlay captures its own close keys before normal key handling.

## Verification

Tests cover the smallest behavior-bearing seams:

- quickstart step order and prompts containing the generated ID prefixes;
- tutor control commands and invalid input preserving the current step;
- launching `aporic tutor` with an isolated data directory does not create the
  normal database;
- TUI help visibility and close-key handling;
- the corrected CLI help text.

The completed change must pass:

```bash
cargo fmt -- --check
cargo test
cargo clippy --all-targets --all-features -- -D warnings
```
