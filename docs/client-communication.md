# Client attribution and server-authored messages (v1)

This is a normative part of the [v1 wire contract](contract-v1.md).

## Version attribution on every communication

Every native CLI/smuggler HTTP request, including unauthenticated device login, bootstrap, logout, and every WS upgrade/reconnect, MUST carry exactly one of each header (names case-insensitive):

```http
Bootleg-Client: cli
Bootleg-Client-Version: 0.1.0
Bootleg-Protocol-Version: 1
```

Smuggler uses `Bootleg-Client: smuggler`. Product version is the actual executable's build/package version, not the core crate version, protocol version, or a user-configurable label. Versions are 1–64 ASCII letters/digits/`.`/`-`/`+` (e.g. `0.1.0-rc.1+abc123`). Headers are attribution claims, not authentication. User-Agent may additionally identify the product but is not the contract. Protocol remains `/api/v1` and header `1`; the header does not negotiate arbitrary versions.

Adapters reject absent/duplicate/invalid metadata with 400 validation, unsupported protocol with protocol_incompatible. They extract case-insensitive header names and call `ClientMetadata::from_headers`; errors must not echo supplied values. Responses include `Bootleg-Protocol-Version: 1`, including bootstrap and protocol-incompatible errors where the API adapter can respond. Browser UI cookie routes are separate existing adapters: they do not pretend to be CLI or supply fake CLI versions.

Web request context includes client kind/version on **every** request, making it available to safe operational diagnostics, version policy and announcement selection. Record the issuing CLI version on device-grant/credential records and the most recently observed version on authenticated use (credentials can be shared by different CLI versions; do not infer current version only from credential issuance). Never use attribution as a permission check.

WS connection context retains smuggler version/protocol metadata from upgrade, so all registration/heartbeat/close events are attributable without repeating version fields in every frame. Register `agent_version` MUST exactly equal the upgrade's Bootleg-Client-Version; register protocol_version MUST equal the header (1). Reject mismatch before accepting the session. Persist agent_version with container metadata; it is already in instances snapshots for the web table. Reconnect sends the headers again. CLI and agent versions are independent; web cannot infer a CLI version from an agent it deployed.

## Server-authored display text

Every ApiError, including WS error frames, includes:

```json
{
  "error": {
    "code": "service_unavailable",
    "level": "error",
    "message": "Registration is temporarily unavailable during maintenance. Please try again shortly.",
    "request_id": "req-example",
    "retry_after_seconds": 30
  }
}
```

`code`/HTTP status drive stable behavior, retry, auth handling and exit status. `message` is the authoritative server-authored user-facing wording. Clients MUST NOT replace it with a local message table indexed by code. They may add local context, request ID and a severity label, but display the received text unchanged in meaning. Wording may change without a client or protocol release. Localized server copy is possible later; no locale negotiation is promised here. Network failures or malformed/missing messages use a safe local fallback (never raw response bodies). Routine device pending/slow_down responses may be presented as progress rather than repeated errors; any displayed server explanation comes from message.

`level` is `info`, `warning`, or `error`; it controls presentation only. Normal failures use error, authorization_pending uses info, and slow_down uses warning. An error envelope still indicates the unsuccessful operation irrespective of its presentation level. Clients must not treat info as success or error-level announcements as command failures.

MessageText is 1–512 Unicode scalar values, single-line, without control characters or leading/trailing whitespace. Messages are **plain text**, never HTML, Markdown, ANSI escape sequences, executable commands or format strings. Web authors must not put secrets in them or interpolate unsafe request data. Clients render as text: escape for HTML, neutralize terminal control/bidi formatting, do not auto-execute or automatically open embedded URLs. Only locally generated styles may color severity. UI may visually wrap long text. Serialization/decoding errors must not expose raw untrusted content.

## Bootstrap and announcements

`GET /api/v1/client/bootstrap` is public, rate-limited (initial ceiling 120/min/source IP), and takes the required client headers; no query parameters or request body. It returns HTTP 200, `Cache-Control: no-store`, and bootstrap-response schema:

```json
{
  "protocol_version": 1,
  "notices": [
    {
      "id": "maintenance-2026-01-02-v1",
      "level": "warning",
      "message": "Scheduled maintenance tomorrow at 02:00 UTC."
    }
  ]
}
```

No credentials are required or sent for bootstrap, so notices can appear before login and during authentication outages. All bootstrap notices are public: never expose tenant/private details. Normal error envelopes cover rate limiting/unavailability. Web owns announcement publishing, active scheduling windows and targeting by product/version; selection is evaluated with server time. No client-side version comparison or scheduling is required. Server admin UI/storage is a separate web implementation concern, not remote command configuration.

Server returns 0–4 currently active notices in intended display order, with unique IDs per response. Each ID is a bounded 1–128-character nonsecret text label, not a credential or TypeID; change the ID when the wording/severity materially changes. IDs support deduplication, never local text lookup. An empty list means no current notice. Bounded `Notices` enforces count and uniqueness at decoding. Total WS frame limits still apply.

CLI requests bootstrap once per network-using invocation, including login and long-running watch startup, **not** only once at installation or credential issuance. Do not add requests to local/offline commands. Bootstrap is best-effort with a maximum 2-second budget and no automatic retry; failure must not prevent the actual command or replace its result. No implicit daily suppression/cache in v1: users must not miss changed maintenance notices. A command reusing connections should render each ID at most once in that invocation. Continuous watches need not fetch later announcements in v1; the next invocation picks them up. Live server push is not promised.

Smuggler receives the same `notices` list in every registered acknowledgment; it does not make an extra bootstrap call. Deduplicate IDs for the process lifetime using a bounded cache (at most 128 IDs, FIFO eviction); new IDs are eligible on reconnect. Announcements are diagnostics only and must not block/kill the supervised application or alter registration/lease handling.

### CLI output and suppression contract

- `--notices=auto|always|never`, default auto; equivalent `BOOTLEG_NOTICES` setting, with explicit flag taking precedence. Invalid values are configuration errors.
- auto displays notices to **stderr only** when stderr is a TTY and the command is not in `--json` mode. always explicitly opts in, still only on stderr. never suppresses informational announcements of **all** levels, including error-level banners; it never suppresses an actual failed-command diagnostic or changes exit status.
- `--json` stdout remains exactly the command's structured result/error contract, never prefixed/appended with a banner. In auto mode, noninteractive scripts see no announcements. No notice ID or severity is interpreted as an exit code.
- In never mode the CLI may skip bootstrap entirely. Bootstrap is not a compatibility/security gate: actual API requests enforce required protocol and authorization even if banners are suppressed.
- Local transport failures use local diagnostics. Well-formed server errors use the server's message in both human diagnostics and structured errors. Secret-bearing API DTOs remain excluded from default output.

### Testing adapters

Core tests cover the wire types and codecs. Test application behavior separately against a real server: CLI output and exit status when notices are present or bootstrap fails; agent notices that don't affect the child process; and server-side version attribution, mismatch rejection, and public notice selection. See [integration and test scope](registration-evidence.md).
