// Ядро игры «Лабиринт» 20×20 — чистая логика, без веба.
// Перенесено 1-в-1 из исходного однофайлового index.html (JS).
// Время передаётся параметрами (now_ms) — детерминированные тесты.
use std::collections::HashSet;

pub(crate) mod bfs;
pub(crate) mod consts;
mod end;
mod enemies;
mod gen;
pub(crate) mod i18n;
mod player;
pub(crate) mod rng;
pub(crate) mod stats;
pub(crate) mod types;
mod visibility;

pub use bfs::*;
pub use consts::*;
pub use i18n::*;
pub use rng::*;
pub use stats::*;
pub use types::*;

// ---------- Игра ----------
pub struct Game {
    pub lang: Lang,
    pub rng: Rng,
    pub seed: u32,
    pub wall: [[[bool; 4]; W]; H],
    pub player: Pos,
    pub chest: Pos,
    pub exit: Pos,
    pub key: Option<Pos>,
    pub lakes: Vec<Lake>,
    pub holes: Vec<Pos>,
    pub arsenal: Vec<Pos>,
    pub hospital: Pos,
    pub trap: Vec<Pos>,
    pub bears: Vec<Pos>,
    pub fire: Vec<Pos>,
    pub zombies: Vec<Pos>,
    pub shooters: Vec<Pos>,
    pub bosses: Vec<Pos>,
    pub swamps: Vec<Pos>,
    pub health: i32,
    pub bombs: i32,
    pub carrying: bool,
    pub carrying_key: bool,
    pub over: bool,
    pub over_kind: Option<OverKind>,
    pub moves: u32,
    pub level: u32,
    pub time_left: i32,
    pub skip_move: i32,
    pub aiming: bool,
    pub paused: bool,
    pub help_open: bool,
    pub score: i32,
    pub combo: i32,
    pub combo_until: u64,
    pub took_damage: bool,
    pub dash_until: u64,
    pub move_lock_until: u64,
    pub grace_until: u64,
    pub hurt_until: u64,
    pub vis: Vec<Vec<bool>>,
    pub explored: Vec<Vec<bool>>,
    pub shooter_aim: HashSet<Pos>,
    pub boss_aim: HashSet<Pos>,
    pub shots: Vec<Shot>,
    pub boom: Option<Fx>,
    pub dash_flash: Option<Fx>,
    pub river_on: bool,
    pub shake: f64,
    pub now_ms: u64,
    pub msg: String,
    pub log: String,
    pub stats: Stats,
    pub best: Best,
    pub sfx: Vec<Sfx>,
    pub new_ach: Vec<String>,
}

impl Game {
    pub fn t(&self, key: &str) -> String {
        t(self.lang, key)
    }
    pub fn say(&mut self, s: String) {
        self.msg = s;
    }
    pub fn sfx(&mut self, s: Sfx) {
        self.sfx.push(s);
    }

    pub fn new(seed: u32, lang: Lang, stats: Stats, best: Best) -> Game {
        let mut g = Game {
            lang,
            rng: Rng::new(seed),
            seed,
            wall: [[[true; 4]; W]; H],
            player: (0, 0),
            chest: (0, 0),
            exit: (0, 0),
            key: None,
            lakes: vec![],
            holes: vec![],
            arsenal: vec![],
            hospital: (0, 0),
            trap: vec![],
            bears: vec![],
            fire: vec![],
            zombies: vec![],
            shooters: vec![],
            bosses: vec![],
            swamps: vec![],
            health: MAX_HEALTH,
            bombs: MAX_BOMBS,
            carrying: false,
            carrying_key: false,
            over: false,
            over_kind: None,
            moves: 0,
            level: 1,
            time_left: LEVEL_TIME,
            skip_move: 0,
            aiming: false,
            paused: false,
            help_open: false,
            score: 0,
            combo: 0,
            combo_until: 0,
            took_damage: false,
            dash_until: 0,
            move_lock_until: 0,
            grace_until: 0,
            hurt_until: 0,
            vis: vec![vec![false; W]; H],
            explored: vec![vec![false; W]; H],
            shooter_aim: HashSet::new(),
            boss_aim: HashSet::new(),
            shots: vec![],
            boom: None,
            dash_flash: None,
            river_on: false,
            shake: 0.0,
            msg: String::new(),
            log: String::new(),
            stats,
            best,
            sfx: vec![],
            new_ach: vec![],
            now_ms: 0,
        };
        g.init_world(now0(), false);
        g
    }

