# Container registration contract v1

Status: frozen bootstrap contract for parallel implementation. This crate implements the **wire contract**, not working login, registration, presence storage, or sync. The versioned proposal is [container-registration-plan.md](container-registration-plan.md); this document resolves its open wire decisions and takes precedence for v1. Changes go through the core coordinator, not individual workstreams. [Client attribution and server-authored messages](client-communication.md) is a normative part of this contract: mandatory client/version headers on every native-client request, server display text/severity, and bootstrap/registration announcements.

## Artifacts and compatibility

- `src/{ids,values,api,protocol}.rs`: portable identities, validated wire scalars, DTOs, bounded JSON frame helpers.
- `schemas/v1/*.json`: generated JSON Schema 2020-12. Semantic constraints below (UUID version/variant, integer range, Unicode whitespace/control validation, permissions, timing) are mandatory even when not expressible in a schema. Schemas do not replace domain validation.
- `fixtures/v1/*.json`: deterministic examples, including all error codes. All credentials in fixtures are public test values and must never be deployed. Fixtures are not mock servers.
- Run `cargo test`, `cargo clippy --all-targets -- -D warnings`, and `cargo fmt --check` independently, with no PostgreSQL/Valkey/network services. Regenerate artifacts with `cargo run --example export-schemas` and `cargo run --example export-fixtures`; tests reject schema drift.
- All workers must use the same core checkout/revision. Sibling dependency during development: `bootleg-core = { path = "../bootleg-core" }`. Record `git -C ../bootleg-core rev-parse HEAD` in handoffs once the coordinator commits this checkpoint. Do not pin the old scaffold revision or independently edit core. No published crate release is implied.

## General encoding and security

HTTPS `/api/v1`, JSON UTF-8 `Content-Type: application/json`; WSS `/api/v1/agent/connect`. No plaintext unless explicitly enabled for development by both clients/server configuration. Base URLs must not contain credentials, query strings, or fragments. Never forward Authorization on cross-origin redirects; credentials are origin-scoped. No bearer in URLs, images, build arguments, ordinary logs or ordinary CLI `--json` output.

HTTP JSON request bodies are at most 16 KiB (413 for larger bodies at the adapter, with `validation` envelope if possible). Responses and WS text messages use these exact DTO fields. Unknown fields/enum variants are rejected in v1. Optional values may be omitted on input or null; canonical responses serialize them as null. All other fields are required. Empty mutation bodies are `{}`. JSON numbers for project IDs are invalid. Dates are RFC 3339 UTC timestamps; producers use `Z`, and server time alone determines presence. Clients must not reinterpret presence as readiness.

ProjectId is a canonical positive decimal i64 string (`1` through `9223372036854775807`); no sign, leading zero or whitespace. ContainerId, SmugglerSessionId, DeviceGrantId, CliCredentialId, OrgId and UserId are distinct canonical mti TypeIDs containing RFC 4122 variant UUIDv4, with prefixes `container`, `smuggler_session`, `device_grant`, `cli_credential`, `org`, `user`. Core OrgId/UserId are wire counterparts, not replacements for existing domain types. Convert explicitly at adapters.

Project token is the existing secret `project_…` ProjectKey, never ProjectId. Its digest remains server-internal and is never accepted as bearer. Core intentionally has no key hashing/authentication implementation. `SecretString` redacts Debug and has no Display; **Serialize exposes the secret for wire delivery**. Never log serialized secret-bearing DTOs or use them directly as CLI output. Explicit `expose_secret()` is for credential storage/header/delivery only. Project eligibility and key parsing/hashing remain in existing web services.

DisplayName: 1–128 Unicode scalar values. MetadataText: 1–256; AgentVersion: 1–64; PlatformName: 1–32; RequestId: 1–128; Cursor: 1–512. All reject Unicode controls and leading/trailing Unicode whitespace. No silent trimming. Display names need not be unique, may contain markup, and must be rendered as escaped text. BuildCommit: exactly 40 or 64 lowercase ASCII hex characters. Null build_commit or null build_dirty is unknown, never evidence of a clean baseline. Runtime metadata is unverified agent claims; no metadata endpoint/orchestrator access is implied.

