# Changelog

All notable changes to `@erebyx/sdk` (Node.js) are documented in this file.

This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html) and [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) format.

The substrate-side release notes live at [erebyx-os `CHANGELOG_v0_1_1.md`](https://github.com/ProjectErebyx/erebyx-os/blob/main/CHANGELOG_v0_1_1.md).

The underlying Rust crate is [`erebyx-sdk`](https://github.com/ProjectErebyx/erebyx-sdk/blob/main/CHANGELOG.md).

---

## [Unreleased]

### Changed

- **License: Apache-2.0 → MIT** per locked canon 2026-05-07 (Anthropic + Stripe + Cohere SDK convention; npm ecosystem norm). `LICENSE` content replaced with canonical MIT text. `package.json` license field updated. README + CONTRIBUTING updated. Source-of-truth: `erebyx-monorepo/docs/distribution/license-canon/README.md` §4.2.

### Removed

- `NOTICE` file removed (MIT does not require attribution file; was Apache-2.0 holdover).

### Added

- `.github/pull_request_template.md` — carve-out PR template with zero-substrate-logic checklist (load-bearing patent defense per Lock 12+28+42).
- `.github/workflows/dco-check.yml` — DCO sign-off enforcement workflow.

---

## [0.1.1] — 2026-04-27 — Genesis Arche

First public release. Native Node.js client over the EREBYX v0.1.1 cognitive surface, built on `erebyx-sdk` (Rust) via napi-rs.

### Added

- **5 cognitive verbs** as async methods: `restoreIdentity()`, `loadContext()`, `save()`, `search()`, `wrapUp()`
- **TypeScript-first API** — full `index.d.ts` with named types for every request / response
- **`Memory.fromEnv()`** — reads `EREBYX_API_KEY`, `EREBYX_API_URL`, `EREBYX_INSTANCE_ID`
- **Constructor**: `new Memory(apiKey, { apiUrl, instanceId })` for explicit construction
- **Circuit breaker** — 3-failure threshold, 30s open window, half-open retry. Inherited from the Rust SDK; surfaced as `code: 'CIRCUIT_OPEN'` JS errors.
- **Typed errors** — `code: 'AUTH'`, `code: 'NETWORK'`, `code: 'CIRCUIT_OPEN'`, `code: 'SERVER'`
- **`X-Erebyx-Hint` protocol support** — every response includes `hints: string[]`. Hint values: `wrap_up_recommended`, `restore_identity_recommended`, `load_context_recommended`, `compact_imminent`. Honoring hints is optional.
- **Cold-session auto-fire transparency** — `response.autoFired: string[]` reports any `restore_identity` / `load_context` runs the substrate fired transparently on first session contact.
- **Encryption** — TLS 1.3 in transit, per-tenant AES-256-GCM at rest server-side. End-to-end client-side encryption ships in v0.1.x.
- **Pre-built native binaries** — macOS (arm64, x64), Linux (arm64, x64), Windows (x64). Source-build fallback for other platforms.

### Configuration

- `EREBYX_API_KEY` (required)
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

- `evolve` — memory reconsolidation
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
