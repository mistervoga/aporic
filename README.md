# Aporic

**Aporic is a local-first developer tool for moving from uncertainty to justified action.**

> Know what you know. Mark what you do not. Act on the implications.

Aporic records reasoning as small, typed entries instead of mixing facts, guesses, questions, and work into one task list:

```text
source -> observation -> claim -> question -> implication -> action -> outcome -> learning
```

It is designed for debugging, technical investigations, architecture decisions, incident learning, dependency evaluation, security analysis, and supervising AI-generated work. SQLite is authoritative; Markdown, JSON, Obsidian, and future MCP tools are interoperable views.

## Current status

Aporic is an early transition from the Nanoshift task-manager prototype. The current release implements the first trustworthy core:

- typed observations, claims, assumptions, questions, implications, actions, outcomes, and learnings;
- directed relations and bounded reasoning traces;
- UUIDv7 public IDs, revisions, actor/origin attribution, and audit events;
- explicit projects and stable JSON output;
- migration of existing Nanoshift tasks into Aporic actions;
- legacy numeric task-ID lookup and an `nsh` compatibility binary;
- non-destructive, versioned Obsidian export.

AI providers, MCP, bidirectional Obsidian sync, source management, and the TUI remain roadmap work. The non-AI workflow must be useful first.

## Build

```bash
cargo build --release
```

To install both binaries into Cargo's binary directory:

```bash
cargo install --path .
```

Ensure Cargo's binary directory (normally `~/.cargo/bin`) is on `PATH`.

Canonical binary:

```text
target/release/aporic
```

The transitional `target/release/nsh` binary runs the same command surface.

Initialize the database and inspect the available commands:

```bash
aporic init
aporic --help
aporic link --help
```

## How to use Aporic

The recommended workflow is to resolve one uncertainty at a time. Do not begin
by creating a large graph. Capture the smallest useful chain, connect only the
relationships that explain a decision, and add structure as the investigation
develops.

Use one explicit project for each investigation or topic:

```bash
aporic --project checkout status
```

`--project` is a global option and can appear before or after a subcommand. An
entry without a project belongs to `global`. Prefer explicit projects in
scripts and when several terminals or agents use Aporic concurrently.

### The idea-resolution lifecycle

The core lifecycle separates what happened, what you currently believe, what
is unknown, and what you decide to do:

| Entry | Use it for | Do not use it for |
|---|---|---|
| `observation` | Something directly noticed or measured | An explanation of why it happened |
| `claim` | A statement that evidence can support or challenge | A fact that needs no scrutiny |
| `assumption` | A premise currently relied upon without enough evidence | A hidden dependency in your reasoning |
| `question` | A meaningful unresolved unknown | A disguised task |
| `implication` | What follows if a premise holds | A conclusion with omitted premises |
| `action` | A concrete next step | A vague intention |
| `outcome` | What happened after an action | Proof that the original hypothesis was correct |
| `learning` | A durable update to future understanding or practice | A transient status update |

Not every investigation needs every entry type. A small chain of observation,
question, action, and outcome is valid. Add claims, assumptions, and
implications when they make the reasoning easier to inspect.

#### 1. Capture the initial evidence and uncertainty

```bash
aporic --project checkout observe "p95 latency increased after the schema change"
aporic --project checkout claim "the schema change caused the regression"
aporic --project checkout assume "the new index increased write contention"
aporic --project checkout ask "does the regression reproduce without the index?"
```

Each command prints a durable ID. The examples below use descriptive
placeholders; replace them with the printed full ID or a unique ID prefix.

#### 2. Record what follows and choose a discriminating action

An implication should expose its premise. A useful action should reduce the
important uncertainty rather than merely produce activity:

```bash
aporic --project checkout imply \
  "if the index is responsible, removing it should restore write latency"
aporic --project checkout act \
  "benchmark both schemas under the production write ratio"
```

#### 3. Connect the reasoning

Relations are directed: `aporic link A RELATION B` means “A has RELATION to
B.” Add a rationale when the edge would otherwise be ambiguous:

```bash
aporic link OBSERVATION_ID supports CLAIM_ID
aporic link ASSUMPTION_ID derived-from OBSERVATION_ID
aporic link IMPLICATION_ID follows-from ASSUMPTION_ID
aporic link ACTION_ID tests ASSUMPTION_ID
aporic link IMPLICATION_ID motivates ACTION_ID \
  --rationale "the benchmark tests the predicted consequence"
```

