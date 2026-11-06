use super::*;

// ---------- События для веба (звук/частицы/тряска) ----------
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(dead_code)]
pub enum Sfx {
    Move,
    Step,
    EsStep,
    Wall,
    Chest,
    Bomb,
    Bear,
    Heal,
    River,
    Hole,
    Key,
    Shoot,
    Danger,
    Win,
    Lose,
    Aim,
    Tick,
    Ach,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OverKind {
    Win,
    Lose,
    Timeout,
}
#[derive(Clone, Debug)]
pub struct Shot {
    pub from: Pos,
    pub to: Pos,
    pub t0: u64,
}
#[derive(Clone, Copy, Debug)]
pub struct Fx {
    pub pos: Pos,
    pub t0: u64,
}

#[derive(Clone, Debug)]
pub struct Lake {
    pub cells: Vec<Pos>,
    pub direction: (i32, i32),
}
