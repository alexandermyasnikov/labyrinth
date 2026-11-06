// Веб-обвязка (только wasm32): рендер, HUD, ввод, звук, таймеры, localStorage.
// Логика — в game.rs (тестируется нативно 1-в-1).
use crate::game::*;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use web_sys::*;

type Shared = Rc<RefCell<Web>>;
thread_local! { static WEAK: RefCell<Option<Shared>> = const { RefCell::new(None) }; }

struct Web {
    g: Game,
    ctx: CanvasRenderingContext2d,
    doc: Document,
    win: Window,
    tick_id: Option<i32>, bear_id: Option<i32>, zombie_id: Option<i32>,
    shooter_id: Option<i32>, boss_id: Option<i32>, river_id: Option<i32>, anim_id: Option<i32>,
    actx: Option<AudioContext>,
    particles: Vec<Particle>,
    anim_from: Option<Pos>,
    cam: f64,
    pad_dir: (i32, i32),
    seed_is_url: bool,
    msg_until: u64,
}
#[derive(Clone)]
struct Particle { x: f64, y: f64, vx: f64, vy: f64, life: u64, t0: u64, color: String, size: f64 }

fn el(doc: &Document, id: &str) -> Option<Element> { doc.get_element_by_id(id) }
fn txt(doc: &Document, id: &str, v: &str) { if let Some(e) = el(doc, id) { e.set_text_content(Some(v)); } }
fn html(doc: &Document, id: &str, v: &str) { if let Some(e) = el(doc, id) { e.set_inner_html(v); } }
fn hidden(doc: &Document, id: &str, h: bool) {
    if let Some(e) = el(doc, id) { let _ = e.toggle_attribute_with_force("hidden", h); }
}
fn now_ms(win: &Window) -> u64 { win.performance().map(|p| p.now() as u64).unwrap_or(0) }
fn shared() -> Shared { WEAK.with(|w| w.borrow().clone().expect("web not registered")) }

fn storage_get(win: &Window, k: &str) -> Option<String> {
    win.local_storage().ok().flatten().and_then(|s| s.get_item(k).ok().flatten())
}
fn storage_set(win: &Window, k: &str, v: &str) {
    if let Ok(Some(s)) = win.local_storage() { let _ = s.set_item(k, v); }
}
fn load_stats(win: &Window) -> Stats {
    let mut st = Stats::default();
    if let Some(v) = storage_get(win, "labyrinth.stats") {
        for part in v.split(',') {
            let line = part.trim();
            if let Some(x) = line.strip_prefix("\"wins\":") { st.wins = x.trim_end_matches('}').parse().unwrap_or(0); }
            if let Some(x) = line.strip_prefix("\"bestCombo\":") { st.best_combo = x.trim_end_matches('}').parse().unwrap_or(0); }
            if let Some(x) = line.strip_prefix("\"exploredMax\":") { st.explored_max = x.trim_end_matches('}').parse().unwrap_or(0); }
            if line.starts_with("\"speedrun\":true") { st.speedrun = true; }
            if line.starts_with("\"cleanWin\":true") { st.clean_win = true; }
        }
        if let Some(i) = v.find("\"unlocked\":[") {
            let rest = &v[i + 13..];
            let end = rest.find(']').unwrap_or(rest.len());
            for s in rest[..end].split('"') {
                if !s.is_empty() && s != "," && s != "[" { st.unlocked.push(s.to_string()); }
            }
        }
    }
    st
}
fn save_stats(win: &Window, st: &Stats) {
    let un: Vec<String> = st.unlocked.iter().map(|s| format!("\"{}\"", s)).collect();
    storage_set(win, "labyrinth.stats", &format!(
        "{{\"wins\":{},\"bestCombo\":{},\"exploredMax\":{},\"speedrun\":{},\"cleanWin\":{},\"unlocked\":[{}]}}",
        st.wins, st.best_combo, st.explored_max, st.speedrun, st.clean_win, un.join(",")
    ));
}
fn load_best(win: &Window) -> Best {
    let mut b = Best::default();
    if let Some(v) = storage_get(win, "labyrinth.best") {
        for part in v.split(',') {
            let line = part.trim();
            if let Some(x) = line.strip_prefix("\"level\":") { b.level = x.trim_end_matches('}').parse().unwrap_or(0); }
            if let Some(x) = line.strip_prefix("\"time1\":") {
                let x = x.trim_end_matches('}');
                if x != "null" { b.time1 = x.parse().ok(); }
            }
        }
    }
    b
}
fn save_best(win: &Window, b: Best) {
    let t1 = b.time1.map(|x| format!("{}", x)).unwrap_or_else(|| "null".to_string());
    storage_set(win, "labyrinth.best", &format!("{{\"level\":{},\"time1\":{}}}", b.level, t1));
}

fn each(ms: i32, f: impl Fn() + 'static) -> i32 {
    let cb = Closure::wrap(Box::new(f) as Box<dyn FnMut()>);
    let id = shared().borrow().win.clone().set_interval_with_callback_and_timeout_and_arguments(cb.as_ref().unchecked_ref(), ms, &js_sys::Array::new()).unwrap();
    cb.forget();
    id
}

