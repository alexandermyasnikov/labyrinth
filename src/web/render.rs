use super::*;
use crate::game::*;

impl Web {
    pub(crate) fn burst(&mut self, color: &str, n: usize) {
        let now = now_ms(&self.win);
        for i in 0..n {
            let ang = (i as f64) * 2.39996;
            let sp = 40.0 + (ang * 37.0).fract().abs() * 120.0;
            self.particles.push(Particle {
                x: 0.0,
                y: 0.0,
                vx: ang.cos() * sp,
                vy: ang.sin() * sp,
                life: 350 + ((ang * 53.0).fract() * 400.0) as u64,
                t0: now,
                color: color.to_string(),
                size: 2.0 + (ang * 29.0).fract() * 4.0,
            });
        }
        if self.particles.len() > 240 {
            self.particles = self.particles.split_off(self.particles.len() - 240);
        }
    }

    pub(crate) fn render(&mut self) {
        let g = &self.g;
        let n = (VIEW * CELL as usize) as f64;
        let c = n / 2.0;
        self.ctx.clear_rect(0.0, 0.0, n, n);
        if g.paused {
            self.ctx
                .set_fill_style(&wasm_bindgen::JsValue::from_str("rgba(6,8,12,.45)"));
            self.ctx.fill_rect(0.0, 0.0, n, n);
            self.ctx.set_font("28px sans-serif");
            self.ctx.set_text_align("center");
            self.ctx
                .set_fill_style(&wasm_bindgen::JsValue::from_str("#ffe9a8"));
            let pt = g.t("pausedTitle");
            self.ctx.fill_text(&pt, c, c).ok();
            return;
        }
        let now = now_ms(&self.win);
        if self.anim_from.is_some() && self.cam < 1.0 {
            self.cam += (1.0 - self.cam) * 0.56_f64.min(1.0);
            if self.cam > 0.999 {
                self.cam = 1.0;
                self.anim_from = None;
            }
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
            (
                (((now / 16) % 5) as f64 - 2.0) * 2.0 * k,
                (((now / 16 + 2) % 5) as f64 - 2.0) * 2.0 * k,
            )
        } else {
            (0.0, 0.0)
        };
        let base_r = dr.round() as i32;
        let base_c = dc.round() as i32;
        let r0 = (base_r - 4).max(0);
        let r1 = (base_r + 5).min(H as i32 - 1);
        let c0 = (base_c - 4).max(0);
        let c1 = (base_c + 5).min(W as i32 - 1);
        let sx = |r: i32| c + (r as f64 - dr) * CELL + shy;
        let sc = |cc: i32| c + (cc as f64 - dc) * CELL + shx;
        let cell_bg = |sym: &str| -> &'static str {
            match sym {
                "🚩" => "#3a2f14",
                "💰" => "#3a3318",
                "🌀" => "#17324a",
                "🕳" => "#1a1a1f",
                "📦" => "#2f2a18",
                "💉" => "#2a1f24",
                "🪷" => "#20301c",
                "⚠️" => "#2b2118",
                _ => "#262a34",
            }
        };
        for r in r0..=r1 {
            for cc in c0..=c1 {
                let (r, cc) = (r as usize, cc as usize);
                if !g.explored[r][cc] || g.visible(r, cc) {
                    continue;
                }
                let x = sc(cc as i32);
                let y = sx(r as i32);
                self.ctx
                    .set_fill_style(&wasm_bindgen::JsValue::from_str("rgba(38,42,52,.35)"));
                self.ctx.fill_rect(x, y, CELL, CELL);
                self.ctx
                    .set_stroke_style(&wasm_bindgen::JsValue::from_str("rgba(255,255,255,.05)"));
                self.ctx.set_line_width(1.0);
                self.ctx.stroke_rect(x, y, CELL, CELL);
            }
        }
        for r in r0..=r1 {
            for cc in c0..=c1 {
                let (r, cc) = (r as usize, cc as usize);
                if !g.visible(r, cc) {
                    continue;
                }
                let x = sc(cc as i32);
                let y = sx(r as i32);
                let sym = g.cell_symbol(r, cc);
                self.ctx
                    .set_fill_style(&wasm_bindgen::JsValue::from_str(cell_bg(sym)));
                self.ctx.fill_rect(x, y, CELL, CELL);
                self.ctx
                    .set_stroke_style(&wasm_bindgen::JsValue::from_str("rgba(255,255,255,.04)"));
                self.ctx.set_line_width(1.0);
                self.ctx.stroke_rect(x, y, CELL, CELL);
                if !sym.is_empty() {
                    self.draw_emoji(sym, x + CELL / 2.0, y + CELL / 2.0, 30.0);
                }
            }
        }
        self.ctx.set_line_cap("round");
        for r in r0..=r1 {
            for cc in c0..=c1 {
                let (r, cc) = (r as usize, cc as usize);
                if !g.visible(r, cc) {
                    continue;
                }
                let x = sc(cc as i32);
                let y = sx(r as i32);
                self.ctx
                    .set_stroke_style(&wasm_bindgen::JsValue::from_str("#05070b"));
                self.ctx.set_line_width(5.0);
                for d in 0..4 {
                    if !g.wall[r][cc][d] {
                        continue;
                    }
                    self.ctx.begin_path();
                    match d {
                        0 => {
                            self.ctx.move_to(x, y);
                            self.ctx.line_to(x + CELL, y);
                        }
                        1 => {
                            self.ctx.move_to(x, y + CELL);
                            self.ctx.line_to(x + CELL, y + CELL);
                        }
                        2 => {
                            self.ctx.move_to(x, y);
                            self.ctx.line_to(x, y + CELL);
                        }
                        _ => {
                            self.ctx.move_to(x + CELL, y);
                            self.ctx.line_to(x + CELL, y + CELL);
                        }
                    }
                    self.ctx.stroke();
                }
            }
        }
        let vis = |p: Pos| g.visible(p.0, p.1);
        for &b in &g.bears {
            if vis(b) {
                self.draw_emoji(
                    "🐻",
                    sc(b.1 as i32) + CELL / 2.0,
                    sx(b.0 as i32) + CELL / 2.0,
                    28.0,
                );
            }
        }
        for &f in &g.fire {
            if vis(f) {
                self.draw_emoji(
                    "🔥",
                    sc(f.1 as i32) + CELL / 2.0,
                    sx(f.0 as i32) + CELL / 2.0,
                    22.0,
                );
            }
        }
        for &z in &g.zombies {
            if vis(z) {
                self.draw_emoji(
                    "🧟",
                    sc(z.1 as i32) + CELL / 2.0,
                    sx(z.0 as i32) + CELL / 2.0,
                    24.0,
                );
            }
        }
        for &s in &g.shooters {
            if vis(s) {
                let e = if g.shooter_aim.contains(&s) {
                    "🏹"
                } else {
                    "👮"
                };
                self.draw_emoji(
                    e,
                    sc(s.1 as i32) + CELL / 2.0,
                    sx(s.0 as i32) + CELL / 2.0,
                    24.0,
                );
            }
        }
        for b in &g.shots {
            let t = (now.saturating_sub(b.t0)) as f64 / 250.0;
            if t >= 1.0 {
                continue;
            }
            let px =
                sc((b.from.1 as f64 + (b.to.1 as f64 - b.from.1 as f64) * t) as i32) + CELL / 2.0;
            let py =
                sx((b.from.0 as f64 + (b.to.0 as f64 - b.from.0 as f64) * t) as i32) + CELL / 2.0;
            self.ctx
                .set_fill_style(&wasm_bindgen::JsValue::from_str("#ffd166"));
            self.ctx.begin_path();
            self.ctx.arc(px, py, 4.0, 0.0, 7.0).ok();
            self.ctx.fill();
        }
        if let Some(bm) = g.boom {
            let bt = (now.saturating_sub(bm.t0)) as f64 / 300.0;
            if bt < 1.0 {
                let bx = sc(bm.pos.1 as i32) + CELL / 2.0;
                let by = sx(bm.pos.0 as i32) + CELL / 2.0;
                self.ctx
                    .set_stroke_style(&wasm_bindgen::JsValue::from_str(&format!(
                        "rgba(255,159,67,{:.2})",
                        1.0 - bt
                    )));
                self.ctx.set_line_width(6.0 * (1.0 - bt));
                self.ctx.begin_path();
                self.ctx
                    .arc(bx, by, CELL * 0.4 + bt * CELL * 0.9, 0.0, 7.0)
                    .ok();
                self.ctx.stroke();
            }
        }
        if let Some(df) = g.dash_flash {
            let dt = (now.saturating_sub(df.t0)) as f64 / 250.0;
            if dt < 1.0 {
                let dx = sc(df.pos.1 as i32) + CELL / 2.0;
                let dy = sx(df.pos.0 as i32) + CELL / 2.0;
                self.ctx
                    .set_stroke_style(&wasm_bindgen::JsValue::from_str(&format!(
                        "rgba(255,255,255,{:.2})",
                        1.0 - dt
                    )));
                self.ctx.set_line_width(3.0 * (1.0 - dt));
                self.ctx.begin_path();
                self.ctx
                    .arc(dx, dy, CELL * 0.3 + dt * CELL * 0.8, 0.0, 7.0)
                    .ok();
                self.ctx.stroke();
            }
        }
        {
            let grad = self
                .ctx
                .create_radial_gradient(
                    c + CELL / 2.0,
                    c + CELL / 2.0,
                    4.0,
                    c + CELL / 2.0,
                    c + CELL / 2.0,
                    28.0,
                )
                .expect("grad");
            grad.add_color_stop(0.0, "rgba(255,209,102,.5)")
                .expect("glow");
            grad.add_color_stop(1.0, "rgba(255,209,102,0)")
                .expect("glow");
            self.ctx.set_fill_style(&wasm_bindgen::JsValue::from(grad));
            self.ctx.begin_path();
            self.ctx
                .arc(c + CELL / 2.0, c + CELL / 2.0, 26.0, 0.0, 7.0)
                .ok();
            self.ctx.fill();
        }
        if !g.shooter_aim.is_empty() || !g.boss_aim.is_empty() {
            let blink = (now / 400) % 2 < 1;
            self.ctx
                .set_stroke_style(&wasm_bindgen::JsValue::from_str(if blink {
                    "#e63946"
                } else {
                    "#ffd166"
                }));
            self.ctx.set_line_width(4.0);
            self.ctx.begin_path();
            self.ctx
                .arc(c + CELL / 2.0, c + CELL / 2.0, CELL / 2.0 + 4.0, 0.0, 7.0)
                .ok();
            self.ctx.stroke();
        }
        self.draw_emoji("🧙", c + CELL / 2.0, c + CELL / 2.0, 50.0);
        {
            let bw = 38.0;
            let bh = 5.0;
            let bx = c + CELL / 2.0 - bw / 2.0;
            let by = c + CELL / 2.0 - CELL / 2.0 + 4.0;
            self.ctx
                .set_fill_style(&wasm_bindgen::JsValue::from_str("rgba(0,0,0,.7)"));
            self.ctx.fill_rect(bx - 1.0, by - 1.0, bw + 2.0, bh + 2.0);
            let frac = ((g.health as f64) / MAX_HEALTH as f64).clamp(0.0, 1.0);
            self.ctx
                .set_fill_style(&wasm_bindgen::JsValue::from_str(if frac > 0.6 {
                    "#58d68d"
                } else if frac > 0.3 {
                    "#f5c542"
                } else {
                    "#e63946"
                }));
            self.ctx.fill_rect(bx, by, bw * frac, bh);
        }
        let (dir, dx, dy, dist) = g.direction_to_target();
        if dir != "🏆" && dist > FOG_RADIUS {
            let off = CELL / 2.0 - 6.0;
            let px = c + CELL / 2.0 + dx as f64 * off;
            let py = c + CELL / 2.0 + dy as f64 * off;
            self.ctx.set_global_alpha(0.85);
            self.ctx
                .set_fill_style(&wasm_bindgen::JsValue::from_str("rgba(24,27,34,.55)"));
            self.ctx.begin_path();
            self.ctx.arc(px, py, 16.0, 0.0, 7.0).ok();
            self.ctx.fill();
            self.ctx.set_font("26px serif");
            self.ctx.set_text_align("center");
            self.ctx.set_text_baseline("middle");
            self.ctx
                .set_fill_style(&wasm_bindgen::JsValue::from_str("#ffd166"));
            self.ctx.fill_text(&dir, px, py).ok();
            self.ctx.set_global_alpha(1.0);
        }
        if g.hurt_until > now {
            let a = (g.hurt_until - now) as f64 / 350.0;
            let gr = self
                .ctx
                .create_radial_gradient(c, c, n * 0.5, c, c, n * 0.75)
                .expect("hurt");
            gr.add_color_stop(0.0, "rgba(230,57,70,0)").expect("hurt");
            gr.add_color_stop(1.0, &format!("rgba(230,57,70,{:.2})", 0.5 * a))
                .expect("hurt");
            self.ctx.set_fill_style(&wasm_bindgen::JsValue::from(gr));
            self.ctx.fill_rect(0.0, 0.0, n, n);
        }
        if g.health <= 2 && g.health > 0 {
            let a = 0.15 + 0.12 * ((now / 300) as f64).sin();
            let gr = self
                .ctx
                .create_radial_gradient(c, c, n * 0.5, c, c, n * 0.8)
                .expect("low");
            gr.add_color_stop(0.0, "rgba(200,40,50,0)").expect("low");
            gr.add_color_stop(1.0, &format!("rgba(200,40,50,{:.2})", a))
                .expect("low");
            self.ctx.set_fill_style(&wasm_bindgen::JsValue::from(gr));
            self.ctx.fill_rect(0.0, 0.0, n, n);
        }
        for &b in &g.bosses {
            let bx = sc(b.1 as i32) + CELL / 2.0;
            let by = sx(b.0 as i32) + CELL / 2.0;
            self.ctx.set_fill_style(&wasm_bindgen::JsValue::from_str(
                if g.boss_aim.contains(&b) {
                    "#7a2a14"
                } else {
                    "#2b0f18"
                },
            ));
            self.ctx.begin_path();
            self.ctx.arc(bx, by, 12.0, 0.0, 7.0).ok();
            self.ctx.fill();
            self.ctx
                .set_stroke_style(&wasm_bindgen::JsValue::from_str("rgba(255,209,102,.5)"));
            self.ctx.set_line_width(2.0);
            self.ctx.begin_path();
            self.ctx.arc(bx, by, 12.0, 0.0, 7.0).ok();
            self.ctx.stroke();
            self.draw_emoji(
                if g.boss_aim.contains(&b) {
                    "🎯"
                } else {
                    "🥷"
                },
                bx,
                by,
                26.0,
            );
        }
        self.particles
            .retain(|p| now.saturating_sub(p.t0) <= p.life);
        for p in &self.particles {
            let age = now.saturating_sub(p.t0);
            let k = age as f64 / p.life as f64;
            self.ctx.set_global_alpha(1.0 - k);
            self.ctx
                .set_fill_style(&wasm_bindgen::JsValue::from_str(&p.color));
            self.ctx.fill_rect(
                p.x + p.vx * age as f64 / 1000.0 - p.size / 2.0,
                p.y + p.vy * age as f64 / 1000.0 - p.size / 2.0,
                p.size,
                p.size,
            );
        }
        self.ctx.set_global_alpha(1.0);
        if self.g.shake > 0.0 {
            self.g.shake = (self.g.shake - 0.8).max(0.0);
        }
    }

    fn draw_emoji(&self, sym: &str, cx: f64, cy: f64, size: f64) {
        // shortcut: fillText-центрирование вместо JS bbox-кэша
        self.ctx.set_font(&format!("{}px sans-serif", size as i32));
        self.ctx.set_text_align("center");
        self.ctx.set_text_baseline("middle");
        self.ctx.fill_text(sym, cx, cy).ok();
    }
}
