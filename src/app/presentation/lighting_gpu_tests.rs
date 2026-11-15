//! Explicit stock-asset GPU witness for live lamp-to-ground lighting.
//! The authored map is a diagnostic copy; this is not a retail pixel-parity oracle.
//!
//! Reproduction (retail files remain local): extract Fight.MAP from multimd.mix
//! with asset-browser, entry 0x9306F050, 91254 bytes, SHA-256
//! d751dce7cd3611077e9228c33235f39c71681fff6ac08ca1f716d963ad6ce070.
//! Make a diagnostic copy with exactly these edits:
//! - In [Lighting], change Ambient=1.000000 to Ambient=0.300000.
//! - Immediately after [Structures], insert this stock-type structure row:
//!   9000000=Neutral,GALITE,256,53,113,0,None,0,0,1,0,0,None,None,None,0,0
//!
//! No rules/type overrides are needed. Set RA2_DIR to the retail installation,
//! VERA20K_LAMP_PROBE_MAP to the absolute diagnostic map path, and optionally
//! VERA20K_LAMP_PROBE_OUTPUT to a PNG output directory. Run:
//! cargo test -p vera20k --lib stock_lamp_loaded_by_scenario_changes_actual_terrain_pixels -- --ignored --nocapture
//! This isolates one terrain tile; full-window draw routing, shroud composition
//! and equivalence to gamemd pixels require separate evidence.
use std::collections::HashSet;
use std::path::PathBuf;

#[test]
#[ignore = "requires stock retail assets, a diagnostic authored lamp map and a GPU"]
fn stock_lamp_loaded_by_scenario_changes_actual_terrain_pixels() {
    use super::lighting::MatchLighting;
    use crate::map::lighting::{CellLightGrid, parse_lighting};

    let root = PathBuf::from(std::env::var_os("RA2_DIR").expect("retail root"));
    let map = std::env::var("VERA20K_LAMP_PROBE_MAP").expect("authored diagnostic map");
    let mut scenario = crate::headless_scenario::load(&root, &map, 0x420)
        .expect("normal scenario construction with stock rules/art/terrain");
    let lamp = scenario
        .sim()
        .entities()
        .values()
        .find(|entity| {
            scenario
                .sim()
                .interner()
                .resolve(entity.type_ref())
                .eq_ignore_ascii_case("GALITE")
                && (entity.position.rx(), entity.position.ry()) == (53, 113)
        })
        .expect("authored stock GALITE must survive normal scenario construction");
    let (id, rx, ry) = (lamp.stable_id(), lamp.position.rx(), lamp.position.ry());
    assert!(
        lamp.lifecycle.object_alive() && !lamp.lifecycle.in_limbo() && lamp.lifecycle.cell_marked()
    );
    let rules = &scenario.runtime.resources.rules;
    let stock = rules.object("GALITE").expect("stock GALITE rules");
    assert_eq!(stock.building_light().visibility_leptons(), 5000);
    assert_eq!(stock.building_light().intensity_units(), 200);
    // Headless construction retains the actual resolved world but omits the
    // app-only immutable template. Snapshot that same terrain for this view.
    let terrain_snapshot = scenario.sim().resolved_terrain().unwrap().clone();
    let terrain = &terrain_snapshot;
    let probe = TerrainPixelProbe::new(&root, &scenario, (rx, ry));
    let mut lighting = MatchLighting::default();
    lighting.install(
        CellLightGrid::new(),
        parse_lighting(&scenario.map.ini),
        2,
        Some((terrain, &scenario.runtime.simulation, rules)),
        None,
    );
    let lit_scalar = lighting
        .grid()
        .cell_light_at((rx, ry))
        .unwrap()
        .common_scalar;
    let lit_additive = lighting
        .grid()
        .cell_light_at((rx, ry))
        .unwrap()
        .raw_additive_intensity;
    assert!(
        lit_additive > 0,
        "stock lamp must reach the live lighting owner"
    );
    let lit_pixels = probe.render(lighting.grid());

    scenario
        .runtime
        .simulation
        .lighting_sources_mut_for_test()
        .discard_pending();
    scenario.runtime.simulation.apply_fatal_lifecycle_stage(
        rules,
        crate::sim::combat::FatalLifecycleStage::BeforeBuildingDestruction,
        id,
        crate::map::entities::EntityCategory::Structure,
        crate::sim::world::UninitContext::with_rules(rules),
    );
    let events = scenario
        .runtime
        .simulation
        .lighting_sources_mut_for_test()
        .take_pending();
    lighting.apply_events(terrain, &events);
    lighting.refresh(terrain, &scenario.runtime.simulation, rules, 2);
    let dark_scalar = lighting
        .grid()
        .cell_light_at((rx, ry))
        .unwrap()
        .common_scalar;
    assert_eq!(
        lighting
            .grid()
            .cell_light_at((rx, ry))
            .unwrap()
            .raw_additive_intensity,
        0
    );
    let dark_pixels = probe.render(lighting.grid());
    let changed = lit_pixels
        .iter()
        .zip(&dark_pixels)
        .filter(|(a, b)| a != b)
        .count();
    let brightness = |pixels: &[[u8; 4]]| {
        pixels
            .iter()
            .map(|p| u64::from(p[0]) + u64::from(p[1]) + u64::from(p[2]))
            .sum::<u64>()
    };
    assert!(
        changed > 0,
        "lamp changed state but never reached GPU pixels"
    );
    assert!(
        brightness(&lit_pixels) > brightness(&dark_pixels),
        "positive stock lamp must brighten actual terrain"
    );
    eprintln!(
        "GALITE({rx},{ry}), scalar {lit_scalar}->{dark_scalar}, additive {lit_additive}->0, GPU changed {changed}/{} pixels",
        lit_pixels.len()
    );
    if let Some(out) = std::env::var_os("VERA20K_LAMP_PROBE_OUTPUT") {
        let out = PathBuf::from(out);
        std::fs::create_dir_all(&out).unwrap();
        for (name, pixels) in [
            ("lamp-present.png", &lit_pixels),
            ("lamp-removed.png", &dark_pixels),
        ] {
            let bytes: Vec<u8> = pixels
                .iter()
                .flat_map(|pixel| pixel.iter().copied())
                .collect();
            image::save_buffer(
                out.join(name),
                &bytes,
                probe.tile.width,
                probe.tile.height,
                image::ColorType::Rgba8,
            )
            .unwrap();
        }
    }
}