impl Web {
    fn new() -> Shared {
        let win = web_sys::window().expect("window");
        let doc = win.document().expect("document");
        let cv: HtmlCanvasElement = doc.get_element_by_id("cv").expect("cv").dyn_into().unwrap();
        let ctx = cv.get_context("2d").expect("ctx").expect("ctx").dyn_into().unwrap();
        let query = win.location().search().unwrap_or_default();
        let url_seed = parse_seed_from_query(&query);
        let seed = url_seed.unwrap_or_else(|| (now_ms(&win) % 1_000_000) as u32);
        let lang = storage_get(&win, "labyrinth.lang").and_then(|s| Lang::from_code(&s)).unwrap_or(Lang::Ru);
        let g = Game::new(seed, lang, load_stats(&win), load_best(&win));
        Rc::new(RefCell::new(Web {
            g, ctx, doc, win, tick_id: None, bear_id: None, zombie_id: None,
            shooter_id: None, boss_id: None, river_id: None, anim_id: None,
            actx: None, particles: vec![], anim_from: None, cam: 0.0, pad_dir: (-1, 0),
            seed_is_url: url_seed.is_some(), msg_until: 0,
        }))
    }

    fn sfx(&mut self, s: Sfx) {
        let (freq, wave, dur, vol): (f64, &str, f64, f64) = match s {
            Sfx::Move => (340.0, "sine", 0.06, 0.08), Sfx::Step => (220.0, "sine", 0.04, 0.05),
            Sfx::EsStep => (90.0, "sine", 0.06, 0.12), Sfx::Wall => (120.0, "triangle", 0.05, 0.18),
            Sfx::Chest => (660.0, "sine", 0.12, 0.25), Sfx::Bomb => (70.0, "sawtooth", 0.25, 0.40),
            Sfx::Bear => (220.0, "square", 0.15, 0.30), Sfx::Heal => (440.0, "sine", 0.20, 0.22),
            Sfx::River => (520.0, "sine", 0.25, 0.20), Sfx::Hole => (160.0, "sine", 0.18, 0.25),
            Sfx::Key => (700.0, "sine", 0.15, 0.22), Sfx::Shoot => (400.0, "square", 0.12, 0.25),
            Sfx::Danger => (300.0, "sawtooth", 0.28, 0.30), Sfx::Win => (880.0, "sine", 0.35, 0.35),
            Sfx::Lose => (90.0, "sawtooth", 0.4, 0.40), Sfx::Aim => (500.0, "sine", 0.08, 0.15),
            Sfx::Tick => (900.0, "square", 0.05, 0.18), Sfx::Ach => (750.0, "sine", 0.18, 0.28),
        };
        let ac = match &self.actx {
            Some(a) => a.clone(),
            None => match AudioContext::new() {
                Ok(a) => { self.actx = Some(a.clone()); a }
                Err(_) => return,
            },
        };
        if ac.state() == AudioContextState::Suspended { let _ = ac.resume(); }
        let t0 = ac.current_time();
        let o = match ac.create_oscillator() { Ok(o) => o, Err(_) => return };
        let gn = match ac.create_gain() { Ok(g) => g, Err(_) => return };
        o.set_type(match wave { "square" => OscillatorType::Square, "sawtooth" => OscillatorType::Sawtooth, "triangle" => OscillatorType::Triangle, _ => OscillatorType::Sine });
        o.frequency().set_value(freq as f32);
        let _ = gn.gain().set_value_at_time(vol as f32, t0);
        let _ = gn.gain().exponential_ramp_to_value_at_time(0.001, t0 + dur);
        o.connect_with_audio_node(&gn).ok();
        gn.connect_with_audio_node(&ac.destination()).ok();
        let _ = o.start_with_when(t0);
        let _ = o.stop_with_when(t0 + dur);
    }

    fn burst(&mut self, color: &str, n: usize) {
        let now = now_ms(&self.win);
        for i in 0..n {
            let ang = (i as f64) * 2.39996;
            let sp = 40.0 + (ang * 37.0).fract().abs() * 120.0;
            self.particles.push(Particle {
                x: 0.0, y: 0.0, vx: ang.cos() * sp, vy: ang.sin() * sp,
                life: 350 + ((ang * 53.0).fract() * 400.0) as u64, t0: now,
                color: color.to_string(), size: 2.0 + (ang * 29.0).fract() * 4.0,
            });
        }
        if self.particles.len() > 240 { self.particles = self.particles.split_off(self.particles.len() - 240); }
    }

