# Security Policy

## Reporting a vulnerability

If you believe you've found a security vulnerability in `@erebyx/sdk`, report it privately so we can fix it before it harms anyone.

- **Email:** `legal@erebyx.com`
- **Alternate:** `privacy@erebyx.com` (data-handling concerns)
- **PGP key:** Available on request to `legal@erebyx.com`
- **Response SLA:** acknowledgment within 24 hours; status update within 72 hours

Please include:
- Clear description of the vulnerability + impact assessment
- Steps to reproduce
- Affected component and `@erebyx/sdk` version
- Whether you intend to disclose publicly, and on what timeline

We follow **coordinated disclosure**. We do not currently run a paid bug-bounty program but will publicly credit responsible disclosures with your permission.

**Do NOT open a public GitHub issue for a security report.**

---

## Supported versions

| Version | Supported            | Notes |
|---------|----------------------|---|
| 0.1.x   | :white_check_mark:   | Active development; security fixes within 72h |
| < 0.1   | :x:                  | Pre-release, unsupported |

When v0.2 ships, v0.1.x receives security fixes for 90 days.

---

## Scope

In scope:
- The `@erebyx/sdk` npm package
- Native binary builds (the napi-rs bindings)
- Underlying Rust SDK (separately tracked in [`erebyx-sdk`](https://github.com/ProjectErebyx/erebyx-sdk) crate)
- TypeScript type definitions

Out of scope:
- Issues in the substrate engine `erebyx-os` (closed-source — same email, separate triage)
- Third-party dependencies (`@napi-rs/cli`, etc.) — please report upstream
- Local-only attacks requiring physical machine access

---

## Known limitations + roadmap

| Area | Current limitation | Target fix |
|---|---|---|
| Client-side encryption | Memory is encrypted in transit (TLS 1.3) and at rest using XChaCha20-Poly1305 envelope encryption (AES-256-GCM legacy supported on existing rows) with per-tenant Key Encryption Keys wrapped under a server-held master KEK. At v0.1.1 EREBYX operationally holds the master KEK; per-user zero-knowledge encryption (passphrase-derived keys, EREBYX cannot decrypt) ships in v0.2. The browser extension already implements client-side AES-256-GCM today. | v0.2 |
| API-key rotation | Manual rotation via `app.erebyx.com/keys`; SDK does not yet auto-rotate | v0.2 |
| Native binary signing | Pre-built binaries are checksummed but not yet signed | v0.1.x |

---

**Built by EREBYX, LLC** — `https://erebyx.com`
