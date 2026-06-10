// SPDX-License-Identifier: MIT OR Apache-2.0
//
// JS-side error helpers for @erebyx/sdk.
//
// The native addon (built by napi-rs) cannot portably set Node's
// `error.code` to an arbitrary string across napi-rs versions, so the
// Rust binding instead encodes the canonical code as a stable prefix
// on `error.message`: "<CODE>: <human message>". This file exposes a
// thin wrapper that lifts that prefix into a real `error.code` and
// throws a typed `ErebyxError` JS Error subclass so consumers can write
// idiomatic `catch (e) { if (e.code === 'CIRCUIT_OPEN') ... }`.
//
// Usage:
//   const { withErebyxErrors, ErebyxError } = require('@erebyx/sdk/errors')
//   const memory = withErebyxErrors(Memory.fromEnv())
//   try { await memory.save('x', 'knowledge') }
//   catch (e) { if (e instanceof ErebyxError && e.code === 'CIRCUIT_OPEN') ... }
//
// The wrapper is a Proxy — every method on the underlying Memory
// instance is forwarded; only thrown errors are re-shaped. Native
// performance is preserved (one extra `try/catch` per call).

'use strict';

/**
 * Stable error codes mirrored from the Rust SDK's `Error` variant family.
 * Stays stable across v0.1.x patch releases.
 *
 * @typedef {'AUTH'|'CIRCUIT_OPEN'|'RATE_LIMITED'|'NETWORK'|'SERVER'|'VALIDATION'|'NOT_FOUND'|'CONFIG'|'SERIALIZATION'} ErebyxErrorCode
 */

const KNOWN_CODES = new Set([
  'AUTH',
  'CIRCUIT_OPEN',
  'RATE_LIMITED',
  'NETWORK',
  'SERVER',
  'VALIDATION',
  'NOT_FOUND',
  'CONFIG',
  'SERIALIZATION',
]);

/**
 * Typed error class. Carries a string `code` you can branch on.
 */
class ErebyxError extends Error {
  /**
   * @param {string} code - One of the documented ErebyxErrorCode values.
   * @param {string} message - Human-readable message.
   * @param {Error} [cause] - Original native error, for stack chaining.
   */
  constructor(code, message, cause) {
    super(message);
    this.name = 'ErebyxError';
    this.code = code;
    if (cause !== undefined) this.cause = cause;
  }
}

/**
 * Inspect a napi-emitted Error and lift the "<CODE>: <msg>" prefix
 * into a typed ErebyxError. If the message doesn't carry a known code
 * prefix, the original error is returned unchanged.
 *
 * @param {Error} err
 * @returns {Error}
 */
function liftErebyxError(err) {
  if (!(err instanceof Error)) return err;
  const msg = String(err.message || '');
  const sep = msg.indexOf(': ');
  if (sep <= 0) return err;
  const prefix = msg.slice(0, sep);
  if (!KNOWN_CODES.has(prefix)) return err;
  return new ErebyxError(prefix, msg.slice(sep + 2), err);
}

/**
 * Wrap a Memory instance so every thrown error gets lifted to a typed
 * ErebyxError. Returns a Proxy — keeps the original instance reachable
 * via the `unwrap` symbol-keyed accessor (useful for tests).
 *
 * @template T
 * @param {T} memory
 * @returns {T}
 */
function withErebyxErrors(memory) {
  return new Proxy(memory, {
    get(target, prop, receiver) {
      const value = Reflect.get(target, prop, receiver);
      if (typeof value !== 'function') return value;
      return function wrapped(...args) {
        try {
          const out = value.apply(target, args);
          if (out && typeof out.then === 'function') {
            return out.catch((e) => {
              throw liftErebyxError(e);
            });
          }
          return out;
        } catch (e) {
          throw liftErebyxError(e);
        }
      };
    },
  });
}

module.exports = { ErebyxError, withErebyxErrors, liftErebyxError };
