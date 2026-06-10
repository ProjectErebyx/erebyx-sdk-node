# Contributing to @erebyx/sdk

Thanks for your interest. The Node SDK is a thin napi-rs binding over the Rust [`erebyx-sdk`](https://github.com/ProjectErebyx/erebyx-sdk) crate — pull requests for ergonomic JS/TS APIs, type definitions, and platform compatibility are welcome.

By contributing, you agree your contributions are licensed under MIT OR Apache-2.0; see [LICENSE-MIT](LICENSE-MIT) and [LICENSE-APACHE-2.0](LICENSE-APACHE-2.0).

---

## Sign-off (DCO)

Every commit must be signed off using the [Developer Certificate of Origin](https://developercertificate.org/):

```bash
git commit -s -m "your message"
```

The DCO bot will block PRs without sign-off.

---

## Local dev setup

```bash
git clone https://github.com/ProjectErebyx/erebyx-sdk-node.git
cd erebyx-sdk-node
npm install
npm run build           # napi build --platform --release
npm test
```

Required toolchain:
- Rust 1.77 or later (`rustup install stable`)
- Node.js 18 or later
- `@napi-rs/cli` (installed via `npm install`)

---

## Test commands

```bash
cargo test                                                    # Rust-side unit tests
cargo fmt --check                                             # Rust formatting
cargo clippy --all-targets -- -D warnings                     # Rust lint (zero warnings)
npm test                                                      # Node-side smoke tests
```

All four must pass before a PR is reviewed.

---

## Native binary builds

Cross-platform binaries are built via `napi build --platform --release`. In v0.1.1 only the **macOS arm64 (Apple Silicon)** prebuilt binary ships; every other platform builds from source on install (requires Rust 1.77+). Per-platform prebuilds for the full target matrix land in v0.1.2 once the cross-compile CI is wired:
- macOS — arm64 (prebuilt in v0.1.1), x64 (from v0.1.2)
- Linux — arm64, x64 (glibc) (from v0.1.2)
- Windows — x64 (from v0.1.2)

To add a new target triple, update `napi.triples.additional` in `package.json` and confirm CI cross-compilation succeeds.

---

## Commit conventions

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```
feat(node): add streaming search via async iterator
fix(napi): correct error code mapping for circuit-open
docs(readme): add TypeScript usage examples
chore(deps): bump @napi-rs/cli to 2.18.1
```

Types we use: `feat`, `fix`, `docs`, `chore`, `refactor`, `test`, `perf`.

---

## Pull request template

When you open a PR, include:

1. **What changed** — one paragraph
2. **Why** — the user-visible problem this fixes
3. **How verified** — `cargo test` + `npm test` output
4. **Risk surface** — backward-compat assessment for the public TypeScript API
5. **Native build impact** — does this require a release of new pre-built binaries?

Public-API changes (`src/lib.rs`, `index.d.ts`) require an extra reviewer.

---

## Scope

The Node SDK exposes the v0.1.1 cognitive verbs: `restoreIdentity`, `loadContext`, `save`, `search`, `wrapUp`. Substrate behavior lives in the closed-source `erebyx-os` engine.

In scope:
- TypeScript ergonomics
- Error code surface
- Native build / packaging
- Platform compatibility
- `X-Erebyx-Hint` parsing surfaced through `response.hints`

Out of scope:
- Underlying Rust SDK behavior — file PRs against [`erebyx-sdk`](https://github.com/ProjectErebyx/erebyx-sdk)
- Substrate behavior — file PRs against `erebyx-os`

---

## Bug reports

Open a [GitHub Issue](https://github.com/ProjectErebyx/erebyx-sdk-node/issues). Include:

- `node --version`
- `npm --version`
- OS + architecture (`uname -a` on Unix)
- `@erebyx/sdk` version
- Minimum reproducer (a `.ts` file we can run)
- Full error output

---

## Security disclosures

Don't open public issues for security findings. See [SECURITY.md](SECURITY.md).

---

**Built by EREBYX, LLC** — `https://erebyx.com`
