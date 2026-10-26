# bootleg-core

Portable Rust wire contracts for the container-registration milestone. No web,
DB/cache, async runtime, process supervision or presence domain dependencies.

**Start here:** [frozen v1 contract and workstream handoffs](docs/contract-v1.md).
The [versioned milestone plan](docs/container-registration-plan.md) describes the
original CLI, smuggler and web workstreams; [registration evidence](docs/registration-evidence.md)
records their completed integration and the supported sibling checkout layout.

- `ids`: canonical typed mti UUIDv4 identities, including `project_…` ProjectId.
- `values`: bounded wire strings, full Git object IDs, redacted transport secrets.
- `api`: device login, identity, projects, snapshots, pagination and errors.
- `communication`: required client/version header parsing, message levels and bounded notices.
  See [client communication contract](docs/client-communication.md) for bootstrap,
  server-authored error text, and script-safe banner suppression.
- `protocol`: registration/heartbeat DTOs and bounded JSON frame codecs.
- `schemas/v1`, `fixtures/v1`: cross-language schemas and golden examples.

```sh
cargo test
cargo clippy --all-targets -- -D warnings
cargo fmt --check
# Regenerate published artifacts after a coordinated contract change:
cargo run --example export-schemas
cargo run --example export-fixtures
```

From a sibling crate:

```toml
[dependencies]
bootleg-core = { path = "../bootleg-core" }
```

Compatible dependency commits are enforced by the sibling CLI's
`scripts/acceptance-revisions.json`; update pins intentionally after coordinated
changes. Don't independently modify core contracts.
Map DTOs explicitly into each bounded context's domain types. Secret-bearing DTOs
serialize secrets for delivery: never use them as log or default CLI output models.

Passing core tests proves wire/scalar behavior, **not** integrated login,
registration, process supervision, presence fencing or file sync.
