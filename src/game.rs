// Ядро игры «Лабиринт» 20×20 — чистая логика, без веба.
// Перенесено 1-в-1 из исходного однофайлового index.html (JS).
// Время передаётся параметрами (now_ms) — детерминированные тесты.
use std::collections::HashSet;

// ---------- Константы баланса (JS: CFG) ----------
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
    if dr == -1 { 0 } else if dr == 1 { 1 } else if dc == -1 { 2 } else { 3 }
}
pub fn in_bounds(r: i32, c: i32) -> bool {
    r >= 0 && r < H as i32 && c >= 0 && c < W as i32
}
pub fn add(p: Pos, d: (i32, i32)) -> Option<Pos> {
    let r = p.0 as i32 + d.0;
    let c = p.1 as i32 + d.1;
    if in_bounds(r, c) { Some((r as usize, c as usize)) } else { None }
}
pub fn manhattan(a: Pos, b: Pos) -> i32 {
    (a.0 as i32 - b.0 as i32).abs() + (a.1 as i32 - b.1 as i32).abs()
}

// ---------- Сидированный RNG (mulberry32, как в JS) ----------
#[derive(Clone, Debug)]
pub struct Rng(u32);
impl Rng {
    pub fn new(seed: u32) -> Self { Rng(seed) }
    pub fn next_f64(&mut self) -> f64 {
        let mut a = self.0;
        a = a.wrapping_add(0x6D2B79F5);
        self.0 = a;
        let mut t = (a ^ (a >> 15)).wrapping_mul(1 | a);
        t = t.wrapping_add((t ^ (t >> 7)).wrapping_mul(61 | t)) ^ t;
        ((t ^ (t >> 14)) as u32 as f64) / 4294967296.0
    }
    pub fn next_usize(&mut self, max: usize) -> usize {
        (self.next_f64() * max as f64) as usize
    }
    pub fn shuffle<T>(&mut self, a: &mut [T]) {
        for i in (1..a.len()).rev() {
            let j = self.next_usize(i + 1);
            a.swap(i, j);
        }
    }
}

// ---------- Локализация RU/EN (тексты 1-в-1 из JS I18N) ----------
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Lang { Ru, En }
impl Lang {
    pub fn other(self) -> Lang { match self { Lang::Ru => Lang::En, Lang::En => Lang::Ru } }
    pub fn code(self) -> &'static str { match self { Lang::Ru => "ru", Lang::En => "en" } }
    pub fn from_code(s: &str) -> Option<Lang> {
        match s { "ru" => Some(Lang::Ru), "en" => Some(Lang::En), _ => None }
    }
}

