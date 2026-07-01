# Aporic Agent and Product Guide

This document is the operating specification for humans and coding agents working on Aporic. It defines the product direction, architectural boundaries, engineering standards, and delivery sequence. When implementation and this document disagree, either restore the documented invariant or update this document in the same change with an explicit rationale.

## 1. Product thesis

Aporic is a local-first system for moving from uncertainty to justified action.

It helps developers, mathematicians, and independent knowledge workers record what they observed, separate facts from assumptions, expose open questions, trace implications, choose actions, and learn from outcomes. The same body of reasoning must remain usable from:

- a fast command-line interface;
- Git hooks, CI jobs, and ordinary scripts through stable JSON;
- Obsidian and other plain-Markdown tools;
- an optional terminal workbench;
- AI clients through the Model Context Protocol (MCP).

The name comes from *aporia*: an honest state of doubt, difficulty, or unresolved inquiry. Aporic treats uncertainty as useful information rather than something to hide behind confident prose.

Aporic is not merely a task manager, note application, issue tracker, or AI chatbot. Its unit of value is a traceable reasoning chain:

```text
source -> observation -> claim -> question -> implication -> action -> outcome -> learning
```

Not every chain needs every element. Minimal capture must stay fast. Structure is added only when it improves a decision, investigation, or future understanding.

### Product promise

> Know what you know. Mark what you do not. Act on the implications.

Secondary tagline: **From uncertainty to implication.**

### Target users

The initial target user is an indie developer, mathematician, operator, security practitioner, technical researcher, or small engineering team member who:

- lives in a terminal and/or Obsidian;
- regularly debugs systems, evaluates technologies, or makes architecture decisions;
- develops definitions, examples, conjectures, proofs, and counterexamples;
- wants reasoning and actions connected without adopting a heavyweight project suite;
- uses one or more AI assistants;
- wants AI to challenge assumptions and find gaps, not manufacture certainty;
- values automation but does not want an agent silently changing evidence or decisions;
- prefers portable data and composable tools over a hosted productivity platform.

### Differentiation

Aporic should win through a distinct epistemic workflow, interoperability, and trust—not feature count:

1. **Local-first:** SQLite and Markdown, with no mandatory service.
2. **Epistemically explicit:** observations, claims, assumptions, questions, implications, actions, outcomes, and learnings are not collapsed into generic notes.
3. **Evidence-aware:** claims can cite files, commits, logs, URLs, commands, measurements, or other entries.
4. **Implication-driven:** the product asks “what follows?” and “what would change this?” before generating more work.
5. **Developer-native:** Git references, source locations, command output, incidents, ADRs, and MCP are first-class integration points.
6. **Agent-native:** AI receives typed context and produces proposals, not unreviewed prose or direct database writes.
7. **Deterministic core:** every mutation has typed input, validation, revision checks, and an audit record.
8. **Composable:** stable JSON, useful exit codes, stdin/stdout support, and protocol-clean output.
9. **Obsidian-friendly:** safe projections and durable identifiers, without requiring a plugin.
10. **Provider-neutral:** local and hosted models are adapters; no model provider owns the domain.

### Product vocabulary

Use plain language in the UI. The philosophical basis should sharpen the product, not make it academic.

| Concept | User-facing meaning | Typical developer example |
|---|---|---|
| Observation | Something directly noticed or measured | “p95 latency rose from 90 ms to 480 ms” |
| Claim | A statement that may be supported or challenged | “The new index caused slower writes” |
| Assumption | A claim currently relied upon without enough evidence | “The cache is not involved” |
| Question | A meaningful unknown | “Does the regression occur on an empty database?” |
| Implication | What follows if a claim or assumption holds | “We should benchmark before deploying the migration” |
| Action | A concrete next step | “Run the benchmark against both schemas” |
| Outcome | What happened after an action | “The index improved reads but doubled write latency” |
| Learning | A durable update to understanding | “This workload is write-bound; index changes require write benchmarks” |

Mathematical entries use an additional `math_kind` rather than replacing epistemic kind. A theorem is still a claim; a conjecture is an unresolved claim; a proof is evidence supporting a mathematical claim. This keeps reasoning semantics consistent across software and mathematics.

| Mathematical kind | Epistemic role | Example |
|---|---|---|
| Definition | Claim that fixes terminology or an object | “A sequence is Cauchy if …” |
| Conjecture | Claim not yet proved or disproved | “Every object in this class has property P” |
| Lemma | Supporting mathematical claim | “The operator is bounded on the dense subspace” |
| Theorem | Main mathematical claim | “The sequence converges in the complete space” |
| Proof | Structured evidence for a claim | Induction, contradiction, construction |
| Counterexample | Evidence disproving or restricting a claim | A discontinuous additive function |
| Example | Concrete instance clarifying a definition | A standard basis calculation |
| Calculation | Symbolic or numerical derivation | Expansion of a determinant |

Internally these are typed entries and relations. Externally they should feel like a concise engineering notebook with executable follow-through.

### Current repository state

The one-time Nanoshift-to-Aporic data migration and the transitional `nsh` compatibility surface are complete and removed from the codebase; there is no legacy code path left to maintain. Schema versioning, typed entries and relations, trace queries, JSON output, versioned Obsidian export, an interactive terminal view (`aporic tui`), and a sandboxed interactive tutor (`aporic tutor`) are implemented. MCP, AI providers, source-management commands, repository automation, and bidirectional Obsidian sync remain roadmap work. Do not document planned capabilities as complete.

Transition requirements:

- preserve existing user data through a tested migration;
- provide a temporary `nsh` compatibility command or clear migration tooling;
- map legacy tasks to `action` entries with explicit migration provenance;
- rename package, binary, paths, documentation, and generated markers in one planned release sequence;
- never silently create a second database that makes existing work appear lost.

## 2. Product principles

### Local operation is the baseline

All core commands must work offline. Network access must be explicit. AI commands must state which provider they use and whether entry or source content leaves the machine.

### AI proposes; the domain layer decides

Models may classify an entry, identify unsupported claims, suggest questions, derive candidate implications, propose experiments, summarize evidence, or suggest edits. They must not decide what is true or write directly to SQLite or Markdown. Model output is parsed into typed proposals, validated, previewed, and then sent through the same application commands used by the CLI and MCP server.

