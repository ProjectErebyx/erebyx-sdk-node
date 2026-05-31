#!/usr/bin/env node
// SPDX-License-Identifier: Apache-2.0
//
// Build-from-source fallback for @erebyx/sdk.
//
// Runs as the package `install` lifecycle script. It is a NO-OP whenever a
// usable native binding is already available — i.e. when:
//   (a) a prebuilt `@erebyx/sdk-<triple>` optionalDependency installed for this
//       platform (the happy path on supported triples), OR
//   (b) a local `erebyx-sdk.<triple>.node` already sits next to index.js (repeat
//       installs / monorepo dev).
//
// Only when NEITHER is present does it compile the bundled Rust source from the
// tarball with `cargo build --release` and copy the resulting cdylib to the
// `erebyx-sdk.<triple>.node` filename that the napi-generated index.js loads.
//
// Design constraints (first-principles):
//   * Customer `npm install @erebyx/sdk` does NOT install devDependencies, so we
//     CANNOT rely on `@napi-rs/cli` (a devDependency) at install time. We invoke
//     `cargo` directly — the only tool the README promises is required (Rust
//     1.77+). index.js / index.d.ts are shipped in the tarball already, so only
//     the .node binary needs producing here.
//   * Cargo.toml ships a pure registry dep `erebyx-sdk = "0.1.1"`, so the build
//     resolves the dependency from crates.io with no sibling checkout required.
//   * Failure is loud and actionable, never silent: if Rust is missing we print
//     the rustup one-liner and exit non-zero so the install fails visibly rather
//     than yielding a package that throws an opaque "binding not found" at runtime.

const { existsSync, copyFileSync, readdirSync } = require('fs')
const { join } = require('path')
const { spawnSync, execSync } = require('child_process')

const root = join(__dirname, '..')

// Map Node's process.platform/arch (+ libc) to the napi triple filename that
// index.js probes for, mirroring package.json `napi.triples`.
function nodeTriple() {
  const { platform, arch } = process
  if (platform === 'darwin') {
    if (arch === 'arm64') return 'darwin-arm64'
    if (arch === 'x64') return 'darwin-x64'
  }
  if (platform === 'linux') {
    // glibc only for v0.1.x prebuilds; source build still works on musl.
    if (arch === 'x64') return 'linux-x64-gnu'
    if (arch === 'arm64') return 'linux-arm64-gnu'
  }
  if (platform === 'win32' && arch === 'x64') return 'win32-x64-msvc'
  return `${platform}-${arch}`
}

const triple = nodeTriple()
const localBinding = join(root, `erebyx-sdk.${triple}.node`)

// (a)/(b): a usable binding already resolves -> nothing to do.
function bindingAlreadyAvailable() {
  if (existsSync(localBinding)) return true
  try {
    // The platform optionalDependency, if it installed, resolves here.
    require.resolve(`@erebyx/sdk-${triple}`)
    return true
  } catch {
    return false
  }
}

if (bindingAlreadyAvailable()) {
  process.exit(0)
}

// No prebuild for this platform -> build from the bundled Rust source.
function haveCargo() {
  const r = spawnSync('cargo', ['--version'], { stdio: 'ignore' })
  return r.status === 0
}

if (!haveCargo()) {
  console.error(
    [
      '',
      '@erebyx/sdk: no prebuilt native binary for this platform (' + triple + ')',
      'and Rust/Cargo was not found to build from source.',
      '',
      'Install Rust 1.77+ (one line):  https://rustup.rs',
      '  curl --proto =https --tlsv1.2 -sSf https://sh.rustup.rs | sh',
      '',
      'then reinstall:  npm install @erebyx/sdk',
      '',
    ].join('\n'),
  )
  process.exit(1)
}

console.log(`@erebyx/sdk: no prebuild for ${triple}; building native binary from source (cargo build --release)...`)

const build = spawnSync('cargo', ['build', '--release'], {
  cwd: root,
  stdio: 'inherit',
})
if (build.status !== 0) {
  console.error('@erebyx/sdk: cargo build failed; see output above.')
  process.exit(build.status || 1)
}

// Locate the produced cdylib in target/release and copy it to the expected
// `erebyx-sdk.<triple>.node` path. cdylib basename derives from the crate name
// (erebyx-sdk-node -> liberebyx_sdk_node) with the platform-specific extension.
const releaseDir = join(root, 'target', 'release')
const ext = process.platform === 'win32' ? '.dll' : process.platform === 'darwin' ? '.dylib' : '.so'
const produced = readdirSync(releaseDir).find(
  (f) => f.endsWith(ext) && f.includes('erebyx') && f.includes('sdk') && f.includes('node'),
)
if (!produced) {
  console.error(`@erebyx/sdk: build succeeded but no cdylib (*${ext}) found in ${releaseDir}.`)
  process.exit(1)
}

copyFileSync(join(releaseDir, produced), localBinding)
console.log(`@erebyx/sdk: built ${triple} native binary -> ${localBinding}`)

// Sanity: confirm the freshly built binding actually loads.
try {
  execSync(`node -e "require('./index.js')"`, { cwd: root, stdio: 'ignore' })
  console.log('@erebyx/sdk: native binary loads OK.')
} catch {
  console.error('@erebyx/sdk: built binary did not load via index.js; please file an issue.')
  process.exit(1)
}