## HTTP routes

`U` means expiring user CLI bearer, rechecking user/org/membership eligibility on every call. Project bearer cannot access any U route. `P` means raw project bearer. No cookie-auth alternative for these CLI mutations; existing browser CSRF rules remain intact. Lists include only eligible resources. Cross-org/non-visible project lookup returns 404, not existence-revealing 403; 403 is for a known visible resource where a permitted action is denied.

| Method/path after /api/v1 | Auth | Request schema | Success/schema |
| --- | --- | --- | --- |
| GET /client/bootstrap | none, rate limited; client headers required | none | 200 bootstrap-response |
| POST /auth/device/code | none, rate limited | device-code-request | 200 device-code-response |
| POST /auth/device/token | device code in body | device-token-request | 200 device-token-response |
| POST /auth/logout | U | no body | 204, no body |
| GET /auth/whoami | U | none | 200 identity-response |
| GET /orgs | U | page-query (URL parameters) | 200 orgs-response |
| GET /orgs/{org_id}/projects | U | page-query | 200 projects-response |
| POST /orgs/{org_id}/projects | U, existing create permission | create-project-request | 201 project-token-response |
| POST /projects/{project_id}/token/rotate | U, existing rotation permission | rotate-project-token-request | 200 project-token-response |
| GET /projects/{project_id}/instances | U, existing project visibility | page-query | 200 instances-response |
| GET /agent/connect | P | WebSocket upgrade | 101, then WS protocol |

Create uses existing project settings/permission services; URL null or a credential-free HTTP(S) URL at most 2048 bytes; existing validation/normalization still applies. End-to-end encryption remains false for new CLI-created projects in this milestone (no promise of file sync/encryption functionality). Project responses expose the existing setting for compatibility. Rotation changes token/hash but not project identity. Only create/rotate return a raw key; lists/select never recover one.

**No automatic mutation retries** for creation, rotation, device-code issuance or device-token exchange after ambiguous transport failures. Creation/rotation require user reconciliation/confirmation; an undelivered rotated key may require another explicitly confirmed rotation. An ambiguously completed single-use device exchange requires a fresh login. GET retry/backoff is safe. Logout revokes the current credential; repeated use of a revoked credential gets 401, which the CLI can treat as locally logged out.

Pagination: `limit` decimal integer 1–200, default 50; `cursor` opaque bounded string, optional. Unknown/duplicate query parameters or malformed/out-of-scope cursors return 400 validation. Cursors must be server-validated and scoped to principal, endpoint and project/org; never contain credentials. Organizations sort by org ID, projects by numeric project ID, instances by `(first_seen_at, container_id)` ascending with keyset pagination. No total count. `next_cursor=null` means end. Snapshots are per-page live reads, not a transaction across pages; watch clients begin each refresh at page one. Each instances response includes server_time. UI polls every 3 seconds while visible without overlap; failed polls mark retained data stale.

All API responses carrying credentials use `Cache-Control: no-store`; use no-store for all authenticated/discovery responses as well. Do not echo request values into error messages. Request IDs are server-generated opaque correlation labels, not user-provided secrets.

### Errors

HTTP error shape is `{"error":{"code":…, "level":…, "message":…, "request_id":…, "retry_after_seconds":null}}`. `level` is info/warning/error; `message` is server-authored MessageText (1–512 characters), not a client-side lookup. Codes drive behavior; level only drives presentation. Stable enum/status mapping is `ErrorCode::http_status()`:

- 401 unauthorized (absent/invalid/expired/revoked bearer; include `WWW-Authenticate: Bearer`).
- 403 forbidden; 404 not_found for isolation; 400 validation.
- 409 conflict (e.g. inconsistent reconnect metadata).
- 429 rate_limited or quota_exceeded; 503 service_unavailable.
- 400 protocol_incompatible (WS post-upgrade uses same code, not an HTTP response).
- 400 authorization_pending, slow_down, access_denied, expired_token, invalid_grant for device exchange.