### Preserve epistemic status

Never silently turn an assumption into a fact, a model inference into an observation, or a suggestion into a decision. Every entry has an author and origin. Derived relationships retain provenance. Confidence may complement evidence but never replace it.

### Truth is approached through traceability

Aporic does not claim to store absolute truth. It stores inspectable claims, evidence, counterevidence, uncertainty, decisions, and outcomes. “Why do we believe this?” and “what would disprove it?” must remain answerable.

### Implications connect knowledge to work

A note system can accumulate knowledge without changing behavior; a task system can accumulate actions without explaining why. Aporic connects them. Actions should be traceable to the questions or implications that motivated them, and outcomes should be able to revise prior beliefs.

### Minimal capture, progressive structure

Creating an entry should take one command and require only text. Links, sources, confidence, projects, and relations are optional at capture time. The system may later propose structure, but it must not force ceremony onto fleeting observations.

### One source of truth

SQLite is the authoritative operational store. Markdown is a projection and integration surface, not a second database. Import and synchronization must use stable IDs, revisions, and explicit conflict rules.

### Plain data before embeddings

Use relational queries and SQLite FTS5 before adding semantic retrieval. Graph traversal is implemented through ordinary relation tables first. Embeddings are optional derived data that can always be rebuilt. A vector index must never be required to read or modify entries.

### Safe automation before autonomous automation

Every tool declares its effect:

- read-only;
- single-item mutation;
- bulk mutation;
- external side effect.

Bulk mutation and external side effects require confirmation by default. `--yes` is allowed only for explicit non-interactive use.

### Compatibility is a feature

Avoid unnecessary format churn. Database migrations are forward-only and tested. JSON fields are additive within a major version. Generated Markdown includes a format version.

## 3. Core user journeys

### Capture with almost no ceremony

```bash
aporic observe "Checkout p95 increased after commit abc123"
aporic ask "Is connection-pool contention responsible?"
aporic assume "The database itself is healthy" --because OBSERVATION_ID
aporic imply "If contention is responsible, increasing pool size should change the profile"
aporic act "Run a load test with pool sizes 10, 20, and 40" --project checkout
```

Fast symbolic aliases may exist for interactive use:

```bash
aporic . "Observed behavior"
aporic ? "Open question"
aporic ~ "Working assumption"
aporic ! "Implication"
aporic '>' "Next action"
aporic = "Measured outcome"
```

Descriptive commands are canonical in scripts and documentation. Capture must remain instant and must not invoke a model implicitly.

### Build a reasoning chain

```bash
aporic link CLAIM_ID --supported-by EVIDENCE_ID
aporic link CLAIM_ID --challenged-by COUNTEREVIDENCE_ID
aporic link IMPLICATION_ID --follows-from CLAIM_ID
aporic link ACTION_ID --tests ASSUMPTION_ID
aporic link OUTCOME_ID --result-of ACTION_ID
aporic trace CLAIM_ID
```

`trace` presents the shortest useful chain plus contradictions and unresolved questions. It must not dump an unreadable graph by default.

### Inspect and automate

```bash
aporic list --project checkout --type question --state open
aporic show ENTRY_ID --json
aporic trace ENTRY_ID --json
aporic actions --ready --json
```

Human-readable output is the terminal default. `--json` is the stable machine interface. Do not encourage parsing display output.

### Ask AI to challenge, not merely generate

```bash
aporic ai examine CLAIM_ID
aporic ai find-gaps --project checkout
aporic ai implications CLAIM_ID
aporic ai propose-test ASSUMPTION_ID
aporic ai synthesize --project checkout
```

These commands create proposals such as missing questions, alternative explanations, relevant evidence, or discriminating experiments. Output must distinguish retrieved evidence from model inference. The default flow is preview, explain, accept, edit, or reject.

### Work through an AI agent

```bash
aporic mcp serve --stdio
```

An MCP client can inspect entries and reasoning chains, add explicitly attributed observations, or create proposals. The server exposes domain operations; it does not expose SQL, arbitrary filesystem access, or shell execution.

### Use Obsidian

```bash
aporic obsidian export "$VAULT/01 Planung/Aporic.md"
aporic obsidian export --project checkout "$VAULT/Projects/Checkout Investigation.md"
aporic obsidian sync "$VAULT/Projects/Checkout Investigation.md" --dry-run
```

Export modifies only Aporic-owned sections. A note can include human-written narrative around generated evidence, questions, implications, and actions. Sync previews conflicts and never treats text as identity.

### Reference MVP command surface

The first coherent release should be small enough to understand from one help screen:

```text
aporic observe TEXT       record something measured or directly noticed
aporic claim TEXT         record a statement to support or challenge
aporic assume TEXT        record a working assumption
aporic ask TEXT           record an unresolved question
aporic imply TEXT         record what follows from existing entries
aporic act TEXT           record a concrete next action
aporic outcome TEXT       record what happened
aporic learn TEXT         record a durable update to understanding
aporic define TEXT        record a mathematical definition
aporic conjecture TEXT    record an open mathematical claim
aporic lemma TEXT         record a supporting mathematical claim
aporic theorem TEXT       record a main mathematical claim
aporic proof TEXT         record proof evidence
aporic counterexample TEXT record disproof or a boundary case
aporic example TEXT       record a clarifying mathematical instance
aporic calculate TEXT     record a symbolic or numerical derivation
aporic link A RELATION B  connect two entries
aporic show ID            inspect one entry and provenance
aporic trace ID           inspect the relevant reasoning chain
aporic list               filter entries
aporic source             manage evidence references
aporic project            manage projects
aporic export             produce Markdown or JSON projections
```

AI, MCP, TUI, bidirectional sync, embeddings, and automated imports are not required for the first coherent release. The non-AI reasoning workflow must prove useful first.

## 4. Developer and IT use cases

Aporic must solve recognizable technical work. Abstract epistemic features are justified only when they improve one or more workflows below.

### Debugging a difficult defect

The developer captures symptoms as observations, competing explanations as assumptions or claims, and experiments as actions. Command output, source locations, commits, and test results become evidence. A trace shows which hypotheses remain possible and prevents repeating failed experiments.

Example chain:

```text
Observation: memory grows only under cancelled requests
  -> Assumption A: response bodies are not released
  -> Assumption B: tracing spans retain request state
  -> Action: compare heap profiles with tracing disabled
  -> Outcome: growth disappears
  -> Implication: inspect span ownership before changing HTTP code
```

