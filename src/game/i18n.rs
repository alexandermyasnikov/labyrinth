#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Lang {
    Ru,
    En,
}
impl Lang {
    pub fn other(self) -> Lang {
        match self {
            Lang::Ru => Lang::En,
            Lang::En => Lang::Ru,
        }
    }
    pub fn code(self) -> &'static str {
        match self {
            Lang::Ru => "ru",
            Lang::En => "en",
        }
    }
    pub fn from_code(s: &str) -> Option<Lang> {
        match s {
            "ru" => Some(Lang::Ru),
            "en" => Some(Lang::En),
            _ => None,
        }
    }
}

pub const RU: &[(&str, &str)] = &[
    ("hudHealth","Здоровье"),("hudBombs","Бомбы"),("hudChest","Клад"),("hudKey","Ключ"),
    ("hudLevel","Уровень"),("hudMoves","Ходы"),("hudTime","Время"),("hudScore","Очки"),("hudCombo","Комбо"),("hudDash","Рывок"),
    ("logHint","Стрелки / WASD — ход · Shift+стрелка — рывок · B — бомба · R — заново · N — новый мир · P — пауза"),
    ("statusStart","Найди сундук и вынеси его через выход."),("title","Лабиринт"),
    ("leg","<span>🧙 игрок</span><span>💰 сундук</span><span>🚩 выход</span><span>🗝️ ключ</span><span>💉 госпиталь</span><span>📦 арсенал</span><span>🌀 озеро</span><span>🕳 дырка</span><span>🪷 болото</span><span>🐻 медведь</span><span>⚠️ ловушка</span><span>🔥 огонь</span><span>🧟 зомби</span><span>👮 стрелок</span><span>🥷 босс</span><span><span class=\"k\">B</span> — взорвать стену · <span class=\"k\">R</span> — заново</span>"),
    ("btnRestart","Заново (R)"),("btnNewWorld","🎲 Новый мир (N)"),("btnReplay","Играть снова"),("btnNextLevel","Следующий уровень"),("btnPause","⏸"),("btnResume","Продолжить (P)"),
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
    ("btnRestart","Restart (R)"),("btnNewWorld","🎲 New maze (N)"),("btnReplay","Play again"),("btnNextLevel","Next level"),("btnPause","⏸"),("btnResume","Resume (P)"),
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
    let table = match lang {
        Lang::Ru => RU,
        Lang::En => EN,
    };
    for (k, v) in table {
        if *k == key {
            return v.to_string();
        }
    }
    for (k, v) in RU {
        if *k == key {
            return v.to_string();
        }
    }
    key.to_string()
}
pub fn dir_name(lang: Lang, d: usize) -> &'static str {
    match (lang, d) {
        (Lang::Ru, 0) => "север",
        (Lang::Ru, 1) => "юг",
        (Lang::Ru, 2) => "запад",
        (Lang::Ru, 3) => "восток",
        (Lang::En, 0) => "north",
        (Lang::En, 1) => "south",
        (Lang::En, 2) => "west",
        _ => "east",
    }
}