Use `relates-to` only when no more precise relation fits. A relation records
provenance; it does not automatically make either entry true.

#### 4. Inspect the active chain

```bash
aporic trace ASSUMPTION_ID
aporic trace ASSUMPTION_ID --depth 3
aporic --project checkout list --type question --state open
aporic --project checkout actions
aporic show ACTION_ID
```

`trace` is intentionally bounded. Increase its depth only when the local chain
does not explain the decision.

#### 5. Record the result without rewriting history

After performing the action, record what happened and then complete the
action:

```bash
aporic --project checkout outcome \
  "without the index, p95 write latency returned from 480 ms to 95 ms"
aporic link OUTCOME_ID result-of ACTION_ID
aporic link OUTCOME_ID supports CLAIM_ID \
  --rationale "the controlled comparison reproduced the predicted change"
aporic complete ACTION_ID
```

Only actions can be completed. Completion does not prove the claim or
assumption that the action tested; the outcome and its evidence remain separate.

#### 6. Preserve the learning

Record a learning only when the outcome changes how future work should be
performed:

```bash
aporic --project checkout learn \
  "schema changes on this workload require read and write benchmarks"
aporic link LEARNING_ID derived-from OUTCOME_ID
aporic trace LEARNING_ID --depth 3
```

This final link preserves both the result and the reason for changing future
practice.

### Practical working habits

- Capture observations before explanations so later conclusions do not rewrite
  the original evidence.
- Keep competing assumptions as separate entries and link evidence that
  supports or challenges each one.
- Write expected consequences before running an experiment. This makes an
  outcome easier to interpret without hindsight bias.
- Prefer an action that distinguishes between competing explanations.
- Record failed actions and negative outcomes; they prevent repeated work.
- Use concise entry bodies. Put one independently testable idea in each entry.
- Use `trace` to answer “why are we doing this?” before adding more entries.
- Treat UUIDs as identity. Text and project names may change conceptually, but
  they are not durable identifiers.

### Available general relations

```text
supports       challenges       derived-from    follows-from
answers        tests            motivates       result-of
contradicts    supersedes       relates-to
```

## Mathematical proofs

Mathematics uses the same epistemic model with an additional mathematical
kind. A theorem is still a claim, a proof is evidence for a claim, and a
counterexample can disprove or restrict one. Numerical agreement can support a
conjecture, but it is not a proof.

The recommended mathematical lifecycle is:

```text
definition -> examples -> conjecture -> counterexample or lemmas
                              |                 |
                              v                 v
                       revised theorem <- proof
```

### 1. Fix definitions and the mathematical setting

```bash
aporic --project functional-analysis \
  --formal-system "normed spaces" \
  define 'A sequence $(x_n)$ is Cauchy if for every $\varepsilon>0$ there is an $N$ such that $\|x_m-x_n\|<\varepsilon$ for all $m,n\ge N$.'

aporic --project functional-analysis example \
  'The sequence $x_n=1/n$ is Cauchy in $\mathbb{R}$.'

aporic link EXAMPLE_ID example-of DEFINITION_ID
```

Single quotes are convenient for LaTeX because most shells then preserve `$`
and backslashes literally.

### 2. State a conjecture without overstating its status

```bash
aporic --project functional-analysis \
  --formal-system "normed spaces" \
  conjecture 'Every Cauchy sequence in a normed space converges.'
aporic link CONJECTURE_ID uses DEFINITION_ID
```

New mathematical entries default to `unverified`. Verification metadata
describes review status, not truth.

### 3. Test the boundary with a counterexample

```bash
aporic --project functional-analysis counterexample \
  '$\mathbb{Q}$ with the Euclidean norm contains Cauchy sequences that do not converge in $\mathbb{Q}$.'
aporic link COUNTEREXAMPLE_ID counterexample-to CONJECTURE_ID
aporic link COUNTEREXAMPLE_ID disproves CONJECTURE_ID
```

Keep the failed conjecture. It documents why the missing completeness
hypothesis became necessary.

### 4. State the corrected theorem and its dependencies

