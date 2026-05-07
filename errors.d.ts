// SPDX-License-Identifier: Apache-2.0
//
// TypeScript declarations for `@erebyx/sdk/errors` — see `errors.js`
// for the runtime contract.

/**
 * Stable error codes mirrored from the Rust SDK's `Error` variant
 * family. Guaranteed stable across v0.1.x patch releases.
 */
export type ErebyxErrorCode =
  | 'AUTH'
  | 'CIRCUIT_OPEN'
  | 'RATE_LIMITED'
  | 'NETWORK'
  | 'SERVER'
  | 'VALIDATION'
  | 'NOT_FOUND'
  | 'CONFIG'
  | 'SERIALIZATION';

/**
 * Typed error class thrown by SDK calls when wrapped via
 * `withErebyxErrors`. Branch on `e.code` rather than parsing
 * `e.message`.
 *
 * @example
 *   try { await memory.save('x', 'knowledge'); }
 *   catch (e) {
 *     if (e instanceof ErebyxError && e.code === 'CIRCUIT_OPEN') {
 *       // back off; substrate is degraded
 *     } else { throw e; }
 *   }
 */
export class ErebyxError extends Error {
  readonly name: 'ErebyxError';
  readonly code: ErebyxErrorCode;
  readonly cause?: Error;
  constructor(code: ErebyxErrorCode, message: string, cause?: Error);
}

/**
 * Wrap a Memory instance so every thrown native error is lifted to a
 * typed `ErebyxError`. The returned Proxy is API-identical to the
 * original instance.
 */
export function withErebyxErrors<T extends object>(memory: T): T;

/**
 * Lift a single error in place. Useful in custom catch sites that
 * don't go through `withErebyxErrors`.
 */
export function liftErebyxError(err: unknown): unknown;
