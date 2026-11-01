# External shield reference component

This component implements the currently ported VERA20K shield chain outside
the game engine. The compatibility build and the authored overload build use
the same `vera:mod-sdk@0.1.0` interface and run under the same engine binary.

The code was moved from VERA20K's Rust implementation, retaining its source
references. It does not include Ares/Phobos C++ code or Mental Omega resources.
The original VERA20K contribution history and GPL licensing continue to apply.

## What runs in the component

- Each reached Rules pass reads the `ShieldTypes` catalog through the host's
  existing INI readers. The final resolved pass binds object and warhead names
  and reads global values. The engine does not interpret shield tags.
- `attach` constructs component-owned points and timers.
- `damage` returns a new component state and receiver damage. It calls the
  host's shared warhead numeric kernel rather than copying that algorithm.
- `tick` owns recovery, powered/temporal pause behavior and cleanup.
- Each plan also returns generic meter/tint presentation data.

Persistent state and configuration are opaque, bounded host-managed bytes.
Every floating-point configuration value is encoded as its IEEE-754 bits.
No module-global mutable value is authoritative. The signed countdown timer
implementation is shared with the engine through `vera-mod-api::timer`.

The port retains the existing scope: idle/hit/break/respawn animations and
weapons, warhead attach/remove/break operations, type-conversion transfer and
the `SelfHealing.EnabledBy` consumer were not completed in the old owner and
are not newly claimed here. The retained original PIPS/PIPBRD presentation is
used; configurable alternative Pips.Background resources are not newly added.

## Compatibility and overload builds

From the engine checkout, after installing the `wasm32-unknown-unknown` Rust
target (`rustup target add wasm32-unknown-unknown`):

```sh
python -m tools.cargo_run --cache-gib 512 --incremental-gib 512 --min-free-gib 1 -- build -p vera-reference-shield --target wasm32-unknown-unknown --release
python -m tools.cargo_run --cache-gib 512 --incremental-gib 512 --min-free-gib 1 -- run -p vera-mod-pack -- "<reported-target>/wasm32-unknown-unknown/release/vera_reference_shield.wasm" "<existing-output-parent>/compatible-new"
```

The runner prints the checkout-owned target directory; substitute that path for
`<reported-target>` rather than assuming `target/`. The packager's output
directory must be new and its parent must already exist. Its `gameplay.wasm`
and `vera-mod.json` form the compatible publication artifact. Preserve this
directory before building and packaging the second variant:

```sh
python -m tools.cargo_run --cache-gib 512 --incremental-gib 512 --min-free-gib 1 -- build -p vera-reference-shield --target wasm32-unknown-unknown --release --features overload
python -m tools.cargo_run --cache-gib 512 --incremental-gib 512 --min-free-gib 1 -- run -p vera-mod-pack -- "<reported-target>/wasm32-unknown-unknown/release/vera_reference_shield.wasm" "<existing-output-parent>/overload-new"
```

The packager uses the embedded WIT metadata to encode and validate a standalone
component; it never executes the guest or adds WASI adapters. A direct
`wasm32-wasip2` build is unsuitable because Rust std can introduce implicit WASI
imports. The runtime links only the SDK imports; no WASI filesystem, clock, entropy,
stdio or network import is available. Native unit tests exercise component
decisions, while the engine acceptance harness must execute the actual built
WASM artifacts and production INI/damage owners.

```sh
python -m tools.cargo_run --cache-gib 512 --incremental-gib 512 --min-free-gib 1 -- test -p vera-reference-shield --lib
python -m tools.cargo_run --cache-gib 512 --incremental-gib 512 --min-free-gib 1 -- test -p vera-reference-shield --lib --features overload
```

## Authored overload behavior

Only the `overload` guest build adds this behavior. It increases heat by the
shield points actually consumed by a hit. Reaching HeatLimit suspends shield
absorption and recovery for Duration simulation frames. Cooling points are
removed on each eligible object AI visit. Temporal, chrono and limbo admission
retain their existing pause/skip semantics. All four recovery countdowns,
including the healing and respawn combat delays, remain paused during overload.
Hits may restart these countdowns, but they stay frozen until an eligible AI
visit ends overload; the existing power rules then apply. For example, overload
from frame 10 through 85 followed by a frame-11 hit resumes a 90-frame healing
countdown at 85, healing at 175. A twelve-frame combat delay moves it to 187.
This behavior is a VERA SDK example, not an Ares/Phobos compatibility
claim, and no engine host operation is specialized for it.

```ini
[VeraOverload]
HeatLimit=60
Duration=75
Cooling=1
```

The compatible build ignores that section. `examples/shield.ini` uses only
original-game unit names and fictional shield data; it contains no artwork.
