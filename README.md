# bootleg-core

Portable Rust types and protocol helpers for Bootleg's v1 HTTP API and agent
WebSocket protocol. This crate defines the wire format; it does not run a server,
authenticate requests, manage presence, supervise processes, or synchronize files.

- [v1 wire contract](docs/contract-v1.md): routes, credentials, errors, and registration frames.
- [Client communication](docs/client-communication.md): required version headers,
  server-authored messages, and script-safe notice handling.
- [Integration and test scope](docs/registration-evidence.md): what core tests
  cover and what requires the companion applications.
- [`schemas/v1`](schemas/v1) and [`fixtures/v1`](fixtures/v1): generated JSON
  schemas and deterministic examples for clients in any language. Example
  credentials are synthetic and must not be used outside tests.

## Development

Run these checks from the repository root with Rust, rustfmt, and Clippy. CI
currently uses Rust 1.99.1. No running Bootleg service, database, or cache is needed.

```sh
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked --all-targets
cargo test --locked --doc
```

After a wire contract change, regenerate and commit the affected artifacts:

```sh
cargo run --locked --example export-schemas
cargo run --locked --example export-fixtures
```

Coordinate wire changes with consumers and update the contract docs, schemas,
and fixtures together. At application boundaries, map wire types to domain
values explicitly. Secret-bearing types redact `Debug` but serialize credential
values for delivery; don't use them for logs or default CLI output.

## License

[Business Source License 1.1](LICENSE) with a limited production-use grant.
Public source availability does not remove the license's use restrictions.
