# VERA Mod SDK — external gameplay reference

Version 0.39 / config28 adds optional `SuperWeaponDeclaration.eva`, retaining five resolved fixed-EVAMD references alongside the existing InsufficientFunds field. The guest owns first initialization and six physical reads; the host reuses the existing immutable binder, event producers and Vox queue. Nuclear and the supported custom handlers consume Ready/Activated/Detected; the sidebar consumes Impatient/SelectTarget. Successful common launch cancels its configured Ready after presentation. Historical nonempty configurations keep domain absence. A paired host is required; other stock redirected handlers and complete Ares remain open.

Version 0.38/config27 adds optional exact `SuperWeaponDeclaration.influence` and
`SuperWeaponResourcePolicy.techno_influence` declarations. The scoped
`super-techno-list` reader resolves the private physical common Type prefix;
the existing atomic SW binder validates category/native ordinals and publishes
nullable ranges through ObjectType. The existing prepared cursor/AI collector
uses actual Type membership, owner/alliance/lifecycle and Building4555D0 power
observations. Unset ranges use raw Type.Sight; explicit zero/negative disable a
site. No new WIT hook, live guest state or per-frame call is added. Historical
nonempty config keeps domain absence until a real reset. Use a paired new host;
old full saves retain their original engine/component pair. Handler overrides,
all target modes and complete Ares remain open.

Version 0.37 adds optional `SuperWeaponDeclaration.common_targeting`: a frozen declaration of resolved targeting mode/preference, flags, masks and binary64 range bits. The paired host projects it into the existing read-only targeting metadata and ordinary AI/cursor/activation consumers. It adds no WIT import, hook ordinal or per-frame call. Absence preserves the historical component configuration and extra-preflight route; shared host predicate fixes still apply. Retain the original engine/component pair when exact old behavior is required. Native common event execution reaches Activate without repeating cursor admission; a failed Generic null-warhead activation consumes readiness but does not settle money/history. Full selector, influence and presentation compatibility remains open.

The current host also retains each native Techno class/ordinal through per-slot rules, construction, paid production, tracking, power and same-build restoration, including equal stored IDs. Native first-match type lookup remains separate from retained pointer identity. This is a shared host prerequisite: the 0.37 guest, config26, API and WIT are unchanged. Other name-addressed resources and the retained sidebar-link model remain separate work; the following 0.38 influence declarations use these same native identities. New full saves use the new host layout; retain original engine/component pairs for older full saves. See the production regressions in [typed_identity_tests.rs](../src/sim/production/typed_identity_tests.rs).

Version 0.36 / config25 adds the pure `MasterTypeLists` declaration and external Nuclear launch/payload references. Actual Weapon and SuperWeapon identities survive direct and silo launch, NukeMaker, flight and same-build restoration. Common `SW.Warhead` follows each physical body with existing-reference lookup; an intermediate custom Type cannot suppress that read. Full Nuclear/Ares compatibility remains open; Light, EVA and shared range semantics remain separate chains.

Version 0.35 / config24 adds optional `MatchRulesPolicy.shrapnel_weapon_assignment`. Fresh configuration and real reset boundaries select it; an older nonempty frozen cohort omits it. `ShrapnelWeapon` retains the actual Weapon allocation, which the installed child keeps through Fire and detonation for Range and RadLevel. Its bare constructor travel budget stays INT_MAX, without a Weapon ProjectileRange copy. This is an admission-time declaration requiring the paired host, with no per-frame Wasm call. Validation covers real-component firing, fragment flight, save/restore and receivers; complete Ares compatibility remains open.

## Development status

The retained delivery is `plugin-dev/dist/sdk-feature-lab-0.7.0`. Current source
is the **0.39.0 development candidate**, config version **28**, state version **1**;
the 0.7.0 descriptions below document the retained package, not a new delivery.
API v1 and `vera:mod-sdk@0.1.0` stay unchanged. New optional messages still require
a matching host; an older host sharing API v1 is not automatically compatible.

Version 0.34 retains the actual native `AirburstWeapon` allocation through saved
rules, resource admission and the retail/Ares child factories. Child fields and
travel budget come from that instance using the existing frozen declarations;
no new SDK callback or configuration schema is added. The Ares child still has
no installed Weapon pointer. Shrapnel and other type/factory identities retain
their separate limits.