Populate retry_after_seconds and HTTP Retry-After with the same integer delay when applicable (429/503). Validation is also used for adapter 413 body-size rejection; this is the only status override. Unknown server errors are a safe 503 rather than leaking internals. Fixtures cover every code, including forbidden/not-found separation.

### Browser-approved device authorization

This is an RFC 8628-*semantics* JSON API, not a form-encoded OAuth endpoint: no client_id, scope, grant_type or refresh token in this first-party v1 surface. Begin request is `{}`; response gives public device_grant_id, secret device_code, secret user_code, verification_uri, expires_in=600, interval=5. No verification_uri_complete: codes must not go into URLs. verification_uri is an absolute HTTPS browser page (explicit development exception), with no secret/query/fragment. CLI may intentionally display the user_code and browser URL for approval, but not device_code/access_token.

Device codes and user bearer values are independently generated opaque secrets (at least 256 bits CSPRNG entropy); public record IDs never authorize exchange. User code uses 8 random characters from `ABCDEFGHJKLMNPQRSTUVWXYZ23456789`, displayed `XXXX-XXXX`, with normalized comparison on the browser side. Bound incoming secret fields to 512 bytes; reject empty/control-bearing values before lookup. Hash server-side, use constant-time credential verification where applicable, never persist raw bearer/device codes. User code uniqueness and online attempt limits must be enforced.

Grant transitions: pending -> approved/denied/expired; approved -> exchanged once or expired. Exchange atomically consumes the grant and creates a credential, or returns no credential. Check expiry at `now >= expires_at` even if approved. Pending returns authorization_pending; polling earlier than interval returns slow_down and permanently increases that grant's interval by 5 seconds (including future polls). retry_after_seconds in slow_down is the **new total interval**, not just +5. Denial returns access_denied, expiry expired_token, unknown/consumed codes invalid_grant. Terminal codes stop polling. Expired records may later be purged and return invalid_grant. Initial CLI credential lifetime is 30 days; no refresh, re-login on expiry. Identity returns its absolute expires_at.

Approval is a web-owned HTML route (verification_uri), requires an existing eligible authenticated browser session and CSRF, clearly identifies Bootleg CLI account-level authorization, and requires explicit approve/deny. No passwords collected by CLI; never silently approve merely by opening a link. Server-side initial limits: device initiation 10/min/source IP, browser code attempts 10/min/account and IP, device exchange 60/min/source IP plus per-grant interval. Limits may be lowered operationally; rate_limited + Retry-After handles this without protocol change. Proxy trust configuration is the web adapter's responsibility.

## WebSocket protocol

Text JSON only, maximum 16,384 **UTF-8 bytes** per complete message including fragmentation (enforce before allocation growth); no compression negotiation. Binary or malformed input is rejected. First application frame must be register within 5 seconds of upgrade. Protocol version is exactly 1; no arbitrary command messages. ApplicationState is required on register and heartbeat: starting/running/exited/unknown. It reports process state, never health/readiness.

Agent creates one ContainerId per smuggler process lifetime and keeps it across reconnects. After authenticating P, server checks claimed project_id against that credential. Register supplies immutable lifetime metadata (display_name, baseline, platform, version, runtime). A reconnect for the same ContainerId must match this metadata; otherwise conflict. Names can duplicate freely. Server assigns a new SmugglerSessionId on **every accepted connection**, atomically replacing/fencing the old one before sending registered. Registered contains heartbeat_interval_seconds=15, lease_timeout_seconds=45, server_time, and notices (a bounded array, empty when there are no announcements). first_seen_at is preserved across reconnects while retained.

Agent sends one heartbeat at a time every 15 seconds after the previous acknowledgement, containing the acknowledged session ID and current application_state. Server returns heartbeat_ack with that session ID and server_time after authorization revalidation and atomic lease renewal. No client timestamp or sequence number: one outstanding heartbeat on one ordered socket is sufficient. Missing registered within 5 seconds or heartbeat_ack within 15 seconds causes agent to close and reconnect. WS ping/pong does **not** renew the application lease.

