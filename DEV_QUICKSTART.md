# Developer Quickstart — @erebyx/sdk

Five minutes from zero to your first wrap-up in Node.js.

---

## 0. Prerequisites

- An EREBYX API key — get one at [app.erebyx.com/keys](https://app.erebyx.com/keys)
- Node.js 18 or later
- TypeScript optional but recommended (full `index.d.ts` ships in the package)

---

## 1. Install (10 seconds)

```bash
npm install @erebyx/sdk
# or
pnpm add @erebyx/sdk
# or
yarn add @erebyx/sdk
```

---

## 2. Configure (10 seconds)

```bash
export EREBYX_API_KEY="<YOUR_API_KEY>"

# Required for tenants registered at v0.1.1+ (Argon2id-default-on).
# Pull this from your dashboard's recovery panel at registration time.
export EREBYX_PASSPHRASE="<YOUR_PASSPHRASE>"
```

Optional overrides (rarely needed):

```bash
export EREBYX_API_URL="https://core.erebyx.com"
export EREBYX_INSTANCE_ID="my-app"
```

---

## 3. First save (10 seconds)

```typescript
import { Memory } from '@erebyx/sdk'

const memory = Memory.fromEnv()

const response = await memory.save('Substrate URL is core.erebyx.com', 'knowledge', {
  anchors: ['setup'],
  importance: 0.6,
})

console.log('Saved', response.memoryId)
```

```bash
node --experimental-strip-types index.ts
# or with tsx
tsx index.ts
```

---

## 4. First search (10 seconds)

```typescript
const results = await memory.search('substrate URL', { limit: 5 })

for (const m of results.memories) {
  console.log(m.id, m.content)
}
```

---

## 5. First wrap-up (10 seconds)

```typescript
await memory.wrapUp('Got the SDK working', 'Wire it into the agent loop', {
  anchors: ['setup', 'node'],
  energy: 'systematic',
})
```

That handoff is now retrievable next session via `memory.loadContext()`.

**You're done.** That's the whole loop.

---

## X-Erebyx-Hint — lifecycle signals

Every SDK response carries lifecycle hints from the substrate:

```typescript
const response = await memory.save('...', 'insight')

for (const hint of response.hints) {
  switch (hint) {
    case 'wrap_up_recommended':
      // Substrate sees a natural consolidation boundary.
      // Call wrapUp at the next clean break.
      break
    case 'restore_identity_recommended':
      // Voice drift detected (v0.2). Re-anchor identity.
      break
    case 'load_context_recommended':
      // Retrieval scores trending low. Refresh working memory.
      break
    case 'compact_imminent':
      // Sustained save volume; consolidate before context fills.
      break
  }
}
```

### When to honor each hint

| Hint | Recommended response |
|---|---|
| `wrap_up_recommended` | Call `memory.wrapUp(...)` at the next clean task boundary. |
| `restore_identity_recommended` | Call `memory.restoreIdentity()` to re-anchor. |
| `load_context_recommended` | Call `memory.loadContext()` to reload working memory. |
| `compact_imminent` | Call `memory.wrapUp(...)` before the harness compacts. |

Honoring hints is **optional** — the SDK never acts on them automatically. The substrate signals what would help; your application decides cadence.

### Disabling hints

```bash
export EREBYX_HINTS_DISABLED=1
```

Disables hint emission server-side. Use only for debugging.

---

## Cold-session auto-fire

The substrate runs `restoreIdentity` + `loadContext` automatically on the first call against a fresh `(instance_id, session_id)` tuple. The SDK surfaces this transparently:

```typescript
const response = await memory.save('first save', 'knowledge')
console.log('Auto-fired:', response.autoFired)
// ['restore_identity', 'load_context'] on first call; [] thereafter
```

### Opting out

```bash
export EREBYX_DISABLE_AUTO_FIRE=1
```

Use this if your service handles its own session warm-up. Most users want auto-fire **on**.

---

## Common errors and fix paths

| Error | Cause | Fix |
|---|---|---|
| `code: 'AUTH'` | API key missing or wrong | Check `process.env.EREBYX_API_KEY` is set and starts with `erebyx_` |
| `code: 'AUTH'` with `passphrase_required` server message | `EREBYX_PASSPHRASE` not set for an Argon2id-default-on tenant | Set `process.env.EREBYX_PASSPHRASE` to the value from the dashboard recovery panel |
| `code: 'NETWORK'` | Connectivity / DNS / TLS | Verify `https://core.erebyx.com` is reachable from the host |
| `code: 'CIRCUIT_OPEN'` | 3 consecutive failures within 30s | SDK is backing off; retries automatically. Check substrate health. |
| `code: 'SERVER'`, `status: 404` | Substrate version mismatch | Verify substrate is `v0.1.1+` |
| `accepted: false, reason: 'below_durability_threshold'` | Save filtered as low-signal | Lower the importance gate or omit; default `min_durability=0.4` |
| Hints never appear | Save volume below substrate threshold | Hints emit at ~12 saves/session by default. Normal early-session behavior. |
| `npm install` fails on native build | Unsupported platform | Pre-built binaries cover macOS / Linux / Windows on common arches. For others, ensure Rust 1.77+ is installed; the install script falls back to source build. |

---

## Per-harness integration examples

The Node SDK is one of several integration paths. Harness-specific integration
patterns (LangGraph, OpenAI Responses API, Anthropic Agent SDK, AutoGen, CrewAI,
Letta, raw HTTP, and any future harness) are at [erebyx.com/core](https://erebyx.com/core),
and common questions are answered at [erebyx.com/faq](https://erebyx.com/faq).

Every harness honors the same `X-Erebyx-Hint` protocol described above.

---

## Patterns for production services

### Long-lived `Memory` instance

`Memory` instances are safe to share across requests. Internally each holds a connection-pooled HTTP client.

```typescript
// app.ts
import { Memory } from '@erebyx/sdk'
export const memory = Memory.fromEnv()
```

```typescript
// route.ts
import { memory } from './app'
await memory.save(...)
```

### Graceful degradation

```typescript
import { Memory } from '@erebyx/sdk'

const memory = Memory.fromEnv()

try {
  await memory.save('...', 'insight')
} catch (e: any) {
  if (e.code === 'CIRCUIT_OPEN') {
    // Don't block the user; log + continue.
    logger.warn('erebyx circuit open; skipping save')
  } else {
    throw e
  }
}
```

### Type imports

```typescript
import type { SaveOptions, SaveResponse, SearchResponse, Hint } from '@erebyx/sdk/types'
```

---

## Next steps

- **Read the substrate overview**: [erebyx.com/core](https://erebyx.com/core) · common questions at [erebyx.com/faq](https://erebyx.com/faq)
- **Rust instead?**: [`erebyx-sdk`](https://github.com/ProjectErebyx/erebyx-sdk) — the underlying crate
- **CLI shortcut?**: [`erebyx-cli`](https://github.com/ProjectErebyx/erebyx-cli) — same surface, shell-callable

---

**Built by EREBYX, LLC** — `https://erebyx.com`
