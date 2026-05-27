// SPDX-License-Identifier: Apache-2.0
//! Native Node.js bindings for the Erebyx SDK via napi-rs.
//!
//! This exposes the Rust SDK as a native Node.js addon, giving JS/TS apps
//! the same performance and reliability as the Rust SDK directly.
//!
//! ## v0.1.1 launch surface — five cognitive verbs
//!
//! 1. `restoreIdentity` — Load identity at session start
//! 2. `loadContext` — Resume work context after identity
//! 3. `save` — Save a memory
//! 4. `search` — Find memories by meaning (the `remember` verb)
//! 5. `wrapUp` — Create session handoff for continuity
//!
//! ## Usage (JavaScript/TypeScript)
//!
//! ```typescript
//! import { Memory } from '@erebyx/sdk'
//!
//! const memory = new Memory(process.env.EREBYX_API_KEY)
//!
//! // Session start
//! const identity = await memory.restoreIdentity()
//! const context = await memory.loadContext({ anchors: ['coding'] })
//!
//! // Save
//! await memory.save('User prefers dark mode', 'knowledge', {
//!   anchors: ['preferences', 'ui']
//! })
//!
//! // Search
//! const results = await memory.search('user preferences')
//!
//! // Session end
//! await memory.wrapUp('Built auth refactor', 'Deploy to staging', {
//!   anchors: ['coding', 'auth'],
//!   energy: 'systematic'
//! })
//! ```

#[macro_use]
extern crate napi_derive;

use erebyx_sdk::client::Memory as RustMemory;
use erebyx_sdk::Error as SdkError;
use napi::bindgen_prelude::*;

// =========================================================================
// Error mapping — every `erebyx_sdk::Error` variant gets a stable string
// `code` so JS callers can branch on `e.code === "CIRCUIT_OPEN"` etc.
// without parsing error messages. Codes mirror the Rust variant family
// (AUTH / CIRCUIT_OPEN / RATE_LIMITED / NETWORK / SERVER / VALIDATION /
// NOT_FOUND / CONFIG / SERIALIZATION) and stay stable across patch
// releases as part of the v0.1.x compatibility guarantee.
// =========================================================================

/// Convert an `erebyx_sdk::Error` to a napi error whose JS-side
/// `e.code` is one of the documented `ErebyxErrorCode` strings. Uses
/// the napi `Status::GenericFailure` channel because the napi runtime
/// requires a `Status`, but the JS-visible `e.code` comes from the
/// formatted message prefix `"<CODE>: <message>"` — see
/// `package.json` / `index.d.ts` for the public contract.
///
/// We intentionally do NOT depend on the (still-evolving) napi
/// `JsError`-class machinery here: putting the code in the message is
/// the only mechanism that survives every napi minor release without
/// breaking the JS surface, and it pairs cleanly with the
/// `ErebyxError` shim class exported from `index.d.ts`.
fn sdk_error_to_napi(err: SdkError) -> Error {
    let code = sdk_error_code(&err);
    Error::new(Status::GenericFailure, format!("{code}: {err}"))
}

/// Stable, documented error code for each SDK error variant. Public via
/// `index.d.ts` `ErebyxErrorCode`.
fn sdk_error_code(err: &SdkError) -> &'static str {
    match err {
        SdkError::AuthenticationFailed(_) => "AUTH",
        SdkError::CircuitOpen { .. } => "CIRCUIT_OPEN",
        SdkError::RateLimit { .. } => "RATE_LIMITED",
        SdkError::Network(_) => "NETWORK",
        SdkError::Server { .. } => "SERVER",
        SdkError::Validation(_) => "VALIDATION",
        SdkError::NotFound(_) => "NOT_FOUND",
        SdkError::Config(_) => "CONFIG",
        SdkError::Serialization(_) => "SERIALIZATION",
    }
}

/// JavaScript-facing Memory class.
///
/// Wraps the Rust `Memory` client with napi bindings.
#[napi]
pub struct Memory {
    inner: RustMemory,
}