The value is not storing another bug ticket. It is preserving why a conclusion became credible.

### Incident response and post-incident learning

During an incident, responders record timestamped observations and actions without prematurely rewriting history. Claims about the cause remain separate from facts. After mitigation, the same chain supports the postmortem:

- what was observed and when;
- which assumptions drove each action;
- which actions changed system state;
- what evidence supports the root-cause claim;
- which questions remain unresolved;
- which learnings should become safeguards or follow-up actions.

Aporic complements, rather than replaces, alerting, logs, and an incident chat room. It provides the reasoning trail those tools usually lose.

### Architecture decisions and ADR preparation

Before an Architecture Decision Record is finalized, Aporic can track requirements, constraints, alternatives, evidence, assumptions, and implications. The accepted chain can then render into an ADR draft containing:

- context supported by sources;
- alternatives considered;
- decision and explicit rationale;
- positive and negative consequences;
- assumptions that should trigger reconsideration if invalidated.

The ADR remains the durable published document. Aporic preserves the investigation that produced it.

### Technology evaluation

An indie developer evaluating a database, framework, model, or deployment platform can connect documentation, benchmarks, experiments, and uncertainties instead of collecting disconnected bookmarks.

```bash
aporic source add https://sqlite.org/fts5.html
aporic claim "FTS5 is sufficient for the first search release" --project search
aporic ask "How does indexing affect write latency at our expected scale?"
aporic act "Benchmark 10k, 100k, and 1m entries"
```

A comparison is useful only when claims cite versioned evidence and assumptions describe the expected workload.

### Security investigation and threat modeling

Assets, trust boundaries, threats, mitigations, and residual questions can be represented without pretending every threat is confirmed. Evidence and counterevidence attach to claims; mitigations are actions derived from implications.

Security rules:

- Aporic is not a secret store.
- Imported scanner output is untrusted evidence, not truth.
- Exploit steps and sensitive findings require explicit export controls.
- AI may suggest threats but must label them as model-generated hypotheses.

### Code review and change-risk analysis

A reviewer or agent can record claims about a change, supporting source references, unanswered questions, and implications for reliability or compatibility. This is particularly useful for large or AI-generated changes where “tests pass” is insufficient evidence.

Potential integrations:

- attach an entry to `repo`, commit, branch, pull request, file, and line;
- import failing test output as an observation;
- render unresolved high-impact questions as a pull-request summary;
- close an action when a linked commit lands, but only through an explicit rule;
- preserve review findings after a hosted pull request disappears.

### AI-agent supervision

An agent receives a bounded reasoning chain instead of an entire vault. It can create attributed proposals, observations from tool results, and candidate implications. A human can ask:

- Which claims came from the agent?
- What evidence did it actually inspect?
- Which assumptions did it make?
- Which proposed action has side effects?
- What changed after the action?

This makes Aporic a control and accountability layer, not another autonomous-agent framework.

### Technical research and learning

Aporic supports learning by turning passive consumption into questions, experiments, and durable implications. A source note can produce claims; claims can be challenged by another source; unresolved questions can become small coding experiments. A learning is retained when it changes how future work should be done.

Example:

```text
Source: Rust async cancellation article
  -> Claim: dropping a future can interrupt work at an await point
  -> Question: which operations in this service are not cancellation-safe?
  -> Action: audit transaction and channel boundaries
  -> Learning: wrap the state transition in one cancellation-safe operation
```

### Decision journal for an indie product

An indie hacker can connect user feedback, analytics, beliefs, product decisions, experiments, and outcomes. This prevents both data theater and hindsight bias:

- feedback is an observation from a particular source, not universal demand;
- a product belief is labeled as an assumption;
- the expected implication is written before the feature ships;
- the outcome is compared with the expected result;
- the learning influences the next decision.

### Lightweight runbooks and operational knowledge

Repeated successful action chains may be promoted into a runbook. Promotion requires human review because a historical action is not automatically a safe general procedure. Each runbook records applicability conditions, verification steps, rollback, and the evidence from which it was learned.

### Personal knowledge and Obsidian

Outside software delivery, the same model supports literature notes, mathematical investigation, and personal decisions. These remain secondary use cases. Developer workflows determine the first releases and prevent the product from becoming an abstract personal-knowledge graph.

### Mathematical research and problem solving

Mathematics is a first-class use case, not a generic note category. Aporic should support the actual lifecycle of mathematical work:

```text
definition -> example -> conjecture -> lemma -> theorem
                         |              |
                         v              v
                   counterexample     proof
                         \              /
                          -> revised claim
```

Typical workflows:

- develop a definition and attach examples and non-examples;
- formulate a conjecture and record evidence without confusing evidence with proof;
- split a proof obligation into lemmas and open questions;
- track which definitions, lemmas, and external results a proof uses;
- attach a counterexample that disproves a conjecture or reveals missing hypotheses;
- preserve failed proof approaches and the exact obstruction encountered;
- record calculations in LaTeX while keeping the conclusion separate;
- compare equivalent formulations of a theorem;
- specialize or generalize a result with explicit relations;
- export a coherent theorem/proof dependency view to Obsidian.

Example:

```bash
aporic --project functional-analysis define \
  'A sequence $(x_n)$ is Cauchy if $\\forall \\varepsilon>0\\;\\exists N\\;\\forall m,n\\ge N: \\|x_m-x_n\\|<\\varepsilon$.'
aporic --project functional-analysis conjecture \
  'Every Cauchy sequence in $X$ converges.' --formal-system "normed spaces"
aporic --project functional-analysis counterexample \
  '$X=\\mathbb{Q}$ with the Euclidean norm is not complete.'
aporic link COUNTEREXAMPLE_ID counterexample-to CONJECTURE_ID
aporic --project functional-analysis theorem \
  'Every Cauchy sequence in a Banach space converges.'
```

The system must distinguish:

- empirical or computed evidence from deductive proof;
- a proof sketch from a completed proof;
- syntactic derivation from semantic interpretation;
- theorem status from peer review or formal verification status;
- an example from a counterexample;
- a failed proof attempt from a disproved claim.

A failed proof attempt is valuable. It should remain attributable and link to the obstruction or unresolved question rather than being deleted.

