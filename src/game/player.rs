use super::*;

impl Game {
    pub fn try_move(&mut self, dr: i32, dc: i32, now: u64) {
        self.now_ms = now;
        if self.over || self.paused || self.help_open {
            return;
        }
        if now < self.move_lock_until {
            return;
        }
        if self.skip_move > 0 {
            self.move_lock_until = now + MOVE_CD;
            self.skip_move -= 1;
            let m = format!(
                "{}{}{}",
                self.t("w_swamp"),
                self.skip_move,
                self.t("w_swamp2")
            );
            self.say(m);
            return;
        }
        let (r, c) = self.player;
        if self.wall[r][c][dir_index(dr, dc)] {
            self.combo = 0;
            self.combo_until = 0;
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
        if !self.over {
            self.moves += 1;
        }
        if now <= self.combo_until {
            self.combo += 1;
        } else {
            self.combo = 1;
        }
        self.combo_until = now + COMBO_WINDOW;
        if self.combo > 0 {
            self.score += COMBO_BONUS * self.combo;
        }
        if (self.combo as u32) > self.stats.best_combo {
            self.stats.best_combo = self.combo as u32;
        }
        if self.combo == 10 {
            self.collect_achievements();
        }
        if self.moves % 25 == 0 {
            self.stats.explored_max = self.stats.explored_max.max(self.count_explored());
            self.collect_achievements();
        }
    }
    pub(crate) fn collect_achievements(&mut self) {
        let new = self.stats.check_achievements();
        for id in &new {
            self.say(format!(
                "{}{}",
                self.t("achUnlock"),
                ach_name(self.lang, id)
            ));
            self.sfx(Sfx::Ach);
            self.new_ach.push(id.clone());
        }
    }
    pub fn dash(&mut self, dr: i32, dc: i32, now: u64) {
        self.now_ms = now;
        if self.over || self.paused || self.help_open {
            return;
        }
        if now < self.dash_until {
            self.say(self.t("w_dashCd").to_string());
            return;
        }
        if self.skip_move > 0 {
            self.skip_move -= 1;
            self.say(format!(
                "{}{}{}",
                self.t("w_swamp"),
                self.skip_move,
                self.t("w_swamp2")
            ));
            return;
        }
        let (r, c) = self.player;
        let d = dir_index(dr, dc);
        if self.wall[r][c][d] {
            self.say(self.t("w_wall").to_string());
            self.sfx(Sfx::Wall);
            return;
        }
        let mid = ((r as i32 + dr) as usize, (c as i32 + dc) as usize);
        if self.wall[mid.0][mid.1][d] {
            self.say(self.t("w_dashWall").to_string());
            return;
        }
        self.player = ((mid.0 as i32 + dr) as usize, (mid.1 as i32 + dc) as usize);
        self.dash_until = now + DASH_CD;
        self.dash_flash = Some(Fx {
            pos: self.player,
            t0: now,
        });
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
            wall[from.0][from.1]
                [dir_index(to.0 as i32 - from.0 as i32, to.1 as i32 - from.1 as i32)]
        };
        if self.rng.next_usize(4) < 3 {
            if let Some(g) = add(p, (dr, dc)) {
                if self.in_lake(g) {
                    return Some(g);
                }
                if open_to(&self.wall, p, g) {
                    return Some(g);
                }
            }
        }
        let mut in_lake_ops = vec![];
        for i in 0..4 {
            if let Some(n) = add(p, DIRS[i]) {
                if self.in_lake(n) && open_to(&self.wall, p, n) {
                    in_lake_ops.push(n);
                }
            }
        }
        if !in_lake_ops.is_empty() {
            return Some(in_lake_ops[self.rng.next_usize(in_lake_ops.len())]);
        }
        for i in 0..4 {
            if let Some(n) = add(p, DIRS[i]) {
                if !self.in_lake(n) && open_to(&self.wall, p, n) {
                    return Some(n);
                }
            }
        }
        None
    }
    pub fn lake_tick(&mut self) {
        if self.over || self.paused {
            if self.over {
                self.river_on = false;
            }
            return;
        }
        match self.lake_flow(self.player) {
            None => {
                self.river_on = false;
            }
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
        // медведь в клетке: игрок получает ранение (и при шаге врага на игрока — bear_turn)
        if self.bears.contains(&cell) {
            self.sfx(Sfx::Bear);
            self.damage_player(1, "w_bear");
        }
        if self.arsenal.contains(&cell) {
            self.sfx(Sfx::Heal);
            self.health = MAX_HEALTH;
            self.bombs = MAX_BOMBS;
            self.say(self.t("w_arsenal").to_string());
            return;
        }
        if cell == self.hospital {
            self.sfx(Sfx::Heal);
            self.health = MAX_HEALTH;
            self.say(self.t("w_hospital").to_string());
            return;
        }
        if self.trap.contains(&cell) {
            if self.rng.next_usize(2) == 0 {
                self.sfx(Sfx::Bear);
                self.damage_player(1, "w_trap");
            } else {
                self.say(self.t("w_trapSafe").to_string());
            }
            return;
        }
        if self.fire.contains(&cell) {
            self.sfx(Sfx::Bear);
            self.damage_player(1, "w_fire");
            return;
        }
        if self.key == Some(cell) {
            self.sfx(Sfx::Key);
            self.carrying_key = true;
            self.say(self.t("w_key").to_string());
            self.key = None;
        }
        if cell == self.chest {
            self.sfx(Sfx::Chest);
            self.carrying = true;
            self.say(self.t("w_chest").to_string());
        }
        if cell == self.exit {
            if self.carrying && self.carrying_key && self.health > 0 {
                self.end_win();
            } else if self.carrying && !self.carrying_key {
                self.say(self.t("w_door").to_string());
            } else {
                self.say(self.t("w_exit").to_string());
            }
        }
        if self.swamps.contains(&cell) {
            self.sfx(Sfx::Hole);
            self.skip_move = SWAMP_SKIPS;
            self.say(format!(
                "{}{}{}",
                self.t("w_swamp"),
                SWAMP_SKIPS,
                self.t("w_swamp3")
            ));
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
        if self.health <= 0 {
            self.end_lose();
        }
    }
    // --- Враги ---
}