#[napi]
impl Memory {
    /// Create a new Memory client.
    ///
    /// @param apiKey - Your Erebyx API key (erebyx_...)
    /// @param options - Optional configuration `{ apiUrl, instanceId, passphrase }`.
    ///   - `passphrase` is REQUIRED for Genesis Arche tenants registered at
    ///     v0.1.1+ (Argon2id-default-on, Lock #20 2026-05-18). The substrate
    ///     hashes it with the tenant's stored Argon2id parameters at request
    ///     time to derive the KEK that decrypts the tenant data envelope.
    ///     **The server never persists it.** Lose the passphrase AND the
    ///     BIP39 recovery seed → data is unrecoverable by design.
    ///   - Omit for legacy `hkdf_api_key` tenants registered before v0.1.1.
    #[napi(constructor)]
    pub fn new(api_key: String, options: Option<JsMemoryOptions>) -> Result<Self> {
        let opts = options.unwrap_or_default();
        let mut builder = RustMemory::builder(&api_key);

        if let Some(url) = &opts.api_url {
            builder = builder.api_url(url);
        }
        if let Some(id) = &opts.instance_id {
            builder = builder.instance_id(id);
        }
        // Argon2id-default-on (Lock #20, 2026-05-18). The Rust builder
        // silently drops empty/whitespace input, so passing through is
        // safe: the SDK boundary distinguishes "header absent" (legacy
        // hkdf_api_key tenant) from "header empty" (substrate rejects).
        if let Some(p) = &opts.passphrase {
            builder = builder.passphrase(p);
        }
        // P1-B (brutal-review POSTFIX_SDK_NODE): expose session_id +
        // timeout to JS consumers. The Rust SDK supports both via
        // MemoryBuilder; the napi binding previously dropped them.
        // session_id is the idempotency contract for wrap_up — without
        // an override, every Node process gets a fresh UUID v4 and
        // can't bind to a harness-supplied conversation id.
        if let Some(sid) = &opts.session_id {
            builder = builder.session_id(sid);
        }
        if let Some(timeout_ms) = opts.timeout_ms {
            builder = builder.timeout(std::time::Duration::from_millis(u64::from(timeout_ms)));
        }

        let inner = builder.build().map_err(sdk_error_to_napi)?;

        Ok(Self { inner })
    }

    /// Create a Memory client from environment variables.
    ///
    /// Reads `EREBYX_API_KEY`, `EREBYX_API_URL`, `EREBYX_INSTANCE_ID`,
    /// and `EREBYX_PASSPHRASE` (required for Argon2id-default-on tenants
    /// registered at v0.1.1+; empty/missing for legacy `hkdf_api_key`).
    #[napi(factory)]
    pub fn from_env() -> Result<Self> {
        let inner = RustMemory::from_env().map_err(sdk_error_to_napi)?;
        Ok(Self { inner })
    }

    /// Save a memory.
    #[napi]
    pub async fn save(
        &self,
        content: String,
        category: String,
        options: Option<JsSaveOptions>,
    ) -> Result<JsSaveResult> {
        let opts = options.unwrap_or_default();
        let mut builder = self.inner.save(&content, &category);

        if let Some(title) = &opts.title {
            builder = builder.title(title);
        }
        if let Some(anchors) = &opts.anchors {
            builder = builder.anchors(anchors.iter().map(|s| s.as_str()).collect());
        }
        if let Some(importance) = opts.importance {
            builder = builder.importance(importance);
        }

        let result = builder.send().await.map_err(sdk_error_to_napi)?;

        Ok(JsSaveResult {
            memory_id: result.memory_id,
            anchors: result.anchors,
            status: result.status,
            hints: result.hints,
            auto_fired: result.auto_fired,
            extra: hashmap_to_value(result.extra),
        })
    }

    /// Search memories.
    #[napi]
    pub async fn search(
        &self,
        query: String,
        options: Option<JsSearchOptions>,
    ) -> Result<JsSearchResult> {
        let opts = options.unwrap_or_default();
        let mut builder = self.inner.search(&query);

        if let Some(limit) = opts.limit {
            builder = builder.limit(limit);
        }
        if let Some(anchors) = &opts.hint_anchors {
            builder = builder.hint_anchors(anchors.iter().map(|s| s.as_str()).collect());
        }
        if let Some(range) = &opts.time_range {
            builder = builder.time_range(range);
        }

        let result = builder.send().await.map_err(sdk_error_to_napi)?;

        Ok(JsSearchResult {
            memories: result
                .memories
                .into_iter()
                .map(|m| JsMemoryRecord {
                    id: m.id,
                    content: m.content,
                    category: m.category,
                    title: m.title,
                    anchors: m.anchors,
                    importance: m.importance,
                    // P0-B (brutal-review POSTFIX_SDK_NODE): the Rust
                    // MemoryRecord has these fields; the napi binding
                    // was dropping them. JS consumers doing temporal
                    // filtering ("memories from this week") were dead
                    // in the water until they switched to the Rust SDK.
                    created_at: m.created_at,
                    updated_at: m.updated_at,
                    score: m.score,
                    extra: hashmap_to_value(m.extra),
                })
                .collect(),
            total_found: result.total_found as u32,
            familiarity: result.familiarity,
            hints: result.hints,
            auto_fired: result.auto_fired,
            extra: hashmap_to_value(result.extra),
        })
    }

