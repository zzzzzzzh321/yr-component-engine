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

The host's `config-read` uses the engine's existing typed INI reader semantics.
Reader names are `int`, `bool`, `double-bits`, `nullable-int`, `nullable-bool`,
`nullable-double-bits`, `string:N`, `name:N`, `nullable-name:N`, `list:N`,
`int-list` and `color-rgb`.
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
