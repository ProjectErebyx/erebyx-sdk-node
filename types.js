// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Runtime stub for the `@erebyx/sdk/types` subpath.
//
// All type aliases in types.d.ts are TYPE-ONLY re-exports from the
// napi-generated `./index.d.ts`. No runtime values to ship — but
// `package.json:exports['./types']` declared a `default` entry to
// satisfy Node's runtime resolver (P1-D fix from POSTFIX_SDK_NODE).
// This empty module fills that slot. Consumers should prefer
// `import type { ... } from '@erebyx/sdk/types'` (TS1484-safe with
// `verbatimModuleSyntax`); the runtime resolution path here is the
// fallback for accidental value-imports.

module.exports = {};