Version 0.33 adds `weapon_instance_resource`, keyed by `WeaponTypeRef.ordinal`
in the actual native Weapon array. Distinct allocations retain separate records,
physical INI histories and frozen Ammo/ApplyDamage/ProjectileRange declarations
even when their stored24-byte names match. `weapon-registry-view` and
`weapon-instance-effective:<scalar>` expose that same source; names describe an
instance and never replace its identity. The combined optional resource bundle
uses the existing policy types. Its outer `Some` owns both projections, including
explicitly empty policies; `None` permits the old strict hooks only for unique
names. Admission binds each instance once, and firing uses the immutable cache.
Bullet payloads keep the actual Weapon handle through flight and restore. New
full saves have a different layout; retain original engine/component pairs for
old full saves. Projectile/Warhead and specialized factory pointer projections
still have separate limits; this does not complete Ares compatibility.

Version 0.32 adds optional `ballistic_scatter` bounds to `projectile_resource`.
The component's independent Projectile domain reads `BallisticScatter.Min/Max`
using `projectile-effective:ares-nullable-leptons`. Only successful native
Leptons conversion sets presence, including zero and masked signed32 results;
missing/invalid reads retain the previous pair. The existing FireAt scatter
resolves absent bounds from the current global value separately for plain and
flak arms. Its Scenario RNG, distance/angle math and Bullet lifetime remain with
the existing owners. Bullet::Fire's extra Inviso/FlakScatter placement retains
its global bounds. Historical omission preserves old behavior; no new hook or
per-frame guest call is added. Full Ares compatibility remains incomplete.

Version 0.31 appends the optional `projectile_resource` hook and a stateless
per-Projectile fan policy. The independent `features/projectiles/airburst` domain
reads `AirburstSpread`, `RetargetAccuracy`, `Splits`, `RetargetSelf` and nullable
`AroundTarget` from `$projectile-config` / `projectile-effective:` physical
bodies. `AresReplacement` selects the shared host fan; live identities, target
array, Scenario RNG and child flight stay with existing owners. Ares children
use weapon Damage/Speed/Bright/range without installing a Weapon pointer.
Admission requires the shared travel budget and retained Warhead relationships.
Historical frozen omission keeps the previous tail. The host explicitly reports
native nonterminating circle inputs and the null-Weapon Inviso/FlakScatter crash
combination; it does not substitute clipping or a factory Weapon pointer.
This is not a claim of complete Ares or all specialized detonation coverage.

Version 0.30 adds optional `MatchRulesPolicy.projectile_fuel` and
`WeaponFiringPolicy.projectile_range` (signed leptons). The independent
`weapons/projectile_range` component domain reads `ProjectileRange` through
`weapon-effective:ares-leptons` at reached physical Rules passes. The matching
host owns the saved/hashed live Bullet budget, true 3D distance and existing
impact tail. Ordinary FireAt supplies the weapon budget; bare and stock
shrapnel constructors retain INT_MAX. `Ranged=no`, retail and older guests
preserve their prior flight behavior. Version 0.31 also supplies this budget to
the ordinary Ares Airburst/Splits factory; Firestorm and other specialized
factories remain separate compatibility residuals. Use the
original engine/component pair for older full saves; layout guards remain.

Version 0.29 adds optional `weapon_firing_resource` queries with actual bound
weapon names, animation counts and Sonic/fire-particle facts. The component
selects ordinary FireAt damage and directional muzzle bins once at admission;
restore rebuilds the same immutable WeaponType cache from the saved component
and frozen configuration. Hot firing paths make no additional guest calls.
Ares `ApplyDamage` uses the shared nullable-bool reader; ordinary weapons default
on and Sonic/fire-particle weapons default off. AI damage estimates and the
independent wave/particle effects retain their original owners and behavior.
Animation references, occupied/open-topped overrides and coordinates stay host-owned.
New enum variants append, preserving accepted saved manifest ordinals. Older
guests receive no new request, and old nonempty configurations do not acquire
the new domain. This is a bounded increment toward full Ares compatibility.

Configuration uses the `$weapon-config` catalog and `weapon-effective:` scalar
service, derived from actual Weapon instances' reached bodies. A later same-ID
Projectile cannot replace this source. The shared INI converters and input-work
budget remain host-owned; the plugin uses `ConfigView::retained_weapon_read`.

