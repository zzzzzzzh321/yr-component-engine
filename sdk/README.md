# VERA Mod SDK — external gameplay reference

One work loads one optional portable gameplay component. The same engine runs
retail YR without a declaration or one reference component whose shields, authored
overload, Ares weapon ammo costs, reload amounts and low-ammo weapon selection are
selected through INI. This prototype migrates the existing supported shield chain;
it does not migrate every Ares/Phobos feature or claim full upstream parity.

The SDK infrastructure lives in `sdk/`; the first author-editable plugin lives
separately in `plugins/vera20k-ares/` at the repository root and publishes
`vera20k-ares.wasm`. Its current features include the retained Phobos-compatible
shield chain and authored overload example; the Ares filename does not claim
complete Ares compatibility.

Plugin authors can use an engine-free development kit containing only the API,
WIT, packager and reference plugin, with its own workspace and lockfile. The
[standalone build guide](dev-kit/README.zh-CN.md) uses standard Cargo and does
not require the engine sources, executable, Python build runner or game assets.
The commands below describe development inside the full engine checkout.

## Boundaries

- `api`: versioned typed messages, bounded opaque state/config, generic meters,
  relationship flags and the shared native countdown timer. `api/src/resource.rs`
  owns the shared immutable weapon/type resource policies.
- `wit/gameplay.wit`: portable `vera:mod-sdk@0.1.0` component interface.
- `runtime`: Wasmtime, with only immutable INI and shared numeric-kernel imports.
  No filesystem, clock, random, network or WASI services are linked.
- `package`: offline `vera-mod-pack` tool; wraps a core Wasm module and emits
  the component plus its content-hashed declaration, without executing it.
- [`plugins/vera20k-ares`](../plugins/vera20k-ares/README.md): author-editable Rust reference. One artifact contains the
  features; `[VeraFeatures] Shield/Overload/AresAmmo/AresReload/AresNoAmmoWeapon`
  selects them through INI. Defense mechanisms live in `features/defense`; weapon
  policies live in `features/weapons`. All use the same `support/host.rs` facade.
  All default to no; Overload requires Shield. Selection is frozen per match.
- `src/mod_sdk.rs`: production asset admission, entity/combat/tick adapter,
  bounded change-plan validation and frozen saved-program admission.

The host owns entity lifecycle and commits validated plans. The module owns its
shield configuration, points, timers and formulas. Generic meter/tint projections
are presentation data; authoritative component bytes plus module/config identity
enter both state hashes and the actual multiplayer FRAMEINFO checksum. The
original two Scenario RNG draws in checksum collection are preserved.

The experimental engine SDK v1 requires four base hooks (an unused hook returns
an unchanged plan). An empty returned entity state unsubscribes that entity; a
stateless behavior can retain a one-byte marker to remain subscribed.

Base hooks are `configure`, `attach`, `damage` and `tick`. An ordered Configure callback
runs for each reached Rules layer. One final merged callback carries complete
object and warhead registries for bindings. During gameplay configuration is
immutable. `damage` can replace receiver input and its own state, but cannot write
health or another entity. Reserved operations are rejected in this prototype.

The reference component also declares two optional immutable queries:
`weapon_resource` returns a weapon's cost policy; `type_resource` returns reload
amounts and low-ammo weapon selection for an object type and actual category.
Admission queries them once and save restoration rebuilds them from frozen
configuration. Hot paths use the resulting read-only projections. These features
do not own a second ammo count or reload clock, and need no additional per-entity
component state. Both queries use the existing `evaluate` message channel and
add no Host service or WIT import.

The component identifies as `vera.reference-gameplay` 0.4.0, config_version=4,
state_version=1. This internal manifest identity is retained independently of the
publication filename and Cargo package name. Older components receive only the queries they declare; changing
INI cannot add a feature absent from their original binary. Existing saves still
require their exact corresponding component bytes.

See the [Ares resource guide](../plugins/vera20k-ares/AMMO.zh-CN.md) and
[resource-cycle.ini](../plugins/vera20k-ares/examples/resource-cycle.ini) for the combined
configuration, defaults, category restrictions and documented compatibility
boundary for unsupported `NoAmmoWeapon` indices.

Every call creates a fresh guest instance with fuel/memory/call limits. Preview
and commit use the same guest leaf; preview publishes nothing. A fault rejects
the plan and stops frame admission, rather than silently removing the module.
No live module replacement or saved schema migration is provided.

## Packaging

Build guests for `wasm32-unknown-unknown`, preserving the embedded WIT metadata
emitted by `wit-bindgen`. Then wrap the core module using the workspace tool:

```sh
python -m tools.cargo_run --cache-gib 512 --incremental-gib 512 --min-free-gib 1 -- build -p vera20k-ares --target wasm32-unknown-unknown --release
python -m tools.cargo_run -- build --locked -p vera-mod-pack --profile fast
python -m tools.cargo_run --resolve vera-mod-pack --profile fast
# Invoke the executable path reported above:
"<reported-packager>" "<reported-target>/wasm32-unknown-unknown/release/vera20k_ares.wasm" "<existing-output-parent>/reference-new" --program vera20k-ares.wasm
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

The explicit `--program vera20k-ares.wasm` selects this plugin's publication
filename; Cargo emits the raw core module as `vera20k_ares.wasm`. For other
plugins the generic packager still defaults to `gameplay.wasm` when `--program`
is omitted. Use the emitted `vera20k-ares.wasm` and `vera-mod.json` as ordinary
work assets.
The generated declaration is:

```json
{"api_version":1,"program":"vera20k-ares.wasm","sha256":"<lowercase SHA-256 of vera20k-ares.wasm>"}
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
