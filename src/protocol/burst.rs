//! Captured patterns (from real client traffic):
//!
//! Stationary mining:  mp (bare, no pM) + HB          [mc:2 per hit]
//! Move+mine:          mp (pM binary) + mP (a:7) + HB  [mc:3]
//! Combat:             mP (a:6) + HAI×N                [mc:N+1]
//! Movement:           mp (pM binary) + mP             [mc:2]

use bson::Document;

use crate::constants::movement;
use crate::constants::protocol as ids;

use super::{
    csharp_ticks, make_collectable_request, make_hit_ai_enemy, make_hit_block, make_map_point,
    make_map_point_bare, make_movement_packet,
};

/// Stationary mining hit: mp (bare, no pM) + N×HB [+ optional collect requests].
/// Use when the bot is already adjacent to the target and not moving.
pub fn make_stationary_hit(
    target_map_x: i32,
    target_map_y: i32,
    n_hits: u32,
    collectables: &[i32],
) -> Vec<Document> {
    let mut burst = Vec::with_capacity(1 + (n_hits as usize) + collectables.len());
    burst.push(make_map_point_bare());
    for _ in 0..n_hits {
        burst.push(make_hit_block(target_map_x, target_map_y));
    }
    for &cid in collectables {
        burst.push(make_collectable_request(cid));
    }
    burst
}

/// Move+mine burst: mp (pM of target) + mP (a:7 HitMove, at target world pos) + N×HB [+ optional collect requests].
/// Use when the bot steps INTO the target tile while swinging.
pub fn make_move_mine_burst(
    target_map_x: i32,
    target_map_y: i32,
    target_world_x: f64,
    target_world_y: f64,
    direction: i32,
    n_hits: u32,
    collectables: &[i32],
) -> Vec<Document> {
    let mut burst = Vec::with_capacity(2 + (n_hits as usize) + collectables.len());
    burst.push(make_map_point(target_map_x, target_map_y));
    burst.push(make_movement_packet(
        target_world_x,
        target_world_y,
        movement::ANIM_HIT_MOVE,
        direction,
        false,
    ));
    for _ in 0..n_hits {
        burst.push(make_hit_block(target_map_x, target_map_y));
    }
    for &cid in collectables {
        burst.push(make_collectable_request(cid));
    }
    burst
}

/// Build a combat burst: mP (a:6) + N×HAI.
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
    let mut burst = Vec::with_capacity((n_hits as usize) + 1);
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
    for _ in 0..n_hits {
        burst.push(make_hit_ai_enemy(target_map_x, target_map_y, ai_id));
    }
    burst
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