Version 0.36 adds the optional pure `MasterTypeLists` declaration (saved hook ordinal31) and immutable Nuclear payload settings. The host runs the existing WeaponTypes registry reader before each physical type-data pass and retains actual source references. A new guest cohort reads common SW.Warhead on every reached body, independently of the selected handler; historical nonempty cohorts retain their earlier gate. No new WIT import or per-frame callback is required. Matched host support is required; saved full engine layouts remain separately versioned. Light/PreImpact/EVA presentation and common range semantics remain separate open compatibility chains.

Version 0.28 moves the existing SuperWeapon extensions into
`features/superweapons/`. `configure_super_read` runs at the original Rules
visit, preserving first-visit initialization and physical SW.Warhead reads
(existing-only lookup; missing, empty and unknown values retain the old slot);
`superweapon_resource` publishes one immutable batch for providers, availability,
initial/virtual charge, payments, GenericWarhead, UnitDelivery and presentation.
Authors use the existing Ares INI keys without an extra enable tag. Shared Host
readers and fixed audio catalogs remain authoritative; the original engine owns
charge/history, money, two-stage selection and delivery queues, without per-frame
Wasm mutations.

This candidate uses config18/state1 and the same API v1/WIT name, with a matching
newer host required for the new hooks. Absent declarations preserve the host
profile; an explicit default policy selects stock behavior. Restore uses the
exact saved component and frozen configuration; preserve old host/component
pairs for old saves. Scope remains the previously implemented subset, excluding
full charge-drain, additional payload kinds and additional AI targeting modes.

Version 0.27 adds the pure `sound_sample_admission` declaration before the
fixed SOUNDMD catalog is first read. The reference selects unindexed sample
admission without an INI switch; absence keeps the host profile, while explicit
false requires indexed names when an index exists. The original catalog, sample
selection and WAV decoder remain host-owned. Reacquiring assets retains the same
frozen catalog. An equal raw INI hash does not prove equal admission semantics;
exact component identity and the fixed source recipe remain authoritative.
Config17/state1/API v1/WIT are unchanged; the new declaration requires a matching
host. Audio-device output and cross-platform execution were not tested.

Version 0.26 binds the ordinary Warhead House filter and existing Airburst
boundary through `match_rules.warhead_house_filter` and `airburst_tail`.
The reference declares the House filter independently of EMP activation; explicit
true without effective Warhead relationships is rejected before publication.
`unsupported_replacement` preserves the unimplemented replacement-fan boundary
while parent damage still executes. Absence keeps the host profile; explicit
false / `retail_fixed_nine` selects stock behavior. Actual component damage,
formal firing, saved rebinding, legacy controls and real retail loading passed.
Config17/state1/API v1/WIT are unchanged and require a matching host. This does
not implement the replacement fan or claim GUI, audio-device, network,
cross-platform or performance validation.

Version 0.25 adds the pure `prism_support_defaults` declaration before the first
Rules read. The reference component selects the existing default-unit correction
without an INI switch; absence keeps the host profile and explicit false selects
retail behavior. A present General section with an absent modifier key retains
the intended percentage across layers. Numeric readers, support counts and firing
remain with their existing owners. Actual-component layer/save/delayed-shot, legacy
and retail loading checks passed. Config17/state1/API v1/WIT stay unchanged, but a
matching host is required. The native rules save layout changed: retain old host
and component pairs for old saves. This adds no PrismForwarding; no GUI or
cross-platform run was performed.

Version 0.24 adds the pure `spawn_identity` declaration for the existing
CnCNet spawn.ini Country and campaign Side width gates. The component selects
extended Country identities and byte Side metadata before gameplay configuration.
The single parser reads the request once; the original Ready transition approves
it against the admitted process recipe. Absent declarations retain the host
profile, while explicit false selects retail limits even in a legacy host.
Country allocation, map/peer/color checks and campaign record0 remain with their
existing owners. Actual component-to-House/Side/MCV, legacy and seven retail-mode
checks passed. Community decoding and the shared approval helper were tested
without installing the global Snapshot; no window/network startup or cross-platform
run was performed. The independent network descriptor keeps its original bounds.
Config17/state1 and API v1/WIT stay unchanged; a matching newer host is required.