### Mathematics and software together

Aporic is particularly useful where mathematics meets implementation:

- verify that code implements the stated mathematical definition;
- connect numerical experiments to conjectures without treating them as proofs;
- trace a theorem to the algorithm or optimization that relies on it;
- record floating-point assumptions and resulting error bounds;
- link a proof obligation to tests, formal specifications, or theorem-prover artifacts;
- document why an approximation is valid in the deployed parameter regime;
- connect benchmark outcomes with complexity claims;
- preserve derivations behind cryptographic, statistical, or machine-learning code.

Repository references may point to Lean, Coq, Isabelle, Agda, TLA+, SMT-LIB, Jupyter, Julia, SageMath, Mathematica, or ordinary source files. Aporic stores provenance and relationships; it does not attempt to replace these systems.

### Product validation scenarios

Before calling the core useful, dogfood these scenarios in real repositories:

1. Diagnose a defect with at least two competing assumptions and show why one was rejected.
2. Evaluate a dependency using documentation, a local benchmark, an open question, and an explicit implication.
3. Produce an ADR draft whose rationale and consequences are traceable to accepted entries.
4. Reconstruct an incident timeline without turning later conclusions into earlier observations.
5. Give an AI client only the relevant trace and verify that all its suggestions remain attributed proposals.
6. Export the result to Obsidian twice without modifying handwritten text.
7. Develop a conjecture into a corrected theorem using a counterexample and at least one lemma.

Useful product signals:

- capture feels as immediate as appending a line to a text file;
- a trace answers “why are we doing this?” without manual archaeology;
- prior failed experiments prevent repeated work;
- a user can recover an important unresolved question in seconds;
- AI finds a concrete missing premise or useful discriminating test;
- the workflow remains valuable with AI disabled.

GitHub stars, number of entry types, and generated-token volume are not product-quality metrics.

## 5. Domain model

Use opaque, durable public IDs such as UUIDv7 or ULID. Internal SQLite row IDs may remain integers but must never be the long-term interoperability identifier.

### Entry

Required fields:

- `id`: durable public ID;
- `kind`: `observation`, `claim`, `assumption`, `question`, `implication`, `action`, `outcome`, or `learning`;
- `body`: concise Markdown text;
- `state`: a kind-specific validated lifecycle state;
- `project_id`: nullable reference;
- `author`: human or machine actor that created the entry;
- `origin`: `human`, `import`, `tool`, or `model` plus optional adapter details;
- `created_at`, `updated_at`: UTC RFC 3339 timestamps;
- `revision`: monotonically increasing integer for optimistic concurrency.

Optional fields:

- `details`: longer Markdown context;
- `occurred_at`: when an observation or outcome happened, distinct from capture time;
- `due_at` and `scheduled_at`;
- `completed_at`;
- `impact`: `low`, `medium`, `high`, or `critical` where meaningful;
- `confidence`: explicitly subjective unless produced by a calibrated mechanism;
- `math_kind`: optional `definition`, `conjecture`, `lemma`, `theorem`, `proof`, `counterexample`, `example`, or `calculation`;
- `formal_system`: optional mathematical setting or proof assistant;
- `verification`: optional `unverified`, `checked`, `peer_reviewed`, or `formally_verified`;
- `source_uri`: origin such as an Obsidian note;
- `source_anchor`: source-specific stable marker;
- `repository`, `commit`, `file`, and `line`: developer context;
- `estimate_minutes`;
- tags through a normalized join table.

State semantics vary by kind and remain explicit:

- observation/outcome: `recorded`, `corrected`, or `retracted`;
- claim/implication/learning: `active`, `superseded`, or `retracted`;
- assumption: `active`, `supported`, `challenged`, or `retired`;
- question: `open`, `answered`, or `deferred`;
- action: `open`, `in_progress`, `blocked`, `done`, or `cancelled`.

State changes never rewrite history. An answered question links to the answering entry; a completed action may link to an outcome; a superseded claim links to its replacement.

### Relation

Relations are directed, typed, and individually addressable. Initial relation kinds:

- `supports` and `challenges`;
- `derived_from` and `follows_from`;
- `answers`;
- `tests`;
- `motivates`;
- `result_of`;
- `contradicts`;
- `supersedes`;
- `relates_to` as a last resort.

Mathematical relation kinds:

- `defines`;
- `proves` and `disproves`;
- `uses`;
- `depends_on`;
- `generalizes` and `specializes`;
- `equivalent_to`;
- `example_of`;
- `counterexample_to`.

`proves` is asserted by a human or trusted formal-verification adapter; a language model may only propose it. Numerical agreement supports a conjecture but does not prove it.

A relation records author, origin, timestamp, and optional rationale. Model-suggested relations are proposals until accepted. Do not infer transitive certainty: if A supports B and B supports C, A is not automatically direct evidence for C.

### Source and evidence reference

A source reference points to material outside or inside Aporic:

- URL plus access time and optional content hash;
- repository and immutable commit;
- file and line range at a commit;
- command invocation, exit code, and captured artifact;
- log query and bounded time range;
- metric name, query, and time range;
- Obsidian note and block anchor;
- another Aporic entry.

Large logs, binaries, source trees, and private documents remain outside the database. Store references, hashes, and small excerpts subject to size and privacy limits. A URL without retrieval metadata is a lead, not durable evidence.

The provenance model should borrow practical concepts from W3C PROV—entities, activities, agents, derivation, and attribution—without requiring RDF, an ontology engine, or W3C terminology in the everyday CLI. Export to a standard provenance representation may be considered later if a real interoperability case emerges.

### Action-specific data

An action may additionally record:

- acceptance criteria;
- expected outcome written before execution;
- rollback or safety notes;
- side-effect class;
- assignee or agent actor;
- execution state;
- linked issue, commit, or pull request.

This preserves ordinary task-management usefulness without making every entry a task.

### Outcome and learning

An outcome reports what happened; it does not automatically validate the hypothesis that motivated the action. A learning is a durable interpretation supported by one or more outcomes or sources. The UI should encourage linking both successful and failed outcomes so survivorship bias is not baked into the knowledge base.

### Project

A project has a durable ID, name, optional description, status, timestamps, and revision. Project names are labels, not identity.

