# Contributing to Aporic

Aporic is a community project. Bug reports, questions, documentation fixes,
and code are all welcome, and none of them require asking permission first.

## Getting set up

```bash
cargo build
cargo test
cargo clippy --all-targets
cargo fmt
```

Aporic depends on a bundled SQLite, so there is nothing to install beyond a
current stable Rust toolchain.

Run the tool against a scratch database instead of your own notes while you
work on it:

```bash
XDG_DATA_HOME=$(mktemp -d) cargo run -- init
```

`cargo run -- tutor` is the fastest way to see the whole reasoning lifecycle;
it always runs against an in-memory sandbox, never your real database.

## Before you open a pull request

- `cargo test`, `cargo clippy --all-targets`, and `cargo fmt --check` all pass.
- New behaviour comes with a test. Bug fixes come with the test that fails without the fix.
- The change is described in the commit message: what changed, and why it was worth changing.

Small, focused pull requests get reviewed quickly. If a change is large or
reshapes the data model, open an issue first so the design discussion happens
before the work.

## What Aporic will not accept

These are product decisions, not review preferences, so a pull request that
crosses one will be turned down regardless of how good the code is:

- Anything that requires a network connection for core capture, listing, or
  export. Aporic is local-first, and the database on disk is authoritative.
- Silent inference: turning an assumption into a fact, a model's guess into an
  observation, or a suggestion into a decision. Every entry keeps its author
  and origin.
- Telemetry, analytics, or any background phone-home.
- Destructive schema changes. Migrations are forward-only, versioned, and
  tested against a database created by the previous version.
- Output changes that break `--json` consumers without a version bump.

## Schema changes

The schema version lives in `src/db.rs`. A change means: a new `migrate_vN`
function, a bumped `SCHEMA_VERSION`, and a test that migrates a database
built by the previous version. Never edit an existing migration — users have
already run it.

## Reporting bugs

Include the output of `aporic status`, the exact command you ran, what you
expected, and what happened. If it involves your own entries, redact the
bodies; the IDs, kinds, and states are usually enough.

## Security

Report suspected vulnerabilities privately to mistervoga@gmail.com rather
than in a public issue.

## License

Contributions are made under the MIT license, the same terms as the rest of
the project. See [LICENSE](LICENSE).
