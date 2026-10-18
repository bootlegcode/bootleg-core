# Container registration and CLI bootstrap — proposed implementation plan

Status: original milestone proposal, archived in the versioned core repository. The shared wire checkpoint is now implemented in bootleg-core; [contract-v1.md](contract-v1.md) freezes the API/WS decisions and workstream handoffs and takes precedence for wire details. CLI, smuggler and web implementations remain separate work. This milestone updates the sequencing in `boot-leg-design.md`: authenticated discovery and dashboard presence come before file sync.

## Milestone

A developer authenticates from the CLI, creates or selects a project, obtains its project token, deploys smuggler as an entrypoint, and sees each running agent in the CLI and web UI. Verify both Kubernetes and plain Docker. No cluster API credentials, inbound agent ports, or Kubernetes API access required.

Defer patch transfer/application, locks, restart-on-sync pipelines, E2EE, exact file drift, and probe proxying. Basic PID 1 supervision is in scope.

## Existing foundations

- `bootleg-cli`, `bootleg-smuggler`, and `bootleg-core` are scaffolds.
- Web already has project/org authorization, project creation, hashed project keys, rotation, deactivation, deletion, and activity history.
- Existing projects have numeric public IDs and one current project key. Creation/rotation disclose the key once; existing keys cannot be retrieved.
- Keep existing authorization rules and services; HTTP endpoints are adapters, not new parallel business logic.
- Project keys authenticate agents, not human users. They must not authorize project creation, membership management, or general account access.

### Project identity versus credentials

Use these terms consistently in documentation, domain types, API schemas, and configuration:

| Term | Meaning | Handling |
| --- | --- | --- |
| ProjectId / `project_id` | Stable public identity of the project; currently numeric. | Nonsecret; safe in URLs, configuration, UI, and logs. Never grants access by itself. |
| ProjectKey / project token | Raw bearer credential (`project_…`) issued once at creation/rotation. | SECRET; injected as `BOOTLEG_PROJECT_TOKEN`. Never place in URLs, ordinary logs, public metadata, or images. |
| ProjectKeyHash / `secret_hash` | One-way digest of the raw ProjectKey stored by the server for verification. | Not the raw key and not a client credential. Internal credential-storage data; exclude from API responses, UI, and logs. It must never be accepted as a bearer credential. |

`BOOTLEG_PROJECT_ID` contains only ProjectId, never ProjectKey or secret_hash. Registration sends the public project ID as metadata and proves authorization with the raw project key in the Authorization header. Hashing happens server-side. A digest does not require the same one-time secret delivery as the raw key, but it is not public project metadata. Rotation changes the key/hash without changing ProjectId.

## Contract decisions to freeze before parallel work

### Transport

- CLI -> web: versioned HTTPS JSON API under `/api/v1`.
- Smuggler -> web: outbound WSS `/api/v1/agent/connect`; bearer project token in Authorization header, never URL.
- First milestone only uses registration, acknowledgment, heartbeat, and errors. No remote command execution.
- UI: authenticated snapshot polling every 3 seconds while visible. This meets near-real-time needs with less complexity than an event stream; SSE can follow without changing the agent protocol.
- Production requires TLS. Local plaintext requires explicit development configuration.

### Identity

Separate these fields:

- `project_id`: existing public numeric ID, serialized as a decimal string on the wire.
- `container_id`: `ContainerId` serialized as `container_<26-character encoded UUIDv4>`, generated at each smuggler process start; stable across network reconnects, new after process/container restart. Unique target identity within a project.
- `display_name`: human label, not unique and never used as a routing key.
- Optional runtime metadata: pod UID/name, namespace, container name, operator-supplied cluster label, runtime hint. These are agent claims, not verified orchestrator identity.
- `smuggler_session_id`: `SmugglerSessionId` serialized as `smuggler_session_<26-character encoded UUIDv4>`, server-generated per accepted connection; old session callbacks must not mark a replacement session offline. This is a public fencing identifier, not a bearer credential.

