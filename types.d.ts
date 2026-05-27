// SPDX-License-Identifier: Apache-2.0
//
// Friendly TypeScript re-exports for `@erebyx/sdk`.
//
// The native addon (built by napi-rs) emits `index.d.ts` automatically
// from the Rust `#[napi(object)]` definitions, using `Js`-prefixed
// names (`JsSaveResult`, `JsSearchResult`, etc.). This file re-exports
// those types under shorter, idiomatic-TypeScript names and adds the
// canonical `Hint` string union for hint-handling code that wants to
// branch with full type-checking.

export {
  JsMemoryOptions as MemoryOptions,
  JsSaveOptions as SaveOptions,
  JsSaveResult as SaveResponse,
  JsSearchOptions as SearchOptions,
  JsSearchResult as SearchResponse,
  JsMemoryRecord as MemoryRecord,
  JsWrapUpOptions as WrapUpOptions,
  JsWrapUpMemory as WrapUpMemory,
  JsWrapUpResult as WrapUpResponse,
  JsRestoreIdentityOptions as RestoreIdentityOptions,
  JsRestoreIdentityResult as RestoreIdentityResponse,
  JsLoadContextOptions as LoadContextOptions,
  JsLoadContextResult as LoadContextResponse,
  // Bucket B shared lifecycle shapes.
  JsSuggestedNextCall as SuggestedNextCall,
  JsToolMeta as ToolMeta,
  // restore_identity nested shapes.
  JsIdentityCore as IdentityCore,
  JsFoundationMemoryItem as FoundationMemoryItem,
  // load_context nested shapes.
  JsHandoffSummary as HandoffSummary,
  JsRelatedMemoryItem as RelatedMemoryItem,
} from './index';

/**
 * Substrate lifecycle hint vocabulary. Carried as comma-separated
 * values on the `X-Erebyx-Hint` response header and surfaced as the
 * `hints: string[]` field on every response.
 *
 * Honoring hints is optional — the SDK never acts on them
 * automatically. The substrate signals what would help; your
 * application decides cadence.
 */
export type Hint =
  | 'wrap_up_recommended'
  | 'restore_identity_recommended'
  | 'load_context_recommended'
  | 'compact_imminent';

/**
 * Tools the substrate auto-fired on a given call. Carried on the
 * `X-Erebyx-Auto-Fired` response header. Typically populated only on
 * the first call against a fresh `(instanceId, sessionId)` tuple.
 */
export type AutoFiredTool = 'restore_identity' | 'load_context';