Server lease is valid while `now < last_renewed_at + 45s`; equal means expired. Only current-session callbacks can renew, update application_state or disconnect. Renewing an expired lease is rejected; reconnect to obtain a new session. Clean disconnect marks the current session offline promptly; abrupt loss expires within 45 seconds. Offline snapshot retains last reported process state (not an assertion the process is still running), last accepted session ID and metadata. last_seen_at is the last accepted registration/heartbeat server time, not a client time or disconnect time. Keep offline records for 24h from offline transition/lease expiry; web worker owns atomic fencing and cleanup.

Recheck project/key/org eligibility before every accepted heartbeat, not just upgrade. Rotation, deactivation and deletion must stop old/new unauthorized participation; once observed, reject all subsequent messages and terminate session without renewing. Stale session traffic must never affect replacement leases. Lease/store unavailability cannot be acknowledged as successful presence: return service_unavailable and close. In-process registries are not authoritative.

Post-upgrade errors are `ServerFrame::Error` with the same envelope fields, followed by close: 1008 for authentication/authorization/validation/conflict/protocol mismatch, 1009 for oversized message, 1013 for service unavailable/quota/rate limiting. Send a safe frame when feasible; no secrets in close reasons. Replacement may close old connection with 1008; stale-session code is conflict, which allows reconnect with backoff. Authentication/forbidden/not-found/protocol_incompatible/validation errors disable agent participation with diagnostics until configuration/process restart; do not kill the child or hot-loop. Transient failures retry with full jitter over [0, min(30s, 1s * 2^attempt)], reset attempt only after a successful heartbeat acknowledgement. Respect retry_after_seconds as a minimum. No registration retry on the same socket.

Initial per-project bounds: 10,000 retained Containers (including offline), 1,000 concurrent accepted sessions, one current session per ContainerId. At quota, reconnect of an existing Container may replace its session; new Containers are rejected until cleanup creates room. Limit pending upgrades to 20/source IP and agent handshakes to 120/min/source IP; web adapters must enforce these across replicas. Quotas can be lowered operationally, with quota_exceeded responses. IDs and metadata are claims from authorized agents, not cryptographic proof of runtime identity.

## Workstream handoff

### CLI (`bootleg-cli` only)

Own workflow/config policies, secure origin-scoped user credential storage, API adapter and safe human/JSON projections. Implement login/logout/whoami at the routes above; preserve one-time key delivery, no blind mutation retries, and key/identity separation. Build baseline classification lives here, not in core; never equate a commit with filesystem equality. Deployment generation writes name-env references without reading laptop environment variables, Downward API entries and runtime Secret references; no automatic apply. Prove workflows, secret redaction, Git edge cases and generator behavior directly with domain/application tests. Integrate later against real web sockets, not canned API responses.

### Smuggler (`bootleg-smuggler` only)

Own config/name precedence, no-credentials/disabled zero-network policy, typed ContainerId lifetime, session/reconnect policy and process supervision. Start child independently of network success. Resolve missing explicitly selected env vars as errors; partial credentials as errors; disabled mode overrides all. Use core bounded frame helpers at the WS boundary, never arbitrary command handling. Test policies with explicit time/IDs and real children/signals; then real web integration and Linux PID 1/container tests. No core or web edits.

### Web (`bootleg-web` only)

Own device grants/user credential domain, reuse existing authorization and project key/settings services, container-presence aggregate and use cases. Implement fencing/replacement/expiry/credential invalidation directly in domain/application methods, with transactional PostgreSQL metadata and atomic shared Valkey leases. DTOs are not aggregates. Test denied access, duplicate labels, stale renew/disconnect, replacement and exact expiry boundaries without mocked HTTP/WS. Prove cross-replica atomicity, TTL and credential invalidation against real stores; transport tests use real clients/server. UI polls bounded authenticated snapshots, escapes metadata and distinguishes stale/offline from readiness. Follow repo rebuild/test instructions.

This checkpoint contains only core scalar/protocol/serialization tests. It does **not** claim these three domain/application suites, backing services, supervision or end-to-end milestones are implemented. Once independent suites pass, run the acceptance scenarios in the versioned plan with actual CLI + smuggler + web on Docker and kind.
