# Aporic

**Aporic is a local-first developer tool for moving from uncertainty to justified action.**

> Know what you know. Mark what you do not. Act on the implications.

Aporic records reasoning as small, typed entries instead of mixing facts,
guesses, questions, and work into one task list:

```text
source -> observation -> claim -> question -> implication -> action -> outcome -> learning
```

It is designed for debugging, technical investigations, architecture
decisions, incident learning, dependency evaluation, security analysis, and
supervising AI-generated work. SQLite is authoritative; Markdown, JSON,
Obsidian, and future MCP tools are interoperable views.

## Current status

Aporic implements the first trustworthy core: typed observations, claims,
assumptions, questions, implications, actions, outcomes, and learnings;
directed relations and bounded reasoning traces; UUIDv7 public IDs, revisions,
actor/origin attribution, and audit events; explicit projects and stable JSON
output; an interactive terminal view (`aporic tui`); and non-destructive,
versioned Obsidian export.

AI providers, MCP, and bidirectional Obsidian sync remain roadmap work. See
[AGENT.md](AGENT.md) for the full design and roadmap.

## Build

```bash
cargo build --release
```

To install into Cargo's binary directory (usually `~/.cargo/bin` — make sure
it's on `PATH`):

```bash
cargo install --path .
```

## Quick start

```bash
aporic init      # create the database and show its schema version
aporic tutor     # learn the tool interactively, in a throw-away sandbox
aporic tui       # browse and inspect your actual entries
```

`aporic tutor` starts with a five-minute quickstart that runs real commands in
a throw-away sandbox. It takes you from an observation through a question and
action to a connected trace, then offers optional topics for capture types,
projects, inspection, the TUI, and Obsidian export. At any tutor prompt, use
`help`, `commands`, `menu`, or `quit`.

The quickstart makes this reasoning path visible as you work:

```text
Observe -> Ask -> Act -> Connect -> Trace
```

It fills the generated entry IDs into the link and trace commands for you.
Those IDs stay unique even when other sandbox entries were created from the
optional topics. The sandbox is discarded when you quit.

### Tutor controls

| Input | Effect |
|---|---|
| `help` | Show the current step's input hint and example |
| `commands` | Show the implemented CLI command reference |
| `menu` | Open the optional learning topics; return to the same quickstart step |
| `quit` / `q` / `exit` | Discard the sandbox and leave the tutor |

Inside `aporic tui`, press `?` for its full key reference. While that overlay
is open, `?`, `q`, or `Esc` closes it before any normal TUI action is handled.

## Commands

Every command accepts `--project <name>` (omit for `global`) and `--json` for
machine-readable output. Run `aporic <command> --help` for a command's full
options.

| Command | Purpose |
|---|---|
| `init` | Initialize the database, show schema version |
| `observe` / `claim` / `assume` / `ask` / `imply` / `act` / `outcome` / `learn` | Capture one typed reasoning entry |
| `define` / `conjecture` / `lemma` / `theorem` / `proof` / `counterexample` / `example` / `calculate` | Capture one typed mathematical entry |
| `list` | List entries, optionally filtered by `--type`, `--state`, `--math-type` |
| `show <id>` | Show one entry by UUID or unique UUID prefix |
| `link <from> <relation> <to>` | Connect two entries with a typed, directed relation |
| `trace <id>` | Show the local reasoning graph around an entry |
| `complete <id>` | Mark an action as done |
| `actions` | List ready actions |
| `projects` | List known projects |
| `status` | Show database and entry counts |
| `tui` | Browse entries and traces; press `?` for the complete key reference |
| `tutor` | Sandboxed five-minute quickstart with optional learning topics |
| `obsidian export <path>` | Export entries into a fenced, versioned section of an Obsidian note |

## Obsidian

Aporic writes only between versioned markers:

```markdown
<!-- aporic:start version=1 -->
...
<!-- aporic:end -->
```

Handwritten content outside the markers is preserved on every re-export.
The tutor's Export topic walks through this against a scratch file before you
try it on a real note.

## Storage

```text
Linux:   ~/.local/share/aporic/aporic.db
macOS:   ~/Library/Application Support/aporic/aporic.db
Windows: %APPDATA%/aporic/aporic.db
```

## Roadmap

Implemented: schema versioning and migrations, typed entries and relations,
trace queries, projects, stable `--json` output, audit events, the TUI, the
tutor, and versioned Obsidian export.

Next, in order:

1. Source-management commands, so evidence is captured and re-checked rather than only referenced.
2. A read-only MCP server for entries, evidence, and traces.
3. Guarded, attributed MCP proposals and mutations.
4. `aporic ai examine`, against a deterministic fake provider before any real one.
5. Bidirectional Obsidian sync, once conflicts have a defined resolution.

## Contributing

Aporic is an open community project. See [CONTRIBUTING.md](CONTRIBUTING.md)
for how to build, test, and propose changes.

## License

MIT. See [LICENSE](LICENSE).