    // (re)генерация мира; now — текущий ms. new_world=true: новый случайный сид (N)
    pub fn init_world(&mut self, now: u64, new_world: bool) {
        if new_world {
            // N без URL-сида в JS: новый случайный сид; здесь детерминированный шаг LCG
            self.seed = self.seed.wrapping_mul(1664525).wrapping_add(1013904223);
            if self.seed == 0 {
                self.seed = 0x9E3779B9;
            }
            self.rng = Rng::new(self.seed);
        }
        // R: rng реинициализируется тем же сидом → тот же лабиринт
        if !new_world {
            self.rng = Rng::new(self.seed);
        }
        self.over = false;
        self.over_kind = None;
        self.paused = false;
        self.help_open = false;
        self.aiming = false;
        self.shooter_aim.clear();
        self.boss_aim.clear();
        self.shots.clear();
        self.boom = None;
        self.dash_flash = None;
        self.wall = self.gen_walls();
        self.ensure_connected();
        self.lakes = self.gen_lakes();
        self.ensure_lake_open();
        self.swamps = self.gen_swamps();
        {
            let sset: HashSet<Pos> = self.swamps.iter().copied().collect();
            for &(r, c) in &self.swamps {
                for (d, dir) in DIRS.iter().enumerate() {
                    if let Some((nr, nc)) = add((r, c), *dir) {
                        if sset.contains(&(nr, nc)) {
                            self.wall[r][c][d] = false;
                            self.wall[nr][nc][d ^ 1] = false;
                        }
                    }
                }
            }
        }
        let lake_set: HashSet<Pos> = self
            .lakes
            .iter()
            .flat_map(|l| l.cells.iter().copied())
            .collect();
        let swamp_set: HashSet<Pos> = self.swamps.iter().copied().collect();
        let mut cells: Vec<Pos> = vec![];
        for r in 0..H {
            for c in 0..W {
                if !lake_set.contains(&(r, c)) && !swamp_set.contains(&(r, c)) {
                    cells.push((r, c));
                }
            }
        }
        self.rng.shuffle(&mut cells);
        self.player = cells[0];
        self.exit = cells[1];
        self.chest = cells[2];
        self.key = Some(cells[6]);
        self.holes = cells[7..11].to_vec();
        self.arsenal = cells[11..13].to_vec();
        self.hospital = cells[13];
        let extra_trap = (self.level - 1).min(3) as usize;
        let extra_bear = (self.level - 1).min(29) as usize;
        let extra_fire = (self.level - 1).min(3) as usize;
        self.trap = cells[14..14 + 2 + extra_trap].to_vec();
        let mut bears = vec![cells[19]];
        bears.extend_from_slice(&cells[20..20 + extra_bear]);
        self.bears = bears;
        self.fire = cells[24..24 + extra_fire].to_vec();
        self.zombies = cells[34..44].to_vec();
        self.shooters = cells[64..71].to_vec();
        self.bosses = cells[74..75].to_vec();
        let player = self.player;
        let far = |c: Pos| manhattan(c, player) > 3;
        let mut occupied: HashSet<Pos> = HashSet::new();
        for c in [self.player, self.exit, self.chest]
            .into_iter()
            .chain(self.key.iter().copied())
            .chain(self.holes.iter().copied())
            .chain(self.arsenal.iter().copied())
            .chain([self.hospital])
        {
            occupied.insert(c);
        }
        let take = |list: Vec<Pos>, occupied: &mut HashSet<Pos>| -> Vec<Pos> {
            list.into_iter()
                .filter(|&c| {
                    if occupied.contains(&c) {
                        return false;
                    }
                    occupied.insert(c);
                    far(c)
                })
                .collect()
        };
        self.trap = take(std::mem::take(&mut self.trap), &mut occupied);
        self.fire = take(std::mem::take(&mut self.fire), &mut occupied);
        self.bears = take(std::mem::take(&mut self.bears), &mut occupied);
        self.zombies = take(std::mem::take(&mut self.zombies), &mut occupied);
        self.shooters = take(std::mem::take(&mut self.shooters), &mut occupied);
        self.bosses = take(std::mem::take(&mut self.bosses), &mut occupied);
        // гарантия: ключ/сундук не на спец-клетках
        let mut special: HashSet<Pos> = HashSet::new();
        for c in self
            .trap
            .iter()
            .chain(self.fire.iter())
            .chain(self.holes.iter())
            .chain(self.arsenal.iter())
        {
            special.insert(*c);
        }
        special.insert(self.hospital);
        for l in &self.lakes {
            for c in &l.cells {
                special.insert(*c);
            }
        }
        for c in &self.swamps {
            special.insert(*c);
        }
        let free_from =
            |cur: Pos, special: &HashSet<Pos>, cells: &Vec<Pos>, player: Pos, exit: Pos| -> Pos {
                if !special.contains(&cur) {
                    return cur;
                }
                for &c in cells {
                    if !special.contains(&c) && c != player && c != exit {
                        return c;
                    }
                }
                cur
            };
        if let Some(k) = self.key {
            self.key = Some(free_from(k, &special, &cells, self.player, self.exit));
        }
        self.chest = free_from(self.chest, &special, &cells, self.player, self.exit);
        self.health = MAX_HEALTH;
        self.bombs = MAX_BOMBS;
        self.carrying = false;
        self.carrying_key = false;
        self.move_lock_until = 0;
        self.dash_until = 0;
        self.hurt_until = 0;
        self.score = 0;
        self.combo = 0;
        self.combo_until = 0;
        self.took_damage = false;
        self.grace_until = now + GRACE_MS;
        self.skip_move = 0;
        self.moves = 0;
        self.time_left = LEVEL_TIME;
        self.river_on = false;
        self.shake = 0.0;
        self.explored = vec![vec![false; W]; H];
        self.compute_visibility();
        self.mark_explored();
        let lv = self.level;
        self.log = format!("{}{}{}", self.t("w_reset1"), lv, self.t("w_reset1b"));
    }

    // «Заново» (R / кнопка под картой): рестарт ТЕКУЩЕГО уровня — тот же мир, тот же уровень.
    // Вызов после победы идёт не сюда, а в new_world (см. web::restart).
    pub fn restart(&mut self, now: u64) {
        self.init_world(now, false);
    }
    pub fn new_world(&mut self, now: u64) {
        self.init_world(now, true);
    }
}

fn now0() -> u64 {
    0
}

pub fn parse_seed_from_query(query: &str) -> Option<u32> {
    for part in query.trim_start_matches('?').split('&') {
        if let Some(v) = part.strip_prefix("seed=") {
            if let Ok(n) = v.parse::<u32>() {
                return Some(n);
            }
        }
    }
    None
}
#[allow(dead_code)]
pub fn is_random_seed(seed: Option<u32>) -> bool {
    seed.is_none()
}