Use the existing `mti` TypeID convention, not raw UUID strings or hand-built encodings. Each identity has a distinct Rust newtype with canonical parsing, expected-prefix and UUID-version validation, and string serialization. New device-grant and CLI-credential record identities use `device_grant_…` and `cli_credential_…`; their public IDs remain separate from secret authorization codes and bearer values. Never interchange IDs solely because their underlying UUID representations match.

Existing project IDs remain numeric for compatibility in this milestone; existing org IDs retain their domain TypeIDs. A project public-ID migration requires an explicit separate contract/data migration: the existing `project_…` value is a SECRET project key, not a public ProjectId. Never expose it as identity metadata.

One pod can have multiple instrumented containers, and identical pod names can occur across clusters/namespaces. Pod UID alone also does not distinguish containers or restarts. Container TypeID identity avoids those collisions without platform dependence. Ubiquitous language: a **Container** is one instrumented process lifetime, a **SmugglerSession** is one accepted control-plane connection, and **Presence** is its time-bounded connectivity—not application health.

### Smuggler configuration

- `BOOTLEG_API_URL`: control-plane base URL.
- `BOOTLEG_PROJECT_ID`: public project ID.
- `BOOTLEG_PROJECT_TOKEN`: current project key, injected at runtime from a Secret.
- `BOOTLEG_DISABLED=true`: unconditional no-network exec-through mode.
- `BOOTLEG_INSTANCE_NAME`: explicit display label.
- `BOOTLEG_NAME_ENV`: name of an environment variable to read for the label; equivalent agent flag `--name-env`.
- `BOOTLEG_BUILD_COMMIT`: immutable build-time Git baseline, optional for discovery.
- Optional `BOOTLEG_CONTAINER_NAME`, `BOOTLEG_CLUSTER_NAME`, `BOOTLEG_BUILD_DIRTY`.

Label precedence: explicit name flag/env -> explicitly selected name-env -> `POD_NAME` -> `KUBERNETES_POD_NAME` -> `HOSTNAME` -> OS hostname -> shortened instance ID. Append known container name when deriving a pod-based label. Explicitly selected missing/empty variables are configuration errors, not silent fallbacks. Bound and validate labels; render as escaped text.

The local CLI's deployment generator accepts `--name-env VARIABLE` and writes `BOOTLEG_NAME_ENV=VARIABLE` into generated configuration. It does not read that variable on the developer's laptop.

Kubernetes does NOT automatically provide POD_NAME/POD_UID/POD_NAMESPACE. Generate Downward API env entries from `metadata.name`, `metadata.uid`, and `metadata.namespace`. Container name is explicitly configured in the manifest. HOSTNAME is only a fallback; Docker's configured container name is not reliably discoverable from inside the container.

No token and no project ID: exec-through, no network. Explicit disable overrides everything. Partial credentials: report configuration error and exit rather than silently pretending registration works. With complete configuration, start/supervise the application independently of control-plane availability. Authentication failure disables remote participation and surfaces diagnostics, but does not kill the application or hot-loop retries.

### Build baseline

A Docker ARG is not available at runtime unless persisted in the final image stage:

```dockerfile
ARG BOOTLEG_BUILD_COMMIT
ENV BOOTLEG_BUILD_COMMIT=${BOOTLEG_BUILD_COMMIT}
ENTRYPOINT ["/usr/local/bin/smuggler", "--"]
CMD ["your-app"]
```

Supply a full Git object ID during build; never bake the project token into an image or build argument. Ensure the recorded commit actually describes the build context; dirty builds must be marked, not reported as clean baselines. Allow full SHA-1 or SHA-256 object IDs, not abbreviated SHAs. Missing baseline permits registration but is displayed as unknown.

Commit identity is not proof of filesystem equality: dirty builds, generated files, bind mounts, and later syncs can differ. Initially CLI reports baseline relationship (equal/ancestor/diverged/unknown), local tracked/untracked changes, and missing local objects. It must not claim exact container drift. Do not automatically fetch arbitrary remotes. Future sync adds applied revision/content manifests separately; never overwrite build_commit with a synced revision.

## Domain-driven design and testing requirements