```bash
aporic --project functional-analysis theorem \
  'Every Cauchy sequence in a Banach space converges.'
aporic link THEOREM_ID supersedes CONJECTURE_ID
aporic link THEOREM_ID uses DEFINITION_ID

aporic --project functional-analysis lemma \
  'A Cauchy sequence is bounded.'
aporic link THEOREM_ID depends-on LEMMA_ID
```

### 5. Record the proof separately

```bash
aporic --project functional-analysis proof \
  'Let $(x_n)$ be Cauchy. Completeness of the Banach space gives an $x$ such that $x_n\to x$; hence the sequence converges in the space.'
aporic link PROOF_ID uses LEMMA_ID
aporic link PROOF_ID proves THEOREM_ID
```

Use `proves` only for a proof assertion made by a human or a trusted formal
verification adapter. A recorded proof is not automatically checked. After an
actual review, capture the appropriate verification status when creating the
reviewed entry:

```bash
aporic --project functional-analysis \
  --verification checked \
  proof 'Reviewed proof text or a reference to the checked artifact.'
```

Supported values are `unverified`, `checked`, `peer_reviewed`, and
`formally_verified`. A formal-system label such as `Lean 4` does not itself
mean formally verified.

### 6. Inspect and export the proof structure

```bash
aporic trace THEOREM_ID --depth 4
aporic --project functional-analysis list --math-type theorem
aporic --project functional-analysis list --math-type proof
aporic --project functional-analysis obsidian export \
  "$HOME/Documents/Notes/Functional Analysis.md"
```

Available mathematical relations are:

```text
defines          proves           disproves        uses
depends-on       generalizes      specializes      equivalent-to
example-of       counterexample-to
```

Use `calculate` for symbolic or numerical derivations and `example` for a
clarifying instance. Neither automatically proves a conjecture.

## Querying and automation

```bash
aporic --project checkout list
aporic --project checkout list --type question --state open
aporic --project checkout actions
aporic show ENTRY_ID
aporic trace ENTRY_ID --depth 3
aporic status
```

Use `--json` for scripts and agents:

```bash
aporic --json --project checkout list --type action --state open
aporic --json show ENTRY_ID
aporic --json trace ENTRY_ID
```

Human display output is not a machine interface. JSON is emitted as exactly one value on stdout.

For reliable scripts, keep the project explicit and extract IDs from JSON with
a JSON parser rather than parsing human-readable output:

```bash
aporic --json --project checkout observe "checkout returned HTTP 503"
```

## Developer context

Source context can be attached explicitly during capture:

```bash
aporic --project parser \
  --repo . \
  --commit "$(git rev-parse HEAD)" \
  --file src/parser.rs \
  --line 142 \
  observe "the parser accepts an unterminated string"
```

Aporic records references; it does not execute captured text or scan an entire repository implicitly.

## Obsidian

```bash
aporic --project checkout obsidian export \
  "$HOME/Dokumente/Obsidian Vault/01 Planung/Checkout Investigation.md"
```

Aporic writes only between versioned markers:

```markdown
<!-- aporic:start version=1 -->
...
<!-- aporic:end -->
```

Handwritten content outside the markers is preserved. Existing Nanoshift markers are upgraded on the next export. Unbalanced or duplicate markers cause export to stop rather than guess.

## Existing Nanoshift data

If `~/.local/share/nanoshift/tasks.db` exists and no Aporic database exists, Aporic opens that database in place and applies a transactional schema migration. It does not copy the database and create two competing sources of truth.

Legacy tasks become `action` entries with:

- their description and completion state preserved;
- creation/update/completion timestamps preserved where available;
- their former integer ID stored as `legacy_task_id`;
- origin `migration:nanoshift`;
- an audit event documenting the migration.

Commands such as `aporic show 7` and `aporic complete 7` continue to resolve a migrated task by its old ID. Back up important data before testing any pre-1.0 migration.

## Storage

Fresh installations use:

```text
Linux:   ~/.local/share/aporic/aporic.db
macOS:   ~/Library/Application Support/aporic/aporic.db
Windows: %APPDATA%/aporic/aporic.db
```

Existing Nanoshift installations temporarily continue using their original database path as described above.

## Design and roadmap

See [AGENT.md](AGENT.md) for the product model, developer/IT use cases, architecture, safety requirements, MCP design, testing strategy, and phased roadmap. [AGENTS.md](AGENTS.md) is the conventional agent-discovery entry point.

## License

MIT. See [LICENSE](LICENSE).
