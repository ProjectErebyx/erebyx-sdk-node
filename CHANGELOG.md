# Changelog

All notable changes to `@erebyx/sdk` (Node.js) are documented in this file.

This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html) and [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) format.

The underlying Rust crate is [`erebyx-sdk`](https://github.com/ProjectErebyx/erebyx-sdk/blob/main/CHANGELOG.md). Substrate-side (private engine) release notes are summarized in each SDK release entry below.

---

## [Unreleased] — pre-Genesis Arche launch tightening

### Changed (breaking, pre-launch)

- **`JsLoadContextOptions.mode` + `.specializationName` fields removed** — matches the substrate's MCP launch surface narrowing (SIMPLIFY-MODES). `mode` was a single-value enum (`"session"`) at v0.1; specialization-loading is a v0.2 feature. The substrate still accepts these fields on the wire if forwarded, but they're not part of the customer-facing TypeScript surface for v0.1.

### Changed (docs)

- `JsSaveResult.extra` catch-all docstring: `atomization` → `enrichment` — reflects the customer-facing rename of the substrate-internal queue envelope at the API boundary (same shape `{queued, queue_id}`, canon-clean key name per Genesis Arche brand canon).

---

## [0.1.1] — 2026-04-27 — Genesis Arche

First public release. Native Node.js client over the EREBYX v0.1.1 cognitive surface, built on `erebyx-sdk` (Rust) via napi-rs.

### Added

- **5 cognitive verbs** as async methods: `restoreIdentity()`, `loadContext()`, `save()`, `search()`, `wrapUp()`
- **TypeScript-first API** — full `index.d.ts` with named types for every request / response
- **`Memory.fromEnv()`** — reads `EREBYX_API_KEY`, `EREBYX_PASSPHRASE` (required for v0.1.1+ Argon2id-default-on tenants), `EREBYX_API_URL`, `EREBYX_INSTANCE_ID`
- **Constructor**: `new Memory(apiKey, { apiUrl, instanceId })` for explicit construction
- **Circuit breaker** — 3-failure threshold, 30s open window, half-open retry. Inherited from the Rust SDK; surfaced as `code: 'CIRCUIT_OPEN'` JS errors.
- **Typed errors** — `code: 'AUTH'`, `code: 'NETWORK'`, `code: 'CIRCUIT_OPEN'`, `code: 'SERVER'`
- **`X-Erebyx-Hint` protocol support** — every response includes `hints: string[]`. Hint values: `wrap_up_recommended`, `restore_identity_recommended`, `load_context_recommended`, `compact_imminent`. Honoring hints is optional.
- **Cold-session auto-fire transparency** — `response.autoFired: string[]` reports any `restore_identity` / `load_context` runs the substrate fired transparently on first session contact.
- **Encryption** — TLS 1.3 in transit, per-tenant AES-256-GCM at rest server-side. End-to-end client-side encryption ships in v0.1.x.
- **Pre-built native binaries** — macOS (arm64, x64), Linux (arm64, x64), Windows (x64). Source-build fallback for other platforms.

### Configuration

- `EREBYX_API_KEY` (required)
- `EREBYX_PASSPHRASE` (required for tenants registered at v0.1.1+ — Argon2id-default-on; surfaced in the dashboard recovery panel)
- `EREBYX_API_URL` (default: `https://core.erebyx.com`)
- `EREBYX_INSTANCE_ID` (default: `default`)
- `EREBYX_HINTS_DISABLED=1` — opt out of `X-Erebyx-Hint` parsing
- `EREBYX_DISABLE_AUTO_FIRE=1` — opt out of substrate-side cold-fire (rare)

### Compatibility

- **Node.js**: 18 or later
- **Rust** (build-from-source path): 1.77+
- **Substrate**: requires `erebyx-os` v0.1.1+
- **Backward compat**: hard guarantee within v0.1.x. Public API and wire protocol are stable.

### Breaking changes

None. First public release.

### Deferred to v0.1.2 / v0.2

- `evolve` — update a memory with new context
- `learn` — explicit relationship formation
- `import` — bulk import from ChatGPT / Claude / Markdown exports
- `pin` / `release` — explicit memory tier control

See the [v0.2 roadmap](https://erebyx.com/docs/roadmap) for cadence.

---

## How to upgrade

```bash
npm install @erebyx/sdk@latest
# or
pnpm add @erebyx/sdk@latest
# or
yarn add @erebyx/sdk@latest
```

Confirm: `npm ls @erebyx/sdk`

---

[0.1.1]: https://github.com/ProjectErebyx/erebyx-sdk-node/releases/tag/v0.1.1
