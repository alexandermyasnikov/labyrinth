// Веб-обвязка (только wasm32): рендер, HUD, ввод, звук, таймеры, localStorage.
// Логика — в game.rs (тестируется нативно 1-в-1).
use crate::game::*;

mod audio;
pub(crate) mod hud;
use hud::{load_best, load_stats, storage_get, storage_set};
mod render;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use web_sys::*;
type Shared = Rc<RefCell<Web>>;
thread_local! { static WEAK: RefCell<Option<Shared>> = const { RefCell::new(None) }; }
pub(crate) struct Web {
    pub(crate) g: Game,
    pub(crate) ctx: CanvasRenderingContext2d,
    pub(crate) doc: Document,
    pub(crate) win: Window,
    pub(crate) tick_id: Option<i32>,
    bear_id: Option<i32>,
    zombie_id: Option<i32>,
    pub(crate) shooter_id: Option<i32>,
    boss_id: Option<i32>,
    river_id: Option<i32>,
    anim_id: Option<i32>,
    pub(crate) actx: Option<AudioContext>,
    pub(crate) particles: Vec<Particle>,
    pub(crate) anim_from: Option<Pos>,
    pub(crate) cam: f64,
    pub(crate) pad_dir: (i32, i32),
    pub(crate) seed_is_url: bool,
    pub(crate) msg_until: u64,
    pub(crate) log_shown: String,
    pub(crate) saved_combo: u32,
    pub(crate) saved_explored: u32,
}
#[derive(Clone)]
struct Particle {
    x: f64,
    y: f64,
    vx: f64,
    vy: f64,
    life: u64,
    t0: u64,
    color: String,
    size: f64,
}
fn el(doc: &Document, id: &str) -> Option<Element> {
    doc.get_element_by_id(id)
}
fn txt(doc: &Document, id: &str, v: &str) {
    if let Some(e) = el(doc, id) {
        e.set_text_content(Some(v));
    }
}
fn html(doc: &Document, id: &str, v: &str) {
    if let Some(e) = el(doc, id) {
        e.set_inner_html(v);
    }
}
fn hidden(doc: &Document, id: &str, h: bool) {
    if let Some(e) = el(doc, id) {
        let _ = e.toggle_attribute_with_force("hidden", h);
    }
}
fn now_ms(win: &Window) -> u64 {
    win.performance().map(|p| p.now() as u64).unwrap_or(0)
}
fn shared() -> Shared {
    WEAK.with(|w| w.borrow().clone().expect("web not registered"))
}
fn each(win: &Window, ms: i32, f: impl Fn() + 'static) -> i32 {
    let cb = Closure::wrap(Box::new(f) as Box<dyn FnMut()>);
    let id = win
        .set_interval_with_callback_and_timeout_and_arguments(
            cb.as_ref().unchecked_ref(),
            ms,
            &js_sys::Array::new(),
        )
        .unwrap();
    cb.forget();
    id
}

impl Web {
    fn new() -> Shared {
        let win = web_sys::window().expect("window");
        let doc = win.document().expect("document");
        let cv: HtmlCanvasElement = doc.get_element_by_id("cv").expect("cv").dyn_into().unwrap();
        let ctx = cv
            .get_context("2d")
            .expect("ctx")
            .expect("ctx")
            .dyn_into()
            .unwrap();
        let query = win.location().search().unwrap_or_default();
        let url_seed = parse_seed_from_query(&query);
        let seed = url_seed.unwrap_or_else(|| (now_ms(&win) % 1_000_000) as u32);
        let lang = storage_get(&win, "labyrinth.lang")
            .and_then(|s| Lang::from_code(&s))
            .unwrap_or_else(|| match win.navigator().language() {
                Some(l) if l.to_lowercase().starts_with("ru") => Lang::Ru,
                _ => Lang::En,
            });
        let g = Game::new(seed, lang, load_stats(&win), load_best(&win));
        Rc::new(RefCell::new(Web {
            g,
            ctx,
            doc,
            win,
            tick_id: None,
            bear_id: None,
            zombie_id: None,
            shooter_id: None,
            boss_id: None,
            river_id: None,
            anim_id: None,
            actx: None,
            particles: vec![],
            anim_from: None,
            cam: 0.0,
            pad_dir: (-1, 0),
            seed_is_url: url_seed.is_some(),
            msg_until: 0,
            log_shown: String::new(),
            saved_combo: 0,
            saved_explored: 0,
        }))
    }
    fn clear_timers(&mut self) {
        for id in [
            &mut self.tick_id,
            &mut self.bear_id,
            &mut self.zombie_id,
            &mut self.shooter_id,
            &mut self.boss_id,
            &mut self.river_id,
        ] {
            if let Some(i) = id.take() {
                self.win.clear_interval_with_handle(i);
            }
        }
    }
    fn start_timers(&mut self) {
        self.clear_timers();
        let win = self.win.clone();
        let wa = shared();
        self.tick_id = Some(each(&win, 1000, move || {
            wa.borrow_mut().tick();
        }));
        self.bear_id = Some(each(&win, BEAR_MS as i32, move || {
            let w = shared();
            w.borrow_mut().turn("bear");
        }));
        self.zombie_id = Some(each(&win, ZOMBIE_MS as i32, move || {
            let w = shared();
            w.borrow_mut().turn("zombie");
        }));
        self.shooter_id = Some(each(&win, SHOOTER_MS as i32, move || {
            let w = shared();
            w.borrow_mut().turn("shooter");
        }));
        self.boss_id = Some(each(&win, BOSS_MS as i32, move || {
            let w = shared();
            w.borrow_mut().turn("boss");
        }));
        let wa = shared();
        self.anim_id = Some(each(&win, 16, move || {
            wa.borrow_mut().animate();
        }));
    }