Version 0.23 adds one `type_sound_resource` catalog for
`Promote.VeteranSound` / `Promote.EliteSound`. Reached Techno reads retain
inherit-global, explicit silence and registered-cue states. The host borrows the
process-selected SOUNDMD registry, validates its source hash and native Type
identities, then binds the catalog once; promotion state, positional SFX and EVA
remain in their existing owners. Registered literal `none` remains a cue, while
silence suppresses only SFX. Absence keeps the old profile; an empty catalog
selects global cues even in a legacy host. Actual AI-to-dispatcher, save/continued
promotion, changed-source rejection, legacy, shared PCX GPU and seven retail-mode
checks passed. Playback on an audio device was not tested. This needs a matching
host; API v1/WIT do not imply compatibility with older hosts. See the
[type sound API](api/src/audio.rs).

Version 0.22 adds one `cameo_resource` catalog for normal, promoted and
SuperWeapon PCX declarations. The component retains names at the actual physical
Rules/ART read points; the host keeps the shared filename reader, PCX/SHP decoder,
60x48 logical canvas and atlas. Missing promoted art falls back to normal PCX,
then the existing SHP path; a fully transparent PCX stays selected. No new INI
switch or per-icon guest call is required. An absent catalog preserves the old
profile; an explicit empty catalog selects stock. Actual-component GPU, exact
save restoration, legacy and seven retail-mode checks passed, including the
previous component. Formal resource binding follows the completed scenario and
common TypeReset; intermediate host configuration is not proof that native
SuperWeapon extensions survive ClearScene. See the [catalog API](api/src/cameo.rs).
This remains a bounded baseline migration, not full Ares or native pixel parity.

Version 0.21 adds optional `match_rules.anim_damage` selectors. Existing Anim
House, or an attached Techno's House, supplies damage provenance; ART Warhead
can take priority over the original own-ID INVISO/C4Warhead or FlameDamage2
fallback. The host retains reference allocation, accumulation, timers, integer
debit and the shared area receiver. No attacking object is invented from House
or visual remap. The component declares both installed fixes without new INI
switches or per-frame guest calls. Current/absent/previous/explicit-off component
controls, legacy and targeted retail loading passed. The full combination matrix
remains a final finite-phase gate. Weapon= and Damage.Delay= remain outside this
bounded migration.

Version 0.20 adds optional `match_rules.building_presentation` selectors for
independent building SHP/MAKE palette groups and DestroyAnim House fallback.
The Ares component declares both installed fixes independently of gameplay INI
switches. Existing retained ART readers, palette decoding, animation state and
rendering stay host-owned; final component binding refreshes the existing asset
catalog before constructors. Absence preserves the original profile; explicit
false selects retail behavior. Actual-component fatal/save, indexed GPU, legacy
and retail loading controls passed. This does not extend bib, active/super
animations, voxel palettes or animation damage ownership.

Version 0.19 adds the separate pure `long_list_readers` capability before cold
Rules processing. `Extended2048` widens only House `Allies` and Techno
`Prerequisite` / `PrerequisiteOverride`; explicit `Retail128` overrides the
legacy profile, and an absent declaration retains it. General group buffers
remain 128 bytes and physical INI lines retain their limit of 511 bytes. The Ares component
declares this installed reader fix independently of its gameplay INI switches.
Actual-component/save/recovery, reader, legacy and retail loading controls passed.
New serialized resource fields still require the corresponding engine/save format;
preserve older engine/component pairs for their complete saves.

The faction increment adds four separate optional capabilities: `eva_catalog`
selects the fixed speech reader before Rules; `configure_country_read` selects the
Country-body reader at its real prelude; `configure_faction_pass` consumes only
actual Country-only reads; `faction_resource` queries frozen Side/Country policies
by family ordinal and stored ID. One component-owned history performs initialization
and one-shot inheritance. Existing host owners still execute crew/disguise,
AI defense and side-token rules, paradrop/canopy selection and starting veterancy.
Completed physical steps publish one accepted Program checkpoint; isolated startup
projections finalize their own receipt without replacing the live process history.

`AresFaction` defaults on for new configurations and can be explicitly disabled in
`[VeraFeatures]`. It is read at the actual Country prelude; the installation-time EVA
catalog layout remains frozen separately. This bounded shared-configuration/gameplay
increment passed actual-Wasm/save, direct-consumer, formal application and legacy
controls, seven retail loading modes and previous-.15 loading. The 0.17 candidate also connects live Country menus/flags, early/late
loading presentation and LoadingTheme. Version 0.18 connects EVA SetSide, sidebar,
message colors, GraphicalText and score resources through the same accepted Side
declarations. These bounded results do not establish full Ares, complete GUI/device,
networking or performance equivalence. See [faction contracts](api/src/faction.rs).

