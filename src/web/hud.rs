use super::*;
use crate::game::*;

pub(crate) fn storage_get(win: &Window, k: &str) -> Option<String> {
    win.local_storage()
        .ok()
        .flatten()
        .and_then(|s| s.get_item(k).ok().flatten())
}
pub(crate) fn storage_set(win: &Window, k: &str, v: &str) {
    if let Ok(Some(s)) = win.local_storage() {
        let _ = s.set_item(k, v);
    }
}
pub(crate) fn load_stats(win: &Window) -> Stats {
    let mut st = Stats::default();
    if let Some(v) = storage_get(win, "labyrinth.stats") {
        for part in v.split(',') {
            let line = part.trim();
            if let Some(x) = line.strip_prefix("\"wins\":") {
                st.wins = x.trim_end_matches('}').parse().unwrap_or(0);
            }
            if let Some(x) = line.strip_prefix("\"bestCombo\":") {
                st.best_combo = x.trim_end_matches('}').parse().unwrap_or(0);
            }
            if let Some(x) = line.strip_prefix("\"exploredMax\":") {
                st.explored_max = x.trim_end_matches('}').parse().unwrap_or(0);
            }
            if line.starts_with("\"speedrun\":true") {
                st.speedrun = true;
            }
            if line.starts_with("\"cleanWin\":true") {
                st.clean_win = true;
            }
        }
        if let Some(i) = v.find("\"unlocked\":[") {
            let rest = &v[i + 13..];
            let end = rest.find(']').unwrap_or(rest.len());
            for s in rest[..end].split('"') {
                if !s.is_empty() && s != "," && s != "[" {
                    st.unlocked.push(s.to_string());
                }
            }
        }
    }
    st
}
pub(crate) fn save_stats(win: &Window, st: &Stats) {
    let un: Vec<String> = st.unlocked.iter().map(|s| format!("\"{}\"", s)).collect();
    storage_set(win, "labyrinth.stats", &format!(
        "{{\"wins\":{},\"bestCombo\":{},\"exploredMax\":{},\"speedrun\":{},\"cleanWin\":{},\"unlocked\":[{}]}}",
        st.wins, st.best_combo, st.explored_max, st.speedrun, st.clean_win, un.join(",")
    ));
}
pub(crate) fn load_best(win: &Window) -> Best {
    let mut b = Best::default();
    if let Some(v) = storage_get(win, "labyrinth.best") {
        for part in v.split(',') {
            let line = part.trim();
            if let Some(x) = line.strip_prefix("\"level\":") {
                b.level = x.trim_end_matches('}').parse().unwrap_or(0);
            }
            if let Some(x) = line.strip_prefix("\"time1\":") {
                let x = x.trim_end_matches('}');
                if x != "null" {
                    b.time1 = x.parse().ok();
                }
            }
        }
    }
    b
}
pub(crate) fn save_best(win: &Window, b: Best) {
    let t1 = b
        .time1
        .map(|x| format!("{}", x))
        .unwrap_or_else(|| "null".to_string());
    storage_set(
        win,
        "labyrinth.best",
        &format!("{{\"level\":{},\"time1\":{}}}", b.level, t1),
    );
}

impl Web {
    pub(crate) fn flush(&mut self) {
        let sfxs = self.g.drain_sfx();
        for s in &sfxs {
            self.sfx(*s);
        }
        if sfxs.contains(&Sfx::Bomb) {
            self.burst("#ff9f43", 14);
            self.g.shake = 4.0;
        }
        if sfxs.contains(&Sfx::Win) {
            self.burst("#ffd166", 40);
        }
        if sfxs.contains(&Sfx::Bear) {
            self.g.shake = 6.0;
        }
        let now = now_ms(&self.win);
        // персистентный #log (info() в JS) vs всплывающий тост #status (msg())
        if self.log_shown != self.g.log {
            txt(&self.doc, "log", &self.g.log);
            self.log_shown = self.g.log.clone();
        }
        if !self.g.msg.is_empty() && self.msg_until <= now {
            txt(&self.doc, "status", &self.g.msg);
            if let Some(s) = el(&self.doc, "status") {
                let _ = s.class_list().add_1("show");
            }
            self.msg_until = now + 2600;
        }
        if self.g.over {
            let (title, color, stats) = match self.g.over_kind {
                Some(OverKind::Win) => (
                    self.g.t("winTitle").to_string(),
                    "#ffd166",
                    self.g.win_stats_html(),
                ),
                Some(OverKind::Lose) => (
                    self.g.t("loseTitle").to_string(),
                    "#e63946",
                    self.g.lose_stats_html(),
                ),
                _ => (
                    self.g.t("timeoutTitle").to_string(),
                    "#f4a261",
                    self.g.timeout_stats_html(),
                ),
            };
            txt(&self.doc, "ov-title", &title);
            if let Some(e) = el(&self.doc, "ov-title") {
                e.set_attribute("style", &format!("color:{}", color)).ok();
            }
            html(&self.doc, "ov-stats", &stats);
            hidden(&self.doc, "overlay", false);
            save_stats(&self.win.clone(), &self.g.stats);
            save_best(&self.win.clone(), self.g.best);
            self.clear_timers();
        }
        // достижения сохраняются сразу при разблокировке (как JS), не только в конце игры
        if !self.g.new_ach.is_empty() {
            save_stats(&self.win.clone(), &self.g.stats);
            self.g.new_ach.clear();
        }
        if self.g.stats.best_combo != self.saved_combo
            || self.g.stats.explored_max != self.saved_explored
        {
            self.saved_combo = self.g.stats.best_combo;
            self.saved_explored = self.g.stats.explored_max;
            save_stats(&self.win.clone(), &self.g.stats);
        }
        // река: старт/стоп автосноса
        if self.g.river_on && self.river_id.is_none() {
            let w = shared();
            let win = self.win.clone();
            self.river_id = Some(each(&win, RIVER_MS as i32, move || {
                w.borrow_mut().lake_tick();
            }));
        } else if !self.g.river_on {
            if let Some(id) = self.river_id.take() {
                self.win.clear_interval_with_handle(id);
            }
        }
        self.update_hud();
        html(&self.doc, "bestline", &self.g.best_text());
    }