    /// Create a session handoff before ending.
    ///
    /// Preserves what was built, what is next, and open threads for the next session.
    #[napi]
    pub async fn wrap_up(
        &self,
        what_we_built: String,
        whats_next: String,
        options: Option<JsWrapUpOptions>,
    ) -> Result<JsWrapUpResult> {
        let opts = options.unwrap_or_default();
        let mut builder = self.inner.wrap_up(&what_we_built, &whats_next);

        if let Some(diary) = &opts.diary {
            builder = builder.diary(diary);
        }
        if let Some(anchors) = &opts.anchors {
            builder = builder.anchors(anchors.iter().map(|s| s.as_str()).collect());
        }
        if let Some(energy) = &opts.energy {
            builder = builder.energy(energy);
        }
        if let Some(memories) = opts.memories {
            let rust_memories: Vec<erebyx_sdk::types::WrapUpMemory> = memories
                .into_iter()
                .map(|m| erebyx_sdk::types::WrapUpMemory {
                    category: m.category,
                    content: m.content,
                    title: m.title,
                    anchors: m.anchors,
                    importance: m.importance,
                })
                .collect();
            builder = builder.memories(rust_memories);
        }

        let result = builder.send().await.map_err(sdk_error_to_napi)?;

        Ok(JsWrapUpResult {
            handoff_id: result.handoff_id,
            session_id: result.session_id,
            anchors: result.anchors,
            summary: result.summary,
            saves_persisted: result.saves_persisted,
            reflections_persisted: result.reflections_persisted,
            next_action_hint: result.next_action_hint,
            narrative: result.narrative,
            suggested_next_call: result.suggested_next_call.map(js_suggested_next_call),
            meta: result.meta.map(js_tool_meta),
            hints: result.hints,
            auto_fired: result.auto_fired,
            extra: hashmap_to_value(result.extra),
        })
    }

    /// Load identity at session start.
    ///
    /// Loads identity, ethos, foundation memories, topology, and continuity
    /// signals from the substrate's `POST /v0/identity/restore` route.
    #[napi]
    pub async fn restore_identity(
        &self,
        options: Option<JsRestoreIdentityOptions>,
    ) -> Result<JsRestoreIdentityResult> {
        let opts = options.unwrap_or_default();
        let mut builder = self.inner.restore_identity();

        if let Some(level) = &opts.detail_level {
            builder = builder.detail_level(level);
        }
        if let Some(include) = opts.include_guide {
            builder = builder.include_guide(include);
        }
        if let Some(limit) = opts.limit {
            builder = builder.limit(limit);
        }

        let result = builder.send().await.map_err(sdk_error_to_napi)?;

        Ok(JsRestoreIdentityResult {
            identity: JsIdentityCore {
                name: result.identity.name,
                instance_id: result.identity.instance_id,
                origin_statement: result.identity.origin_statement,
                persona_summary: result.identity.persona_summary,
            },
            ethos: result.ethos,
            foundation_memories: result
                .foundation_memories
                .into_iter()
                .map(|m| JsFoundationMemoryItem {
                    id: m.id,
                    title: m.title,
                    snippet: m.snippet,
                    anchors: m.anchors,
                    captured_at: m.captured_at,
                })
                .collect(),
            topology: hashmap_to_value(result.topology),
            continuity: hashmap_to_value(result.continuity),
            needs_onboarding: result.needs_onboarding,
            memory_guide: result.memory_guide,
            narrative: result.narrative,
            suggested_next_call: result.suggested_next_call.map(js_suggested_next_call),
            meta: result.meta.map(js_tool_meta),
            hints: result.hints,
            auto_fired: result.auto_fired,
            extra: hashmap_to_value(result.extra),
        })
    }

