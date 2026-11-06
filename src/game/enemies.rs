use super::*;

impl Game {
    fn open_neighbors(&self, p: Pos) -> Vec<Pos> {
        (0..4)
            .filter(|&d| !self.wall[p.0][p.1][d])
            .filter_map(|d| add(p, DIRS[d]))
            .collect()
    }
    pub fn bear_turn(&mut self, now: u64) {
        self.now_ms = now;
        if self.over || now < self.grace_until {
            return;
        }
        for i in 0..self.bears.len() {
            if self.over {
                break;
            }
            let bear = self.bears[i];
            let ops = self.open_neighbors(bear);
            if ops.is_empty() {
                continue;
            }
            let m = if self.rng.next_usize(2) == 0 {
                let mut best = ops[0];
                let mut bd = manhattan(best, self.player);
                for &o in &ops {
                    let d = manhattan(o, self.player);
                    if d < bd {
                        bd = d;
                        best = o;
                    }
                }
                best
            } else {
                ops[self.rng.next_usize(ops.len())]
            };
            self.bears[i] = m;
            if self.vis[m.0][m.1] {
                self.sfx(Sfx::EsStep);
            }
            if m == self.player {
                self.sfx(Sfx::Bear);
                self.damage_player(1, "w_bear");
            }
        }
    }
    pub fn zombie_turn(&mut self, now: u64) {
        self.now_ms = now;
        if self.over || now < self.grace_until {
            return;
        }
        for i in 0..self.zombies.len() {
            if self.over {
                break;
            }
            let z = self.zombies[i];
            if manhattan(z, self.player) > 5 {
                continue;
            }
            let ops = self.open_neighbors(z);
            if ops.is_empty() {
                continue;
            }
            let mut best = ops[0];
            let mut bd = manhattan(best, self.player);
            for &o in &ops {
                let d = manhattan(o, self.player);
                if d < bd {
                    bd = d;
                    best = o;
                }
            }
            self.zombies[i] = best;
            if self.vis[best.0][best.1] {
                self.sfx(Sfx::EsStep);
            }
            if manhattan(best, self.player) == 1 {
                self.sfx(Sfx::Bear);
                self.damage_player(1, "w_zombie");
            }
        }
    }
    pub fn has_clear_shot(&self, a: Pos, b: Pos) -> bool {
        if a == b {
            return true;
        }
        let (mut r, mut c) = (a.0 as i32, a.1 as i32);
        let (tr, tc) = (b.0 as i32, b.1 as i32);
        while r != tr || c != tc {
            let (dr, dc) = if r != tr {
                (if r < tr { 1 } else { -1 }, 0)
            } else {
                (0, if c < tc { 1 } else { -1 })
            };
            if self.wall[r as usize][c as usize][dir_index(dr, dc)] {
                return false;
            }
            r += dr;
            c += dc;
            if r == tr && c == tc {
                return true;
            }
        }
        true
    }
    pub fn next_step_to(&self, from: Pos, to: Pos) -> Option<Pos> {
        if from == to {
            return None;
        }
        let b = bfs(&self.wall, from, None);
        b.prev[to.0][to.1]?;
        let mut node = to;
        while let Some(p) = b.prev[node.0][node.1] {
            if p == from {
                return Some(node);
            }
            node = p;
        }
        Some(node)
    }
    pub fn boss_turn(&mut self, now: u64) {
        self.now_ms = now;
        if self.over || now < self.grace_until {
            return;
        }
        for i in 0..self.bosses.len() {
            if self.over {
                break;
            }
            let b = self.bosses[i];
            let dist = manhattan(b, self.player);
            if dist <= BOSS_AIM_DIST && self.has_clear_shot(b, self.player) {
                if self.boss_aim.contains(&b) {
                    self.sfx(Sfx::Danger);
                    self.sfx(Sfx::Shoot);
                    self.damage_player(1, "w_boss");
                    self.boss_aim.remove(&b);
                    self.shots.push(Shot {
                        from: b,
                        to: self.player,
                        t0: now,
                    });
                } else {
                    self.sfx(Sfx::Danger);
                    self.boss_aim.insert(b);
                }
                continue;
            }
            self.boss_aim.remove(&b);
            if b == self.player {
                self.sfx(Sfx::Bear);
                self.damage_player(2, "w_bossMelee");
                continue;
            }
            let step = match self.next_step_to(b, self.player) {
                Some(s) => s,
                None => continue,
            };
            self.bosses[i] = step;
            if self.vis[step.0][step.1] {
                self.sfx(Sfx::EsStep);
            }
            if manhattan(step, self.player) == 1 {
                self.sfx(Sfx::Bear);
                self.damage_player(2, "w_bossMelee");
            }
        }
    }
    pub fn shooter_turn(&mut self, now: u64) {
        self.now_ms = now;
        if self.over || now < self.grace_until {
            return;
        }
        for i in 0..self.shooters.len() {
            if self.over {
                break;
            }
            let s = self.shooters[i];
            let dist = manhattan(s, self.player);
            if dist <= 3 && self.vis[s.0][s.1] {
                if self.shooter_aim.contains(&s) {
                    self.sfx(Sfx::Danger);
                    self.sfx(Sfx::Shoot);
                    self.damage_player(1, "w_shooter");
                    self.shooter_aim.remove(&s);
                    self.shots.push(Shot {
                        from: s,
                        to: self.player,
                        t0: now,
                    });
                } else {
                    self.sfx(Sfx::Danger);
                    self.shooter_aim.insert(s);
                }
                continue;
            }
            self.shooter_aim.remove(&s);
            let ops = self.open_neighbors(s);
            if ops.is_empty() {
                continue;
            }
            self.shooters[i] = ops[self.rng.next_usize(ops.len())];
        }
    }
}