    pub(crate) fn apply_i18n(&self) {
        let lang = self.g.lang;
        self.doc.set_title(&t(lang, "title"));
        if let Some(d) = self.doc.document_element() {
            let _ = d.set_attribute("lang", lang.code());
        }
        if let Ok(nodes) = self.doc.query_selector_all("[data-i18n]") {
            for i in 0..nodes.length() {
                let Some(e) = nodes.get(i).and_then(|n| n.dyn_into::<Element>().ok()) else {
                    continue;
                };
                let k = e.get_attribute("data-i18n").unwrap_or_default();
                let v = t(lang, &k);
                if e.get_attribute("data-html").as_deref() == Some("1") {
                    e.set_inner_html(&v);
                } else {
                    e.set_text_content(Some(&v));
                }
            }
        }
        txt(
            &self.doc,
            "lang-btn",
            if lang == Lang::Ru { "EN" } else { "RU" },
        );
        let date = js_sys::Date::new_0();
        let daily = format!(
            "?seed={}{:02}{:02}",
            date.get_full_year(),
            date.get_month() + 1,
            date.get_date()
        );
        if let Some(a) = el(&self.doc, "daily-link") {
            let _ = a.set_attribute("href", &daily);
        }
        if let Some(a) = el(&self.doc, "cur-link") {
            let _ = a.set_attribute("href", &format!("?seed={}", self.g.seed));
            a.set_text_content(Some(&format!("сид текущей партии: {}", self.g.seed)));
        }
    }

    fn update_hud(&self) {
        let g = &self.g;
        let dim = |s: &str| format!("<span style=\"opacity:.3\">{}</span>", s);
        let hp_on = g.health.max(0);
        let hp_off = (MAX_HEALTH - g.health).max(0);
        let bm_on = g.bombs.max(0);
        let bm_off = (MAX_BOMBS - g.bombs).max(0);
        html(
            &self.doc,
            "hud-hp",
            &format!(
                "{}{}",
                "♥".repeat(hp_on as usize),
                if hp_off > 0 {
                    dim(&"♥".repeat(hp_off as usize))
                } else {
                    String::new()
                }
            ),
        );
        html(
            &self.doc,
            "hud-bombs",
            &format!(
                "{}{}",
                "💣".repeat(bm_on as usize),
                if bm_off > 0 {
                    dim(&"💣".repeat(bm_off as usize))
                } else {
                    String::new()
                }
            ),
        );
        txt(&self.doc, "hud-chest", if g.carrying { "✔" } else { "✖" });
        txt(&self.doc, "hud-key", if g.carrying_key { "✔" } else { "✖" });
        txt(&self.doc, "hud-level", &format!("{}", g.level));
        txt(&self.doc, "hud-moves", &format!("{}", g.moves));
        txt(&self.doc, "hud-score", &format!("{}", g.score));
        txt(&self.doc, "hud-combo", &format!("x{}", g.combo.max(1)));
        let now = now_ms(&self.win);
        let ready = now >= g.dash_until;
        txt(&self.doc, "hud-dash", if ready { "⚡" } else { "·" });
        if let Some(e) = el(&self.doc, "hud-dash") {
            let _ = e.set_attribute("class", if ready { "ready" } else { "" });
        }
        txt(
            &self.doc,
            "hud-time",
            &format!("{}{}", g.time_left.min(999), g.t("secs")),
        );
        if let Some(e) = el(&self.doc, "hud-time") {
            let _ = e
                .class_list()
                .toggle_with_force("low", g.time_left <= LOW_TIME && g.time_left > 0);
        }
    }
}
