use super::*;

#[derive(Clone, Default, Debug)]
pub struct Stats {
    pub wins: u32,
    pub best_combo: u32,
    pub explored_max: u32,
    pub speedrun: bool,
    pub clean_win: bool,
    pub unlocked: Vec<String>,
}
#[derive(Clone, Copy, Default, Debug)]
pub struct Best {
    pub level: u32,
    pub time1: Option<f64>,
}

#[allow(dead_code)]
pub const ACH_IDS: [&str; 5] = ["first", "combo10", "explorer", "speedrun", "clean"];
pub fn ach_name(lang: Lang, id: &str) -> String {
    match id {
        "first" => t(lang, "ach_first"),
        "combo10" => t(lang, "ach_combo"),
        "explorer" => t(lang, "ach_explorer"),
        "speedrun" => t(lang, "ach_speed"),
        "clean" => t(lang, "ach_clean"),
        _ => id.to_string(),
    }
}
impl Stats {
    pub fn ach_unlocked(&self, id: &str) -> bool {
        self.unlocked.iter().any(|x| x == id)
    }
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
