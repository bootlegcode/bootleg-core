# Registration implementation and evidence

Registration is implemented across CLI, web and smuggler. Browser presence uses
SSE with one-second server-side shared-store observation and session/project
visibility revalidation each iteration, not browser polling or Valkey pub/sub.
Public project identities are canonical UUIDv4 `project_…` TypeIDs, never project
bearer keys. Presence is connectivity, not readiness or filesystem equality.

## Compatible sibling checkouts

Use one parent containing `bootleg-core/`, `bootleg-cli/`, `bootleg-smuggler/` and
`bootleg-web/`. Cargo path dependencies require this layout; Cargo.lock does not
pin their source revisions. The CLI harness enforces exact dependency commits in
[`acceptance-revisions.json`](../../bootleg-cli/scripts/acceptance-revisions.json).
It does not reset checkouts; tracked dependency edits require explicit opt-in.
See [reproduction and current pins](../../bootleg-cli/docs/acceptance.md).

## Historical executed checkpoint

The initial full registration acceptance used these exact clean revisions:

| Repository | Revision |
| --- | --- |
| CLI | `f29e2ccac873f60b6b9aad200bc7ec18d880c22e` |
| web | `515974dcdfcc03f60ab4e0ca488726860f49c126` |
| core | `05144d6e047fc5c88afd58143d9f6771fd262af7` |
| smuggler | `7869c5da4015894eb523232e0285c11140d03143` |

[Sanitized successful report](../../bootleg-cli/.pi/acceptance/bootleg-acceptance-d045c3a7f1ac/report.json)
records `registration_complete: true` and completed cleanup. This local handoff
artifact lives in the sibling checkout; later durable reports and quota/capacity
and read-only preview results are linked from the CLI acceptance document.

Lifecycle, browser, Docker and kind scenarios run actual executables. Stale-session
heartbeat/disconnect fencing has **supplemental real-store and WebSocket protocol
client evidence**, not executable-agent evidence. The smuggler's two standalone
opt-in real-web tests are separate and are not retroactively counted as run.

Registration acceptance is not a full web CI pass: historical dashboard assertion
failures and formatting drift remain documented in
[web acceptance](../../bootleg-web/docs/registration-acceptance.md).
This documentation reconciliation does not rerun acceptance or alter its reports.
Read-only CLI diff preview has subsequently completed; file transfer, remote
filesystem mutation, sync locks and apply/ack remain future work.
