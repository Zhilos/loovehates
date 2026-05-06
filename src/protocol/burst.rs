//! mc:N burst bundles matching the official PixelWorlds client's voice.
//!
//! Captured patterns:
//! - Mining: mP (header) + N×HB + p (footer)
//! - Combat: mP (header) + N×HA + ST (footer)
//! - Movement: mp (binary point) + mP (with U field)

use bson::Document;

use crate::constants::movement;
use crate::constants::protocol as ids;

use super::{
    csharp_ticks, make_hit_ai_enemy, make_hit_block, make_map_point, make_movement_packet,
    make_st,
};

/// Build a mining burst: mP header + N×HB + p footer (mc:5 when n_hits=3).
pub fn make_mining_burst(
    player_world_x: f64,
    player_world_y: f64,
    target_map_x: i32,
    target_map_y: i32,
    n_hits: u32,
    user_id: Option<&str>,
    direction: i32,
    anim: i32,
) -> Vec<Document> {
    let mut burst =
        Vec::with_capacity((n_hits as usize) + 2);
    let mut header = make_movement_packet(
        player_world_x,
        player_world_y,
        anim,
        direction,
        false,
    );
    if let Some(u) = user_id {
        header.insert("U", u.to_string());
    }
    burst.push(header);
    burst.push(make_map_point(target_map_x, target_map_y));
    for _ in 0..n_hits {
        burst.push(make_hit_block(target_map_x, target_map_y));
    }
    burst.push(make_st());
    burst
}

/// Build a combat burst: mP header + N×HA + ST footer.
pub fn make_combat_burst(
    player_world_x: f64,
    player_world_y: f64,
    target_map_x: i32,
    target_map_y: i32,
    ai_id: i32,
    n_hits: u32,
    user_id: Option<&str>,
    direction: i32,
) -> Vec<Document> {
    let mut burst = Vec::with_capacity((n_hits as usize) + 2);
    let mut header = make_movement_packet(
        player_world_x,
        player_world_y,
        movement::ANIM_HIT,
        direction,
        false,
    );
    if let Some(u) = user_id {
        header.insert("U", u.to_string());
    }
    burst.push(header);
    burst.push(make_map_point(target_map_x, target_map_y));
    for _ in 0..n_hits {
        burst.push(make_hit_ai_enemy(target_map_x, target_map_y, ai_id));
    }
    burst.push(bson::doc! { "ID": ids::PACKET_ID_PUNCH });
    burst.push(make_st());
    burst
}

/// Build a movement step: mp (binary point) + mP (with U field).
pub fn make_movement_step(
    from_world_x: f64,
    from_world_y: f64,
    to_world_x: f64,
    to_world_y: f64,
    to_map_x: i32,
    to_map_y: i32,
    start_anim: i32,
    target_anim: i32,
    direction: i32,
    user_id: Option<&str>,
) -> Vec<Document> {
    let mut mP_from = make_movement_packet(from_world_x, from_world_y, start_anim, direction, false);
    if let Some(u) = user_id {
        mP_from.insert("U", u.to_string());
    }
    let mut mP_to = make_movement_packet(to_world_x, to_world_y, target_anim, direction, false);
    if let Some(u) = user_id {
        mP_to.insert("U", u.to_string());
    }
    vec![
        mP_from,
        make_map_point(to_map_x, to_map_y),
        mP_to,
        make_st(),
    ]
}

/// C# DateTime.UtcNow.Ticks alias for callers that want a fresh timestamp
/// without importing the parent module's helper.
pub fn current_csharp_ticks() -> i64 {
    csharp_ticks()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ids(batch: &[Document]) -> Vec<String> {
        batch.iter().map(|d| d.get_str("ID").unwrap().to_string()).collect()
    }

    #[test]
    fn mining_burst_shape_is_mP_then_HBs_then_p() {
        let burst = make_mining_burst(10.24, 4.0, 32, 13, 3, Some("u123"), movement::DIR_RIGHT, movement::ANIM_HIT);
        assert_eq!(ids(&burst), vec!["mP", "HB", "HB", "HB", "p"]);
        assert_eq!(burst[0].get_str("U").unwrap(), "u123");
        assert_eq!(burst[1].get_i32("x").unwrap(), 32);
        assert_eq!(burst[1].get_i32("y").unwrap(), 13);
    }

    #[test]
    fn combat_burst_shape_is_mP_then_HAs_then_ST() {
        let burst = make_combat_burst(10.24, 4.0, 32, 13, 99, 3, Some("u123"), movement::DIR_LEFT);
        assert_eq!(ids(&burst), vec!["mP", "HAI", "HAI", "HAI", "ST"]);
        assert_eq!(burst[0].get_str("U").unwrap(), "u123");
        assert_eq!(burst[1].get_i32("AIid").unwrap(), 99);
    }

    #[test]
    fn movement_step_shape_is_mp_then_mP_with_U() {
        let burst = make_movement_step(
            10.24, 4.0, 10.56, 4.0, 33, 13,
            movement::ANIM_WALK, movement::ANIM_WALK, movement::DIR_RIGHT,
            Some("u123"),
        );
        assert_eq!(ids(&burst), vec!["mP", "mp", "mP", "ST"]);
        assert_eq!(burst[2].get_str("U").unwrap(), "u123");
    }

    #[test]
    fn missing_user_id_omits_U_field() {
        let burst = make_mining_burst(0.0, 0.0, 0, 0, 3, None, movement::DIR_RIGHT, movement::ANIM_HIT);
        assert!(burst[0].get_str("U").is_err()); // field absent
    }
}
