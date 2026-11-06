use super::*;

pub const W: usize = 20;
pub const H: usize = 20;
pub const MAX_HEALTH: i32 = 5;
pub const MAX_BOMBS: i32 = 5;
pub const LEVEL_TIME: i32 = 300; // секунд на уровень
pub const MOVE_CD: u64 = 150;
pub const DASH_CD: u64 = 1500;
pub const FOG_RADIUS: i32 = 3;
pub const VIEW: usize = 10;
pub const CELL: f64 = 56.0;
pub const SWAMP_SKIPS: i32 = 3;
pub const BOSS_AIM_DIST: i32 = 3;
pub const BEAR_MS: u64 = 1800;
pub const ZOMBIE_MS: u64 = 2000;
pub const SHOOTER_MS: u64 = 2500;
pub const BOSS_MS: u64 = 1800;
pub const RIVER_MS: u64 = 2000;
pub const GRACE_MS: u64 = 5000;
pub const COMBO_WINDOW: u64 = 900;
pub const COMBO_BONUS: i32 = 5;
pub const LOW_TIME: i32 = 30;
// направления: 0=север,1=юг,2=запад,3=восток (JS DIRS)
pub const DIRS: [(i32, i32); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];
pub type Pos = (usize, usize);
pub fn dir_index(dr: i32, dc: i32) -> usize {
    if dr == -1 {
        0
    } else if dr == 1 {
        1
    } else if dc == -1 {
        2
    } else {
        3
    }
}
pub fn in_bounds(r: i32, c: i32) -> bool {
    r >= 0 && r < H as i32 && c >= 0 && c < W as i32
}
pub fn add(p: Pos, d: (i32, i32)) -> Option<Pos> {
    let r = p.0 as i32 + d.0;
    let c = p.1 as i32 + d.1;
    if in_bounds(r, c) {
        Some((r as usize, c as usize))
    } else {
        None
    }
}
pub fn manhattan(a: Pos, b: Pos) -> i32 {
    (a.0 as i32 - b.0 as i32).abs() + (a.1 as i32 - b.1 as i32).abs()
}