The Country presentation chain uses one accepted live Program and the existing
menu/loading snapshots. ScenarioClear keeps still-live faction inheritance history;
TypeReset clears it when those types are actually destroyed. Independent optional
`loading_theme_resource` reads bounded names from the retained root Side/Country
or parsed campaign Basic through the canonical host reader and original Theme owner.
It requires cold Rules admission and `configure_boundary`, without requiring
faction or Country-reader hooks. Related actual-component and legacy controls passed;
seven real-retail loading modes and previous-.16 loading also passed. Full GUI/device output is not claimed.
See [loading-theme contract](api/src/loading.rs).

The startup increment adds optional `startup_resources`: source composition,
audio pairs and string supplements are selected before their original resource
frontiers, separately from gameplay Configure. The component supplies bounded,
ordered recipes; the existing host owns VFS resolution, Include expansion, audio
index/BAG merging and CSF parsing. The admitted identity and bytes remain frozen
across Rules binding and process-asset recovery. Old components without this hook
keep the original profile fallback. A component needing startup resources must be
visible before audio selection; a module found only in later archives is refused
if it missed that frontier. Raw inspection remains separate from active admission.
This bounded increment passed actual-component core and application checks,
LAN worker install/cache and existing network controls, original PCM/text controls,
legacy Ares resource tests, seven retail loading modes and previous-component
loading. It does not establish full network, GUI/device output or performance parity.
The startup increment itself kept configuration version 14; the current faction
increment uses version 15. Entity state remains version 1; retain matching old
engine/component pairs for old full saves. See [startup recipes](api/src/startup.rs).

The Foundation increment adds `foundation_resource` for class-qualified Building
identities. A component selects a stock selector or a small immutable section
reference into the process-selected fixed ART. The host validates its source hash,
section and geometry, then publishes through the original Foundation owner before
construction. Cell and complete outline arrays stay outside guest messages and
configuration. A separate `fixed-art-view` preserves the earlier closed production
view. Saved configuration is rebound against the existing matching fixed ART,
without replaying Configure. This bounded increment passed actual-component
spatial/save tests, direct-consumer and legacy controls, formal application and
existing placement GPU checks, seven retail loading modes and a previous-component
loading control. Making the Custom enum available in the ordinary build changes
its traced full-save layout and derived configuration digest; retain corresponding
older engines and components for older saves. Stock world-state hash encoding
remains unchanged. This does not establish universal rendered Custom parity.
Legacy Ares-profile invalid raw Custom declarations still fail their original
early registry validation, including when an external component later disables them.

The conversion increment adds optional type targets and signed secondary ROT,
plus an explicit match-wide Script68 selector. The shared Team VM and same-object
conversion owner retain entity identity, health, ammo, mining and locomotor state.
Configuration references use the actual common Techno construction prefix. This
bounded increment has passed connected behavior, legacy controls, application
resource loading and seven real-retail loading modes. Making the retained
inactive miner variant available changes the traced full-save layout; preserve
the corresponding older engines and components for older saves.

The spotlight increment extends the existing optional type resource with a
capability override, type-rule selector, attachment, height/distance and channel
flags. Original building behavior and extended attached lights share the same host
light store, lifecycle and rendering. This bounded increment has passed focused
and legacy behavior tests, application resource loading, existing GPU pixel
controls and seven real-retail loading modes. Existing rendering residuals remain.

The EMP increment adds optional `emp_resource` declarations for rules, class-qualified
Techno types and warheads. Nullable resource names distinguish inheritance, explicit
clear and a registered name. The component interprets configuration; the existing
host owns counters, mission transitions, ordered trigger dispatch and animations.
Focused behavior and legacy controls, application loading and seven real-retail
loading modes have passed for this bounded increment. Existing unsupported receiver
paths remain unsupported. Frozen trigger classification changes the traced full-save
layout; retain corresponding older engines and components for older saves.