Organize each implementation around domain concepts and application use cases, with transport/storage/process adapters at the edges. Documentation and code should use the same ubiquitous language: Container, SmugglerSession, PresenceLease, BuildBaseline, Project, and ProjectCredential. Do not model the business domain as WebSocket handlers or serialized JSON structs.

- Reuse the Projects/Organizations/Accounts bounded contexts and their authorization services. Add a container-presence context owning Container registration, session replacement, lease renewal/expiry, and disconnect decisions. CLI workflows and smuggler lifecycle policies likewise live outside Clap/network/process adapters.
- Domain value objects validate names, typed identities, baselines, and configuration. Domain behavior enforces allowed transitions; application services coordinate authorization, repositories, clocks, and transactions. HTTP/WS handlers only decode/validate wire input, invoke use cases, and encode results. Database/cache adapters enforce atomic persistence and fencing.
- A Container owns the current SmugglerSession identity. Renew/disconnect from a stale session cannot change current Presence. A reconnect preserves ContainerId but replaces SmugglerSessionId; a new process produces a new ContainerId. Keep these invariants explicit in domain methods and application tests, not hidden in socket callbacks.
- Shared wire DTOs are not domain aggregates. Map explicitly at boundaries; keep infrastructure and web-framework dependencies out of domain modules. Narrow ports represent business needs, not copies of HTTP or WebSocket APIs.

**Fully test business logic without mocking API or WebSocket calls.**

1. Domain tests call real value objects, policies, and state machines directly. Use explicit time/ID inputs for deterministic boundary tests; cover every rule and rejected transition, including typed-ID parsing/round trips/wrong prefixes, duplicate names, session replacement, stale renewal/disconnect, expiry boundaries, configuration precedence, no-op policy, and Git-baseline classification.
2. Application tests invoke use cases directly, not routes. Small in-memory repository/clock implementations are allowed at domain ports where useful; they are not network mocks. Exercise actual authorization/business policies. Cover denied access, credential invalidation, failures, and lifecycle outcomes. Verify transaction, concurrency, TTL, and fencing behavior separately against real PostgreSQL/Valkey adapters.
3. Adapter/integration tests run the real HTTP/WS server and real clients over local sockets with isolated real backing services. Test authentication, serialization, reconnects, permission isolation, and malformed input through actual transport. For failures stop connections/services or use controlled fault injection; do not substitute canned API/WS responses or mock servers.
4. Linux process/container tests use real children and signals. End-to-end tests run real CLI, smuggler, and web components against Docker/kind.

JSON fixtures are serialization examples/golden contract tests only, never substitutes for business-rule coverage or fake network services. Each handoff lists domain invariants, use cases, adapter responsibilities, and tests proving them. Independently runnable domain/application suites are the parallel-development checkpoint; full integration follows once the real adapters are available.

## Proposed v1 surface

Freeze exact JSON schemas, status codes, enum values, limits, and fixtures before workers start. `bootleg-core` holds only portable wire DTOs/protocol helpers, not web DB/domain dependencies. Use a pinned shared revision for all workers.

Human authorization: implement browser-approved device login (RFC 8628 semantics), yielding an expiring, revocable user API credential. Approval uses existing logged-in browser session and CSRF protections. Credentials are hashed server-side and every API request rechecks existing user/org/membership eligibility. No CLI password collection and no project-token substitution for user auth. Device codes must be short-lived, single-use, rate-limited, and support pending/slow_down/denied/expired states. Browser approval must clearly show what is being authorized. Initial credentials can require re-login on expiry; refresh tokens are not needed today.

| Endpoint | Purpose |
| --- | --- |
| POST /api/v1/auth/device/code | Begin device authorization |
| POST /api/v1/auth/device/token | Poll/exchange approved grant |
| POST /api/v1/auth/logout | Revoke current CLI credential |
| GET /api/v1/orgs | List eligible workspaces |
| GET/POST /api/v1/orgs/{org_id}/projects | List/create using existing permissions |
| POST /api/v1/projects/{project_id}/token/rotate | Rotate current key, return once; warn this invalidates agents' old key |
| GET /api/v1/projects/{project_id}/instances | Authorized bounded/paginated snapshot |
| GET /api/v1/agent/connect (WS upgrade) | Project-key-authenticated agent connection |