    fn tick(&mut self) {
        self.g.tick_second(now_ms(&self.win));
        self.flush();
        self.render();
    }
    fn turn(&mut self, kind: &str) {
        let now = now_ms(&self.win);
        if self.g.over || self.g.paused {
            return;
        }
        match kind {
            "bear" => self.g.bear_turn(now),
            "zombie" => self.g.zombie_turn(now),
            "shooter" => self.g.shooter_turn(now),
            _ => self.g.boss_turn(now),
        }
        self.flush();
        self.render();
    }
    fn lake_tick(&mut self) {
        self.g.lake_tick();
        self.flush();
        self.render();
    }
    fn animate(&mut self) {
        let now = now_ms(&self.win);
        if self.msg_until > 0 && now >= self.msg_until {
            if let Some(s) = el(&self.doc, "status") {
                let _ = s.class_list().remove_1("show");
            }
            self.msg_until = 0;
        }
        self.g.shots.retain(|s| now.saturating_sub(s.t0) < 250);
        if self
            .g
            .boom
            .map(|b| now.saturating_sub(b.t0) >= 300)
            .unwrap_or(false)
        {
            self.g.boom = None;
        }
        if self
            .g
            .dash_flash
            .map(|b| now.saturating_sub(b.t0) >= 250)
            .unwrap_or(false)
        {
            self.g.dash_flash = None;
        }
        self.render();
    }
    fn restart(&mut self, new_world: bool) {
        let now = now_ms(&self.win);
        // после победы следующий уровень — НОВЫЙ лабиринт (сид шагает),
        // рестарт посреди уровня — тот же мир (как было в JS: R = same world)
        let fresh = new_world || self.g.over_kind == Some(OverKind::Win);
        if fresh && !self.seed_is_url {
            self.g.seed = (now % 1_000_000) as u32;
        }
        if fresh {
            self.g.new_world(now);
        } else {
            self.g.restart(now);
        }
        self.anim_from = None;
        self.cam = 0.0;
        self.msg_until = 0;
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
        if self.g.player != prev {
            self.anim_from = Some(prev);
            self.cam = 0.0;
        }
        self.flush();
        self.render();
    }
    fn do_dash(&mut self, dr: i32, dc: i32) {
        let now = now_ms(&self.win);
        let prev = self.g.player;

        self.g.dash(dr, dc, now);
        if self.g.player != prev {
            self.anim_from = Some(prev);
            self.cam = 0.0;
        }
        self.flush();
        self.render();
    }
    fn do_bomb(&mut self, dr: i32, dc: i32) {
        let now = now_ms(&self.win);
        self.g.bomb(dr, dc, now);
        self.flush();
        self.render();
    }
    fn toggle_pause(&mut self) {
        self.g.toggle_pause();
        hidden(&self.doc, "pause", !self.g.paused);
        self.render();
    }
    fn on_key(&mut self, e: &KeyboardEvent) {
        let code = e.code();
        match code.as_str() {
            "KeyP" | "Escape" => {
                self.toggle_pause();
                return;
            }
            "KeyR" => {
                self.restart(false);
                return;
            }
            "KeyN" => {
                self.restart(true);
                return;
            }
            _ => {}
        }
        if self.g.paused {
            return;
        }
        match code.as_str() {
            "KeyH" => {
                self.g.toggle_help();
                hidden(&self.doc, "help", !self.g.help_open);
                self.render();
                return;
            }
            "KeyB" => {
                if self.g.over {
                    return;
                }
                if self.g.bombs <= 0 {
                    self.g.say(self.g.t("w_noBombs").to_string());
                    self.flush();
                    return;
                }
                self.g.aiming = true;
                self.sfx(Sfx::Aim);
                self.g.say(self.g.t("w_aim").to_string());
                self.flush();
                self.render();
                return;
            }
            _ => {}
        }
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
            if self.g.aiming {
                self.g.aiming = false;
                self.do_bomb(dr, dc);
            } else if e.shift_key() {
                self.do_dash(dr, dc);
            } else {
                self.do_move(dr, dc);
            }
        }
    }
}