pub const RU: &[(&str, &str)] = &[
    ("hudHealth","Здоровье"),("hudBombs","Бомбы"),("hudChest","Клад"),("hudKey","Ключ"),
    ("hudLevel","Уровень"),("hudMoves","Ходы"),("hudTime","Время"),("hudScore","Очки"),("hudCombo","Комбо"),("hudDash","Рывок"),
    ("logHint","Стрелки / WASD — ход · Shift+стрелка — рывок · B — бомба · R — заново · N — новый мир · P — пауза"),
    ("statusStart","Найди сундук и вынеси его через выход."),("title","Лабиринт"),
    ("leg","<span>🧙 игрок</span><span>💰 сундук</span><span>🚩 выход</span><span>🗝️ ключ</span><span>💉 госпиталь</span><span>📦 арсенал</span><span>🌀 озеро</span><span>🕳 дырка</span><span>🪷 болото</span><span>🐻 медведь</span><span>⚠️ ловушка</span><span>🔥 огонь</span><span>🧟 зомби</span><span>👮 стрелок</span><span>🥷 босс</span><span><span class=\"k\">B</span> — взорвать стену · <span class=\"k\">R</span> — заново</span>"),
    ("btnRestart","Заново (R)"),("btnNewWorld","🎲 Новый мир (N)"),("btnReplay","Играть снова"),("btnPause","⏸"),("btnResume","Продолжить (P)"),
    ("helpTitle","Помощь"),
    ("help1","<b>Цель:</b> найди сундук 💰, возьми его и вынеси через выход 🚩."),
    ("help2","<b>Управление:</b> стрелки / WASD — ход · <span class=\"k\">Shift</span>+стрелка — рывок через клетку · <span class=\"k\">B</span> — бомба (стена) · <span class=\"k\">R</span> — заново · <span class=\"k\">H</span> — помощь"),
    ("help3","<b>Спец-клетки:</b> 💉 госпиталь и 📦 арсенал восстанавливают здоровье и бомбы · 🌀 озеро несёт игрока течением каждые 2с · 🕳 дырка выкидывает из другой дырки · 🪷 болото — задерживает на 3 хода · ⚠️ ловушка ранит · 🐻 медведь, 🧟 зомби, 👮 стрелки и 🥷 босс ходят по своим таймерам (босс всегда знает, где ты)."),
    ("help4","<b>Прочее:</b> <span class=\"k\">P</span> — пауза · быстрые ходы подряд дают <b>очки комбо</b> · на телефоне — кнопки на экране · сид лабиринта: <span class=\"k\">?seed=123</span> или дата <span class=\"k\">?seed=20261106</span>."),
    ("w_river","Течение озера несёт тебя."),("w_hole","Дырка — провал! Ты выходишь из другой дырки."),
    ("w_arsenal","Арсенал! Здоровье и бомбы восстановлены (5)."),("w_hospital","Госпиталь! Здоровье восстановлено."),
    ("w_trap","Ловушка! Ты ранен. Здоровье: "),("w_trapSafe","Осторожно — здесь была ловушка, но ты не сработал её."),
    ("w_fire","Огненная ловушка! Ты обожжён. Здоровье: "),("w_key","Ключ взят! Теперь можно открыть дверь к выходу."),
    ("w_chest","Ты взял сундук! Отнеси его к выходу (🚩)."),("w_door","Дверь закрыта. Нужен ключ 🗝️."),
    ("w_exit","Это выход. Но ты без клада. Вернись за сундуком."),
    ("w_bear","Медведь настиг тебя! Ранен. Здоровье: "),("w_zombie","Зомби укусил! Здоровье: "),
    ("w_boss","Охотник-босс подстрелил тебя! Здоровье: "),("w_bossMelee","Охотник-босс настиг тебя! Тяжёлое ранение. Здоровье: "),
    ("w_shooter","Стрелок ранил тебя выстрелом! Здоровье: "),("w_dropChest","Клад выпал и остался тут."),
    ("w_dropChestBack","Сундук вернулся на исходную клетку."),
    ("w_swampSkip1","Ты завяз в болоте — пропускаешь ход ("),("w_swampSkip2"," осталось)."),
    ("ach_first","Первый выход!"),("ach_combo","Серия из 10 быстрых ходов!"),("ach_explorer","Разведал >=150 клеток!"),("ach_speed","Победа за 90 секунд!"),("ach_clean","Победа без урона!"),
    ("achUnlock","🏅 Достижение: "),
    ("pausedTitle","⏸ ПАУЗА"),
    ("w_dropChestTrap","Клад остался на месте ранения."),("w_noBombs","Нет бомб! Найди арсенал."),
    ("w_wall","Стена!"),("w_dashCd","Рывок ещё не готов."),("w_dashWall","За стеной прохода нет — рывок не прошёл."),("w_dash","Рывок!"),
    ("w_swamp","Болото! Ноги увязли — пропускаешь "),("w_swamp2"," ход"),("w_swamp3"," хода"),
    ("w_edge","За краем стена — взрывать нечего."),("w_stone","Каменная стена — её не взорвать. Заряд израсходован."),
    ("w_boom","Бум! Стена "),("w_boom2"," взорвана. Бомб осталось: "),("w_noWall","Здесь нет стены."),("w_aim","Выбери направление взрыва (стрелка / WASD)."),
    ("w_winMsg","🏆 ПОБЕДА! Ты вынес сундук из лабиринта."),("winTitle","🏆 ПОБЕДА"),
    ("loseTitle","💀 ПОРАЖЕНИЕ"),("timeoutTitle","⏰ ВРЕМЯ ВЫШЛО"),
    ("w_winStats","Уровень пройден за <b>"),("w_winStats2","</b> ходов · "),("w_newLevel","<br>Новый уровень: <b>"),("w_newLevel2","</b> — сложность растёт"),
    ("w_loseMsg","💀 Ты погиб. Игра окончена."),("timeoutMsg","⏰ Время вышло!"),
    ("w_timeStats","</b> ходов · "),("w_loseStats","Достигнуто: <b>"),("w_resetLevel","<br>Уровень сброшен до 1"),
    ("w_timeoutStats","Достигнуто: <b>"),("w_resetLevel2","Уровень сброшен до 1"),
    ("w_reset1","Уровень "),("w_reset1b",". У тебя нет карты. Ходи с помощью стрелок, изучай лабиринт."),
    ("w_newLevelR","! Сложность растёт. Нажми R, чтобы сыграть ещё."),("w_R","Нажми R, чтобы сыграть ещё."),
    ("secs","с"),
    ("record","Рекорд: уровень <b>"),("record2","</b>"),("bestL1"," · лучшее время L1 <b>"),
];
pub const EN: &[(&str, &str)] = &[
    ("hudHealth","Health"),("hudBombs","Bombs"),("hudChest","Chest"),("hudKey","Key"),
    ("hudLevel","Level"),("hudMoves","Moves"),("hudTime","Time"),("hudScore","Score"),("hudCombo","Combo"),("hudDash","Dash"),
    ("logHint","Arrows / WASD — move · Shift+arrow — dash · B — bomb · R — restart · N — new maze · P — pause"),
    ("statusStart","Find the chest and carry it out through the exit."),("title","labyrinth"),
    ("leg","<span>🧙 player</span><span>💰 chest</span><span>🚩 exit</span><span>🗝️ key</span><span>💉 hospital</span><span>📦 arsenal</span><span>🌀 lake</span><span>🕳 hole</span><span>🪷 swamp</span><span>🐻 bear</span><span>⚠️ trap</span><span>🔥 fire</span><span>🧟 zombie</span><span>👮 shooter</span><span>🥷 boss</span><span><span class=\"k\">B</span> — destroy wall · <span class=\"k\">R</span> — restart</span>"),
    ("btnRestart","Restart (R)"),("btnNewWorld","🎲 New maze (N)"),("btnReplay","Play again"),("btnPause","⏸"),("btnResume","Resume (P)"),
    ("helpTitle","Help"),
    ("help1","<b>Goal:</b> find the chest 💰, pick it up and carry it through the exit 🚩."),
    ("help2","<b>Controls:</b> arrows / WASD — move · <span class=\"k\">Shift</span>+arrow — dash one cell · <span class=\"k\">B</span> — bomb (wall) · <span class=\"k\">R</span> — restart · <span class=\"k\">H</span> — help"),
    ("help3","<b>Special cells:</b> 💉 hospital and 📦 arsenal restore health and bombs · 🌀 lake carries you every 2s · 🕳 hole drops you out of another hole · 🪷 swamp — keeps you for 3 moves · ⚠️ trap hurts · 🐻 bear, 🧟 zombie, 👮 shooter and 🥷 boss move on their own timers (the boss always knows where you are)."),
    ("help4","<b>More:</b> <span class=\"k\">P</span> — pause · fast consecutive moves build a <b>combo score</b> · on touch devices use the on-screen pad · maze seed: <span class=\"k\">?seed=123</span> or a date <span class=\"k\">?seed=20261106</span>."),
    ("w_river","The lake carries you away."),("w_hole","Hole — drop! You exit from another hole."),
    ("w_arsenal","Arsenal! Health and bombs restored (5)."),("w_hospital","Hospital! Health restored."),
    ("w_trap","Trap! You got hurt. Health: "),("w_trapSafe","Careful — there was a trap here, but you did not trigger it."),
    ("w_fire","Fire trap! You got burned. Health: "),("w_key","Key picked up! Now you can open the exit door."),
    ("w_chest","You picked up the chest! Carry it to the exit (🚩)."),("w_door","Door is locked. You need the key 🗝️."),
    ("w_exit","This is the exit. But you have no chest. Go back for it."),
    ("w_bear","A bear caught you! Hurt. Health: "),("w_zombie","A zombie bit you! Health: "),
    ("w_boss","The hunter-boss shot you! Health: "),("w_bossMelee","The hunter-boss caught you! Heavy wound. Health: "),
    ("w_shooter","The shooter wounded you! Health: "),("w_dropChest","The chest dropped and stayed here."),
    ("w_dropChestBack","The chest returned to its original cell."),
    ("w_swampSkip1","You are stuck in the swamp — skipping a move ("),("w_swampSkip2"," left)."),
    ("ach_first","First escape!"),("ach_combo","10-move streak!"),("ach_explorer","Explored >=150 cells!"),("ach_speed","Win in 90 seconds!"),("ach_clean","Win without taking damage!"),
    ("achUnlock","🏅 Achievement: "),
    ("pausedTitle","⏸ PAUSED"),
    ("w_dropChestTrap","The chest stayed where you were hurt."),("w_noBombs","No bombs! Find an arsenal."),
    ("w_wall","Wall!"),("w_dashCd","Dash not ready."),("w_dashWall","No opening ahead — dash failed."),("w_dash","Dash!"),
    ("w_swamp","Swamp! Your feet are stuck — you skip "),("w_swamp2"," move"),("w_swamp3"," moves"),
    ("w_edge","It is a wall at the edge — nothing to blow up."),("w_stone","Stone wall — cannot be blown up. Charge consumed."),
    ("w_boom","Boom! Wall "),("w_boom2"," is destroyed. Bombs left: "),("w_noWall","No wall here."),("w_aim","Choose a bomb direction (arrow / WASD)."),
    ("w_winMsg","🏆 VICTORY! You carried the chest out of the maze."),("winTitle","🏆 VICTORY"),
    ("loseTitle","💀 DEFEAT"),("timeoutTitle","⏰ TIME OUT"),
    ("w_winStats","Level beaten in <b>"),("w_winStats2","</b> moves · "),("w_newLevel","<br>New level: <b>"),("w_newLevel2","</b> — difficulty rises"),
    ("w_loseMsg","💀 You died. Game over."),("timeoutMsg","⏰ Time out!"),
    ("w_timeStats","</b> moves · "),("w_loseStats","Reached: <b>"),("w_resetLevel","<br>Level reset to 1"),
    ("w_timeoutStats","Reached: <b>"),("w_resetLevel2","Level reset to 1"),
    ("w_reset1","Level "),("w_reset1b",". You have no map. Use the arrows, explore the maze."),
    ("w_newLevelR","! Difficulty rises. Press R to play again."),("w_R","Press R to play again."),
    ("secs","s"),
    ("record","Record: level <b>"),("record2","</b>"),("bestL1"," · best time L1 <b>"),
];
pub fn t(lang: Lang, key: &str) -> String {
    let table = match lang { Lang::Ru => RU, Lang::En => EN };
    for (k, v) in table { if *k == key { return v.to_string(); } }
    for (k, v) in RU { if *k == key { return v.to_string(); } }
    key.to_string()
}
pub fn dir_name(lang: Lang, d: usize) -> &'static str {
    match (lang, d) {
        (Lang::Ru, 0) => "север", (Lang::Ru, 1) => "юг", (Lang::Ru, 2) => "запад", (Lang::Ru, 3) => "восток",
        (Lang::En, 0) => "north", (Lang::En, 1) => "south", (Lang::En, 2) => "west", _ => "east",
    }
}

