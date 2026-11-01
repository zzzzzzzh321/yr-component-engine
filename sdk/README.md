# VERA Mod SDK — first shield prototype

One work loads one optional portable gameplay component. The same engine runs
retail YR without a declaration or one reference component whose shields, authored
overload and Ares weapon ammo are selected through INI. This prototype migrates the existing supported shield chain;
it does not migrate every Ares/Phobos feature or claim full upstream parity.

## Boundaries

- `api`: versioned typed messages, bounded opaque state/config, generic meters,
  relationship flags and the shared native countdown timer.
- `wit/gameplay.wit`: portable `vera:mod-sdk@0.1.0` component interface.
- `runtime`: Wasmtime, with only immutable INI and shared numeric-kernel imports.
  No filesystem, clock, random, network or WASI services are linked.
- `package`: offline `vera-mod-pack` tool; wraps a core Wasm module and emits
  the component plus its content-hashed declaration, without executing it.
- `plugins/shield`: author-editable Rust reference. One artifact contains the
  features; `[VeraFeatures] Shield/Overload/AresAmmo` selects them through INI.
  All default to no; Overload requires Shield. Selection is frozen per match.
- `src/mod_sdk.rs`: production asset admission, entity/combat/tick adapter,
  bounded change-plan validation and frozen saved-program admission.

The host owns entity lifecycle and commits validated plans. The module owns its
shield configuration, points, timers and formulas. Generic meter/tint projections
are presentation data; authoritative component bytes plus module/config identity
enter both state hashes and the actual multiplayer FRAMEINFO checksum. The
original two Scenario RNG draws in checksum collection are preserved.

The experimental engine SDK v1 requires all four hooks (an unused hook returns
an unchanged plan). An empty returned entity state unsubscribes that entity; a
stateless behavior can retain a one-byte marker to remain subscribed.

Hooks are `configure`, `attach`, `damage` and `tick`. An ordered Configure callback
runs for each reached Rules layer. One final merged callback carries complete
object and warhead registries for bindings. During gameplay configuration is
immutable. `damage` can replace receiver input and its own state, but cannot write
health or another entity. Reserved operations are rejected in this prototype.

Every call creates a fresh guest instance with fuel/memory/call limits. Preview
and commit use the same guest leaf; preview publishes nothing. A fault rejects
the plan and stops frame admission, rather than silently removing the module.
No live module replacement or saved schema migration is provided.

## Packaging

Build guests for `wasm32-unknown-unknown`, preserving the embedded WIT metadata
emitted by `wit-bindgen`. Then wrap the core module using the workspace tool:

```sh
python -m tools.cargo_run --cache-gib 512 --incremental-gib 512 --min-free-gib 1 -- build -p vera-reference-shield --target wasm32-unknown-unknown --release
python -m tools.cargo_run -- build --locked -p vera-mod-pack --profile fast
python -m tools.cargo_run --resolve vera-mod-pack --profile fast
# Invoke the executable path reported above:
"<reported-packager>" "<reported-target>/wasm32-unknown-unknown/release/vera_reference_shield.wasm" "<existing-output-parent>/reference-new"
```

`<reported-target>` is the checkout-owned target directory printed by the build
runner; it may be outside the checkout. Use that reported path rather than
assuming `target/`. Install the Rust target with
`rustup target add wasm32-unknown-unknown` if needed. The output directory must
not exist and its parent must exist. Inputs and encoded components are bounded
to 16 MiB. The tool validates the encoded component before creating output and
never overwrites an existing directory or artifact. It neither executes the
guest nor supplies WASI adapters. Engine admission still checks the SDK
manifest, content hash and allowed imports.

Use the emitted `gameplay.wasm` and `vera-mod.json` as ordinary work assets.
The generated declaration is:

```json
{"api_version":1,"program":"gameplay.wasm","sha256":"<lowercase SHA-256 of gameplay.wasm>"}
```

These are ordinary work assets and can use the existing community memory-package
path. Keep configuration in the work's Rules/map layers. No declaration means
vanilla. Raw VXL/SHP/INI assets are independent of this SDK; this is an extension
mechanism and does not provide asset-extraction protection.

Do not substitute a `wasm32-wasip2` artifact: Rust's standard library can add
WASI interface imports even if guest logic does not call them, while this host
intentionally links only the SDK interface. The unknown-unknown build plus
explicit component wrapping keeps the guest independent of WASI.

The current gameplay component bytes and its frozen parsed configuration are
saved with the world. Restore checks the current work's component identity and
reuses the saved configuration. Updating a component requires a new match;
existing saves require the corresponding original version.

## Scope

SDK v1 admits authored maps. Works declaring external gameplay are explicitly
rejected at generated-map admission; generator hooks are not part of this version.

This is a native desktop prototype for Windows/Linux/macOS. Windows is the local
validated platform; other platform builds and transport-connected multiplayer
are separate validation. The architecture does not certify 20,000-unit throughput:
fresh instantiation and whole-config decoding per entity are intentionally simple
for the first isolation/behavior experiment. A batched interface and measured
performance work should precede scale claims. See the reference component README
for the inherited animation/transfer/warhead-operation residuals.
