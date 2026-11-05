//! Optional external gameplay: this facade keeps engine call sites stable.
//! Program owns saved identity/configuration, assets owns admission/restoration,
//! host exposes native readers/math, and simulation adapts lifecycle phases.
#[path = "mod_sdk/assets.rs"]
mod assets;
#[path = "mod_sdk/host.rs"]
mod host;
#[path = "mod_sdk/program.rs"]
mod program;
#[path = "mod_sdk/simulation.rs"]
mod simulation;
#[path = "mod_sdk/startup.rs"]
mod startup;

pub use assets::adopt_saved_program;
pub(crate) use assets::{PreparedProcessModBinding, prepare_process_mod_binding};
pub(crate) use assets::{early_configuration_seed, load_from_assets};
pub use program::Program;
pub(crate) use simulation::{
    attach_entity, live_entity_context, pre_receive_damage, preview_receive_damage, tick_entity,
    weapon_user,
};
pub use startup::{open_active_assets, reopen_active_assets};
pub use vera_mod_api::{InputPolicy, Meter, Projection, Tint};

#[path = "mod_sdk/acceptance.rs"]
pub mod acceptance;
#[cfg(test)]
#[path = "mod_sdk/tests.rs"]
mod tests;

#[cfg(test)]
#[path = "mod_sdk/startup_tests.rs"]
mod startup_tests;