// ---------- События для веба (звук/частицы/тряска) ----------
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(dead_code)]
pub enum Sfx { Move, Step, EsStep, Wall, Chest, Bomb, Bear, Heal, River, Hole, Key, Shoot, Danger, Win, Lose, Aim, Tick, Ach }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OverKind { Win, Lose, Timeout }
#[derive(Clone, Debug)]
pub struct Shot { pub from: Pos, pub to: Pos, pub t0: u64 }
#[derive(Clone, Copy, Debug)]
pub struct Fx { pub pos: Pos, pub t0: u64 }

#[derive(Clone, Debug)]
pub struct Lake { pub cells: Vec<Pos>, pub direction: (i32, i32) }

#[derive(Clone, Default, Debug)]
pub struct Stats {
    pub wins: u32, pub best_combo: u32, pub explored_max: u32,
    pub speedrun: bool, pub clean_win: bool, pub unlocked: Vec<String>,
}
#[derive(Clone, Copy, Default, Debug)]
pub struct Best { pub level: u32, pub time1: Option<f64> }

#[allow(dead_code)]
pub const ACH_IDS: [&str; 5] = ["first", "combo10", "explorer", "speedrun", "clean"];
pub fn ach_name(lang: Lang, id: &str) -> String {
    match id {
        "first" => t(lang, "ach_first"), "combo10" => t(lang, "ach_combo"),
        "explorer" => t(lang, "ach_explorer"), "speedrun" => t(lang, "ach_speed"),
        "clean" => t(lang, "ach_clean"), _ => id.to_string(),
    }
}
impl Stats {
    pub fn ach_unlocked(&self, id: &str) -> bool { self.unlocked.iter().any(|x| x == id) }
    // Возвращает newly unlocked id (проверка как в JS checkAchievements)
    pub fn check_achievements(&mut self) -> Vec<String> {
        let mut out = Vec::new();
        let checks: [(&str, bool); 5] = [
            ("first", self.wins >= 1),
            ("combo10", self.best_combo >= 10),
            ("explorer", self.explored_max >= 150),
            ("speedrun", self.speedrun),
            ("clean", self.clean_win),
        ];
        for (id, ok) in checks {
            if ok && !self.ach_unlocked(id) {
                self.unlocked.push(id.to_string());
                out.push(id.to_string());
            }
        }
        out
    }
}

// ---------- BFS ----------
pub struct Bfs {
    pub dist: Vec<Vec<i32>>,
    pub prev: Vec<Vec<Option<Pos>>>,
}
pub fn bfs(wall: &[[[bool; 4]; W]; H], start: Pos, max_dist: Option<i32>) -> Bfs {
    let mut dist = vec![vec![-1i32; W]; H];
    let mut prev = vec![vec![None; W]; H];
    let mut q = vec![start];
    dist[start.0][start.1] = 0;
    let mut qi = 0;
    while qi < q.len() {
        let (r, c) = q[qi]; qi += 1;
        if let Some(m) = max_dist { if dist[r][c] >= m { continue; } }
        for d in 0..4 {
            if wall[r][c][d] { continue; }
            let nr = r as i32 + DIRS[d].0;
            let nc = c as i32 + DIRS[d].1;
            if !in_bounds(nr, nc) { continue; }
            let (nr, nc) = (nr as usize, nc as usize);
            if dist[nr][nc] != -1 { continue; }
            dist[nr][nc] = dist[r][c] + 1;
            prev[nr][nc] = Some((r, c));
            q.push((nr, nc));
        }
    }
    Bfs { dist, prev }
}

// ---------- Игра ----------
pub struct Game {
    pub lang: Lang,
    pub rng: Rng,
    pub seed: u32,
    pub wall: [[[bool; 4]; W]; H],
    pub player: Pos, pub chest: Pos, pub exit: Pos, pub key: Option<Pos>,
    pub lakes: Vec<Lake>, pub holes: Vec<Pos>, pub arsenal: Vec<Pos>, pub hospital: Pos,
    pub trap: Vec<Pos>, pub bears: Vec<Pos>, pub fire: Vec<Pos>, pub zombies: Vec<Pos>,
    pub shooters: Vec<Pos>, pub bosses: Vec<Pos>, pub swamps: Vec<Pos>,
    pub health: i32, pub bombs: i32, pub carrying: bool, pub carrying_key: bool,
    pub over: bool, pub over_kind: Option<OverKind>,
    pub moves: u32, pub level: u32, pub time_left: i32,
    pub skip_move: i32, pub aiming: bool, pub paused: bool, pub help_open: bool,
    pub score: i32, pub combo: i32, pub combo_until: u64, pub took_damage: bool,
    pub dash_until: u64, pub move_lock_until: u64, pub grace_until: u64, pub hurt_until: u64,
    pub vis: Vec<Vec<bool>>, pub explored: Vec<Vec<bool>>,
    pub shooter_aim: HashSet<Pos>, pub boss_aim: HashSet<Pos>,
    pub shots: Vec<Shot>, pub boom: Option<Fx>, pub dash_flash: Option<Fx>,
    pub river_on: bool, pub shake: f64,
    pub now_ms: u64,
    pub msg: String, pub log: String, pub stats: Stats, pub best: Best,
    pub sfx: Vec<Sfx>, pub new_ach: Vec<String>,
}

