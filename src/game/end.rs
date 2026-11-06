use super::*;

impl Game {
    pub fn bomb(&mut self, dr: i32, dc: i32, now: u64) {
        self.now_ms = now;
        if self.over || self.paused || self.help_open {
            return;
        }
        if self.bombs <= 0 {
            self.say(self.t("w_noBombs").to_string());
            return;
        }
        let (r, c) = self.player;
        let d = dir_index(dr, dc);
        let target = match add((r, c), (dr, dc)) {
            Some(x) => x,
            None => {
                self.say(self.t("w_edge").to_string());
                return;
            }
        };
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
            self.say(format!(
                "{}{}{}",
                self.t("w_boom"),
                dir_name(self.lang, d),
                self.t("w_boom2")
            ));
            self.boom = Some(Fx {
                pos: target,
                t0: now,
            });
            self.shake = 4.0;
            self.compute_visibility();
        } else {
            self.say(self.t("w_noWall").to_string());
        }
    }
    pub fn tick_second(&mut self, _now: u64) {
        if self.over || self.paused {
            return;
        }
        self.time_left -= 1;
        if self.time_left <= 10 && self.time_left > 0 {
            self.sfx(Sfx::Tick);
        }
        if self.time_left <= 0 {
            self.time_left = 0;
            self.end_timeout();
        }
    }
    pub fn active_target(&self) -> Pos {
        if !self.carrying_key {
            return self.key.unwrap_or(self.chest);
        }
        if !self.carrying {
            return self.chest;
        }
        self.exit
    }
    // {dir glyph, dx, dy, dist} — прямой линии к цели (8 сторон)
    pub fn direction_to_target(&self) -> (String, i32, i32, i32) {
        let tgt = self.active_target();
        if self.player == tgt {
            return ("🏆".to_string(), 0, 0, 0);
        }
        let b = bfs(&self.wall, self.player, None);
        let d = b.dist[tgt.0][tgt.1];
        let dr = self.player.0 as i32 - tgt.0 as i32;
        let dc = self.player.1 as i32 - tgt.1 as i32;
        let dy = if dr != 0 {
            if dr > 0 {
                -1
            } else {
                1
            }
        } else {
            0
        };
        let dx = if dc != 0 {
            if dc > 0 {
                -1
            } else {
                1
            }
        } else {
            0
        };
        let glyph = if dy < 0 {
            if dx < 0 {
                "↖"
            } else if dx > 0 {
                "↗"
            } else {
                "⬆"
            }
        } else if dy > 0 {
            if dx < 0 {
                "↙"
            } else if dx > 0 {
                "↘"
            } else {
                "⬇"
            }
        } else if dx < 0 {
            "⬅"
        } else {
            "➡"
        };
        (glyph.to_string(), dx, dy, if d == -1 { 999 } else { d })
    }
    // --- Финал ---
    pub fn end_win(&mut self) {
        self.over = true;
        self.over_kind = Some(OverKind::Win);
        self.level += 1;
        self.sfx(Sfx::Win);
        self.score += 1000;
        let win_time = LEVEL_TIME - self.time_left;
        if win_time <= 90 && self.level - 1 == 1 {
            self.stats.speedrun = true;
        }
        if !self.took_damage {
            self.stats.clean_win = true;
        }
        self.stats.wins += 1;
        self.stats.explored_max = self.stats.explored_max.max(self.count_explored());
        self.save_best(win_time as f64);
        self.collect_achievements();
        self.say(self.t("w_winMsg").to_string());
        self.log = format!(
            "{}{}{}",
            self.t("w_reset1"),
            self.level,
            self.t("w_newLevelR")
        );
    }
    pub fn end_lose(&mut self) {
        self.over = true;
        self.over_kind = Some(OverKind::Lose);
        self.level = 1;
        self.sfx(Sfx::Lose);
        self.say(self.t("w_loseMsg").to_string());
        self.log = self.t("w_R").to_string();
    }
    pub fn end_timeout(&mut self) {
        if self.over {
            return;
        }
        self.over = true;
        self.over_kind = Some(OverKind::Timeout);
        self.sfx(Sfx::Lose);
        self.carrying = false;
        self.level = 1;
        self.say(self.t("timeoutMsg").to_string());
        self.log = self.t("w_R").to_string();
    }
    fn save_best(&mut self, win_time: f64) {
        let passed = self.level - 1;
        if passed > self.best.level {
            self.best.level = passed;
        }
        if passed == 1 && (self.best.time1.is_none() || win_time < self.best.time1.unwrap()) {
            self.best.time1 = Some(win_time);
        }
    }
    pub fn best_text(&self) -> String {
        let mut s = format!(
            "{}{}{}",
            self.t("record"),
            self.best.level,
            self.t("record2")
        );
        if let Some(tt) = self.best.time1 {
            s = format!("{}{}{:.1}</b>{}", s, self.t("bestL1"), tt, self.t("secs"));
        }
        s
    }
    pub fn toggle_pause(&mut self) {
        if !self.over {
            self.paused = !self.paused;
        }
    }
    pub fn toggle_help(&mut self) {
        self.help_open = !self.help_open;
        self.aiming = false;
    }
    pub fn drain_sfx(&mut self) -> Vec<Sfx> {
        std::mem::take(&mut self.sfx)
    }
    pub fn win_stats_html(&self) -> String {
        let ach_names: Vec<String> = self
            .stats
            .unlocked
            .iter()
            .map(|id| ach_name(self.lang, id).to_string())
            .collect();
        let mut s = format!(
            "{}{}{}{}{}{}{}{}",
            self.t("w_winStats"),
            self.moves,
            self.t("w_winStats2"),
            LEVEL_TIME - self.time_left,
            self.t("secs"),
            self.t("w_newLevel"),
            self.level,
            self.t("w_newLevel2")
        );
        s = format!("{}<br>{}: <b>{}</b>", s, self.t("hudScore"), self.score);
        if !ach_names.is_empty() {
            s = format!("{}<br>{}{}", s, self.t("achUnlock"), ach_names.join(", "));
        }
        let b = self.best_text();
        if !b.is_empty() {
            s = format!("{}<br>{}", s, b);
        }
        s
    }
    pub fn lose_stats_html(&self) -> String {
        format!(
            "{}{}{}{}{}{}{}{}",
            self.t("w_loseStats"),
            self.moves,
            self.t("w_timeStats"),
            LEVEL_TIME - self.time_left,
            self.t("secs"),
            self.t("w_resetLevel"),
            self.t("hudScore"),
            self.score
        )
    }
    pub fn timeout_stats_html(&self) -> String {
        format!(
            "{}{}{}",
            self.t("w_timeoutStats"),
            self.moves,
            self.t("w_resetLevel2")
        )
    }
    pub fn cell_symbol(&self, r: usize, c: usize) -> &'static str {
        if self.key == Some((r, c)) {
            return "🗝️";
        }
        if (r, c) == self.chest {
            return "💰";
        }
        if (r, c) == self.exit {
            return "🚩";
        }
        if self.fire.contains(&(r, c)) {
            return "🔥";
        }
        if (r, c) == self.hospital {
            return "💉";
        }
        if self.arsenal.contains(&(r, c)) {
            return "📦";
        }
        if self.in_lake((r, c)) {
            return "🌀";
        }
        if self.holes.contains(&(r, c)) {
            return "🕳";
        }
        if self.swamps.contains(&(r, c)) {
            return "🪷";
        }
        if self.trap.contains(&(r, c)) {
            return "⚠️";
        }
        ""
    }
}

// now0: для damage_player нужен ms; храним последний now в поле через thread-local нельзя —
// проще: hurt_until считается web слоем от времени кадра; здесь ставим относительное значение 350.