The production increment adds `production_resource` for frozen prerequisite
groups, type qualification and existing production-owner selectors. The optional
`configure_boundary` capability receives actual ScenarioClear/TypeReset events
and pending physical Process reads from the same native Rules owner. Exact
component identity gates opaque process continuation; saves adopt their frozen
Program without replay. Focused production and loader regressions, legacy
controls and seven real-retail loading modes have passed for this increment;
the wider migration remains in progress. Added
live Country/House-plan fields change the automatically traced full-save layout;
keep old saves with their corresponding old engine and component.

The armor increment adds optional `armor_resource` catalog, class-qualified
object/Terrain bindings and warhead interaction tables. `AresArmor` defaults to
enabled only for new configurations; historical missing flags stay disabled.
The host reuses the same interaction query for damage, weapon choice and target
admission. Configuration uses the original `verse-token` numeric reader, with
no duplicate guest percentage or x87 parser. Focused armor/consumer and retail
loader checks passed for this increment; the broader migration remains open
and this is not a new stable delivery. See [armor resources](api/src/armor.rs).

The candidate adds `match_rules`, paired `effect_resource` / `effects`, and
optional type survival and cloak permissions. `AresKeepAlive` and
`AresAttachEffect` join the four existing default-enabled Ares modules for new
matches. Ares effects own configuration and ordered lifecycle decisions in the
plugin; the original host owns the only instance list, animation references,
derived factors, permanent crate bonuses and cloak clock. Effects do not
overwrite the shield's opaque entity state. See [match rules](api/src/match_rules.rs)
and [effect transactions](api/src/effects.rs).

Fresh authored and generated scenarios bind the same frozen program after final
Rules processing and before constructors. Saved scenarios adopt the saved exact
module and configuration without replaying Configure, then bind animation roots
through the shared resource closure. Generator modification hooks are not
provided. These source paths are undergoing concentrated acceptance; this is
not a completed migration of all existing Ares support or a release claim.

New serialized host fields also change the complete-world snapshot layout.
Retain the corresponding old engine, component and declaration for old saves;
matching component state version alone does not migrate an old host snapshot.

One work loads one optional portable gameplay component. The same engine runs
retail YR without a declaration or one reference component whose shields, authored
overload, Ares weapon ammo costs, reload amounts, low-ammo weapon selection and
manual-action restrictions are configured through INI. New matches with component
0.7.0 enable its four implemented Ares modules by default; Shield and Overload
still need explicit opt-in. This prototype migrates the existing supported shield chain;
it does not migrate every Ares/Phobos feature or claim full upstream parity.

This engine checkout retains the shared API, WIT contract, Wasmtime runtime and
engine adapters. Plugin implementation and authoring tools live in the sibling
[`plugin-dev` workspace](../../plugin-dev/README.md), whose first plugin is
`plugins/vera20k-ares` and publishes `vera20k-ares.wasm`. Its current features
include the retained Phobos-compatible shield chain and authored overload
example; the Ares filename does not claim complete Ares compatibility.

Plugin authors work in that independent workspace using standard Cargo. The
[build guide](../../plugin-dev/sdk/dev-kit/README.zh-CN.md) needs no engine
sources, executable, Python build runner or game assets. The engine's `sdk/api`
and `sdk/wit` are the authoritative contract; the plugin workspace builds against
a pinned development snapshot. A contract change updates the engine definition
and host support before refreshing that SDK snapshot.

## Boundaries

- `api`: versioned typed messages, bounded opaque state/config, generic meters,
  relationship flags and the shared native countdown timer. `api/src/resource.rs`
  owns the shared immutable weapon/type resource policies.
- `wit/gameplay.wit`: portable `vera:mod-sdk@0.1.0` component interface.
- `runtime`: Wasmtime, with only immutable INI and shared numeric-kernel imports.
  No filesystem, clock, random, network or WASI services are linked.
- The independent [packager](../../plugin-dev/sdk/package/src/main.rs) wraps a
  core Wasm module and emits the component plus its content-hashed declaration,
  without executing it; it is not an engine workspace member.
- [`plugin-dev/plugins/vera20k-ares`](../../plugin-dev/plugins/vera20k-ares/README.md):
  author-editable Rust reference. One artifact contains the
  features. New matches default AresAmmo, AresReload, AresNoAmmoWeapon and
  AresManualControl to yes; `[VeraFeatures]` can explicitly set each to no for
  comparison. Shield and Overload still default to no, and Overload requires
  Shield. Individual Ares tags retain their own defaults, including no for all
  four manual restrictions. Defense mechanisms live in `features/defense`, weapon
  policies in `features/weapons`, and input policy in `features/units`.
  All use the same `support/host.rs` facade. Selection is frozen per match.
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