impl Game {
    pub fn t(&self, key: &str) -> String { t(self.lang, key) }
    pub fn say(&mut self, s: String) { self.msg = s; }
    pub fn sfx(&mut self, s: Sfx) { self.sfx.push(s); }

    pub fn new(seed: u32, lang: Lang, stats: Stats, best: Best) -> Game {
        let mut g = Game {
            lang, rng: Rng::new(seed), seed,
            wall: [[[true; 4]; W]; H],
            player: (0, 0), chest: (0, 0), exit: (0, 0), key: None,
            lakes: vec![], holes: vec![], arsenal: vec![], hospital: (0, 0),
            trap: vec![], bears: vec![], fire: vec![], zombies: vec![],
            shooters: vec![], bosses: vec![], swamps: vec![],
            health: MAX_HEALTH, bombs: MAX_BOMBS, carrying: false, carrying_key: false,
            over: false, over_kind: None, moves: 0, level: 1, time_left: LEVEL_TIME,
            skip_move: 0, aiming: false, paused: false, help_open: false,
            score: 0, combo: 0, combo_until: 0, took_damage: false,
            dash_until: 0, move_lock_until: 0, grace_until: 0, hurt_until: 0,
            vis: vec![vec![false; W]; H], explored: vec![vec![false; W]; H],
            shooter_aim: HashSet::new(), boss_aim: HashSet::new(),
            shots: vec![], boom: None, dash_flash: None, river_on: false, shake: 0.0,
            msg: String::new(), log: String::new(), stats, best, sfx: vec![], new_ach: vec![],
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
            if self.seed == 0 { self.seed = 0x9E3779B9; }
            self.rng = Rng::new(self.seed);
        }
        // R: rng реинициализируется тем же сидом → тот же лабиринт
        if !new_world { self.rng = Rng::new(self.seed); }
        self.over = false; self.over_kind = None;
        self.paused = false; self.help_open = false; self.aiming = false;
        self.shooter_aim.clear(); self.boss_aim.clear();
        self.shots.clear(); self.boom = None; self.dash_flash = None;
        self.wall = self.gen_walls();
        self.ensure_connected();
        self.lakes = self.gen_lakes();
        self.ensure_lake_open();
        self.swamps = self.gen_swamps();
        {
            let sset: HashSet<Pos> = self.swamps.iter().copied().collect();
            for &(r, c) in &self.swamps {
                for d in 0..4 {
                    if let Some((nr, nc)) = add((r, c), DIRS[d]) {
                        if sset.contains(&(nr, nc)) {
                            self.wall[r][c][d] = false;
                            self.wall[nr][nc][d ^ 1] = false;
                        }
                    }
                }
            }
        }
        let lake_set: HashSet<Pos> = self.lakes.iter().flat_map(|l| l.cells.iter().copied()).collect();
        let swamp_set: HashSet<Pos> = self.swamps.iter().copied().collect();
        let mut cells: Vec<Pos> = vec![];
        for r in 0..H { for c in 0..W { if !lake_set.contains(&(r, c)) && !swamp_set.contains(&(r, c)) { cells.push((r, c)); } } }
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
        for c in [self.player, self.exit, self.chest].into_iter().chain(self.key.iter().copied())
            .chain(self.holes.iter().copied()).chain(self.arsenal.iter().copied()).chain([self.hospital].into_iter()) {
            occupied.insert(c);
        }
        let take = |list: Vec<Pos>, occupied: &mut HashSet<Pos>| -> Vec<Pos> {
            list.into_iter().filter(|&c| { if occupied.contains(&c) { return false; } occupied.insert(c); far(c) }).collect()
        };
        self.trap = take(std::mem::take(&mut self.trap), &mut occupied);
        self.fire = take(std::mem::take(&mut self.fire), &mut occupied);
        self.bears = take(std::mem::take(&mut self.bears), &mut occupied);
        self.zombies = take(std::mem::take(&mut self.zombies), &mut occupied);
        self.shooters = take(std::mem::take(&mut self.shooters), &mut occupied);
        self.bosses = take(std::mem::take(&mut self.bosses), &mut occupied);
        // гарантия: ключ/сундук не на спец-клетках
        let mut special: HashSet<Pos> = HashSet::new();
        for c in self.trap.iter().chain(self.fire.iter()).chain(self.holes.iter()).chain(self.arsenal.iter()) { special.insert(*c); }
        special.insert(self.hospital);
        for l in &self.lakes { for c in &l.cells { special.insert(*c); } }
        for c in &self.swamps { special.insert(*c); }
        let free_from = |cur: Pos, special: &HashSet<Pos>, cells: &Vec<Pos>, player: Pos, exit: Pos| -> Pos {
            if !special.contains(&cur) { return cur; }
            for &c in cells { if !special.contains(&c) && c != player && c != exit { return c; } }
            cur
        };
        if let Some(k) = self.key { self.key = Some(free_from(k, &special, &cells, self.player, self.exit)); }
        self.chest = free_from(self.chest, &special, &cells, self.player, self.exit);
        self.health = MAX_HEALTH; self.bombs = MAX_BOMBS;
        self.carrying = false; self.carrying_key = false;
        self.move_lock_until = 0; self.dash_until = 0; self.hurt_until = 0;
        self.score = 0; self.combo = 0; self.combo_until = 0; self.took_damage = false;
        self.grace_until = now + GRACE_MS;
        self.skip_move = 0; self.moves = 0; self.time_left = LEVEL_TIME;
        self.river_on = false; self.shake = 0.0;
        self.explored = vec![vec![false; W]; H];
        self.compute_visibility();
        self.mark_explored();
        let lv = self.level;
        self.log = format!("{}{}{}", self.t("w_reset1"), lv, self.t("w_reset1b"));
    }

    // «Заново» (R / кнопка посреди уровня): бросить уровень, начать с 1-го, тот же мир.
    // Вызов после победы идёт не сюда, а в new_world — см. web::restart.
    pub fn restart(&mut self, now: u64) { self.level = 1; self.init_world(now, false); }
    pub fn new_world(&mut self, now: u64) { self.init_world(now, true); }

