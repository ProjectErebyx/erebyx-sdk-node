# @erebyx/sdk

> Native Node.js SDK for the EREBYX memory substrate. Built on the Rust SDK via napi-rs. Persistent AI memory across every AI you use.

[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)
[![Version](https://img.shields.io/badge/version-0.1.1-green.svg)](CHANGELOG.md)
[![npm](https://img.shields.io/badge/npm-%40erebyx%2Fsdk-red.svg)](https://www.npmjs.com/package/@erebyx/sdk)

---

## Install in 5 lines

```bash
npm install @erebyx/sdk
export EREBYX_API_KEY="erebyx_..."
```

```typescript
import { Memory } from '@erebyx/sdk'
const memory = Memory.fromEnv()
const res = await memory.save('Anchor retrieval improves recall by 40%', 'insight')
console.log('saved', res.memoryId, 'hints:', res.hints)
```

That's the whole loop: install -> save. Memory is encrypted in transit (TLS 1.3) and at rest (per-tenant AES-256-GCM, server-side). End-to-end client-side encryption (true zero-knowledge — server NEVER sees plaintext) is on the v0.2+ roadmap.

---

## What it is

`@erebyx/sdk` is the Node.js / TypeScript client for the EREBYX memory substrate. It wraps the Rust [`erebyx-sdk`](https://github.com/ProjectErebyx/erebyx-sdk) crate via napi-rs — same circuit breaker, same wire protocol, same lifecycle hint surface, native performance.

It exposes the v0.1.1 cognitive surface as five async methods:

| Method | Purpose |
|---|---|
| `restoreIdentity()` | Wake up — load identity at session start |
| `loadContext({ anchors })` | Resume — load handoff + recent work |
| `save(content, category, opts)` | Store a memory worth keeping |
| `search(query, opts)` | Find what you know by meaning (the `remember` verb) |
| `wrapUp(whatWeBuilt, whatsNext, opts)` | Create a session handoff at the end |

Substrate behavior (atomization, retrieval, dream cycle, encryption) lives behind the API — you never need to think about it.

---

## Quickstart

```typescript
import { Memory } from '@erebyx/sdk'

const memory = new Memory(process.env.EREBYX_API_KEY!, {
  apiUrl: 'https://core.erebyx.com',
  instanceId: 'my-app',
})

// Save
await memory.save('User prefers dark mode', 'knowledge', {
  anchors: ['preferences', 'ui'],
  importance: 0.7,
})

// Search
const results = await memory.search('user preferences', { limit: 5 })
for (const m of results.memories) {
  console.log(m.id, m.content)
}

// Session handoff
await memory.wrapUp('Wired the SDK', 'Add streaming search', {
  anchors: ['sdk', 'node'],
  energy: 'systematic',
})
```

Or use `Memory.fromEnv()` to read everything from env vars:

```typescript
const memory = Memory.fromEnv()
```

---

## Configuration

```bash
# Required
export EREBYX_API_KEY="erebyx_..."

# Required for tenants registered at v0.1.1+ (Argon2id-default-on).
# Find this in your dashboard recovery panel; treat it like a master
# encryption key — losing it AND the BIP39 recovery seed is
# unrecoverable by design (zero-knowledge property).
export EREBYX_PASSPHRASE="<YOUR_PASSPHRASE>"

# Optional (defaults shown)
export EREBYX_API_URL="https://core.erebyx.com"
export EREBYX_INSTANCE_ID="default"
```

Get your API key at [app.erebyx.com/keys](https://app.erebyx.com/keys).
The dashboard surfaces the matching `EREBYX_PASSPHRASE` value at
registration; both are required for new tenants.

---

## X-Erebyx-Hint — lifecycle signals for free

Every SDK call surfaces the substrate's lifecycle hints (`X-Erebyx-Hint` response header) and any tools the substrate auto-fired (`X-Erebyx-Auto-Fired`). Both are typed string arrays on every response:

```typescript
import { Hint } from '@erebyx/sdk/types'

const response = await memory.save('...', 'insight')

for (const hint of response.hints as Hint[]) {
  switch (hint) {
    case 'wrap_up_recommended':       /* call wrapUp at next natural break */ break
    case 'restore_identity_recommended': /* identity drift; re-anchor */ break
    case 'load_context_recommended':  /* call loadContext to refresh */ break
    case 'compact_imminent':          /* consolidate before context fills */ break
  }
}

// First call against a fresh (instanceId, sessionId) tuple typically
// reports ['restore_identity', 'load_context']; empty thereafter.
console.log('auto-fired this call:', response.autoFired)
```

Hint values:
- `wrap_up_recommended` — substrate sees a natural consolidation boundary
- `restore_identity_recommended` — voice drift detected
- `load_context_recommended` — retrieval scores trending low
- `compact_imminent` — sustained save volume; consolidate before context fills

Honoring hints is optional. Disable globally with `EREBYX_HINTS_DISABLED=1`. Full hint protocol at [DEV_QUICKSTART.md](DEV_QUICKSTART.md#x-erebyx-hint--lifecycle-signals).

---

## Reliability

Built-in circuit breaker — after 3 consecutive failures, the SDK refuses calls for 30s, then attempts a half-open retry. Memory failures degrade gracefully and never crash your app.

Wrap your `Memory` instance with `withErebyxErrors` to get typed errors with stable string `code` values:

```typescript
import { Memory } from '@erebyx/sdk'
import { withErebyxErrors, ErebyxError } from '@erebyx/sdk/errors'

const memory = withErebyxErrors(Memory.fromEnv())

try {
  await memory.save('...', 'insight')
} catch (e) {
  if (e instanceof ErebyxError && e.code === 'CIRCUIT_OPEN') {
    // Substrate degraded; backed off. Don't block the user.
    console.warn('erebyx circuit open; skipping save')
  } else {
    throw e
  }
}
```

Stable `code` values: `AUTH`, `CIRCUIT_OPEN`, `RATE_LIMITED`, `NETWORK`, `SERVER`, `VALIDATION`, `NOT_FOUND`, `CONFIG`, `SERIALIZATION`. See `@erebyx/sdk/errors` for the full type definition.

---

## Native binaries

Pre-built native binaries are published per platform on every npm release:
- macOS — arm64, x64
- Linux — arm64, x64 (glibc)
- Windows — x64

If your platform is unsupported, `@erebyx/sdk` falls back to building from source on install (requires Rust 1.77+).

---

## How to upgrade

Track releases in [CHANGELOG.md](CHANGELOG.md). Backward compatibility is a hard guarantee within v0.1.x — every release lists explicit breaking changes (none expected before v0.2).

```bash
npm install @erebyx/sdk@latest
```

---

## Build from source

```bash
git clone https://github.com/ProjectErebyx/erebyx-sdk-node.git
cd erebyx-sdk-node
npm install
npm run build
```

Requires Rust 1.77+ and Node.js 18+. The native build is driven by `napi build --platform --release`.

---

## See also

- [`erebyx-cli`](https://github.com/ProjectErebyx/erebyx-cli) — Native CLI for MCP integration
- [`erebyx-sdk`](https://github.com/ProjectErebyx/erebyx-sdk) — Rust SDK (this package wraps it)
- [`erebyx-extension`](https://github.com/ProjectErebyx/erebyx-extension) — browser extension for ChatGPT + Claude.ai
- [Substrate docs](https://erebyx.com/docs)
- [Per-harness integration examples](https://github.com/ProjectErebyx/erebyx-cookbook) — 11 harnesses, copy-paste integration
- [LangGraph example](https://github.com/ProjectErebyx/erebyx-cookbook/blob/main/langgraph/README.md) — closest match for Node-based agent graphs
- [OpenAI Responses API example](https://github.com/ProjectErebyx/erebyx-cookbook/blob/main/openai-responses-api/README.md) — closest match for raw Node API loops

---

## Contributing

Pull requests welcome. DCO sign-off required (`git commit -s`). See [CONTRIBUTING.md](CONTRIBUTING.md).

## Security

Vulnerability reports → `legal@erebyx.com`. See [SECURITY.md](SECURITY.md).

## License

Apache-2.0. See [LICENSE](LICENSE) and [NOTICE](NOTICE).

---

**Built by EREBYX, LLC** — `https://erebyx.com`