    fn flush(&mut self) {
        let sfxs = self.g.drain_sfx();
        for s in &sfxs { self.sfx(*s); }
        if sfxs.contains(&Sfx::Bomb) { self.burst("#ff9f43", 14); self.g.shake = 4.0; }
        if sfxs.contains(&Sfx::Win) { self.burst("#ffd166", 40); }
        if sfxs.contains(&Sfx::Bear) { self.g.shake = 6.0; }
        let now = now_ms(&self.win);
        if !self.g.msg.is_empty() && self.msg_until <= now {
            txt(&self.doc, "log", &self.g.msg);
            txt(&self.doc, "status", &self.g.msg);
            if let Some(s) = el(&self.doc, "status") { let _ = s.class_list().add_1("show"); }
            self.msg_until = now + 2600;
        }
        if self.g.over {
            let (title, color, stats) = match self.g.over_kind {
                Some(OverKind::Win) => (self.g.t("winTitle").to_string(), "#ffd166", self.g.win_stats_html()),
                Some(OverKind::Lose) => (self.g.t("loseTitle").to_string(), "#e63946", self.g.lose_stats_html()),
                _ => (self.g.t("timeoutTitle").to_string(), "#f4a261", self.g.timeout_stats_html()),
            };
            txt(&self.doc, "ov-title", &title);
            if let Some(e) = el(&self.doc, "ov-title") { e.set_attribute("style", &format!("color:{}", color)).ok(); }
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
        // река: старт/стоп автосноса
        if self.g.river_on && self.river_id.is_none() {
            let w = shared();
            self.river_id = Some(each(RIVER_MS as i32, move || { w.borrow_mut().lake_tick(); }));
        } else if !self.g.river_on {
            if let Some(id) = self.river_id.take() { self.win.clear_interval_with_handle(id); }
        }
        self.update_hud();
        html(&self.doc, "bestline", &self.g.best_text());
    }

    fn clear_timers(&mut self) {
        for id in [&mut self.tick_id, &mut self.bear_id, &mut self.zombie_id, &mut self.shooter_id, &mut self.boss_id, &mut self.river_id] {
            if let Some(i) = id.take() { self.win.clear_interval_with_handle(i); }
        }
    }
    fn start_timers(&mut self) {
        self.clear_timers();
        let wa = shared();
        self.tick_id = Some(each(1000, move || { wa.borrow_mut().tick(); }));
        self.bear_id = Some(each(BEAR_MS as i32, move || { let w = shared(); let n = now_ms(&w.borrow().win.clone()); w.borrow_mut().turn("bear", n); }));
        self.zombie_id = Some(each(ZOMBIE_MS as i32, move || { let w = shared(); let n = now_ms(&w.borrow().win.clone()); w.borrow_mut().turn("zombie", n); }));
        self.shooter_id = Some(each(SHOOTER_MS as i32, move || { let w = shared(); let n = now_ms(&w.borrow().win.clone()); w.borrow_mut().turn("shooter", n); }));
        self.boss_id = Some(each(BOSS_MS as i32, move || { let w = shared(); let n = now_ms(&w.borrow().win.clone()); w.borrow_mut().turn("boss", n); }));
        let wa = shared();
        self.anim_id = Some(each(16, move || { wa.borrow_mut().animate(); }));
    }

    fn tick(&mut self) { self.g.tick_second(now_ms(&self.win)); self.flush(); self.render(); }
    fn turn(&mut self, kind: &str, now: u64) {
        if self.g.over || self.g.paused { return; }
        match kind {
            "bear" => self.g.bear_turn(now),
            "zombie" => self.g.zombie_turn(now),
            "shooter" => self.g.shooter_turn(now),
            _ => self.g.boss_turn(now),
        }
        self.flush(); self.render();
    }
    fn lake_tick(&mut self) { self.g.lake_tick(); self.flush(); self.render(); }
    fn animate(&mut self) {
        let now = now_ms(&self.win);
        if self.msg_until > 0 && now >= self.msg_until {
            if let Some(s) = el(&self.doc, "status") { let _ = s.class_list().remove_1("show"); }
            self.msg_until = 0;
        }
        self.g.shots.retain(|s| now.saturating_sub(s.t0) < 250);
        if self.g.boom.map(|b| now.saturating_sub(b.t0) >= 300).unwrap_or(false) { self.g.boom = None; }
        if self.g.dash_flash.map(|b| now.saturating_sub(b.t0) >= 250).unwrap_or(false) { self.g.dash_flash = None; }
        self.render();
    }
    fn restart(&mut self, new_world: bool) {
        let now = now_ms(&self.win);
        if new_world && !self.seed_is_url { self.g.seed = (now % 1_000_000) as u32; }
        if new_world { self.g.new_world(now); } else { self.g.restart(now); }
        self.anim_from = None; self.cam = 0.0; self.msg_until = 0;
        hidden(&self.doc, "overlay", true);
        hidden(&self.doc, "pause", true);
        hidden(&self.doc, "help", true);
        self.start_timers();
        self.apply_i18n();
        self.flush();
        self.render();
    }
    fn do_move(&mut self, dr: i32, dc: i32) {
        let now = now_ms(&self.win);
        let prev = self.g.player;
        self.g.try_move(dr, dc, now);
        if self.g.player != prev { self.anim_from = Some(prev); self.cam = 0.0; }
        self.flush(); self.render();
    }
    fn do_dash(&mut self, dr: i32, dc: i32) {
        let now = now_ms(&self.win);
        let prev = self.g.player;

        self.g.dash(dr, dc, now);
        if self.g.player != prev { self.anim_from = Some(prev); self.cam = 0.0; }
        self.flush(); self.render();
    }
    fn do_bomb(&mut self, dr: i32, dc: i32) {
        let now = now_ms(&self.win);
        self.g.bomb(dr, dc, now);
        self.flush(); self.render();
    }
    fn toggle_pause(&mut self) {
        self.g.toggle_pause();
        hidden(&self.doc, "pause", !self.g.paused);
        self.render();
    }
    fn on_key(&mut self, e: &KeyboardEvent) {
        let code = e.code();
        match code.as_str() {
            "KeyP" | "Escape" => { self.toggle_pause(); return; }
            "KeyR" => { self.restart(false); return; }
            "KeyN" => { self.restart(true); return; }
            "KeyH" => { self.g.toggle_help(); hidden(&self.doc, "help", !self.g.help_open); self.render(); return; }
            "KeyB" => {
                if self.g.over { return; }
                if self.g.bombs <= 0 { self.g.say(self.g.t("w_noBombs").to_string()); self.flush(); return; }
                self.g.aiming = true; self.sfx(Sfx::Aim);
                self.g.say(self.g.t("w_aim").to_string());
                self.flush(); self.render();
                return;
            }
            _ => {}
        }
        if self.g.paused { return; }
        let dir = match code.as_str() {
            "ArrowUp" | "KeyW" => Some((-1, 0)),
            "ArrowDown" | "KeyS" => Some((1, 0)),
            "ArrowLeft" | "KeyA" => Some((0, -1)),
            "ArrowRight" | "KeyD" => Some((0, 1)),
            _ => None,
        };
        if let Some((dr, dc)) = dir {
            e.prevent_default();
            self.pad_dir = (dr, dc);
            if self.g.aiming { self.g.aiming = false; self.do_bomb(dr, dc); }
            else if e.shift_key() { self.do_dash(dr, dc); }
            else { self.do_move(dr, dc); }
        }
    }

    fn apply_i18n(&self) {
        let lang = self.g.lang;
        if let Some(d) = self.doc.document_element() { let _ = d.set_attribute("lang", lang.code()); }
        if let Ok(nodes) = self.doc.query_selector_all("[data-i18n]") {
            for i in 0..nodes.length() {
                let Some(e) = nodes.get(i).and_then(|n| n.dyn_into::<Element>().ok()) else { continue };
                let k = e.get_attribute("data-i18n").unwrap_or_default();
                let v = t(lang, &k);
                if e.get_attribute("data-html").as_deref() == Some("1") { e.set_inner_html(&v); }
                else { e.set_text_content(Some(&v)); }
            }
        }
        txt(&self.doc, "lang-btn", if lang == Lang::Ru { "EN" } else { "RU" });
        let date = js_sys::Date::new_0();
        let daily = format!("?seed={}{:02}{:02}", date.get_full_year(), date.get_month() + 1, date.get_date());
        if let Some(a) = el(&self.doc, "daily-link") { let _ = a.set_attribute("href", &daily); }
        if let Some(a) = el(&self.doc, "cur-link") {
            let _ = a.set_attribute("href", &format!("?seed={}", self.g.seed));
            a.set_text_content(Some(&format!("сид текущей партии: {}", self.g.seed)));
        }
    }
    fn update_hud(&self) {
        let g = &self.g;
        let dim = |s: &str| format!("<span style=\"opacity:.3\">{}</span>", s);
        let hp_on = g.health.max(0); let hp_off = (MAX_HEALTH - g.health).max(0);
        let bm_on = g.bombs.max(0); let bm_off = (MAX_BOMBS - g.bombs).max(0);
        html(&self.doc, "hud-hp", &format!("{}{}", "♥".repeat(hp_on as usize), if hp_off > 0 { dim(&"♥".repeat(hp_off as usize)) } else { String::new() }));
        html(&self.doc, "hud-bombs", &format!("{}{}", "💣".repeat(bm_on as usize), if bm_off > 0 { dim(&"💣".repeat(bm_off as usize)) } else { String::new() }));
        txt(&self.doc, "hud-chest", if g.carrying { "✔" } else { "✖" });
        txt(&self.doc, "hud-key", if g.carrying_key { "✔" } else { "✖" });
        txt(&self.doc, "hud-level", &format!("{}", g.level));
        txt(&self.doc, "hud-moves", &format!("{}", g.moves));
        txt(&self.doc, "hud-score", &format!("{}", g.score));
        txt(&self.doc, "hud-combo", &format!("x{}", g.combo.max(1)));
        let now = now_ms(&self.win);
        let ready = now >= g.dash_until;
        txt(&self.doc, "hud-dash", if ready { "⚡" } else { "·" });
        if let Some(e) = el(&self.doc, "hud-dash") { let _ = e.set_attribute("class", if ready { "ready" } else { "" }); }
        txt(&self.doc, "hud-time", &format!("{}{}", g.time_left.min(999), g.t("secs")));
        if let Some(e) = el(&self.doc, "hud-time") { let _ = e.class_list().toggle_with_force("low", g.time_left <= LOW_TIME && g.time_left > 0); }
    }

    fn render(&mut self) {
        let g = &self.g;
        let n = (VIEW * CELL as usize) as f64;
        let c = n / 2.0;
        self.ctx.clear_rect(0.0, 0.0, n, n);
        if g.paused {
            self.ctx.set_fill_style(&wasm_bindgen::JsValue::from_str("rgba(6,8,12,.45)"));
            self.ctx.fill_rect(0.0, 0.0, n, n);
            self.ctx.set_font("28px sans-serif");
            self.ctx.set_text_align("center");
            self.ctx.set_fill_style(&wasm_bindgen::JsValue::from_str("#ffe9a8"));
            let pt = g.t("pausedTitle");
            self.ctx.fill_text(&pt, c, c).ok();
            return;
        }
        let now = now_ms(&self.win);
        if self.anim_from.is_some() && self.cam < 1.0 {
            self.cam += (1.0 - self.cam) * 0.56_f64.min(1.0);
            if self.cam > 0.999 { self.cam = 1.0; self.anim_from = None; }
        }
        let (dr, dc) = match self.anim_from {
            Some((fr, fc)) if self.cam < 1.0 => (
                fr as f64 + (g.player.0 as f64 - fr as f64) * self.cam,
                fc as f64 + (g.player.1 as f64 - fc as f64) * self.cam,
            ),
            _ => (g.player.0 as f64, g.player.1 as f64),
        };
        let (shx, shy) = if g.shake > 0.0 {
            let k = g.shake / 6.0;
            ((((now / 16) % 5) as f64 - 2.0) * 2.0 * k, (((now / 16 + 2) % 5) as f64 - 2.0) * 2.0 * k)
        } else { (0.0, 0.0) };
        let base_r = dr.round() as i32; let base_c = dc.round() as i32;
        let r0 = (base_r - 4).max(0); let r1 = (base_r + 5).min(H as i32 - 1);
        let c0 = (base_c - 4).max(0); let c1 = (base_c + 5).min(W as i32 - 1);
        let sx = |r: i32| c + (r as f64 - dr) * CELL + shy;
        let sc = |cc: i32| c + (cc as f64 - dc) * CELL + shx;
        let cell_bg = |sym: &str| -> &'static str {
            match sym {
                "🚩" => "#3a2f14", "💰" => "#3a3318", "🌀" => "#17324a", "🕳" => "#1a1a1f",
                "📦" => "#2f2a18", "💉" => "#2a1f24", "🪷" => "#20301c", "⚠️" => "#2b2118",
                _ => "#262a34",
            }
        };
        for r in r0..=r1 { for cc in c0..=c1 {
            let (r, cc) = (r as usize, cc as usize);
            if !g.explored[r][cc] || g.visible(r, cc) { continue; }
            let x = sc(cc as i32); let y = sx(r as i32);
            self.ctx.set_fill_style(&wasm_bindgen::JsValue::from_str("rgba(38,42,52,.35)"));
            self.ctx.fill_rect(x, y, CELL, CELL);
            self.ctx.set_stroke_style(&wasm_bindgen::JsValue::from_str("rgba(255,255,255,.05)"));
            self.ctx.set_line_width(1.0);
            self.ctx.stroke_rect(x, y, CELL, CELL);
        }}
        for r in r0..=r1 { for cc in c0..=c1 {
            let (r, cc) = (r as usize, cc as usize);
            if !g.visible(r, cc) { continue; }
            let x = sc(cc as i32); let y = sx(r as i32);
            let sym = g.cell_symbol(r, cc);
            self.ctx.set_fill_style(&wasm_bindgen::JsValue::from_str(cell_bg(sym)));
            self.ctx.fill_rect(x, y, CELL, CELL);
            self.ctx.set_stroke_style(&wasm_bindgen::JsValue::from_str("rgba(255,255,255,.04)"));
            self.ctx.set_line_width(1.0);
            self.ctx.stroke_rect(x, y, CELL, CELL);
            if !sym.is_empty() { self.draw_emoji(sym, x + CELL / 2.0, y + CELL / 2.0, 30.0); }
        }}
        self.ctx.set_line_cap("round");
        for r in r0..=r1 { for cc in c0..=c1 {
            let (r, cc) = (r as usize, cc as usize);
            if !g.visible(r, cc) { continue; }
            let x = sc(cc as i32); let y = sx(r as i32);
            self.ctx.set_stroke_style(&wasm_bindgen::JsValue::from_str("#05070b"));
            self.ctx.set_line_width(5.0);
            for d in 0..4 {
                if !g.wall[r][cc][d] { continue; }
                self.ctx.begin_path();
                match d {
                    0 => { self.ctx.move_to(x, y); self.ctx.line_to(x + CELL, y); }
                    1 => { self.ctx.move_to(x, y + CELL); self.ctx.line_to(x + CELL, y + CELL); }
                    2 => { self.ctx.move_to(x, y); self.ctx.line_to(x, y + CELL); }
                    _ => { self.ctx.move_to(x + CELL, y); self.ctx.line_to(x + CELL, y + CELL); }
                }
                self.ctx.stroke();
            }
        }}
        let vis = |p: Pos| g.visible(p.0, p.1);
        for &b in &g.bears { if vis(b) { self.draw_emoji("🐻", sc(b.1 as i32) + CELL/2.0, sx(b.0 as i32) + CELL/2.0, 28.0); } }
        for &f in &g.fire { if vis(f) { self.draw_emoji("🔥", sc(f.1 as i32) + CELL/2.0, sx(f.0 as i32) + CELL/2.0, 22.0); } }
        for &z in &g.zombies { if vis(z) { self.draw_emoji("🧟", sc(z.1 as i32) + CELL/2.0, sx(z.0 as i32) + CELL/2.0, 24.0); } }
        for &s in &g.shooters {
            if vis(s) {
                let e = if g.shooter_aim.contains(&s) { "🏹" } else { "👮" };
                self.draw_emoji(e, sc(s.1 as i32) + CELL/2.0, sx(s.0 as i32) + CELL/2.0, 24.0);
            }
        }
        for b in &g.shots {
            let t = (now.saturating_sub(b.t0)) as f64 / 250.0;
            if t >= 1.0 { continue; }
            let px = sc((b.from.1 as f64 + (b.to.1 as f64 - b.from.1 as f64) * t) as i32) + CELL/2.0;
            let py = sx((b.from.0 as f64 + (b.to.0 as f64 - b.from.0 as f64) * t) as i32) + CELL/2.0;
            self.ctx.set_fill_style(&wasm_bindgen::JsValue::from_str("#ffd166"));
            self.ctx.begin_path();
            self.ctx.arc(px, py, 4.0, 0.0, 7.0).ok();
            self.ctx.fill();
        }
        if let Some(bm) = g.boom {
            let bt = (now.saturating_sub(bm.t0)) as f64 / 300.0;
            if bt < 1.0 {
                let bx = sc(bm.pos.1 as i32) + CELL/2.0; let by = sx(bm.pos.0 as i32) + CELL/2.0;
                self.ctx.set_stroke_style(&wasm_bindgen::JsValue::from_str(&format!("rgba(255,159,67,{:.2})", 1.0 - bt)));
                self.ctx.set_line_width(6.0 * (1.0 - bt));
                self.ctx.begin_path();
                self.ctx.arc(bx, by, CELL * 0.4 + bt * CELL * 0.9, 0.0, 7.0).ok();
                self.ctx.stroke();
            }
        }
        if let Some(df) = g.dash_flash {
            let dt = (now.saturating_sub(df.t0)) as f64 / 250.0;
            if dt < 1.0 {
                let dx = sc(df.pos.1 as i32) + CELL/2.0; let dy = sx(df.pos.0 as i32) + CELL/2.0;
                self.ctx.set_stroke_style(&wasm_bindgen::JsValue::from_str(&format!("rgba(255,255,255,{:.2})", 1.0 - dt)));
                self.ctx.set_line_width(3.0 * (1.0 - dt));
                self.ctx.begin_path();
                self.ctx.arc(dx, dy, CELL * 0.3 + dt * CELL * 0.8, 0.0, 7.0).ok();
                self.ctx.stroke();
            }
        }
        {
            let grad = self.ctx.create_radial_gradient(c + CELL/2.0, c + CELL/2.0, 4.0, c + CELL/2.0, c + CELL/2.0, 28.0).expect("grad");
            grad.add_color_stop(0.0, "rgba(255,209,102,.5)").expect("glow");
            grad.add_color_stop(1.0, "rgba(255,209,102,0)").expect("glow");
            self.ctx.set_fill_style(&wasm_bindgen::JsValue::from(grad));
            self.ctx.begin_path();
            self.ctx.arc(c + CELL/2.0, c + CELL/2.0, 26.0, 0.0, 7.0).ok();
            self.ctx.fill();
        }
        if !g.shooter_aim.is_empty() || !g.boss_aim.is_empty() {
            let blink = (now / 400) % 2 < 1;
            self.ctx.set_stroke_style(&wasm_bindgen::JsValue::from_str(if blink { "#e63946" } else { "#ffd166" }));
            self.ctx.set_line_width(4.0);
            self.ctx.begin_path();
            self.ctx.arc(c + CELL/2.0, c + CELL/2.0, CELL/2.0 + 4.0, 0.0, 7.0).ok();
            self.ctx.stroke();
        }
        self.draw_emoji("🧙", c + CELL/2.0, c + CELL/2.0, 50.0);
        {
            let bw = 38.0; let bh = 5.0; let bx = c + CELL/2.0 - bw/2.0; let by = c + CELL/2.0 - CELL/2.0 + 4.0;
            self.ctx.set_fill_style(&wasm_bindgen::JsValue::from_str("rgba(0,0,0,.7)"));
            self.ctx.fill_rect(bx - 1.0, by - 1.0, bw + 2.0, bh + 2.0);
            let frac = ((g.health as f64) / MAX_HEALTH as f64).clamp(0.0, 1.0);
            self.ctx.set_fill_style(&wasm_bindgen::JsValue::from_str(if frac > 0.6 { "#58d68d" } else if frac > 0.3 { "#f5c542" } else { "#e63946" }));
            self.ctx.fill_rect(bx, by, bw * frac, bh);
        }
        let (dir, dx, dy, dist) = g.direction_to_target();
        if dir != "🏆" && dist > FOG_RADIUS {
            let off = CELL/2.0 - 6.0;
            let px = c + CELL/2.0 + dx as f64 * off; let py = c + CELL/2.0 + dy as f64 * off;
            self.ctx.set_global_alpha(0.85);
            self.ctx.set_fill_style(&wasm_bindgen::JsValue::from_str("rgba(24,27,34,.55)"));
            self.ctx.begin_path();
            self.ctx.arc(px, py, 16.0, 0.0, 7.0).ok();
            self.ctx.fill();
            self.ctx.set_font("26px serif");
            self.ctx.set_text_align("center");
            self.ctx.set_text_baseline("middle");
            self.ctx.set_fill_style(&wasm_bindgen::JsValue::from_str("#ffd166"));
            self.ctx.fill_text(&dir, px, py).ok();
            self.ctx.set_global_alpha(1.0);
        }
        if g.hurt_until > now {
            let a = (g.hurt_until - now) as f64 / 350.0;
            let gr = self.ctx.create_radial_gradient(c, c, n * 0.5, c, c, n * 0.75).expect("hurt");
            gr.add_color_stop(0.0, "rgba(230,57,70,0)").expect("hurt");
            gr.add_color_stop(1.0, &format!("rgba(230,57,70,{:.2})", 0.5 * a)).expect("hurt");
            self.ctx.set_fill_style(&wasm_bindgen::JsValue::from(gr));
            self.ctx.fill_rect(0.0, 0.0, n, n);
        }
        if g.health <= 2 && g.health > 0 {
            let a = 0.15 + 0.12 * ((now / 300) as f64).sin();
            let gr = self.ctx.create_radial_gradient(c, c, n * 0.5, c, c, n * 0.8).expect("low");
            gr.add_color_stop(0.0, "rgba(200,40,50,0)").expect("low");
            gr.add_color_stop(1.0, &format!("rgba(200,40,50,{:.2})", a)).expect("low");
            self.ctx.set_fill_style(&wasm_bindgen::JsValue::from(gr));
            self.ctx.fill_rect(0.0, 0.0, n, n);
        }
        for &b in &g.bosses {
            let bx = sc(b.1 as i32) + CELL/2.0; let by = sx(b.0 as i32) + CELL/2.0;
            self.ctx.set_fill_style(&wasm_bindgen::JsValue::from_str(if g.boss_aim.contains(&b) { "#7a2a14" } else { "#2b0f18" }));
            self.ctx.begin_path();
            self.ctx.arc(bx, by, 12.0, 0.0, 7.0).ok();
            self.ctx.fill();
            self.ctx.set_stroke_style(&wasm_bindgen::JsValue::from_str("rgba(255,209,102,.5)"));
            self.ctx.set_line_width(2.0);
            self.ctx.begin_path();
            self.ctx.arc(bx, by, 12.0, 0.0, 7.0).ok();
            self.ctx.stroke();
            self.draw_emoji(if g.boss_aim.contains(&b) { "🎯" } else { "🥷" }, bx, by, 26.0);
        }
        self.particles.retain(|p| now.saturating_sub(p.t0) <= p.life);
        for p in &self.particles {
            let age = now.saturating_sub(p.t0);
            let k = age as f64 / p.life as f64;
            self.ctx.set_global_alpha(1.0 - k);
            self.ctx.set_fill_style(&wasm_bindgen::JsValue::from_str(&p.color));
            self.ctx.fill_rect(p.x + p.vx * age as f64 / 1000.0 - p.size/2.0, p.y + p.vy * age as f64 / 1000.0 - p.size/2.0, p.size, p.size);
        }
        self.ctx.set_global_alpha(1.0);
        if self.g.shake > 0.0 { self.g.shake = (self.g.shake - 0.8).max(0.0); }
    }
    fn draw_emoji(&self, sym: &str, cx: f64, cy: f64, size: f64) {
        // shortcut: fillText-центрирование вместо JS bbox-кэша
        self.ctx.set_font(&format!("{}px sans-serif", size as i32));
        self.ctx.set_text_align("center");
        self.ctx.set_text_baseline("middle");
        self.ctx.fill_text(sym, cx, cy).ok();
    }
}

