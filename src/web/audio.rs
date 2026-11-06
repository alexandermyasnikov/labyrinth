use super::*;
use crate::game::*;

impl Web {
    pub(crate) fn sfx(&mut self, s: Sfx) {
        let (freq, wave, dur, vol): (f64, &str, f64, f64) = match s {
            Sfx::Move => (340.0, "sine", 0.06, 0.08),
            Sfx::Step => (220.0, "sine", 0.04, 0.05),
            Sfx::EsStep => (90.0, "sine", 0.06, 0.12),
            Sfx::Wall => (120.0, "triangle", 0.05, 0.18),
            Sfx::Chest => (660.0, "sine", 0.12, 0.25),
            Sfx::Bomb => (70.0, "sawtooth", 0.25, 0.40),
            Sfx::Bear => (220.0, "square", 0.15, 0.30),
            Sfx::Heal => (440.0, "sine", 0.20, 0.22),
            Sfx::River => (520.0, "sine", 0.25, 0.20),
            Sfx::Hole => (160.0, "sine", 0.18, 0.25),
            Sfx::Key => (700.0, "sine", 0.15, 0.22),
            Sfx::Shoot => (400.0, "square", 0.12, 0.25),
            Sfx::Danger => (300.0, "sawtooth", 0.28, 0.30),
            Sfx::Win => (880.0, "sine", 0.35, 0.35),
            Sfx::Lose => (90.0, "sawtooth", 0.4, 0.40),
            Sfx::Aim => (500.0, "sine", 0.08, 0.15),
            Sfx::Tick => (900.0, "square", 0.05, 0.18),
            Sfx::Ach => (750.0, "sine", 0.18, 0.28),
        };
        let ac = match &self.actx {
            Some(a) => a.clone(),
            None => match AudioContext::new() {
                Ok(a) => {
                    self.actx = Some(a.clone());
                    a
                }
                Err(_) => return,
            },
        };
        if ac.state() == AudioContextState::Suspended {
            let _ = ac.resume();
        }
        let t0 = ac.current_time();
        let o = match ac.create_oscillator() {
            Ok(o) => o,
            Err(_) => return,
        };
        let gn = match ac.create_gain() {
            Ok(g) => g,
            Err(_) => return,
        };
        o.set_type(match wave {
            "square" => OscillatorType::Square,
            "sawtooth" => OscillatorType::Sawtooth,
            "triangle" => OscillatorType::Triangle,
            _ => OscillatorType::Sine,
        });
        o.frequency().set_value(freq as f32);
        let _ = gn.gain().set_value_at_time(vol as f32, t0);
        let _ = gn.gain().exponential_ramp_to_value_at_time(0.001, t0 + dur);
        o.connect_with_audio_node(&gn).ok();
        gn.connect_with_audio_node(&ac.destination()).ok();
        let _ = o.start_with_when(t0);
        let _ = o.stop_with_when(t0 + dur);
    }
}
