# Gameplay component runtime

`ComponentRuntime` verifies the SHA-256 of portable component bytes and evaluates
the exported `manifest` before admitting the module's API and schema identity.
Compiled Components are cached by content hash. Every `evaluate` instantiates a
fresh Store and component, then validates the returned change plan. The caller
owns state/config persistence, preview isolation and the atomic gameplay commit.

Only `vera:mod-sdk/host@0.1.0` is linked. No WASI filesystem, clock, networking or
random interfaces are available. Stores enforce fuel, one bounded linear memory,
bounded tables/instances and a host-call budget. Input/output and state/config
messages have separate bounds. Host adapters must themselves be trusted and
bounded; fuel limits guest instructions, not arbitrary Rust callbacks or code
compilation. Content admission and host operation validation remain mandatory.

Defaults allocate base fuel of 50 million to each one-time physical Configure
pass or `configure_boundary` call and 5 million to each manifest/resource/gameplay call. Every call also
receives 256 fuel per serialized input byte, since decoding opaque JSON byte
arrays consumes guest instructions before the requested logic runs. This total
is shared by decoding and logic; it is not a separate strict 5-million gameplay
cap. `max_input_fuel` caps this extra allowance at 50 million even for the largest
valid input, so default total budgets never exceed 55 million for ordinary calls
or 100 million for Configure. Configuration also counts the frozen registry
payload supplied through the host bulk reader within that same extra allowance.
The host memoizes and measures the exact payload once per immutable read view;
gameplay calls receive no allowance for host configuration metadata. The 8 MiB
wire limit remains separately enforced.
Setting `fuel_per_input_byte=0` restores a fixed budget. Memory, host-call and
wire bounds remain independent; infinite loops still exhaust fuel. Traps report
input size and consumed fuel/host calls; Rules admission adds its physical pass
index and frozen configuration size. This allowance does not reduce repeated
configuration decoding cost or guarantee arbitrary large-Mod throughput.

The host's `config-read` uses the engine's existing typed INI reader semantics.
Reader names are `int`, `bool`, `double-bits`, `nullable-int`, `nullable-bool`,
`nullable-double-bits`, `string:N`, `name:N`, `nullable-name:N`, `list:N`,
`int-list` and `color-rgb`.
The development host also exposes the existing `ares-int`, `ares-bool`,
`ares-nullable-bool`, `ares-double-bits` readers and
`ares-ref:$animations@<scope>` for registered references. These delegate to the
same native readers used by the engine. Reference scopes preserve the visible
registry prefix at each reached physical Rules reader; the final registry must
not retroactively resolve an earlier unknown reference.
`verse-token` takes a literal JSON string as its current argument and returns
the existing armor numeric parser's binary64/independent-flag record. It does
not perform another INI read or implement a second numeric algorithm.
`rules-registry-view` returns the typed catalogs and actual read-point prefixes
for a physical Rules pass. Its bulk output uses the 8 MiB message cap; ordinary
INI reader outputs and all reader current-value inputs retain their 64 KiB cap.
`comma-list:N` delegates the native comma-only `IniSection::read_list`, retaining
the current value on missing/empty reads. The earlier `list:N` reader retains
its comma/space/tab tokenization; callers select the semantics their native
reader requires rather than sharing a misleading common list interpretation.
Current values and results are JSON; doubles use raw `u64` bits. `N` is the native
read capacity. `registry` returns physical section keys in authored order, so the
component reads their values through the same typed reader. The engine adapter
retains ownership of defaults, case rules and Rules pass selection.

`manifest` returns `ModuleManifest` JSON. The declared configure/attach/damage/tick
hooks take `Request` JSON and return `Plan` JSON. Configuration, entity state and
presentation are separate values; no Shield-specific state is defined in this
runtime or in the shared API. `damage-kernel` delegates to the existing engine
numeric receiver and returns one JSON `i32`; the runtime contains no copy of that
algorithm.

Optional `weapon_resource`, `type_resource`, `match_rules` and the paired
`effect_resource` / `effects` queries use that same evaluate channel. Admission
rejects an unpaired effect capability. Ordered effect plans are validated before
the host commits them to its existing instance and animation owners; the runtime
does not keep another gameplay state store. API version 1 does not imply that an
older host understands newly declared optional hooks.
The optional `armor_resource` query uses the same stateless channel for catalog,
object/Terrain and warhead projections; the engine validates their indices and
lengths together before binding them to existing rules owners.
The optional `production_resource` channel similarly publishes immutable groups
and type policies. `configure_boundary` may replace configuration but cannot
write actor state or presentation; the existing process Rules owner supplies
the real reset/read chronology and exact-component opaque continuation.

`emp_resource` publishes immutable rules, type and warhead declarations. It has
no entity state or operations. EMP counters and phase-sensitive status transitions
remain in the existing host owner. `ares-nullable-ref` preserves explicit unset /
clear / named resource states through the same native reference reader and prefix.