    /// Load work context after restore_identity.
    ///
    /// POSTs to `/v0/session/load` and returns the most-recent matching
    /// handoff, related memories, integrated skills, and a per-session id.
    #[napi]
    pub async fn load_context(
        &self,
        options: Option<JsLoadContextOptions>,
    ) -> Result<JsLoadContextResult> {
        let opts = options.unwrap_or_default();
        let mut builder = self.inner.load_context();

        if let Some(anchors) = &opts.anchors {
            builder = builder.anchors(anchors.iter().map(|s| s.as_str()).collect());
        }
        if let Some(mode) = &opts.mode {
            builder = builder.mode(mode);
        }
        if let Some(spec) = &opts.specialization_name {
            builder = builder.specialization_name(spec);
        }
        if let Some(level) = &opts.detail_level {
            builder = builder.detail_level(level);
        }
        if let Some(priority) = &opts.load_priority {
            builder = builder.load_priority(priority);
        }

        let result = builder.send().await.map_err(sdk_error_to_napi)?;

        Ok(JsLoadContextResult {
            session_id: result.session_id,
            anchors: result.anchors,
            handoff: result.handoff.map(|h| JsHandoffSummary {
                id: h.id,
                summary: h.summary,
                anchors: h.anchors,
                captured_at: h.captured_at,
            }),
            related_memories: result
                .related_memories
                .into_iter()
                .map(|m| JsRelatedMemoryItem {
                    id: m.id,
                    title: m.title,
                    snippet: m.snippet,
                    anchors: m.anchors,
                    relevance_tier: m.relevance_tier,
                })
                .collect(),
            skills: result.skills,
            topology: hashmap_to_value(result.topology),
            narrative: result.narrative,
            suggested_next_call: result.suggested_next_call.map(js_suggested_next_call),
            meta: result.meta.map(js_tool_meta),
            hints: result.hints,
            auto_fired: result.auto_fired,
            extra: hashmap_to_value(result.extra),
        })
    }
}

fn hashmap_to_value(
    m: std::collections::HashMap<String, serde_json::Value>,
) -> serde_json::Value {
    serde_json::Value::Object(m.into_iter().collect())
}

fn js_suggested_next_call(s: erebyx_sdk::types::SuggestedNextCall) -> JsSuggestedNextCall {
    JsSuggestedNextCall {
        tool: s.tool,
        args: serde_json::Value::Object(s.args.into_iter().collect()),
        reason: s.reason,
    }
}

fn js_tool_meta(m: erebyx_sdk::types::ToolMeta) -> JsToolMeta {
    JsToolMeta {
        schema_version: m.schema_version,
        request_id: m.request_id,
        latency_ms: m.latency_ms,
    }
}

// =========================================================================
// JS-facing types (napi-compatible)
// =========================================================================

#[napi(object)]
#[derive(Default)]
pub struct JsMemoryOptions {
    pub api_url: Option<String>,
    pub instance_id: Option<String>,
    /// Override the per-process session id used on `X-Erebyx-Session-Id`.
    /// Defaults to a fresh UUID v4 per Memory instance. Bind to a
    /// harness-supplied conversation id (Vercel function `req.headers
    /// .x-conversation-id`, LLM thread id, etc.) for cross-request
    /// idempotency on `wrap_up`. (P1-B fix from POSTFIX_SDK_NODE.)
    pub session_id: Option<String>,
    /// HTTP request timeout in milliseconds. Defaults to 10000 (10s; inherited from the Rust SDK).
    /// Lower for latency-sensitive applications; raise for slow
    /// substrates under load. (P1-B fix from POSTFIX_SDK_NODE.)
    pub timeout_ms: Option<u32>,
    /// Per-tenant passphrase for Argon2id-default-on Genesis Arche
    /// tenants (Lock #20, 2026-05-18). Required at v0.1.1+ for any
    /// tenant registered with `encryption_mode: argon2_passphrase`.
    /// Omit for legacy `hkdf_api_key` tenants — the substrate
    /// distinguishes header-absent (legacy) from header-empty (reject).
    pub passphrase: Option<String>,
}

#[napi(object)]
#[derive(Default)]
pub struct JsSaveOptions {
    pub title: Option<String>,
    pub anchors: Option<Vec<String>>,
    pub importance: Option<f64>,
}