Weapon and type policies use these optional immutable queries:
`weapon_resource` returns a weapon's cost policy; `type_resource` returns reload
amounts, low-ammo weapon selection and local input policy for an object type and actual category.
`weapon_firing_resource` additionally selects ordinary shot damage and muzzle
facing bins without changing the old `weapon_resource` request shape.
Admission queries them once and save restoration rebuilds them from frozen
configuration. Hot paths use the resulting read-only projections. These features
do not own a second ammo count or reload clock, and need no additional per-entity
component state. These queries use the existing `evaluate` message channel and
add no Host service or WIT import.

The component identifies as `vera.reference-gameplay` 0.7.0, config_version=5,
state_version=1; API v1 and WIT remain unchanged. Its new defaults apply only to a
new match's empty initial configuration, not to nonempty frozen configurations.
This internal manifest identity is retained independently of the
publication filename and Cargo package name. Older components receive only the queries they declare; changing
INI cannot add a feature absent from their original binary. Existing saves still
require their exact corresponding component bytes.

The matching host must support `use_weapon_rof`, optional `input` type policies
and complete weapon indices in building delayed-fire state/save paths. Sharing
API v1 alone does not make an older host compatible with those requirements.

See the [Ares resource guide](../../plugin-dev/plugins/vera20k-ares/AMMO.zh-CN.md) and
[resource-cycle.ini](../../plugin-dev/plugins/vera20k-ares/examples/resource-cycle.ini) for the combined
configuration, defaults, category restrictions and the documented 0..17 weapon
array and safe no-weapon behavior for `NoAmmoWeapon` indices at or above 18.
The [manual-control guide](../../plugin-dev/plugins/vera20k-ares/MANUAL.zh-CN.md)
retains the tunnel and mixed/multi-healer gaps; default enabling does not complete them.

Every call creates a fresh guest instance with fuel/memory/call limits. Preview
and commit use the same guest leaf; preview publishes nothing. A fault rejects
the plan and stops frame admission, rather than silently removing the module.
No live module replacement or saved schema migration is provided.

## Packaging

Build and package plugins in the sibling `plugin-dev` workspace. Starting from
the engine checkout root, these commands enter that workspace; create `dist`
first and choose a new output subdirectory for each package:

```sh
cd ../plugin-dev
cargo build --locked --release --target wasm32-unknown-unknown -p vera20k-ares
cargo run --locked --release -p vera-mod-pack -- target/wasm32-unknown-unknown/release/vera20k_ares.wasm dist/reference-new --program vera20k-ares.wasm
```

The `target/` path is Cargo's default; adjust the input path when using
`CARGO_TARGET_DIR`. Install the Rust target with
`rustup target add wasm32-unknown-unknown` if needed. Plugin commands use standard
Cargo; engine builds and tests continue to use `python -m tools.cargo_run` from
the engine checkout. Full authoring instructions are in the
[independent workspace guide](../../plugin-dev/sdk/dev-kit/README.zh-CN.md).

The output directory must not exist and its parent must exist. Inputs and
encoded components are bounded to 16 MiB. The tool validates the encoded
component before creating output and never overwrites an existing directory or
artifact. It neither executes the guest nor supplies WASI adapters. Engine
admission still checks the SDK manifest, content hash and allowed imports.

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
saved with the world. Restore requires the current work's component digest to
match the saved digest exactly, then uses the saved original component bytes and
frozen configuration. It neither reapplies current INI nor inserts new defaults.
Updating a component requires a new match; retain the old artifact and declaration
for existing saves, even though those saves also contain component bytes.

## Scope

The retained 0.7.0 host admits authored maps and rejects declared gameplay at
generated-map admission. The development candidate admits both through the
shared scenario binding described above; it does not add generator hooks.

This is a native desktop prototype for Windows/Linux/macOS. Windows is the local
validated platform; other platform builds and transport-connected multiplayer
are separate validation. The architecture does not certify 20,000-unit throughput:
fresh instantiation and whole-config decoding per entity are intentionally simple
for the first isolation/behavior experiment. A batched interface and measured
performance work should precede scale claims. See the reference component README
for the inherited animation/transfer/warhead-operation residuals.
