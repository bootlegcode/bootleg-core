# Integration and test scope

`bootleg-core` tests validate wire types, frame encoding and limits, client
metadata, and agreement between schemas and fixtures. They do not exercise
login, server-side authorization, process supervision, or presence storage.

The CLI, the smuggler agent, and the web control plane are separate applications.
If you maintain all four repositories, check them out as siblings (`bootleg-core`,
`bootleg-cli`, `bootleg-smuggler`, and `bootleg-web`) for local Cargo path
dependencies. Path dependency revisions are not pinned by `Cargo.lock`; use the
companion CLI's acceptance instructions and revision pins when running the
integrated suite. Access to those repositories is not needed to test this crate.

Historical registration acceptance exercised the actual CLI, web control plane,
and smuggler with Docker and kind. Supplemental stale-session heartbeat and
disconnect fencing checks used real stores and WebSocket protocol clients, not
the smuggler executable. This was registration acceptance, not a full web CI run
or proof that file transfer or remote filesystem mutation works. Presence reports
connectivity, not application readiness or filesystem equality.