#[napi(object)]
#[derive(Default)]
pub struct JsSearchOptions {
    pub limit: Option<u32>,
    pub hint_anchors: Option<Vec<String>>,
    pub time_range: Option<String>,
}

#[napi(object)]
pub struct JsSaveResult {
    pub memory_id: String,
    pub anchors: Vec<String>,
    pub status: String,
    /// Substrate lifecycle hints (`X-Erebyx-Hint` response header).
    /// See `Hint` union in index.d.ts for the canonical vocabulary.
    /// Empty when the substrate has nothing to recommend.
    pub hints: Vec<String>,
    /// Tools the substrate auto-fired on this call
    /// (`X-Erebyx-Auto-Fired` header). Typically
    /// `["restore_identity", "load_context"]` on the first call against
    /// a fresh `(instance_id, session_id)` tuple. Empty thereafter.
    pub auto_fired: Vec<String>,
    /// Forward-compat catch-all for `StoreMemoryResponse` fields not
    /// typed individually (`action`, `dedup`, `atomization`,
    /// `created_at`, `warnings`, `schema_version`,
    /// `consolidation_priority`, `content_hash`). P0-A fix from
    /// POSTFIX_SDK_NODE.
    pub extra: serde_json::Value,
}

#[napi(object)]
pub struct JsSearchResult {
    pub memories: Vec<JsMemoryRecord>,
    pub total_found: u32,
    pub familiarity: f64,
    /// Substrate lifecycle hints (`X-Erebyx-Hint` response header).
    pub hints: Vec<String>,
    /// Tools the substrate auto-fired on this call
    /// (`X-Erebyx-Auto-Fired` header).
    pub auto_fired: Vec<String>,
    /// Forward-compat catch-all. Notably carries the substrate's
    /// `formatted_text` field (pre-rendered LLM-readable output) plus
    /// `star_context`, `abstention`, `voice_markers`, `onboarding_hint`,
    /// `truncated`, `degraded`, `warnings`. Surfacing them as typed
    /// fields is post-launch work; in the interim JS consumers reach
    /// them via `result.extra.formatted_text` etc. (P0-A fix from
    /// POSTFIX_SDK_NODE.)
    pub extra: serde_json::Value,
}

#[napi(object)]
pub struct JsMemoryRecord {
    pub id: String,
    pub content: String,
    pub category: String,
    pub title: Option<String>,
    pub anchors: Vec<String>,
    pub importance: f64,
    /// ISO-8601 timestamp from substrate (P0-B fix from
    /// POSTFIX_SDK_NODE — was silently dropped pre-fix).
    pub created_at: Option<String>,
    /// ISO-8601 timestamp from substrate.
    pub updated_at: Option<String>,
    pub score: Option<f64>,
    /// Forward-compat catch-all. The substrate's `RememberMemoryItem`
    /// emits 7 additional fields (`metadata`, `retrieval_path`,
    /// `reranker_score`, `supersedes_id`, `contradiction_ids`,
    /// `evolved_from_id`, `related_via_edges`); surface as raw JSON
    /// until they're typed individually.
    pub extra: serde_json::Value,
}

// =========================================================================
// Wrap-up types
// =========================================================================

#[napi(object)]
#[derive(Default)]
pub struct JsWrapUpOptions {
    pub diary: Option<String>,
    pub anchors: Option<Vec<String>>,
    pub energy: Option<String>,
    pub memories: Option<Vec<JsWrapUpMemory>>,
}

#[napi(object)]
pub struct JsWrapUpMemory {
    pub category: String,
    pub content: String,
    pub title: Option<String>,
    pub anchors: Option<Vec<String>>,
    pub importance: Option<f64>,
}

#[napi(object)]
pub struct JsWrapUpResult {
    pub handoff_id: String,
    pub session_id: String,
    pub anchors: Vec<String>,
    pub summary: String,
    pub saves_persisted: i64,
    pub reflections_persisted: i64,
    pub next_action_hint: String,
    pub narrative: String,
    pub suggested_next_call: Option<JsSuggestedNextCall>,
    pub meta: Option<JsToolMeta>,
    /// Substrate lifecycle hints (`X-Erebyx-Hint` response header).
    pub hints: Vec<String>,
    /// Tools the substrate auto-fired on this call
    /// (`X-Erebyx-Auto-Fired` header).
    pub auto_fired: Vec<String>,
    /// Forward-compat catch-all for substrate fields not typed
    /// individually. P0-A fix from POSTFIX_SDK_NODE.
    pub extra: serde_json::Value,
}