Projects group work but do not isolate the graph. Cross-project links are allowed and visible. A `context` may represent a repository, service, incident, decision, or learning topic without requiring a new project.

### Audit event

Every mutation creates an append-only audit event containing:

- event ID and timestamp;
- actor: `human`, `cli`, `mcp:<client>`, or `ai:<provider>/<model>`;
- operation name;
- entity or relation ID;
- before/after patch or structured payload;
- correlation ID for a multi-operation proposal.

The audit log enables undo, debugging, and trustworthy agent behavior. Sensitive model prompts and secrets do not belong in it.

### Proposal

AI-generated work is represented separately from committed entry state:

- proposal ID;
- provider and model identifier;
- source entry and relation revisions;
- typed operations;
- human-readable rationale;
- confidence is optional and must not be presented as calibrated unless it is measured;
- status: `pending`, `accepted`, `rejected`, or `stale`.

A proposal becomes stale when an input entry or relation revision changes.

### Invariants

- Public identity never depends on mutable text.
- Deleting an entry with inbound relations is rejected or converted to an auditable tombstone.
- Actor and origin attribution cannot be erased by ordinary edits.
- Model-generated content cannot be stored as a human observation.
- Evidence references do not imply endorsement.
- An action can complete without proving the claim it tested.
- Relations cannot point to nonexistent entities.
- Every bulk change is transactional and shares a correlation ID.

## 6. Architecture

Evolve toward a Cargo workspace only when the boundaries below become real. Do not split crates merely to appear modular.

```text
CLI / TUI / MCP / Obsidian adapter
              |
        application layer
       commands and queries
              |
          domain model
              |
 repositories / audit / events
              |
            SQLite

AI providers -> typed proposals -> validation -> application commands
```

Recommended eventual layout:

```text
crates/
  aporic-core/       domain types, commands, validation, errors
  aporic-store/      SQLite repositories and migrations
  aporic-ai/         provider traits, structured proposals, redaction
  aporic-mcp/        MCP tools/resources mapped to application commands
  aporic-obsidian/   Markdown projection, parsing, conflict detection
  aporic-cli/        CLI and optional TUI entry points
```

Until the code justifies a workspace, use equivalent modules under `src/`:

```text
src/
  cli.rs
  domain/
  application/
  store/
  integrations/obsidian/
  integrations/mcp/
  ai/
```

### Dependency direction

- Domain code has no dependency on Clap, SQLite, HTTP, MCP, or terminal UI crates.
- Application code depends on domain interfaces, not concrete storage.
- Adapters translate their protocol into application commands.
- Storage and integrations may depend on external crates.
- No adapter calls another adapter directly.

### Command/query separation

Mutations are explicit application commands such as `CreateEntry`, `LinkEntries`, `ResolveQuestion`, `CompleteAction`, and `ApplyProposal`. Reads are queries. This is not full event sourcing; SQLite tables remain current state, with audit events recording changes.

### Async policy

Keep local database and simple CLI paths synchronous unless concurrency provides measurable value. Use Tokio at network/MCP boundaries. Do not spread async through the domain solely because an adapter needs it.

## 7. Technology choices

Dependencies are selected for a concrete capability. Pin compatible released versions and review release notes before upgrades.

### Keep

- **Rust:** a strong fit for a portable, reliable single binary.
- **SQLite:** authoritative local storage with transactional migrations.
- **Clap:** typed CLI parsing and generated help.
- **Serde:** shared JSON and protocol types.
- **anyhow at binary boundaries:** convenient context for top-level failures.

Use a typed error enum such as `thiserror` inside libraries so callers can distinguish validation, conflict, storage, and provider failures.

### Add incrementally

- **`tracing` + `tracing-subscriber`:** structured diagnostics written to stderr; JSON logs only when requested.
- **`serde_json` + `schemars`:** stable machine output and schemas for AI/MCP tools.
- **`uuid` with UUIDv7:** durable sortable public identity.
- **`rmcp`:** the official Rust MCP SDK for agent interoperability. Follow released APIs rather than a Git branch.
- **`reqwest` with rustls:** provider-neutral hosted-model adapters when required.
- **`ratatui` + `crossterm`:** optional TUI after command and JSON APIs stabilize.
- **`insta`:** snapshots for CLI help, JSON, and Markdown projections.
- **`proptest`:** parser, marker, and synchronization invariants.

Consider `sqlx` only if compile-time query checks, async access, or its migration tooling clearly outweigh the migration cost from `rusqlite`. Replacing a working database layer is not itself modernization.

### Search and retrieval

1. Start with indexed relational filters.
2. Add SQLite FTS5 for entry bodies and details.
3. Add embeddings only behind an experimental feature and a rebuildable index.
4. Evaluate SQLite's official `vec1` extension when its packaging, platform support, and stability meet release requirements.

Embedding metadata must include model, dimensions, normalization, and source revision. Never compare vectors produced by incompatible models.

### Model providers

Define a small internal provider trait around the product's needs, not a universal LLM abstraction:

```rust
trait ReasoningProvider {
    fn generate_proposal(&self, request: ProposalRequest) -> Result<Proposal, ProviderError>;
}
```

Initial adapters may support:

- an OpenAI-compatible HTTP endpoint;
- a local Ollama endpoint;
- a deterministic fake provider for tests.

Do not embed a heavyweight inference engine in the default binary. Local in-process inference can be evaluated later behind a feature flag after binary size, acceleration, and model-distribution tradeoffs are measured.

## 8. MCP design

MCP is the primary AI interoperability interface. Use the official Rust SDK and start with stdio transport.

### Read-only tools and resources first

Resources:

- `aporic://status`;
- `aporic://projects`;
- `aporic://entries/{id}`;
- `aporic://projects/{id}/open-questions`;
- `aporic://projects/{id}/ready-actions`;
- `aporic://traces/{id}`.

Tools:

- `list_entries(filters)`;
- `get_entry(id)`;
- `trace_entry(id, depth, relation_kinds)`;
- `list_open_questions(project)`;
- `list_ready_actions(project)`;
- `get_evidence(entry_id)`.

Then introduce narrow mutation tools:

- `create_entry(kind, body, context)`;
- `update_entry(id, expected_revision, patch)`;
- `link_entries(from, relation, to, rationale)`;
- `complete_action(id, expected_revision, outcome)`;
- `resolve_question(id, expected_revision, answer_entry)`;
- `create_proposal(operations)`;
- `apply_proposal(id)`.