    fn gen_walls(&mut self) -> [[[bool; 4]; W]; H] {
        let mut w = [[[true; 4]; W]; H];
        for r in 0..H {
            for c in 0..W {
                w[r][c][1] = r == H - 1 || self.rng.next_f64() < 0.30;
                w[r][c][3] = c == W - 1 || self.rng.next_f64() < 0.30;
                if r > 0 { w[r][c][0] = w[r - 1][c][1]; }
                if c > 0 { w[r][c][2] = w[r][c - 1][3]; }
            }
        }
        w
    }
    pub fn cell_count(&self) -> usize {
        let b = bfs(&self.wall, (0, 0), None);
        b.dist.iter().flat_map(|r| r.iter()).filter(|&&d| d != -1).count()
    }
    pub fn reachable(&self) -> Vec<Vec<bool>> {
        let b = bfs(&self.wall, (0, 0), None);
        b.dist.iter().map(|row| row.iter().map(|&d| d != -1).collect()).collect()
    }
    fn ensure_connected(&mut self) {
        for _ in 0..600 {
            if self.cell_count() == H * W { return; }
            let mark = self.reachable();
            'outer: for r in 0..H {
                for c in 0..W {
                    if mark[r][c] { continue; }
                    for &d in &DIRS {
                        let nr = r as i32 + d.0; let nc = c as i32 + d.1;
                        if !in_bounds(nr, nc) || !mark[nr as usize][nc as usize] { continue; }
                        let di = dir_index(d.0, d.1);
                        self.wall[r][c][di] = false;
                        self.wall[nr as usize][nc as usize][di ^ 1] = false;
                        break 'outer;
                    }
                }
            }
        }
    }
    fn gen_lakes(&mut self) -> Vec<Lake> {
        let mut used = vec![vec![false; W]; H];
        let mut res: Vec<Lake> = vec![];
        let n_lakes = 2 + self.rng.next_usize(2);
        let mut attempts = 0;
        while res.len() < n_lakes && attempts < 200 {
            attempts += 1;
            let size = 1 + self.rng.next_usize(10);
            let sr = self.rng.next_usize(H);
            let sc = self.rng.next_usize(W);
            if used[sr][sc] { continue; }
            let mut cluster = vec![(sr, sc)];
            used[sr][sc] = true;
            let mut grown = 1;
            while grown < size {
                let pool = cluster[self.rng.next_usize(cluster.len())];
                let dd = DIRS[self.rng.next_usize(4)];
                let nr = pool.0 as i32 + dd.0; let nc = pool.1 as i32 + dd.1;
                if !in_bounds(nr, nc) || used[nr as usize][nc as usize] { continue; }
                used[nr as usize][nc as usize] = true;
                cluster.push((nr as usize, nc as usize));
                grown += 1;
            }
            let direction = DIRS[self.rng.next_usize(4)];
            res.push(Lake { cells: cluster, direction });
        }
        if res.is_empty() { res.push(Lake { cells: vec![(1, 1)], direction: (0, 1) }); }
        res
    }
    fn ensure_lake_open(&mut self) {
        let lake_set: HashSet<Pos> = self.lakes.iter().flat_map(|l| l.cells.iter().copied()).collect();
        let cells: Vec<Pos> = self.lakes.iter().flat_map(|l| l.cells.iter().copied()).collect();
        for (r, c) in cells {
            for d in 0..4 {
                if let Some(n) = add((r, c), DIRS[d]) {
                    if lake_set.contains(&n) {
                        self.wall[r][c][d] = false;
                        self.wall[n.0][n.1][d ^ 1] = false;
                    }
                }
            }
        }
    }
    fn gen_swamps(&mut self) -> Vec<Pos> {
        let lake_set: HashSet<Pos> = self.lakes.iter().flat_map(|l| l.cells.iter().copied()).collect();
        let mut used = vec![vec![false; W]; H];
        for &(rr, cc) in &lake_set { used[rr][cc] = true; }
        let mut res: Vec<Pos> = vec![];
        let n_swamp = self.rng.next_usize(2) + 1;
        let mut attempts = 0;
        while res.len() < n_swamp && attempts < 200 {
            attempts += 1;
            let size = self.rng.next_usize(4) + 2;
            let sr = self.rng.next_usize(H); let sc = self.rng.next_usize(W);
            if used[sr][sc] { continue; }
            let mut cluster = vec![(sr, sc)];
            used[sr][sc] = true;
            let mut grown = 1;
            while grown < size {
                let pool = cluster[self.rng.next_usize(cluster.len())];
                let dd = DIRS[self.rng.next_usize(4)];
                let nr = pool.0 as i32 + dd.0; let nc = pool.1 as i32 + dd.1;
                if !in_bounds(nr, nc) || used[nr as usize][nc as usize] { continue; }
                used[nr as usize][nc as usize] = true;
                cluster.push((nr as usize, nc as usize));
                grown += 1;
            }
            res.extend(cluster);
        }
        res
    }

    pub fn compute_visibility(&mut self) {
        let b = bfs(&self.wall, self.player, Some(FOG_RADIUS));
        self.vis = b.dist.iter().map(|row| row.iter().map(|&d| d != -1).collect()).collect();
    }
    pub fn visible(&self, r: usize, c: usize) -> bool { self.vis[r][c] }
    pub fn mark_explored(&mut self) {
        for r in 0..H { for c in 0..W { if self.vis[r][c] { self.explored[r][c] = true; } } }
    }
    pub fn count_explored(&self) -> u32 {
        self.explored.iter().flat_map(|r| r.iter()).filter(|&&v| v).count() as u32
    }
    pub fn lake_of(&self, p: Pos) -> Option<&Lake> {
        self.lakes.iter().find(|l| l.cells.contains(&p))
    }
    pub fn in_lake(&self, p: Pos) -> bool { self.lake_of(p).is_some() }

    pub fn try_move(&mut self, dr: i32, dc: i32, now: u64) {
        self.now_ms = now;
        if self.over || self.paused || self.help_open { return; }
        if now < self.move_lock_until { return; }
        if self.skip_move > 0 {
            self.move_lock_until = now + MOVE_CD;
            self.skip_move -= 1;
            let m = format!("{}{}{}", self.t("w_swamp"), self.skip_move, self.t("w_swamp2"));
            self.say(m);
            return;
        }
        let (r, c) = self.player;
        if self.wall[r][c][dir_index(dr, dc)] {
            self.combo = 0; self.combo_until = 0;
            self.say(self.t("w_wall").to_string());
            self.sfx(Sfx::Wall);
            return;
        }
        self.move_lock_until = now + MOVE_CD;
        self.player = ((r as i32 + dr) as usize, (c as i32 + dc) as usize);
        self.compute_visibility();
        self.mark_explored();
        self.sfx(Sfx::Step);
        self.on_enter(self.player);
        if !self.over { self.moves += 1; }
        if now <= self.combo_until { self.combo += 1; } else { self.combo = 1; }
        self.combo_until = now + COMBO_WINDOW;
        if self.combo > 0 { self.score += COMBO_BONUS * self.combo; }
        if (self.combo as u32) > self.stats.best_combo { self.stats.best_combo = self.combo as u32; }
        if self.combo == 10 { self.collect_achievements(); }
        if self.moves % 25 == 0 {
            self.stats.explored_max = self.stats.explored_max.max(self.count_explored());
            self.collect_achievements();
        }
    }
    fn collect_achievements(&mut self) {
        let new = self.stats.check_achievements();
        for id in &new {
            self.say(format!("{}{}", self.t("achUnlock"), ach_name(self.lang, id)));
            self.sfx(Sfx::Ach);
            self.new_ach.push(id.clone());
        }
    }
    pub fn dash(&mut self, dr: i32, dc: i32, now: u64) {
        self.now_ms = now;
        if self.over || self.paused || self.help_open { return; }
        if now < self.dash_until { self.say(self.t("w_dashCd").to_string()); return; }
        if self.skip_move > 0 {
            self.skip_move -= 1;
            self.say(format!("{}{}{}", self.t("w_swamp"), self.skip_move, self.t("w_swamp2")));
            return;
        }
        let (r, c) = self.player;
        let d = dir_index(dr, dc);
        if self.wall[r][c][d] { self.say(self.t("w_wall").to_string()); self.sfx(Sfx::Wall); return; }
        let mid = ((r as i32 + dr) as usize, (c as i32 + dc) as usize);
        if self.wall[mid.0][mid.1][d] { self.say(self.t("w_dashWall").to_string()); return; }
        self.player = ((mid.0 as i32 + dr) as usize, (mid.1 as i32 + dc) as usize);
        self.dash_until = now + DASH_CD;
        self.dash_flash = Some(Fx { pos: self.player, t0: now });
        self.sfx(Sfx::Step);
        self.mark_explored();
        self.compute_visibility();
        self.say(self.t("w_dash").to_string());
        self.on_enter(self.player);
    }

    pub fn lake_flow(&mut self, p: Pos) -> Option<Pos> {
        let l = self.lake_of(p)?.clone();
        let (dr, dc) = l.direction;
        let open_to = |wall: &[[[bool; 4]; W]; H], from: Pos, to: Pos| -> bool {
            wall[from.0][from.1][dir_index(to.0 as i32 - from.0 as i32, to.1 as i32 - from.1 as i32)]
        };
        if self.rng.next_usize(4) < 3 {
            if let Some(g) = add(p, (dr, dc)) {
                if self.in_lake(g) { return Some(g); }
                if open_to(&self.wall, p, g) { return Some(g); }
            }
        }
        let mut in_lake_ops = vec![];
        for i in 0..4 {
            if let Some(n) = add(p, DIRS[i]) {
                if self.in_lake(n) && open_to(&self.wall, p, n) { in_lake_ops.push(n); }
            }
        }
        if !in_lake_ops.is_empty() { return Some(in_lake_ops[self.rng.next_usize(in_lake_ops.len())]); }
        for i in 0..4 {
            if let Some(n) = add(p, DIRS[i]) {
                if !self.in_lake(n) && open_to(&self.wall, p, n) { return Some(n); }
            }
        }
        None
    }
    pub fn lake_tick(&mut self) {
        if self.over || self.paused {
            if self.over { self.river_on = false; }
            return;
        }
        match self.lake_flow(self.player) {
            None => { self.river_on = false; }
            Some(m) => {
                self.player = m;
                self.sfx(Sfx::River);
                self.mark_explored();
                self.say(self.t("w_river").to_string());
                if !self.in_lake(self.player) && !self.holes.contains(&self.player) && !self.over {
                    self.cell_effects(self.player);
                }
            }
        }
    }
    fn on_enter(&mut self, cell: Pos) {
        if self.in_lake(cell) {
            if let Some(m) = self.lake_flow(cell) {
                self.player = m;
                self.sfx(Sfx::River);
                self.mark_explored();
                self.say(self.t("w_river").to_string());
            }
            self.river_on = true;
            if !self.in_lake(self.player) && !self.holes.contains(&self.player) && !self.over {
                self.cell_effects(self.player);
            }
            return;
        }
        if self.holes.contains(&cell) {
            self.sfx(Sfx::Hole);
            self.say(self.t("w_hole").to_string());
            let idx = self.holes.iter().position(|&h| h == cell).unwrap_or(0);
            self.player = self.holes[(idx + 1) % self.holes.len()];
            self.mark_explored();
            if !self.in_lake(self.player) && !self.holes.contains(&self.player) && !self.over {
                self.cell_effects(self.player);
            }
            return;
        }
        self.cell_effects(cell);
    }
    fn cell_effects(&mut self, cell: Pos) {
        let (r, c) = cell;
        if self.arsenal.contains(&cell) {
            self.sfx(Sfx::Heal); self.health = MAX_HEALTH; self.bombs = MAX_BOMBS;
            self.say(self.t("w_arsenal").to_string()); return;
        }
        if cell == self.hospital {
            self.sfx(Sfx::Heal); self.health = MAX_HEALTH;
            self.say(self.t("w_hospital").to_string()); return;
        }
        if self.trap.contains(&cell) {
            if self.rng.next_usize(2) == 0 { self.sfx(Sfx::Bear); self.damage_player(1, "w_trap"); }
            else { self.say(self.t("w_trapSafe").to_string()); }
            return;
        }
        if self.fire.contains(&cell) { self.sfx(Sfx::Bear); self.damage_player(1, "w_fire"); return; }
        if self.key == Some(cell) {
            self.sfx(Sfx::Key); self.carrying_key = true;
            self.say(self.t("w_key").to_string()); self.key = None;
        }
        if cell == self.chest {
            self.sfx(Sfx::Chest); self.carrying = true;
            self.say(self.t("w_chest").to_string());
        }
        if cell == self.exit {
            if self.carrying && self.carrying_key && self.health > 0 { self.end_win(); }
            else if self.carrying && !self.carrying_key { self.say(self.t("w_door").to_string()); }
            else { self.say(self.t("w_exit").to_string()); }
        }
        if self.swamps.contains(&cell) {
            self.sfx(Sfx::Hole);
            self.skip_move = SWAMP_SKIPS;
            self.say(format!("{}{}{}", self.t("w_swamp"), SWAMP_SKIPS, self.t("w_swamp3")));
        }
    }
    pub fn damage_player(&mut self, amount: i32, msg_key: &str) {
        self.health -= amount;
        self.took_damage = true;
        self.hurt_until = self.now_ms + 350;
        self.shake = 6.0;
        let mut m = format!("{}{}", self.t(msg_key), self.health);
        if self.carrying && self.health <= 2 {
            self.carrying = false;
            m = format!("{} · {}", m, self.t("w_dropChestBack"));
        }
        self.say(m);
        if self.health <= 0 { self.end_lose(); }
    }
    // --- Враги ---
    fn open_neighbors(&self, p: Pos) -> Vec<Pos> {
        (0..4).filter(|&d| !self.wall[p.0][p.1][d])
            .filter_map(|d| add(p, DIRS[d])).collect()
    }
    pub fn bear_turn(&mut self, now: u64) {
        self.now_ms = now;
        if self.over || now < self.grace_until { return; }
        for i in 0..self.bears.len() {
            if self.over { break; }
            let bear = self.bears[i];
            let ops = self.open_neighbors(bear);
            if ops.is_empty() { continue; }
            let m = if self.rng.next_usize(2) == 0 {
                let mut best = ops[0]; let mut bd = manhattan(best, self.player);
                for &o in &ops { let d = manhattan(o, self.player); if d < bd { bd = d; best = o; } }
                best
            } else { ops[self.rng.next_usize(ops.len())] };
            self.bears[i] = m;
            if self.vis[m.0][m.1] { self.sfx(Sfx::EsStep); }
            if m == self.player { self.sfx(Sfx::Bear); self.damage_player(1, "w_bear"); }
        }
    }
    pub fn zombie_turn(&mut self, now: u64) {
        self.now_ms = now;
        if self.over || now < self.grace_until { return; }
        for i in 0..self.zombies.len() {
            if self.over { break; }
            let z = self.zombies[i];
            if manhattan(z, self.player) > 5 { continue; }
            let ops = self.open_neighbors(z);
            if ops.is_empty() { continue; }
            let mut best = ops[0]; let mut bd = manhattan(best, self.player);
            for &o in &ops { let d = manhattan(o, self.player); if d < bd { bd = d; best = o; } }
            self.zombies[i] = best;
            if self.vis[best.0][best.1] { self.sfx(Sfx::EsStep); }
            if manhattan(best, self.player) == 1 { self.sfx(Sfx::Bear); self.damage_player(1, "w_zombie"); }
        }
    }
    pub fn has_clear_shot(&self, a: Pos, b: Pos) -> bool {
        if a == b { return true; }
        let (mut r, mut c) = (a.0 as i32, a.1 as i32);
        let (tr, tc) = (b.0 as i32, b.1 as i32);
        while r != tr || c != tc {
            let (dr, dc) = if r != tr { (if r < tr { 1 } else { -1 }, 0) } else { (0, if c < tc { 1 } else { -1 }) };
            if self.wall[r as usize][c as usize][dir_index(dr, dc)] { return false; }
            r += dr; c += dc;
            if r == tr && c == tc { return true; }
        }
        true
    }
    pub fn next_step_to(&self, from: Pos, to: Pos) -> Option<Pos> {
        if from == to { return None; }
        let b = bfs(&self.wall, from, None);
        if b.prev[to.0][to.1].is_none() { return None; }
        let mut node = to;
        while let Some(p) = b.prev[node.0][node.1] {
            if p == from { return Some(node); }
            node = p;
        }
        Some(node)
    }
    pub fn boss_turn(&mut self, now: u64) {
        self.now_ms = now;
        if self.over || now < self.grace_until { return; }
        for i in 0..self.bosses.len() {
            if self.over { break; }
            let b = self.bosses[i];
            let dist = manhattan(b, self.player);
            if dist <= BOSS_AIM_DIST && self.has_clear_shot(b, self.player) {
                if self.boss_aim.contains(&b) {
                    self.sfx(Sfx::Danger); self.sfx(Sfx::Shoot);
                    self.damage_player(1, "w_boss");
                    self.boss_aim.remove(&b);
                    self.shots.push(Shot { from: b, to: self.player, t0: now });
                } else {
                    self.sfx(Sfx::Danger);
                    self.boss_aim.insert(b);
                }
                continue;
            }
            self.boss_aim.remove(&b);
            if b == self.player { self.sfx(Sfx::Bear); self.damage_player(2, "w_bossMelee"); continue; }
            let step = match self.next_step_to(b, self.player) { Some(s) => s, None => continue };
            self.bosses[i] = step;
            if self.vis[step.0][step.1] { self.sfx(Sfx::EsStep); }
            if manhattan(step, self.player) == 1 { self.sfx(Sfx::Bear); self.damage_player(2, "w_bossMelee"); }
        }
    }
    pub fn shooter_turn(&mut self, now: u64) {
        self.now_ms = now;
        if self.over || now < self.grace_until { return; }
        for i in 0..self.shooters.len() {
            if self.over { break; }
            let s = self.shooters[i];
            let dist = manhattan(s, self.player);
            if dist <= 3 && self.vis[s.0][s.1] {
                if self.shooter_aim.contains(&s) {
                    self.sfx(Sfx::Danger); self.sfx(Sfx::Shoot);
                    self.damage_player(1, "w_shooter");
                    self.shooter_aim.remove(&s);
                    self.shots.push(Shot { from: s, to: self.player, t0: now });
                } else {
                    self.sfx(Sfx::Danger);
                    self.shooter_aim.insert(s);
                }
                continue;
            }
            self.shooter_aim.remove(&s);
            let ops = self.open_neighbors(s);
            if ops.is_empty() { continue; }
            self.shooters[i] = ops[self.rng.next_usize(ops.len())];
        }
    }
    pub fn bomb(&mut self, dr: i32, dc: i32, now: u64) {
        self.now_ms = now;
        if self.over || self.paused || self.help_open { return; }
        if self.bombs <= 0 { self.say(self.t("w_noBombs").to_string()); return; }
        let (r, c) = self.player;
        let d = dir_index(dr, dc);
        let target = match add((r, c), (dr, dc)) { Some(x) => x, None => { self.say(self.t("w_edge").to_string()); return; } };
        if self.wall[r][c][d] {
            if self.rng.next_usize(5) < 3 {
                self.bombs -= 1;
                self.sfx(Sfx::Wall);
                self.say(self.t("w_stone").to_string());
                return;
            }
            self.wall[r][c][d] = false;
            self.wall[target.0][target.1][d ^ 1] = false;
            self.bombs -= 1;
            self.sfx(Sfx::Bomb);
            self.say(format!("{}{}{}", self.t("w_boom"), dir_name(self.lang, d), self.t("w_boom2")));
            self.boom = Some(Fx { pos: target, t0: now });
            self.shake = 4.0;
            self.compute_visibility();
        } else {
            self.say(self.t("w_noWall").to_string());
        }
    }
    pub fn tick_second(&mut self, _now: u64) {
        if self.over || self.paused { return; }
        self.time_left -= 1;
        if self.time_left <= 10 && self.time_left > 0 { self.sfx(Sfx::Tick); }
        if self.time_left <= 0 { self.time_left = 0; self.end_timeout(); }
    }
    pub fn active_target(&self) -> Pos {
        if !self.carrying_key { return self.key.unwrap_or(self.chest); }
        if !self.carrying { return self.chest; }
        self.exit
    }
    // {dir glyph, dx, dy, dist} — прямой линии к цели (8 сторон)
    pub fn direction_to_target(&self) -> (String, i32, i32, i32) {
        let tgt = self.active_target();
        if self.player == tgt { return ("🏆".to_string(), 0, 0, 0); }
        let b = bfs(&self.wall, self.player, None);
        let d = b.dist[tgt.0][tgt.1];
        let dr = self.player.0 as i32 - tgt.0 as i32;
        let dc = self.player.1 as i32 - tgt.1 as i32;
        let dy = if dr != 0 { if dr > 0 { -1 } else { 1 } } else { 0 };
        let dx = if dc != 0 { if dc > 0 { -1 } else { 1 } } else { 0 };
        let glyph = if dy < 0 { if dx < 0 { "↖" } else if dx > 0 { "↗" } else { "⬆" } }
            else if dy > 0 { if dx < 0 { "↙" } else if dx > 0 { "↘" } else { "⬇" } }
            else if dx < 0 { "⬅" } else { "➡" };
        (glyph.to_string(), dx, dy, if d == -1 { 999 } else { d })
    }
    // --- Финал ---
    pub fn end_win(&mut self) {
        self.over = true; self.over_kind = Some(OverKind::Win);
        self.level += 1;
        self.sfx(Sfx::Win);
        self.score += 1000;
        let win_time = LEVEL_TIME - self.time_left;
        if win_time <= 90 && self.level - 1 == 1 { self.stats.speedrun = true; }
        if !self.took_damage { self.stats.clean_win = true; }
        self.stats.wins += 1;
        self.stats.explored_max = self.stats.explored_max.max(self.count_explored());
        self.save_best(win_time as f64);
        self.collect_achievements();
        self.say(self.t("w_winMsg").to_string());
        self.log = format!("{}{}{}", self.t("w_reset1"), self.level, self.t("w_newLevelR"));
    }
    pub fn end_lose(&mut self) {
        self.over = true; self.over_kind = Some(OverKind::Lose);
        self.level = 1;
        self.sfx(Sfx::Lose);
        self.say(self.t("w_loseMsg").to_string());
        self.log = self.t("w_R").to_string();
    }
    pub fn end_timeout(&mut self) {
        if self.over { return; }
        self.over = true; self.over_kind = Some(OverKind::Timeout);
        self.sfx(Sfx::Lose);
        self.carrying = false;
        self.level = 1;
        self.say(self.t("timeoutMsg").to_string());
        self.log = self.t("w_R").to_string();
    }
    fn save_best(&mut self, win_time: f64) {
        let passed = self.level - 1;
        if passed > self.best.level { self.best.level = passed; }
        if passed == 1 && (self.best.time1.is_none() || win_time < self.best.time1.unwrap()) {
            self.best.time1 = Some(win_time);
        }
    }
    pub fn best_text(&self) -> String {
        let mut s = format!("{}{}{}", self.t("record"), self.best.level, self.t("record2"));
        if let Some(tt) = self.best.time1 {
            s = format!("{}{}{:.1}</b>{}", s, self.t("bestL1"), tt, self.t("secs"));
        }
        s
    }
    pub fn toggle_pause(&mut self) { if !self.over { self.paused = !self.paused; } }
    pub fn toggle_help(&mut self) { self.help_open = !self.help_open; self.aiming = false; }
    pub fn drain_sfx(&mut self) -> Vec<Sfx> { std::mem::take(&mut self.sfx) }
    pub fn win_stats_html(&self) -> String {
        let ach_names: Vec<String> = self.stats.unlocked.iter().map(|id| ach_name(self.lang, id).to_string()).collect();
        let mut s = format!("{}{}{}{}{}{}{}{}",
            self.t("w_winStats"), self.moves, self.t("w_winStats2"), LEVEL_TIME - self.time_left, self.t("secs"),
            self.t("w_newLevel"), self.level, self.t("w_newLevel2"));
        s = format!("{}<br>{}: <b>{}</b>", s, self.t("hudScore"), self.score);
        if !ach_names.is_empty() { s = format!("{}<br>{}{}", s, self.t("achUnlock"), ach_names.join(", ")); }
        let b = self.best_text();
        if !b.is_empty() { s = format!("{}<br>{}", s, b); }
        s
    }
    pub fn lose_stats_html(&self) -> String {
        format!("{}{}{}{}{}{}{}{}", self.t("w_loseStats"), self.moves, self.t("w_timeStats"), LEVEL_TIME - self.time_left, self.t("secs"), self.t("w_resetLevel"), self.t("hudScore"), self.score)
    }
    pub fn timeout_stats_html(&self) -> String {
        format!("{}{}{}", self.t("w_timeoutStats"), self.moves, self.t("w_resetLevel2"))
    }
    pub fn cell_symbol(&self, r: usize, c: usize) -> &'static str {
        if self.key == Some((r, c)) { return "🗝️"; }
        if (r, c) == self.chest { return "💰"; }
        if (r, c) == self.exit { return "🚩"; }
        if self.fire.contains(&(r, c)) { return "🔥"; }
        if (r, c) == self.hospital { return "💉"; }
        if self.arsenal.contains(&(r, c)) { return "📦"; }
        if self.in_lake((r, c)) { return "🌀"; }
        if self.holes.contains(&(r, c)) { return "🕳"; }
        if self.swamps.contains(&(r, c)) { return "🪷"; }
        if self.trap.contains(&(r, c)) { return "⚠️"; }
        ""
    }
}

// now0: для damage_player нужен ms; храним последний now в поле через thread-local нельзя —
// проще: hurt_until считается web слоем от времени кадра; здесь ставим относительное значение 350.
fn now0() -> u64 { 0 }

pub fn parse_seed_from_query(query: &str) -> Option<u32> {
    for part in query.trim_start_matches('?').split('&') {
        if let Some(v) = part.strip_prefix("seed=") {
            if let Ok(n) = v.parse::<u32>() { return Some(n); }
        }
    }
    None
}
#[allow(dead_code)]
pub fn is_random_seed(seed: Option<u32>) -> bool { seed.is_none() }