pub fn run() {
    let w = Web::new();
    WEAK.with(|x| *x.borrow_mut() = Some(w.clone()));
    // клавиатура
    {
        let ww = w.clone();
        let cb = Closure::wrap(Box::new(move |e: KeyboardEvent| { ww.borrow_mut().on_key(&e); }) as Box<dyn FnMut(KeyboardEvent)>);
        web_sys::window().unwrap().add_event_listener_with_callback("keydown", cb.as_ref().unchecked_ref()).ok();
        cb.forget();
    }
    let doc = w.borrow().doc.clone();
    for (id, is_new) in [("restart", false), ("new-world", true), ("ov-replay", false)] {
        let ww = w.clone();
        if let Some(e) = el(&doc, id) {
            let cb = Closure::wrap(Box::new(move || { ww.borrow_mut().restart(is_new); }) as Box<dyn FnMut()>);
            e.add_event_listener_with_callback("click", cb.as_ref().unchecked_ref()).ok();
            cb.forget();
        }
    }
    for (id, dir) in [("pad-up", (-1, 0)), ("pad-down", (1, 0)), ("pad-left", (0, -1)), ("pad-right", (0, 1))] {
        let ww = w.clone();
        if let Some(e) = el(&doc, id) {
            let cb = Closure::wrap(Box::new(move |ev: PointerEvent| {
                ev.prevent_default();
                let mut b = ww.borrow_mut();
                b.pad_dir = dir;
                if b.g.aiming { b.g.aiming = false; b.do_bomb(dir.0, dir.1); }
                else { b.do_move(dir.0, dir.1); }
            }) as Box<dyn FnMut(PointerEvent)>);
            e.add_event_listener_with_callback("pointerdown", cb.as_ref().unchecked_ref()).ok();
            cb.forget();
        }
    }
    {
        let ww = w.clone();
        if let Some(e) = el(&doc, "pad-dash") {
            let cb = Closure::wrap(Box::new(move |ev: PointerEvent| { ev.prevent_default(); let (dr, dc) = ww.borrow().pad_dir; ww.borrow_mut().do_dash(dr, dc); }) as Box<dyn FnMut(PointerEvent)>);
            e.add_event_listener_with_callback("pointerdown", cb.as_ref().unchecked_ref()).ok();
            cb.forget();
        }
    }
    {
        let ww = w.clone();
        if let Some(e) = el(&doc, "pad-bomb") {
            let cb = Closure::wrap(Box::new(move |ev: PointerEvent| {
                ev.prevent_default();
                let mut b = ww.borrow_mut();
                if b.g.over || b.g.paused { return; }
                let no_bombs = b.g.t("w_noBombs");
                let aim = b.g.t("w_aim");
                if b.g.bombs <= 0 { b.g.say(no_bombs); b.flush(); return; }
                b.g.aiming = true; b.sfx(Sfx::Aim);
                b.g.say(aim); b.flush(); b.render();
            }) as Box<dyn FnMut(PointerEvent)>);
            e.add_event_listener_with_callback("pointerdown", cb.as_ref().unchecked_ref()).ok();
            cb.forget();
        }
    }
    {
        let ww = w.clone();
        if let Some(e) = el(&doc, "pause-btn") {
            let cb = Closure::wrap(Box::new(move || { ww.borrow_mut().toggle_pause(); }) as Box<dyn FnMut()>);
            e.add_event_listener_with_callback("click", cb.as_ref().unchecked_ref()).ok();
            cb.forget();
        }
    }
    {
        let ww = w.clone();
        if let Some(e) = el(&doc, "resume-btn") {
            let cb = Closure::wrap(Box::new(move || { let mut b = ww.borrow_mut(); if b.g.paused { b.toggle_pause(); } }) as Box<dyn FnMut()>);
            e.add_event_listener_with_callback("click", cb.as_ref().unchecked_ref()).ok();
            cb.forget();
        }
    }
    {
        let ww = w.clone();
        if let Some(e) = el(&doc, "lang-btn") {
            let cb = Closure::wrap(Box::new(move || {
                let mut b = ww.borrow_mut();
                b.g.lang = b.g.lang.other();
                storage_set(&b.win.clone(), "labyrinth.lang", b.g.lang.code());
                b.apply_i18n(); b.flush(); b.render();
            }) as Box<dyn FnMut()>);
            e.add_event_listener_with_callback("click", cb.as_ref().unchecked_ref()).ok();
            cb.forget();
        }
    }
    for id in ["help", "pause"] {
        let ww = w.clone();
        if let Some(e) = el(&doc, id) {
            let cb = Closure::wrap(Box::new(move |ev: MouseEvent| {
                let mut b = ww.borrow_mut();
                if ev.target() == ev.current_target() {
                    if id == "help" && b.g.help_open { b.g.toggle_help(); hidden(&b.doc, "help", true); }
                    if id == "pause" && b.g.paused { b.toggle_pause(); hidden(&b.doc, "pause", true); }
                }
            }) as Box<dyn FnMut(MouseEvent)>);
            e.add_event_listener_with_callback("click", cb.as_ref().unchecked_ref()).ok();
            cb.forget();
        }
    }
    w.borrow_mut().start_timers();
    w.borrow_mut().apply_i18n();
    w.borrow_mut().render();
}