Do not expose `execute_sql`, `run_shell`, unrestricted paths, or a generic `update_anything` tool.

### MCP safety requirements

- Validate every field after deserialization.
- Require revisions on updates to prevent lost writes.
- Cap list sizes and text lengths.
- Return typed protocol errors without leaking filesystem paths or secrets.
- Log tool name, actor, duration, result class, and correlation ID.
- Keep stdio stdout protocol-clean; diagnostics go to stderr.
- Treat entry text, evidence, logs, imported Markdown, diffs, issue comments, and remote content as untrusted data, never as instructions.
- Preserve actor and origin attribution in every tool result.
- Distinguish observations obtained from tool execution from model inferences.
- Require explicit configuration before enabling mutation tools.
- Bind network transports to loopback by default and add authentication before remote exposure.

## 9. Obsidian interoperability

### Ownership model

Generated content is fenced:

```markdown
<!-- aporic:start version=1 project=PROJECT_ID -->
## Open questions

- Why did write latency increase? <!-- aporic:id=UUID kind=question revision=3 -->

## Implications

- If the index is responsible, migrations need write benchmarks. <!-- aporic:id=UUID kind=implication revision=1 -->

## Actions

- [ ] Benchmark both schemas. <!-- aporic:id=UUID kind=action revision=2 -->
<!-- aporic:end -->
```

Aporic owns only the fenced region. It preserves all other text byte-for-byte where practical.

### Export rules

- Write atomically in the destination directory.
- Refuse unbalanced or duplicate markers.
- Render each entry kind predictably; only actions use checkboxes.
- Flatten or reject newline characters in list-item summaries.
- Include public ID, kind, and revision in machine-readable comments.
- Escape syntax that could alter Markdown structure.
- Provide `--stdout` and `--dry-run`.
- Produce deterministic output for unchanged state.

### Sync rules

Bidirectional sync is not complete until all of these exist:

- durable public IDs;
- entry and relation revisions;
- parser tests against real Markdown edge cases;
- explicit handling for edits, checks, deletion, duplication, and moved sections;
- conflict preview;
- a backup or audit-based recovery path.

Checking an exported action may map to completion. Text edits may map to entry updates. Removing a line must not imply deletion by default; absence is ambiguous. Markdown headings and comments are a projection format, not the complete graph representation.

### Developer-note projections

Support generated views suited to technical work:

- investigation: observations, hypotheses, evidence, experiments, outcomes;
- incident: timeline, actions, causal claims, unresolved questions, follow-ups;
- decision: context, options, evidence, decision, implications;
- learning: sources, claims, questions, experiments, learnings;
- mathematics: definitions, conjectures, lemmas, theorems, proofs, counterexamples, examples, calculations, and dependency relations;
- project: current implications, ready actions, blockers, recent outcomes.

These views read from the same domain model. Do not create separate database schemas for each template.

### Obsidian plugin policy

Keep the Markdown integration plugin-free. A native Obsidian plugin may later improve live updates and commands, but it must use the same stable CLI/JSON or local protocol and must not become required for data access.

## 10. CLI contract

### Global behavior

- `--json` emits one documented JSON value and no prose.
- Diagnostics and progress go to stderr.
- `--quiet` suppresses success messages, not errors.
- `--no-color` and `NO_COLOR` are honored.
- Non-interactive commands never prompt unless explicitly requested.
- Destructive bulk commands support `--dry-run` and require confirmation or `--yes`.
- Commands return documented non-zero exit codes for usage, not-found, conflict, provider, and storage failures.

### Project and repository context

Do not rely exclusively on a database-global active scope. Support explicit selection and safe repository detection:

```bash
aporic --project checkout list
aporic --repo . observe "Test fails on the current branch"
```

A configured default is allowed, but two terminals and two agents must operate independently. Repository detection may attach context; it must not silently choose a project or read the whole repository.

### Input and references

- Accept text as a positional argument or through `--stdin`.
- Accept source references through explicit flags such as `--url`, `--repo`, `--commit`, `--file`, and `--line`.
- Never execute a captured command merely because text resembles shell syntax.
- Provide `aporic import` adapters for bounded formats rather than one permissive parser.
- Make shell completion useful for entry IDs, project names, kinds, and relation names.

### Compatibility

Snapshot CLI help. Treat command names, option names, exit behavior, and JSON schemas as public interfaces. Deprecate before removal.

## 11. AI behavior and trust

### Appropriate AI features

- classify raw captures while preserving their original form;
- identify claims lacking evidence or relying on circular support;
- distinguish candidate observations, inferences, and assumptions in imported text;
- propose alternative hypotheses and counterevidence to seek;
- derive candidate implications with the premises shown;
- propose an experiment that discriminates between competing explanations;
- summarize an investigation without flattening disagreement;
- find potentially related prior incidents, decisions, or learnings;
- draft an ADR or postmortem from accepted entries and relations;
- suggest ready actions that reduce important uncertainty.

### Features to avoid

- automatic priority changes without review;
- declaring a claim true because a model assigned high confidence;
- inventing evidence, citations, command results, or source-code inspection;
- rewriting an observation to fit a later conclusion;
- fabricated deadlines or project context;
- background agents with unrestricted mutation rights;
- motivational text presented as productivity;
- opaque scores with no explanation;
- uploading the whole database when a small context window is sufficient.

### Prompt construction

- Separate system policy, trusted application data, and untrusted user/imported text.
- Minimize supplied context.
- Ask for schema-constrained output.
- Validate output independently of the model.
- Record provider/model and hashes or IDs needed for diagnosis, not secrets or unnecessary private prompt content.
- Require model responses to identify premises for every proposed implication.
- Require citations to supplied source IDs; reject invented identifiers.
- Test prompt-injection fixtures from entries, logs, diffs, issue comments, websites, and imported notes.

### Useful AI response shape

AI adapters should return typed candidates rather than an essay:

```json
{
  "proposals": [
    {
      "operation": "create_entry",
      "kind": "question",
      "body": "Does the regression reproduce with tracing disabled?",
      "derived_from": ["ENTRY_ID"],
      "rationale": "This distinguishes two currently viable explanations."
    }
  ],
  "limitations": ["No production heap profile was supplied."]
}
```