/// The existing stock-lamp witness and the SDK witness use the same real TMP,
/// ordinary terrain instance builder and WGSL/readback consumer.
struct TerrainPixelProbe {
    tile: crate::map::theater::TileImage,
    grid: crate::map::terrain::TerrainGrid,
    placement: crate::map::terrain::TilePlacement,
}
impl TerrainPixelProbe {
    fn new(
        root: &std::path::Path,
        scenario: &crate::headless_scenario::HeadlessScenario,
        at: (u16, u16),
    ) -> Self {
        use crate::assets::asset_manager::{AssetManager, MediaArchiveMode};
        use crate::map::terrain::{TerrainCell, TerrainGrid, TilePlacement};
        use crate::map::theater::{TileKey, load_theater, load_tile_images};
        let terrain = scenario.sim().resolved_terrain().unwrap();
        let cell = terrain
            .cell(at.0, at.1)
            .expect("probe has real resolved terrain");
        let (tile_id, sub_tile) = terrain.presentation_tile(cell);
        let key = TileKey {
            tile_id,
            sub_tile,
            variant: 0,
        };
        let mut assets = AssetManager::new(root, MediaArchiveMode::STOCK_DIGITAL).unwrap();
        let theater = load_theater(&mut assets, &scenario.map.header.theater).unwrap();
        let tile = load_tile_images(
            &assets,
            &theater.lookup,
            &theater.iso_palette,
            &HashSet::from([key]),
        )
        .remove(&key)
        .expect("actual map TMP decodes");
        let placement = TilePlacement {
            uv_origin: [0.0; 2],
            uv_size: [1.0; 2],
            pixel_size: [tile.width as f32, tile.height as f32],
            draw_offset: [tile.offset_x as f32, tile.offset_y as f32],
        };
        let grid = TerrainGrid {
            cells: vec![TerrainCell {
                screen_x: -(tile.offset_x as f32),
                screen_y: -(tile.offset_y as f32),
                tile_id,
                sub_tile,
                z: cell.level,
                rx: at.0,
                ry: at.1,
                variant: 0,
                tint: [1.0; 3],
                radar_left: [0; 3],
                radar_right: [0; 3],
            }],
            world_width: tile.width as f32,
            world_height: tile.height as f32,
            origin_x: 0.0,
            origin_y: 0.0,
            bridge_middle_tiles: None,
        };
        Self {
            tile,
            grid,
            placement,
        }
    }
    fn render(&self, lights: &crate::map::lighting::CellLightGrid) -> Vec<[u8; 4]> {
        let lookup = |_, _, _| Some(self.placement);
        let instance = crate::render::terrain_instances::build_visible_instances(
            &self.grid,
            Some(lights),
            0.0,
            0.0,
            self.tile.width as f32,
            self.tile.height as f32,
            Some(&lookup),
            None,
        )
        .normal
        .remove(0);
        crate::render::depth_gpu_tests::render_terrain_lighting_probe(&self.tile, instance)
    }
}