pub fn run() {
    let w = Web::new();
    WEAK.with(|x| *x.borrow_mut() = Some(w.clone()));
    // клавиатура
    {
        let ww = w.clone();
        let cb = Closure::wrap(Box::new(move |e: KeyboardEvent| {
            ww.borrow_mut().on_key(&e);
        }) as Box<dyn FnMut(KeyboardEvent)>);
        web_sys::window()
            .unwrap()
            .add_event_listener_with_callback("keydown", cb.as_ref().unchecked_ref())
            .ok();
        cb.forget();
    }
    let doc = w.borrow().doc.clone();
    for (id, is_new) in [
        ("restart", false),
        ("new-world", true),
        ("ov-replay", false),
    ] {
        let ww = w.clone();
        if let Some(e) = el(&doc, id) {
            let cb = Closure::wrap(Box::new(move || {
                ww.borrow_mut().restart(is_new);
            }) as Box<dyn FnMut()>);
            e.add_event_listener_with_callback("click", cb.as_ref().unchecked_ref())
                .ok();
            cb.forget();
        }
    }
    for (id, dir) in [
        ("pad-up", (-1, 0)),
        ("pad-down", (1, 0)),
        ("pad-left", (0, -1)),
        ("pad-right", (0, 1)),
    ] {
        let ww = w.clone();
        if let Some(e) = el(&doc, id) {
            let cb = Closure::wrap(Box::new(move |ev: PointerEvent| {
                ev.prevent_default();
                let mut b = ww.borrow_mut();
                b.pad_dir = dir;
                if b.g.aiming {
                    b.g.aiming = false;
                    b.do_bomb(dir.0, dir.1);
                } else {
                    b.do_move(dir.0, dir.1);
                }
            }) as Box<dyn FnMut(PointerEvent)>);
            e.add_event_listener_with_callback("pointerdown", cb.as_ref().unchecked_ref())
                .ok();
            cb.forget();
        }
    }
    {
        let ww = w.clone();
        if let Some(e) = el(&doc, "pad-dash") {
            let cb = Closure::wrap(Box::new(move |ev: PointerEvent| {
                ev.prevent_default();
                let (dr, dc) = ww.borrow().pad_dir;
                ww.borrow_mut().do_dash(dr, dc);
            }) as Box<dyn FnMut(PointerEvent)>);
            e.add_event_listener_with_callback("pointerdown", cb.as_ref().unchecked_ref())
                .ok();
            cb.forget();
        }
    }
    {
        let ww = w.clone();
        if let Some(e) = el(&doc, "pad-bomb") {
            let cb = Closure::wrap(Box::new(move |ev: PointerEvent| {
                ev.prevent_default();
                let mut b = ww.borrow_mut();
                if b.g.over || b.g.paused {
                    return;
                }
                let no_bombs = b.g.t("w_noBombs");
                let aim = b.g.t("w_aim");
                if b.g.bombs <= 0 {
                    b.g.say(no_bombs);
                    b.flush();
                    return;
                }
                b.g.aiming = true;
                b.sfx(Sfx::Aim);
                b.g.say(aim);
                b.flush();
                b.render();
            }) as Box<dyn FnMut(PointerEvent)>);
            e.add_event_listener_with_callback("pointerdown", cb.as_ref().unchecked_ref())
                .ok();
            cb.forget();
        }
    }
    {
        let ww = w.clone();
        if let Some(e) = el(&doc, "pause-btn") {
            let cb = Closure::wrap(Box::new(move || {
                ww.borrow_mut().toggle_pause();
            }) as Box<dyn FnMut()>);
            e.add_event_listener_with_callback("click", cb.as_ref().unchecked_ref())
                .ok();
            cb.forget();
        }
    }
    {
        let ww = w.clone();
        if let Some(e) = el(&doc, "resume-btn") {
            let cb = Closure::wrap(Box::new(move || {
                let mut b = ww.borrow_mut();
                if b.g.paused {
                    b.toggle_pause();
                }
            }) as Box<dyn FnMut()>);
            e.add_event_listener_with_callback("click", cb.as_ref().unchecked_ref())
                .ok();
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
                b.apply_i18n();
                b.flush();
                b.render();
            }) as Box<dyn FnMut()>);
            e.add_event_listener_with_callback("click", cb.as_ref().unchecked_ref())
                .ok();
            cb.forget();
        }
    }
    for id in ["help", "pause"] {
        let ww = w.clone();
        if let Some(e) = el(&doc, id) {
            let cb = Closure::wrap(Box::new(move |ev: MouseEvent| {
                let mut b = ww.borrow_mut();
                if ev.target() == ev.current_target() {
                    if id == "help" && b.g.help_open {
                        b.g.toggle_help();
                        hidden(&b.doc, "help", true);
                    }
                    if id == "pause" && b.g.paused {
                        b.toggle_pause();
                        hidden(&b.doc, "pause", true);
                    }
                }
            }) as Box<dyn FnMut(MouseEvent)>);
            e.add_event_listener_with_callback("click", cb.as_ref().unchecked_ref())
                .ok();
            cb.forget();
        }
    }
    w.borrow_mut().start_timers();
    w.borrow_mut().apply_i18n();
    w.borrow_mut().flush();
    w.borrow_mut().render();
}