Project creation returns metadata plus one-time project token. Selecting an existing project does not recover its key. For today preserve one-key semantics; multiple named agent credentials are a later migration, not an accidental new API promise. Creation requires an idempotency strategy or an explicit no-automatic-retry policy on ambiguous failures; rotation must never retry blindly.

Registration frame fields: `type=register`, `protocol_version=1`, `project_id`, `container_id`, `display_name`, `build_commit` (nullable), `build_dirty` (nullable), `platform` (os/arch), `agent_version`, `runtime` (optional metadata). Server checks claimed project against authenticated key before acknowledging.

Acknowledgment: `type=registered`, `smuggler_session_id`, `heartbeat_interval_seconds=15`, `lease_timeout_seconds=45`, server timestamp. Heartbeats keep that connection lease alive; server acknowledges them. Agent reconnects on missing acknowledgments with capped exponential backoff and jitter. The same instance reconnect replaces/fences the older connection. Require registration within a short bounded timeout and bound frame size (suggest 16 KiB).

Snapshot includes identity/metadata, server `first_seen_at`/`last_seen_at`, and presence status. Presence means control-plane connectivity, NOT application readiness. Keep application state separate (starting/running/exited/unknown) and never label process-running as application-ready.

Server time determines lease expiry. Clean disconnect marks offline promptly; abrupt loss expires within 45 seconds. Never trust client timestamps for liveness. Keep recently offline instances for a defined bounded window (initially 24 hours); page results and enforce per-project quotas. Heartbeats are operational data, not activity-history events.

Revalidate project-key eligibility during connected sessions (at heartbeat at latest): rotation, project/org deactivation, and deletion must stop existing sessions as well as new handshakes. Reject further traffic immediately after invalidation is observed. Do not rely only on opening-handshake auth.

Store instance metadata/history in PostgreSQL and bounded connection leases in shared Valkey, with atomic compare-by-SmugglerSessionId updates. Existing Valkey TTL rules apply. Web snapshots must work across replicas; an in-process registry cannot be authoritative. Connection ownership can remain local for this presence-only milestone; cross-replica command routing is deferred.

Use a consistent error envelope with stable machine code, safe message, and request ID. Explicit fixtures cover auth failure, forbidden/not-found isolation, validation, rate limiting, service unavailable, and protocol incompatibility. Bound input/connection counts and never log bearer headers or secrets.

## Three independent workstreams

### Agent A — CLI (`bootleg-cli` only)

1. Clap command tree, config precedence, API client, safe error handling, human and `--json` output.
2. `bootleg login`, `logout`, `whoami` (add matching identity endpoint to frozen contract), workspace selection.
3. `bootleg projects list/create/use`; `bootleg project token rotate` with explicit confirmation.
4. `bootleg init` stores only nonsecret API/project/deployment settings in repository config. Store user credentials in OS keychain or explicitly documented owner-only file fallback; CI credential via environment. Keep credentials scoped to their API origin.
5. `bootleg instances list --watch` and baseline inspection against local Git.
6. Generate Docker/Kubernetes setup snippets, including Downward API, build metadata, name-env override, and runtime Secret references. Do not auto-apply Kubernetes resources or print secret-bearing manifests by default. Project secrets require explicit secure delivery, never default JSON/log output.
7. Direct domain/application tests for workflows, name-env generation, Git edge cases, and secret-redaction policies; serialization fixture tests; real-server integration tests for auth expiration, permission errors, and ambiguous mutation failures. No mocked API calls.

Independent checkpoint: passing domain/application tests and offline configuration generation. Integrated demo: actual CLI creates/selects projects and watches real registered containers against the actual web service. Do not edit core or web during parallel work.

### Agent B — smuggler (`bootleg-smuggler` only)