/// Installed impact10036998 can replace CurrentNuke without calling
/// ChangeLighting10032370. Existing Convert pixels must stay; later
/// LightConvert555DA0 uses GetLightingColor10032700's replacement source.
#[test]
#[cfg(not(any(feature = "ares", feature = "phobos")))]
#[ignore = "requires actual .40 VERA_SDK_NUCLEAR_WASM, retail assets and a GPU; isolated process"]
fn sdk_nuclear_source_only_change_preserves_old_pixels_and_changes_new_convert() {
    use super::lighting::MatchLighting;
    use crate::map::lighting::{CellLightGrid, parse_lighting};
    use crate::sim::command::Command;
    const TEST: &str = "app::presentation::lighting_gpu_tests::sdk_nuclear_source_only_change_preserves_old_pixels_and_changes_new_convert";
    const CHILD: &str = "VERA_NUCLEAR_LIGHT_GPU_CHILD";
    if std::env::var_os(CHILD).as_deref() != Some(std::ffi::OsStr::new("1")) {
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                TEST,
                "--ignored",
                "--test-threads=1",
                "--nocapture",
            ])
            .env(CHILD, "1")
            .output()
            .unwrap();
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(output.status.success(), "{stdout}\n{stderr}");
        assert!(
            stdout.contains("running 1 test") && stdout.contains("1 passed; 0 failed"),
            "{stdout}"
        );
        return;
    }
    let root = PathBuf::from(std::env::var_os("RA2_DIR").expect("retail root"));
    let module =
        std::fs::read(std::env::var_os("VERA_SDK_NUCLEAR_WASM").expect("actual .40 component"))
            .unwrap();
    // One authored clear map; Rules/ART, the TMP and palette remain retail.
    // Direct delivery removes the unrelated silo animation/flight delay.
    let map = b"[Map]\nTheater=TEMPERATE\nSize=0,0,40,40\nLocalSize=2,2,36,32\n\
[IsoMapPack5]\n1=CAAEABUAAAAAEQAA\n[Basic]\nName=SDK light GPU\nNewINIFormat=4\n\
[Waypoints]\n0=30030\n1=50050\n[Lighting]\nAmbient=0.3\nRed=1\nGreen=1\nBlue=1\n\
[SuperWeaponTypes]\nSDKLightA=SDKLightA\nSDKLightB=SDKLightB\n\
[SDKLightA]\nType=MultiMissile\nNuke.SiloLaunch=no\nNuke.Payload=SDKLightPayload\n\
SW.Warhead=SDKLightWH\nIsPowered=no\nSW.InitialReady=yes\nRechargeTime=1\n\
Light.Enabled=yes\nLight.Ambient=30\nLight.Red=200\nLight.Green=0\nLight.Blue=0\n\
[SDKLightB]\nType=MultiMissile\nNuke.SiloLaunch=no\nNuke.Payload=SDKLightPayload\n\
SW.Warhead=SDKLightWH\nIsPowered=no\nSW.InitialReady=yes\nRechargeTime=1\nLight.Enabled=no\n\
[BuildingTypes]\nSDKLightLauncher=SDKLightLauncher\n\
[SDKLightLauncher]\nImage=NANUK\nStrength=1000\nArmor=wood\nPower=1000\n\
SuperWeapon=SDKLightA\nSuperWeapon2=SDKLightB\nNukeSilo=no\n\
[WeaponTypes]\nSDKLightPayload=SDKLightPayload\n\
[SDKLightPayload]\nDamage=0\nRange=30\nSpeed=100\nProjectile=SDKLightFall\nWarhead=SDKLightWH\n\
[SDKLightFall]\nArm=2\nShadow=no\nAcceleration=1\nVertical=yes\nDetonationAltitude=30000\n\
[Warheads]\nSDKLightWH=SDKLightWH\n\
[SDKLightWH]\nVerses=100%,100%,100%,100%,100%,100%,100%,100%,100%,100%,100%\n\
NukeFlash.Duration=6000\nPreImpactAnim=none\n";
    let declaration = serde_json::to_vec(&serde_json::json!({
        "api_version": vera_mod_api::API_VERSION, "program": "light.wasm",
        "sha256": crate::util::sha256::sha256_hex(&module),
    }))
    .unwrap();
    crate::assets::community::install(
        crate::assets::community::Snapshot::new(
            "sdk-nuclear-light-gpu".into(),
            vec![
                ("vera-mod.json".into(), declaration),
                ("light.wasm".into(), module),
                ("SDKLight.map".into(), map.to_vec()),
            ],
        )
        .unwrap(),
    )
    .unwrap();
    // Keep the ordinary cold-start owner while preserving its detailed error;
    // the convenience headless loader intentionally reports only Option loss.
    let assets = crate::mod_sdk::open_active_assets(
        &root,
        crate::assets::asset_manager::MediaArchiveMode::STOCK_DIGITAL,
    )
    .unwrap();
    let audio = crate::rules::audio_sources::AudioDefinitions::select(&assets);
    let (_, _, mut owner) = crate::rules::retail_sources::RetailRulesSources::select(&assets)
        .and_then(|sources| sources.into_startup(audio.sounds().clone(), audio.eva().clone()))
        .expect("actual component cold-start Rules admission")
        .into_parts();
    let launch = crate::sim::scenario_bootstrap::MatchLaunchDescriptor::from_resolved(
        crate::headless_scenario::battle_session("SDKLight.map", &[]),
    )
    .unwrap();
    let mut scenario = crate::headless_scenario::load_with_launch_and_ini(
        &root,
        "SDKLight.map",
        0x420,
        launch,
        &Default::default(),
        crate::sim::scenario_session::ScenarioGameMode::Skirmish,
        None,
        Some(&mut owner),
    )
    .unwrap();
    let at = (40, 40);
    scenario.add_human_house("SDKOwner", crate::skirmish_launch::LaunchCountry::America);
    let owner = "SDKOwner";
    let runtime = &mut scenario.runtime;
    runtime
        .simulation
        .spawn_object_at_height(
            "SDKLightLauncher",
            owner,
            at.0,
            at.1,
            0,
            0,
            &runtime.resources.rules,
        )
        .expect("normal launcher construction");
    // House's ordinary update grants the building's initially charged supers.
    for _ in 0..2 {
        runtime.advance_frame_for_tooling(&[], 33).unwrap();
    }
    let terrain = runtime.simulation.resolved_terrain().unwrap().clone();
    let mut lighting = MatchLighting::default();
    lighting.install(
        CellLightGrid::new(),
        parse_lighting(&scenario.map.ini),
        2,
        Some((&terrain, &runtime.simulation, &runtime.resources.rules)),
        None,
    );
    let initial_events = runtime.simulation.take_lighting_events();
    lighting.apply_events(&terrain, &initial_events);
    let probe = TerrainPixelProbe::new(&root, &scenario, at);
    let normal_pixels = probe.render(lighting.grid());
    let mut enabled_pixels = None;
    let mut enabled_cell = None;
    let mut enabled_future_pixels = None;
    for name in ["SDKLightA", "SDKLightB"] {
        let runtime = &mut scenario.runtime;
        let nuclear = runtime
            .resources
            .rules
            .super_weapon(name)
            .unwrap()
            .nuclear
            .as_ref()
            .unwrap();
        let source = nuclear.source_ordinal;
        let payload = nuclear.payload.expect("registered direct payload Weapon");
        let sw = runtime.simulation.interner().get(name).unwrap();
        assert!(runtime.apply_command_for_test(
            owner,
            &Command::LaunchSuperWeapon {
                sw_type_id: sw,
                target_rx: at.0,
                target_ry: at.1,
            }
        ));
        assert!(
            runtime
                .simulation
                .projectiles()
                .iter()
                .any(|(_, bullet)| { bullet.payload().installed_weapon() == Some(payload) }),
            "accepted direct launch must create its actual payload"
        );
        let first_frame = runtime.simulation.session().binary_frame();
        // Activate10073209 starts Vertical payloads20000 above the target.
        // Observe real flight; this bound is not a native flight-time golden.
        for step in 0..3000 {
            runtime.advance_frame_for_tooling(&[], 33).unwrap();
            if runtime
                .simulation
                .session()
                .lighting()
                .current_nuke_source()
                == Some(source)
            {
                break;
            }
            if step == 2999 {
                let bullets: Vec<_> = runtime
                    .simulation
                    .projectiles()
                    .iter()
                    .map(|(id, bullet)| {
                        (
                            id,
                            bullet.launch_origin(),
                            bullet.position(),
                            bullet.velocity(),
                            bullet.trajectory(),
                            bullet.in_logic_vector(),
                            bullet.awaiting_anim(),
                        )
                    })
                    .collect();
                panic!(
                    "actual impact did not reach source assignment for {name}; frames={first_frame}..{}, bullets={bullets:?}",
                    runtime.simulation.session().binary_frame()
                );
            }
        }
        let events = runtime.simulation.take_lighting_events();
        if name == "SDKLightB" {
            assert!(
                !events.iter().any(|event| matches!(
                    event,
                    crate::sim::light_sources::LightingEvent::Global { .. }
                )),
                "disabled second source must not globally relight"
            );
        }
        lighting.apply_events(&terrain, &events);
        lighting.refresh(&terrain, &runtime.simulation, &runtime.resources.rules, 2);
        let pixels = probe.render(lighting.grid());
        if name == "SDKLightA" {
            assert_ne!(
                pixels, normal_pixels,
                "actual enabled Light override must reach ordinary WGSL pixels"
            );
            assert_eq!(
                runtime
                    .simulation
                    .palette_constructor_rgb(&runtime.resources.rules),
                Some([2000, 0, 0])
            );
            enabled_pixels = Some(pixels);
            enabled_cell = Some(lighting.grid().cell_light_at(at).unwrap().clone());
            // Same fresh RGB key and brightness on both sides: differing
            // pixels must come from the constructor context, not the input.
            let mut future = lighting.grid().clone();
            let scalar = lighting.grid().cell_light_at(at).unwrap().common_scalar;
            future.insert_profiled_light(at, [0.512, 0.640, 0.768], scalar as f32 / 1000.0);
            let profile = future.cell_light_at(at).unwrap().profile_id;
            assert_eq!(future.palette_rgb(profile), [2000, 0, 0]);
            enabled_future_pixels = Some(probe.render(&future));
        } else {
            assert_eq!(
                pixels,
                *enabled_pixels.as_ref().unwrap(),
                "source-only change recolored old terrain"
            );
            assert_eq!(
                lighting.grid().cell_light_at(at).unwrap(),
                enabled_cell.as_ref().unwrap()
            );
            assert_eq!(
                runtime
                    .simulation
                    .palette_constructor_rgb(&runtime.resources.rules),
                None
            );
            assert_eq!(lighting.grid().constructor_alternate_rgb(), None);
            // A later Cell sample goes through the ordinary Convert cache
            // with the newly selected constructor context. Reuse this tile
            // and brightness, so the changed pixels witness its palette.
            let old = lighting.grid().cell_light_at(at).unwrap();
            let mut future = lighting.grid().clone();
            future.insert_profiled_light(
                at,
                [0.512, 0.640, 0.768],
                old.common_scalar as f32 / 1000.0,
            );
            let new = future.cell_light_at(at).unwrap();
            assert_ne!(new.profile_id, old.profile_id);
            assert_eq!(future.palette_rgb(new.profile_id), [512, 640, 768]);
            assert_ne!(
                probe.render(&future),
                *enabled_future_pixels.as_ref().unwrap(),
                "same new Convert input must render differently after constructor context changes"
            );
        }
    }
}
