//! GPU-independent draw geometry shared by tactical drawing and static previews.
//! These functions consume presentation inputs; they do not alter map or sim state.

/// BuildingClass DrawBody (0x0043D787) draws from the render-coordinate
/// override, not GetCoords' foundation centre (0x00447AC0). From the Location
/// cell centre, a half-cell subtraction on both lepton axes changes only Y.
pub const BUILDING_ART_LIFT_PX: f32 = super::terrain::TILE_HEIGHT / 2.;

/// Body, bib and attached-art anchor; footprint dimensions do not enter it.
pub fn building_art_anchor(sx: f32, sy: f32) -> (f32, f32) {
    (sx, sy - BUILDING_ART_LIFT_PX)
}

const BRIDGE_BODY_LATIN_SQUARE: [u8; 16] = [0, 1, 2, 3, 3, 2, 1, 0, 2, 3, 0, 1, 1, 0, 3, 2];

/// CellClass::DrawOverlay_Body (0x0047F6A0): only healthy states 0 and 9
/// add g_LatinSquare. Retained native controls: bridge_body_render.json.
pub fn compute_bridge_body_shp_frame(state: u8, rx: u16, ry: u16) -> u8 {
    if state == 0 || state == 9 {
        state + BRIDGE_BODY_LATIN_SQUARE[usize::from((ry & 3) * 4 + (rx & 3))]
    } else {
        state
    }
}

/// CellClass::Get_Draw_Offset (0x00480110) for a bridge anchor (flag 0x80).
/// The raw cell state, not the selected SHP frame, owns this offset.
pub fn bridge_body_y_offset(state: u8) -> f32 {
    if (9..=17).contains(&state) {
        -31.
    } else {
        -16.
    }
}

pub fn is_high_bridge_body_name(name: &str) -> bool {
    ["BRIDGE1", "BRIDGEB1", "BRIDGE2", "BRIDGEB2"]
        .iter()
        .any(|candidate| name.eq_ignore_ascii_case(candidate))
}

/// OverlayClass::Mark (0x005FC570) dispatches by numeric type identity.
/// Keep the tactical renderer's canonical-name compatibility path as well.
pub fn is_high_bridge_body_identity(overlay_id: u8, name: &str) -> bool {
    crate::rules::overlay_types::is_high_bridge_index(overlay_id) || is_high_bridge_body_name(name)
}

/// CC_Draw_Shape (0x004AED70) centres by the integer canvas half, then adds
/// the stored frame rectangle; odd canvas dimensions do not add half a pixel.
pub fn stored_frame_offset(
    canvas_width: u16,
    canvas_height: u16,
    frame_x: u16,
    frame_y: u16,
    y_offset: f32,
) -> (f32, f32) {
    (
        f32::from(frame_x) - f32::from(canvas_width / 2),
        f32::from(frame_y) - f32::from(canvas_height / 2) + y_offset,
    )
}