1. Domain configuration and deterministic name resolution, startup ContainerId, SmugglerSessionId lifecycle, disabled exec-through mode.
2. Enabled PID 1 supervision: launch child promptly, forward termination signals to its process group, reap orphaned children, bounded shutdown, and preserve meaningful exit status. Network failures must not block child startup or shutdown.
3. WebSocket client, register/ack, application heartbeat/ack, reconnect/backoff, bounded queues/timeouts, graceful disconnect. No incoming arbitrary-command handler.
4. Gather explicit runtime metadata without Docker socket, Kubernetes API, service-account token, privileged permissions, or automatic metadata endpoint requests.
5. Direct domain/application tests for lifecycle policies and typed identities; real-server WS integration tests; Linux container tests for PID 1 behavior, SIGTERM/SIGKILL cases, duplicate labels, reconnect identity, new-process identity, invalid credentials, missing build SHA, and disabled zero-network operation. No mocked WebSocket calls.
6. Minimal Docker example and kind fixture with two replicas and an instrumented multi-container case. Preserve existing app probes; no probe proxy yet.

Independent checkpoint: passing domain/application tests and real local process supervision. Integrated demo: container supervises a sample process and registers/reconnects to the actual web service. Do not edit core or web during parallel work.

### Agent C — web/control plane/UI (`bootleg-web` only)

This is deliberately more than UI: the missing API/control plane must have an owner.

1. Device authorization and user API credentials, using existing account eligibility and permission services.
2. Project API adapters, one-time secret responses, key rotation, API documentation.
3. Agent WS endpoint, project-scoped registration, fenced shared leases, expiry/cleanup, snapshot endpoint, live-session revocation handling.
4. Project-detail instances table: display label plus short instance ID, runtime metadata, build SHA/unknown/dirty, connected/offline, process status, and last seen. Empty, loading, stale-data, and permission-error states.
5. Poll authenticated UI fragments/snapshots every 3 seconds while visible; prevent overlapping polls. On failures show stale state, not falsely current online labels. Recheck access on every request. Existing cookie mutation routes retain CSRF protection.
6. Tests for cross-org isolation, spoofed project ID, rotation/deactivation/deletion while connected, reconnect fencing, lease expiry, multiple web replicas, escaped labels, and no secret leakage. Use inline Tailwind; follow repo `just rebuild` / `just test` instructions.

Independent checkpoint: passing domain/application and real PostgreSQL/Valkey adapter tests. Integrated demo: real smuggler connects, table updates and expires, and real CLI performs project operations. Do not edit CLI/smuggler during parallel work.

## Coordination and integration

Before launch: approve this proposal, resolve protocol questions, then have one coordinator publish minimal bootleg-core DTOs, exact API/WS schemas, and shared JSON fixtures. All three workers use those fixtures immediately. No concurrent core edits; changes go through the coordinator. Each repo is independently versioned; parent directory is not a Git repo, so move this planning document into a chosen versioned home before relying on its history.

Prioritize web API/auth first because it is the largest workstream. Other agents implement and directly test domain/application behavior rather than waiting for network adapters; no API/WS mocks are required. This is a milestone scope, not a guarantee that hardened device auth plus supervision fits one day. If needed, today's checkpoint is independently domain/application-tested components; do not replace human auth with an overprivileged project key just to get a demo.

End-to-end acceptance:

1. CLI login approves in browser; CLI creates project without web project forms.
2. Deploy two kind replicas: two unique instances appear in CLI/UI within 5 seconds of successful registration.
3. Deploy two instrumented containers in one pod and a plain Docker container; none collide even with identical labels.
4. Interrupt network: status goes offline within lease timeout plus polling delay; reconnect preserves instance identity and fences stale disconnect callbacks.
5. Restart container: new instance identity; old instance becomes offline.
6. Rotate token or disable project: existing and new unauthorized participation stops; unrelated org cannot view instances.
7. Stop app/container: correct signal/reaping/exit behavior; web outage does not stop the application.
8. Missing/dirty SHA visibly prevents claims of a clean sync baseline. No secret appears in Git, images, logs, URLs, or normal list output.
9. Disabled/no-credentials mode performs no network I/O.

Next milestone: read-only diff preview, then locked, revisioned transfer/apply/ack. Registration must not be described as working file sync.