The application validates identifiers, kinds, relations, lengths, permissions, and source revisions before previewing the proposal.

### Privacy modes

Expose clear modes:

- `off`: no AI commands;
- `local`: only configured local providers;
- `remote`: explicitly configured hosted provider;
- `ask`: confirm before sending content remotely.

The active mode and provider must be visible through `aporic status` and MCP metadata.

## 12. Security baseline

- Never accept SQL, shell commands, or filesystem paths from model output without an independent allowlist and validation layer.
- Keep secrets in environment variables or OS credential storage, never SQLite entries, committed config, logs, or audit payloads.
- Restrict Obsidian operations to a configured vault root after canonicalizing paths.
- Reject path traversal and symlink escapes where an operation writes files.
- Set maximum lengths for entry bodies, details, model output, excerpts, and imported files.
- Use transactions for multi-step mutations.
- Run dependency auditing and deny known vulnerable or unmaintained dependencies in CI.
- Document the trust boundary for every new integration.

## 13. Testing strategy

Every behavior change requires proportionate tests.

### Unit tests

- domain validation and state transitions;
- entry-kind and relation invariants;
- attribution and origin preservation;
- scope/project isolation;
- revision conflicts;
- Markdown escaping and marker replacement;
- date parsing boundaries;
- AI proposal validation.

### Integration tests

- migrations from every supported schema version;
- CLI stdout, stderr, and exit codes;
- JSON schema snapshots;
- Obsidian round trips with fixtures;
- MCP tool schemas and calls over stdio;
- provider adapters using a local mock HTTP server.

### Property tests

- arbitrary entry text never escapes its generated Markdown structure;
- LaTeX delimiters and backslashes survive database and Markdown round trips;
- proof/counterexample relations preserve direction and identity;
- numerical outcomes never automatically change a conjecture into a theorem;
- export followed by parse retains identity, kind, and revision;
- generated relation graphs never contain dangling references;
- an action outcome cannot silently change the tested claim;
- generated-section replacement preserves surrounding content;
- malformed markers never cause partial writes.

### End-to-end tests

Use isolated temporary data and config directories. Never read or mutate the developer's real database, repository, or vault. Include one smoke test for a clean install, reasoning-chain lifecycle, Obsidian export, and MCP read.

### Required local checks

```bash
cargo fmt -- --check
cargo test
cargo clippy --all-targets --all-features -- -D warnings
```

Add `cargo nextest` and security/license checks in CI when configured, but keep `cargo test` supported for contributors.

## 14. Observability and diagnostics

- Normal CLI output is a product interface, not a log stream.
- Structured tracing goes to stderr and is opt-in through verbosity or environment configuration.
- Generate a correlation ID for AI proposals and MCP calls.
- Measure provider latency, token usage when available, parse failures, proposal acceptance, and conflicts without collecting entry content.
- `aporic doctor` should inspect database access, migrations, config, vault paths, provider connectivity, and MCP setup while redacting secrets.
- Crash reports or telemetry are opt-in only.

## 15. Configuration

Use a versioned TOML configuration in the platform-standard config directory. Environment variables override secrets and automation-specific values. CLI flags override configuration for the current invocation.

Illustrative configuration:

```toml
version = 1
default_project = "global"

[obsidian]
vault = "~/Dokumente/Obsidian Vault"
default_note = "01 Planung/Aporic.md"

[repository]
detect = true
capture_commit = true

[ai]
mode = "ask"
provider = "ollama"
model = "configured-by-user"

[mcp]
mutations = false
```

Do not hard-code a model name as “best.” Models and endpoints change faster than the domain. Validate configuration and provide actionable errors.

## 16. Release engineering

- Maintain a changelog once external users exist.
- Use semantic versioning; pre-1.0 changes may move quickly but must be documented.
- Produce reproducible binaries for Linux, macOS, and Windows.
- Consider `cargo-dist` for release artifacts and installers after the CLI contract stabilizes.
- Generate shell completions and man pages from Clap.
- Publish checksums and a software bill of materials for releases.
- Keep the default binary lean; optional capabilities belong behind Cargo features only when that improves distribution materially.

## 17. Roadmap

### Phase 0 — rename and preserve trust

- Introduce migrations with an explicit schema version.
- Add durable public IDs and revisions.
- Rename the product, binary, package, paths, and documentation through a compatibility plan.
- Migrate legacy tasks into action entries without losing IDs, completion state, or timestamps.
- Replace the global-only scope workflow with explicit `--project` selection.
- Add stable `--json` output and documented exit codes.
- Build integration tests around isolated databases.

Exit criterion: an existing Nanoshift database upgrades safely, and scripts can create, read, update, and complete actions without parsing display text or racing silent updates.

### Phase 1 — epistemic core

- Add typed entries and directed relations.
- Implement observations, assumptions, questions, implications, actions, outcomes, and learnings.
- Preserve author, origin, evidence references, revisions, and audit events.
- Add `trace`, unresolved-question, unsupported-claim, and ready-action queries.
- Add repository, commit, file, line, URL, and command-result source references.
- Ship a complete debugging workflow without AI.

Exit criterion: a developer can capture a defect investigation from observation through learning and later explain why each action was taken.

### Phase 1b — mathematical core

- Add mathematical subtypes without forking the epistemic model.
- Add definitions, conjectures, lemmas, theorems, proofs, counterexamples, examples, and calculations.
- Add mathematical dependency and proof relations.
- Preserve LaTeX exactly and export structured mathematical views.
- Support formal-system and verification metadata without claiming verification automatically.
- Complete one conjecture-to-theorem workflow and one mathematics-to-code workflow.

Exit criterion: a mathematician can explain which definitions and lemmas a theorem uses, why a conjecture changed, and whether its proof is merely recorded, checked, peer reviewed, or formally verified.

### Phase 2 — excellent Obsidian bridge

- Version generated Markdown markers.
- Export UUID, kind, and revision.
- Add `--stdout`, `--dry-run`, and deterministic output.
- Implement a parser and conflict preview.
- Add investigation, incident, decision, learning, and project projections.
- Add guarded action-checkbox synchronization.
- Add vault-root configuration and path safety.

Exit criterion: repeated export is lossless outside the owned section, and sync never silently overwrites conflicting edits.

### Phase 3 — developer integrations