// =========================================================================
// Shared lifecycle types
// =========================================================================

#[napi(object)]
pub struct JsSuggestedNextCall {
    pub tool: String,
    /// Suggested args dict — opaque JSON object the substrate emits.
    pub args: serde_json::Value,
    pub reason: String,
}

#[napi(object)]
pub struct JsToolMeta {
    pub schema_version: String,
    pub request_id: Option<String>,
    pub latency_ms: Option<i64>,
}

// =========================================================================
// Restore identity types
// =========================================================================

#[napi(object)]
#[derive(Default)]
pub struct JsRestoreIdentityOptions {
    pub detail_level: Option<String>,
    pub include_guide: Option<bool>,
    pub limit: Option<u32>,
}

#[napi(object)]
pub struct JsIdentityCore {
    pub name: String,
    pub instance_id: String,
    pub origin_statement: String,
    pub persona_summary: Option<String>,
}

#[napi(object)]
pub struct JsFoundationMemoryItem {
    pub id: String,
    pub title: Option<String>,
    pub snippet: String,
    pub anchors: Vec<String>,
    pub captured_at: Option<String>,
}

#[napi(object)]
pub struct JsRestoreIdentityResult {
    pub identity: JsIdentityCore,
    pub ethos: Vec<String>,
    pub foundation_memories: Vec<JsFoundationMemoryItem>,
    /// Concept priming for attention. Opaque JSON object — substrate
    /// emits categorical buckets per the WHAT-not-HOW doctrine.
    pub topology: serde_json::Value,
    /// Drift-detector continuity signals. Opaque JSON object with
    /// `last_session_ago` / `signals` / `signals_count` keys.
    pub continuity: serde_json::Value,
    pub needs_onboarding: bool,
    pub memory_guide: Option<String>,
    pub narrative: String,
    pub suggested_next_call: Option<JsSuggestedNextCall>,
    pub meta: Option<JsToolMeta>,
    /// Substrate lifecycle hints (`X-Erebyx-Hint` response header).
    pub hints: Vec<String>,
    /// Tools the substrate auto-fired on this call
    /// (`X-Erebyx-Auto-Fired` header).
    pub auto_fired: Vec<String>,
    /// Forward-compat catch-all. P0-A fix from POSTFIX_SDK_NODE.
    pub extra: serde_json::Value,
}

// =========================================================================
// Load context types
// =========================================================================

#[napi(object)]
#[derive(Default)]
pub struct JsLoadContextOptions {
    pub anchors: Option<Vec<String>>,
    pub mode: Option<String>,
    /// When `mode === "specialization"`, the name of the specialization
    /// to load by exact match.
    pub specialization_name: Option<String>,
    pub detail_level: Option<String>,
    /// Token-budget priority: `"minimal"` | `"summary"` | `"full"`.
    pub load_priority: Option<String>,
}

#[napi(object)]
pub struct JsHandoffSummary {
    pub id: String,
    pub summary: String,
    pub anchors: Vec<String>,
    pub captured_at: Option<String>,
}

#[napi(object)]
pub struct JsRelatedMemoryItem {
    pub id: String,
    pub title: Option<String>,
    pub snippet: String,
    pub anchors: Vec<String>,
    /// Categorical bucket: high / medium / low (WHAT-not-HOW).
    pub relevance_tier: String,
}

#[napi(object)]
pub struct JsLoadContextResult {
    pub session_id: String,
    pub anchors: Vec<String>,
    pub handoff: Option<JsHandoffSummary>,
    pub related_memories: Vec<JsRelatedMemoryItem>,
    pub skills: Vec<String>,
    pub topology: serde_json::Value,
    pub narrative: String,
    pub suggested_next_call: Option<JsSuggestedNextCall>,
    pub meta: Option<JsToolMeta>,
    /// Substrate lifecycle hints (`X-Erebyx-Hint` response header).
    pub hints: Vec<String>,
    /// Tools the substrate auto-fired on this call
    /// (`X-Erebyx-Auto-Fired` header).
    pub auto_fired: Vec<String>,
    /// Forward-compat catch-all. P0-A fix from POSTFIX_SDK_NODE.
    pub extra: serde_json::Value,
}