- Detect Git repository context without scanning content implicitly.
- Attach immutable commit and source references.
- Import bounded test output and benchmark artifacts.
- Render accepted decision chains into ADR drafts.
- Render incident chains into postmortem drafts.
- Document integration contracts for Git hooks and CI without requiring either.

Exit criterion: Aporic materially helps with debugging, a technical decision, and a postmortem in real repositories.

### Phase 4 — agent-native MCP server

- Add read-only MCP resources and tools over stdio.
- Publish tool schemas and setup examples.
- Add revision-aware, attributed, opt-in mutation tools.
- Add audit actors and correlation IDs.
- Test with at least two independent MCP clients.

Exit criterion: an AI client can inspect evidence and reasoning chains without shell access, and every generated entry remains attributable.

### Phase 5 — useful AI proposals

- Add provider configuration and privacy modes.
- Add a fake provider and contract tests.
- Implement unsupported-claim review and discriminating-experiment proposals.
- Add preview, accept, reject, stale detection, and audit history.
- Add prompt-injection and malformed-output tests.

Exit criterion: AI exposes a useful gap or alternative explanation in a real technical investigation, degrades cleanly offline, and cannot bypass attribution or domain validation.

### Phase 6 — terminal workbench

- Add an optional Ratatui interface over the same application layer.
- Focus on capture triage, trace navigation, unresolved questions, action review, proposals, and conflicts.
- Preserve complete CLI parity for automation and accessibility.

Exit criterion: the TUI adds workflow speed without creating a second business-logic implementation.

### Phase 7 — discoverability and ecosystem

- Publish a concise demo of a real defect investigation across shell, Git, Obsidian, and an AI agent.
- Provide copy-paste MCP client configuration examples.
- Add architecture, epistemic-model, and security documentation.
- Package releases and shell completions.
- Create small, well-scoped issues labeled for contributors.

Exit criterion: a new user can understand the differentiator and complete the first integrated workflow in under five minutes.

## 18. Contribution rules for coding agents

### Before editing

1. Read this file, `README.md`, `Cargo.toml`, and the relevant modules.
2. Inspect `git status`; existing changes belong to the user unless proven otherwise.
3. State the invariant being changed and the smallest end-to-end outcome.
4. Identify whether the work touches a public interface, migration, security boundary, or external side effect.

### While editing

- Preserve unrelated changes.
- Prefer small domain types over strings and booleans with ambiguous meaning.
- Parameterize SQL; never construct it with user input.
- Add context at I/O boundaries without erasing typed domain errors.
- Keep stdout stable and protocol-clean.
- Use UTC internally and convert for display only.
- Make writes atomic and multi-record state changes transactional.
- Do not add a dependency for functionality that is clear and safer with the standard library.
- Do not add speculative abstraction without a second real implementation or test seam.
- Do not let model-generated text cross a trust boundary untreated.

### Documentation requirement

Update documentation in the same change when modifying:

- commands or flags;
- JSON or MCP schemas;
- configuration;
- database fields or migrations;
- Obsidian format markers;
- privacy, security, or network behavior.

Use an Architecture Decision Record under `docs/adr/` for choices that are expensive to reverse, including database replacement, embedded inference, sync semantics, network MCP transport, or a native Obsidian plugin.

### Definition of done

A change is done only when:

- behavior is implemented through the correct layer;
- validation and failure behavior are explicit;
- relevant tests include the failure path;
- formatting, tests, and strict Clippy pass;
- user-facing documentation is current;
- no real user database, vault, or remote service was mutated by tests;
- security and privacy implications were considered;
- the final report distinguishes completed work from future recommendations.

## 19. Anti-goals

Do not turn Aporic into:

- a hosted team project-management suite;
- a model-specific wrapper;
- a framework for arbitrary autonomous shell execution;
- a second Obsidian vault or proprietary note format;
- a vector database benchmark;
- a TUI whose business logic cannot be automated;
- a philosophical ontology users must study before recording an observation;
- a generic knowledge graph with no opinionated developer workflows;
- a theorem prover, computer algebra system, or LaTeX editor;
- a system that treats model-generated prose as mathematical proof;
- an issue tracker replacement competing on boards, sprints, or team administration;
- a system that equates confidence with truth;
- a collection of “AI” commands without measurable user outcomes.

## 20. Near-term implementation order

The next changes should be delivered in this order:

1. Write and test the Nanoshift-to-Aporic data and naming migration plan.
2. Add a schema-version table, UUIDv7 public IDs, revisions, actors, origins, and audit events.
3. Add `entries`, `relations`, and `sources`, migrating legacy tasks to actions.
4. Implement descriptive capture commands and stable JSON output.
5. Implement `link`, `trace`, open-question, unsupported-claim, and ready-action queries.
6. Add mathematical subtypes, relations, verification metadata, and LaTeX-safe projections.
7. Complete one end-to-end debugging workflow and one mathematical workflow.
8. Update Obsidian export to typed, versioned projections.
9. Introduce a read-only MCP server for entries, evidence, and traces.
10. Add guarded, attributed MCP proposals and mutations.
11. Implement `ai examine` with a deterministic fake provider before a real provider.

Do not start the TUI, embeddings, multiple hosted providers, or bidirectional sync before identity, attribution, relations, revision control, audit history, and machine interfaces are complete.

## 21. Primary references

- [Model Context Protocol specification and documentation](https://modelcontextprotocol.io/)
- [Official MCP Rust SDK](https://github.com/modelcontextprotocol/rust-sdk)
- [Ratatui documentation](https://ratatui.rs/)
- [SQLite vec1 documentation](https://sqlite.org/vec1/doc/trunk/doc/vec1.md)
- [SQLite FTS5 documentation](https://sqlite.org/fts5.html)
- [W3C PROV overview](https://www.w3.org/TR/prov-overview/)
- [Google SRE: Postmortem Culture—Learning from Failure](https://sre.google/sre-book/postmortem-culture/)
- [Architecture Decision Record guide and examples](https://github.com/architecture-decision-record/architecture-decision-record)
- [Cargo workspaces](https://doc.rust-lang.org/cargo/reference/workspaces.html)
- [The Twelve-Factor App: logs as event streams](https://12factor.net/logs)

These references inform choices; they do not override project requirements, release stability, or the security constraints in this document.
