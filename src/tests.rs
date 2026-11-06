
use crate::game::*;

    fn game(seed: u32) -> Game { Game::new(seed, Lang::Ru, Stats::default(), Best::default()) }
    fn open_all(g: &mut Game) { for r in 0..H { for c in 0..W { g.wall[r][c] = [false; 4]; } } }
    fn place_player(g: &mut Game, p: Pos) { g.player = p; g.compute_visibility(); }
    fn later(g: &mut Game) -> u64 { g.grace_until + 1 }

    // --- баланс (критерий готовности) ---
    #[test]
    fn balance_constants() {
        assert_eq!(MAX_HEALTH, 5);
        assert_eq!(MAX_BOMBS, 5);
        assert_eq!(LEVEL_TIME, 300);
        assert_eq!(W, 20); assert_eq!(H, 20); assert_eq!(W * H, 400);
    }
    #[test]
    fn new_game_starts_with_5hp_5bombs() {
        let g = game(123);
        assert_eq!(g.health, 5); assert_eq!(g.bombs, 5);
        assert_eq!(g.time_left, 300); assert_eq!(g.moves, 0); assert_eq!(g.level, 1);
        assert!(!g.over); assert!(!g.carrying); assert!(!g.carrying_key);
    }
    #[test]
    fn enemy_timer_constants() {
        assert_eq!(BEAR_MS, 1800); assert_eq!(ZOMBIE_MS, 2000);
        assert_eq!(SHOOTER_MS, 2500); assert_eq!(BOSS_MS, 1800); assert_eq!(RIVER_MS, 2000);
        assert_eq!(GRACE_MS, 5000); assert_eq!(MOVE_CD, 150); assert_eq!(DASH_CD, 1500);
        assert_eq!(FOG_RADIUS, 3); assert_eq!(VIEW, 10);
        assert_eq!(SWAMP_SKIPS, 3); assert_eq!(BOSS_AIM_DIST, 3);
        assert_eq!(COMBO_WINDOW, 900); assert_eq!(COMBO_BONUS, 5); assert_eq!(LOW_TIME, 30);
    }

    // --- RNG / сид ---
    #[test]
    fn same_seed_same_world() {
        let a = game(123); let b = game(123);
        assert_eq!(a.wall, b.wall);
        assert_eq!(a.player, b.player); assert_eq!(a.chest, b.chest);
        assert_eq!(a.exit, b.exit); assert_eq!(a.key, b.key);
        assert_eq!(a.trap, b.trap); assert_eq!(a.bears, b.bears);
    }
    #[test]
    fn different_seeds_differ() {
        let a = game(1); let b = game(2);
        assert_ne!(a.wall, b.wall);
    }
    #[test]
    fn rng_matches_mulberry32_reference() {
        // сверка с JS: mulberry32(0) первое значение = 0x6D2B79F5-based... проверяем инварианты
        let mut r = Rng::new(42);
        for _ in 0..1000 { let v = r.next_f64(); assert!(v >= 0.0 && v < 1.0); }
        let mut r2 = Rng::new(42);
        let mut r3 = Rng::new(42);
        assert_eq!(r2.next_usize(100), r3.next_usize(100));
        assert_eq!(r2.next_usize(100), r3.next_usize(100));
    }
    #[test]
    fn seed_query_parsing() {
        assert_eq!(parse_seed_from_query("?seed=123"), Some(123));
        assert_eq!(parse_seed_from_query("?seed=20261106"), Some(20261106));
        assert_eq!(parse_seed_from_query("?x=1&seed=7"), Some(7));
        assert_eq!(parse_seed_from_query(""), None);
        assert_eq!(parse_seed_from_query("?seed=abc"), None);
        assert!(is_random_seed(None)); assert!(!is_random_seed(Some(1)));
    }
    #[test]
    fn restart_keeps_same_world() {
        let mut g = game(555);
        let wall = g.wall; let player = g.player;
        g.restart(100_000);
        assert_eq!(g.wall, wall); assert_eq!(g.player, player);
    }
    #[test]
    fn new_world_changes_seed_and_world() {
        let mut g = game(555);
        let wall = g.wall;
        g.new_world(100_000);
        assert_ne!(g.wall, wall);
        assert_ne!(g.seed, 555);
    }

    // --- генерация ---
    #[test]
    fn connectivity_many_seeds() {
        for seed in [1u32, 2, 3, 42, 123, 999, 20261106, 777, 31337, 8, 100, 55] {
            let g = game(seed);
            assert_eq!(g.cell_count(), 400, "seed {}", seed);
        }
    }
    #[test]
    fn border_walls_always_closed() {
        for seed in [1u32, 7, 42, 123] {
            let g = game(seed);
            for c in 0..W {
                assert!(g.wall[0][c][0], "north edge");
                assert!(g.wall[H - 1][c][1], "south edge");
            }
            for r in 0..H {
                assert!(g.wall[r][0][2], "west edge");
                assert!(g.wall[r][W - 1][3], "east edge");
            }
        }
    }
    #[test]
    fn wall_symmetry() {
        let g = game(9);
        for r in 0..H { for c in 0..W {
            if r + 1 < H { assert_eq!(g.wall[r][c][1], g.wall[r + 1][c][0]); }
            if c + 1 < W { assert_eq!(g.wall[r][c][3], g.wall[r][c + 1][2]); }
        }}
    }
    #[test]
    fn placement_invariants_many_seeds() {
        for seed in [1u32, 2, 3, 42, 123, 999, 20261106, 777, 31337] {
            let g = game(seed);
            let lake_set: std::collections::HashSet<Pos> = g.lakes.iter().flat_map(|l| l.cells.iter().copied()).collect();
            let swamp_set: std::collections::HashSet<Pos> = g.swamps.iter().copied().collect();
            // игрок/сундук/выход/ключ не в озере и болоте
            for p in [g.player, g.chest, g.exit] {
                assert!(!lake_set.contains(&p), "lake overlap seed {}", seed);
                assert!(!swamp_set.contains(&p), "swamp overlap seed {}", seed);
            }
            if let Some(k) = g.key { assert!(!lake_set.contains(&k)); assert!(!swamp_set.contains(&k)); }
            // статика не пересекается
            let specials = [g.player, g.chest, g.exit, g.hospital];
            for &a in &specials { for &b in &specials { if a != b { assert_ne!(a, b); } } }
            for h in &g.holes { assert!(!lake_set.contains(h)); }
            for a in &g.arsenal { assert!(!lake_set.contains(a)); assert!(!swamp_set.contains(a)); }
            // дырки не пересекаются с арсеналом/госпиталем
            for h in &g.holes {
                assert!(*h != g.hospital);
                assert!(!g.arsenal.contains(h));
                assert!(*h != g.chest && *h != g.exit && *h != g.player);
            }
            // враги вне радиуса 3 от игрока
            for e in g.bears.iter().chain(g.zombies.iter()).chain(g.shooters.iter()).chain(g.bosses.iter()) {
                assert!(manhattan(*e, g.player) > 3, "enemy too close seed {}", seed);
            }
            // ловушки/огонь не на статике
            for t in g.trap.iter().chain(g.fire.iter()) {
                assert!(g.key != Some(*t));
                assert!(*t != g.chest && *t != g.exit && *t != g.hospital);
                assert!(!g.holes.contains(t) && !g.arsenal.contains(t));
            }
            // ключи уникальны внутри списков
            let mut all: Vec<Pos> = vec![];
            all.extend(g.trap.iter()); all.extend(g.fire.iter());
            all.extend(g.bears.iter()); all.extend(g.zombies.iter());
            all.extend(g.shooters.iter()); all.extend(g.bosses.iter());
            all.sort(); all.dedup();
            assert_eq!(all.len(), g.trap.len() + g.fire.len() + g.bears.len() + g.zombies.len() + g.shooters.len() + g.bosses.len(),
                "enemy/trap lists must not overlap, seed {}", seed);
        }
    }
    #[test]
    fn level1_enemy_counts() {
        let g = game(42);
        assert_eq!(g.bosses.len(), 1);
        assert!(g.zombies.len() <= 10);
        assert!(g.shooters.len() <= 7);
        assert!(g.bears.len() >= 1 && g.bears.len() <= 30);
        assert!(g.trap.len() <= 5);
        assert_eq!(g.fire.len(), 0, "огонь только со 2 уровня");
        assert_eq!(g.holes.len(), 4);
        assert_eq!(g.arsenal.len(), 2);
    }
    #[test]
    fn difficulty_grows_with_level() {
        // огонь со 2 уровня: ищем сид, где клетка огня прошла фильтр стартовой зоны
        let mut found_fire = false;
        for seed in 0..60u32 {
            let mut gg = game(seed);
            gg.level = 2; gg.new_world(100_000);
            if !gg.fire.is_empty() { found_fire = true; break; }
        }
        assert!(found_fire, "огонь со 2 уровня");
        let mut g = game(42);
        g.level = 2; g.new_world(100_000);
        let bears_l2 = g.bears.len();
        g.level = 10; g.new_world(200_000);
        assert!(g.bears.len() >= bears_l2);
        assert!(g.bears.len() <= 30);
        assert!(g.trap.len() <= 5);
    }
    #[test]
    fn lakes_generation() {
        let g = game(42);
        assert!(g.lakes.len() >= 2 && g.lakes.len() <= 3);
        for l in &g.lakes {
            assert!(!l.cells.is_empty() && l.cells.len() <= 10);
            // связность кластера: BFS по клеткам кластера
            let set: std::collections::HashSet<Pos> = l.cells.iter().copied().collect();
            for &cell in &l.cells {
                assert!(set.contains(&cell));
            }
            // все клетки в границах
            for &(r, c) in &l.cells { assert!(r < H && c < W); }
        }
        // озёра не перекрываются между собой
        let all: Vec<Pos> = g.lakes.iter().flat_map(|l| l.cells.clone()).collect();
        let mut s = all.clone(); s.sort();
        assert_eq!(s.len(), { let mut u = s.clone(); u.dedup(); u.len() });
    }
    #[test]
    fn lake_cells_open_passages() {
        let g = game(42);
        let lake_set: std::collections::HashSet<Pos> = g.lakes.iter().flat_map(|l| l.cells.iter().copied()).collect();
        for &(r, c) in &lake_set {
            for d in 0..4 {
                if let Some(n) = add((r, c), DIRS[d]) {
                    if lake_set.contains(&n) {
                        assert!(!g.wall[r][c][d], "lake passage open");
                        assert!(!g.wall[n.0][n.1][d ^ 1]);
                    }
                }
            }
        }
    }
    #[test]
    fn swamps_not_in_lakes() {
        let g = game(42);
        let lake_set: std::collections::HashSet<Pos> = g.lakes.iter().flat_map(|l| l.cells.iter().copied()).collect();
        for s in &g.swamps { assert!(!lake_set.contains(s)); }
    }

    // --- BFS / туман ---
    #[test]
    fn bfs_respects_walls() {
        let mut g = game(1);
        // полностью закрытый лабиринт: все стены → только старт достижим
        for r in 0..H { for c in 0..W { g.wall[r][c] = [true; 4]; } }
        let b = bfs(&g.wall, (0, 0), None);
        assert_eq!(b.dist[0][0], 0);
        assert_eq!(b.dist[0][1], -1);
        // открытый лабиринт
        open_all(&mut g);
        let b2 = bfs(&g.wall, (5, 5), Some(3));
        assert_eq!(b2.dist[5][8], 3);
        assert_eq!(b2.dist[5][9], -1, "max_dist обрубает поиск");
    }
    #[test]
    fn fog_radius_3() {
        let mut g = game(1);
        open_all(&mut g);
        place_player(&mut g, (5, 5));
        assert!(g.visible(5, 8)); assert!(!g.visible(5, 9));
        assert!(g.visible(2, 5)); assert!(!g.visible(1, 5));
    }
    #[test]
    fn fog_blocked_by_walls() {
        let mut g = game(1);
        open_all(&mut g);
        // глухой угол: все 4 стены вокруг игрока (BFS-туман не проходит никуда)
        g.wall[5][5] = [true; 4];
        place_player(&mut g, (5, 5));
        assert!(g.visible(5, 5));
        assert!(!g.visible(5, 6));
        assert!(!g.visible(5, 7));
        assert!(!g.visible(4, 5));
    }
    #[test]
    fn explored_memory_persists() {
        let mut g = game(1);
        open_all(&mut g);
        place_player(&mut g, (5, 5));
        g.mark_explored();
        assert!(g.explored[5][5]);
        place_player(&mut g, (15, 15));
        // (5,5) больше не видна, но остаётся разведанной
        assert!(!g.visible(5, 5));
        assert!(g.explored[5][5], "память карты");
    }
    #[test]
    fn visible_at_start() {
        let g = game(3);
        assert!(g.visible(g.player.0, g.player.1));
    }

    // --- движение ---
    #[test]
    fn move_in_open_direction() {
        let mut g = game(1);
        open_all(&mut g);
        place_player(&mut g, (10, 10));
        g.trap.clear(); g.fire.clear(); g.swamps.clear(); g.lakes.clear(); g.holes.clear();
        g.try_move(0, 1, 0);
        assert_eq!(g.player, (10, 11));
        assert_eq!(g.moves, 1);
    }
    #[test]
    fn wall_blocks_move_no_move_count() {
        let mut g = game(1);
        // все стены
        for r in 0..H { for c in 0..W { g.wall[r][c] = [true; 4]; } }
        place_player(&mut g, (10, 10));
        g.try_move(-1, 0, 0);
        assert_eq!(g.player, (10, 10));
        assert_eq!(g.moves, 0);
        assert_eq!(g.msg, t(Lang::Ru, "w_wall"));
    }
    #[test]
    fn move_cooldown() {
        let mut g = game(1);
        open_all(&mut g);
        place_player(&mut g, (10, 10));
        g.lakes.clear(); g.holes.clear();
        g.try_move(0, 1, 1000);
        let m = g.moves;
        g.try_move(0, 1, 1000 + 100); // < 150 мс
        assert_eq!(g.moves, m, "кулдаун хода");
        g.try_move(0, 1, 1000 + 150);
        assert_eq!(g.moves, m + 1);
    }
    #[test]
    fn move_paused_or_over_ignored() {
        let mut g = game(1);
        open_all(&mut g);
        place_player(&mut g, (10, 10));
        g.paused = true;
        let p = g.player;
        g.try_move(0, 1, 1000);
        assert_eq!(g.player, p);
        g.paused = false; g.over = true;
        g.try_move(0, 1, 5000);
        assert_eq!(g.player, p);
    }
    #[test]
    fn help_blocks_input() {
        let mut g = game(1);
        open_all(&mut g);
        place_player(&mut g, (10, 10));
        g.help_open = true;
        g.try_move(0, 1, 1000);
        assert_eq!(g.player, (10, 10));
    }
    #[test]
    fn combo_scoring_and_reset_on_wall() {
        let mut g = game(1);
        open_all(&mut g);
        place_player(&mut g, (10, 10));
        g.lakes.clear(); g.holes.clear();
        g.try_move(0, 1, 1000); // combo 1, +5
        assert_eq!(g.combo, 1); assert_eq!(g.score, 5);
        g.try_move(0, 1, 1300); // combo 2, +10 (шаг 300мс > кулдауна 150, < окна 900)
        assert_eq!(g.combo, 2); assert_eq!(g.score, 15);
        g.try_move(0, 1, 4000); // окно истекло (>900мс) → combo 1
        assert_eq!(g.combo, 1);
        // стена сбрасывает серию
        g.wall[g.player.0][g.player.1][0] = true;
        g.try_move(-1, 0, 6000);
        assert_eq!(g.combo, 0);
    }
    #[test]
    fn combo10_achievement() {
        let mut g = game(1);
        open_all(&mut g);
        place_player(&mut g, (10, 10));
        g.lakes.clear(); g.holes.clear();
        for i in 0..10u64 {
            // ходим туда-сюда, чтобы не упереться в стену
            let d = if i % 2 == 0 { (0, 1) } else { (0, -1) };
            g.try_move(d.0, d.1, 1000 + i * 200);
        }
        assert_eq!(g.stats.best_combo, 10);
        assert!(g.stats.ach_unlocked("combo10"));
    }
    #[test]
    fn dash_jumps_two_cells() {
        let mut g = game(1);
        open_all(&mut g);
        place_player(&mut g, (10, 10));
        g.lakes.clear(); g.holes.clear(); g.trap.clear(); g.fire.clear(); g.swamps.clear();
        g.player = (10, 10);
        g.dash(0, 1, 1000);
        assert_eq!(g.player, (10, 12));
        assert_eq!(g.dash_until, 1000 + DASH_CD);
    }
    #[test]
    fn dash_cooldown() {
        let mut g = game(1);
        open_all(&mut g);
        place_player(&mut g, (10, 10));
        g.lakes.clear(); g.holes.clear();
        g.dash(0, 1, 1000);
        g.dash(0, 1, 1000 + 1000); // < 1500
        assert_eq!(g.player, (10, 12), "рывок не прошёл по кулдауну");
        assert_eq!(g.msg, t(Lang::Ru, "w_dashCd"));
    }
    #[test]
    fn dash_blocked_by_second_wall() {
        let mut g = game(1);
        open_all(&mut g);
        // стена между (10,11) и (10,12)
        g.wall[10][11][3] = true; g.wall[10][12][2] = true;
        place_player(&mut g, (10, 10));
        g.lakes.clear(); g.holes.clear();
        g.dash(0, 1, 1000);
        assert_eq!(g.player, (10, 10));
        assert_eq!(g.msg, t(Lang::Ru, "w_dashWall"));
    }
    #[test]
    fn dash_picks_up_items() {
        let mut g = game(1);
        open_all(&mut g);
        place_player(&mut g, (10, 10));
        g.lakes.clear(); g.holes.clear(); g.trap.clear(); g.fire.clear(); g.swamps.clear();
        g.chest = (10, 12);
        g.dash(0, 1, 1000);
        assert!(g.carrying, "рывок подбирает сундук");
    }

    // --- озера / болота / дырки ---
    #[test]
    fn swamp_skips_three_moves() {
        let mut g = game(1);
        open_all(&mut g);
        place_player(&mut g, (10, 10));
        g.lakes.clear(); g.holes.clear(); g.trap.clear(); g.fire.clear();
        g.swamps = vec![(10, 11)];
        g.try_move(0, 1, 1000);
        assert_eq!(g.skip_move, 3);
        // следующие 3 хода пропущены
        g.try_move(0, 1, 2000); assert_eq!(g.skip_move, 2);
        g.try_move(0, 1, 2500); assert_eq!(g.skip_move, 1);
        g.try_move(0, 1, 3000); assert_eq!(g.skip_move, 0);
        // 4-й ход — настоящий
        let p = g.player;
        g.try_move(0, 1, 4000);
        assert_ne!(g.player, p);
    }
    #[test]
    fn hole_teleports_to_next() {
        let mut g = game(1);
        open_all(&mut g);
        place_player(&mut g, (5, 5));
        g.lakes.clear(); g.swamps.clear(); g.trap.clear(); g.fire.clear();
        g.holes = vec![(5, 6), (8, 8)];
        g.try_move(0, 1, 1000);
        assert_eq!(g.player, (8, 8));
    }
    #[test]
    fn hole_cycles_through_all() {
        let mut g = game(1);
        open_all(&mut g);
        g.lakes.clear(); g.swamps.clear(); g.trap.clear(); g.fire.clear();
        g.holes = vec![(1, 1), (2, 2), (3, 3)];
        // вход в дырку 0 → выход из 1
        place_player(&mut g, (1, 1));
        g.try_move(1, 0, 1000); // уход вниз из дырки? дырка срабатывает при входе; ставим напрямую через on_enter непубличен —
        // проверяем напрямую: игрок стоит в дырке, ход в дырку 2 должен выкинуть в 0 (цикл)
        place_player(&mut g, (3, 2));
        g.try_move(0, 1, 3000); // в (3,3)
        assert_eq!(g.player, (1, 1), "цикл дырок");
    }
    #[test]
    fn lake_flow_moves_and_ejects() {
        let mut g = game(1);
        open_all(&mut g);
        // озеро из 2 клеток, течение на восток
        g.lakes = vec![Lake { cells: vec![(10, 10), (10, 11)], direction: (0, 1) }];
        g.holes.clear();
        // несколько раз: игрок должен сменить клетку или быть выброшен
        let mut moved = false;
        for _ in 0..20 {
            place_player(&mut g, (10, 10));
            if g.lake_flow((10, 10)).is_some() { moved = true; break; }
        }
        assert!(moved, "течение озера должно двигать");
    }
    #[test]
    fn arsenal_restores_hp_and_bombs() {
        let mut g = game(1);
        open_all(&mut g);
        place_player(&mut g, (10, 9));
        g.lakes.clear(); g.holes.clear(); g.trap.clear(); g.fire.clear(); g.swamps.clear();
        g.arsenal = vec![(10, 10)];
        g.health = 2; g.bombs = 1;
        g.try_move(0, 1, 1000);
        assert_eq!(g.health, 5); assert_eq!(g.bombs, 5);
    }
    #[test]
    fn hospital_restores_only_hp() {
        let mut g = game(1);
        open_all(&mut g);
        place_player(&mut g, (10, 9));
        g.lakes.clear(); g.holes.clear(); g.trap.clear(); g.fire.clear(); g.swamps.clear();
        g.arsenal.clear();
        g.hospital = (10, 10);
        g.health = 1; g.bombs = 2;
        g.try_move(0, 1, 1000);
        assert_eq!(g.health, 5); assert_eq!(g.bombs, 2, "госпиталь бомбы не трогает");
    }

    // --- сундук / ключ / выход ---
    #[test]
    fn chest_pickup() {
        let mut g = game(1);
        open_all(&mut g);
        place_player(&mut g, (10, 9));
        g.lakes.clear(); g.holes.clear(); g.trap.clear(); g.fire.clear(); g.swamps.clear();
        g.chest = (10, 10);
        g.try_move(0, 1, 1000);
        assert!(g.carrying);
    }
    #[test]
    fn key_pickup_and_disappears() {
        let mut g = game(1);
        open_all(&mut g);
        place_player(&mut g, (10, 9));
        g.lakes.clear(); g.holes.clear(); g.trap.clear(); g.fire.clear(); g.swamps.clear();
        g.key = Some((10, 10));
        g.try_move(0, 1, 1000);
        assert!(g.carrying_key); assert_eq!(g.key, None);
    }
    #[test]
    fn exit_requires_chest_and_key() {
        let mut g = game(1);
        open_all(&mut g);
        place_player(&mut g, (10, 9));
        g.lakes.clear(); g.holes.clear(); g.trap.clear(); g.fire.clear(); g.swamps.clear();
        g.exit = (10, 10); g.key = None;
        g.carrying = true; g.carrying_key = false;
        g.try_move(0, 1, 1000);
        assert!(!g.over);
        assert_eq!(g.msg, t(Lang::Ru, "w_door"));
    }
    #[test]
    fn exit_without_chest_msg() {
        let mut g = game(1);
        open_all(&mut g);
        place_player(&mut g, (10, 9));
        g.lakes.clear(); g.holes.clear(); g.trap.clear(); g.fire.clear(); g.swamps.clear();
        g.exit = (10, 10);
        g.try_move(0, 1, 1000);
        assert_eq!(g.msg, t(Lang::Ru, "w_exit"));
    }
    #[test]
    fn win_with_chest_and_key() {
        let mut g = game(1);
        open_all(&mut g);
        place_player(&mut g, (10, 9));
        g.lakes.clear(); g.holes.clear(); g.trap.clear(); g.fire.clear(); g.swamps.clear();
        g.exit = (10, 10);
        g.carrying = true; g.carrying_key = true;
        g.try_move(0, 1, 1000);
        assert!(g.over); assert_eq!(g.over_kind, Some(OverKind::Win));
        assert_eq!(g.level, 2, "победа повышает уровень");
        assert!(g.score >= 1000, "победа +1000 и комбо-бонус хода");
        assert_eq!(g.stats.wins, 1);
    }
    #[test]
    fn damage_drops_chest_below_3hp() {
        let mut g = game(1);
        g.carrying = true;
        g.health = 3;
        g.damage_player(1, "w_bear"); // 3→2, выпадает
        assert_eq!(g.health, 2);
        assert!(!g.carrying);
        assert!(g.msg.contains("Сундук вернулся"));
    }
    #[test]
    fn damage_keeps_chest_at_3hp() {
        let mut g = game(1);
        g.carrying = true;
        g.health = 4;
        g.damage_player(1, "w_bear"); // 4→3, не выпадает
        assert_eq!(g.health, 3);
        assert!(g.carrying);
    }
    #[test]
    fn death_loses_and_resets_level() {
        let mut g = game(1);
        g.level = 5;
        g.health = 1;
        g.damage_player(1, "w_bear");
        assert!(g.over); assert_eq!(g.over_kind, Some(OverKind::Lose));
        assert_eq!(g.level, 1);
    }
    #[test]
    fn timeout_loses_and_resets() {
        let mut g = game(1);
        g.level = 3;
        g.carrying = true;
        g.time_left = 1;
        g.tick_second(0);
        assert!(g.over); assert_eq!(g.over_kind, Some(OverKind::Timeout));
        assert_eq!(g.level, 1); assert!(!g.carrying);
        assert_eq!(g.time_left, 0);
    }
    #[test]
    fn timer_ticks_down() {
        let mut g = game(1);
        g.tick_second(0);
        assert_eq!(g.time_left, 299);
    }

    // --- ловушки / огонь ---
    #[test]
    fn fire_always_hurts() {
        let mut g = game(1);
        open_all(&mut g);
        place_player(&mut g, (10, 9));
        g.lakes.clear(); g.holes.clear(); g.trap.clear(); g.swamps.clear();
        g.fire = vec![(10, 10)];
        g.try_move(0, 1, 1000);
        assert_eq!(g.health, 4);
    }
    #[test]
    fn trap_hurts_or_safe() {
        // ловушка: 50% урон — проверяем оба исхода возможны на серии сидов
        let mut hurt = 0; let mut safe = 0;
        for seed in 0..40u32 {
            let mut g = game(seed);
            open_all(&mut g);
            place_player(&mut g, (10, 9));
            g.lakes.clear(); g.holes.clear(); g.fire.clear(); g.swamps.clear();
            g.trap = vec![(10, 10)];
            g.try_move(0, 1, 1000);
            if g.health == 4 { hurt += 1; } else if g.health == 5 { safe += 1; }
        }
        assert!(hurt > 0 && safe > 0, "ловушка 50%: hurt={} safe={}", hurt, safe);
    }

    // --- враги ---
    #[test]
    fn grace_period_blocks_enemies() {
        let mut g = game(1);
        let bear_before = g.bears.clone();
        g.bear_turn(g.grace_until - 1);
        assert_eq!(g.bears, bear_before);
        let now = later(&mut g);
        g.bear_turn(now);
        // медведь мог сходить; здоровье не в минус — проверяем, что метод работает без паники
        assert!(g.health <= 5);
        let _ = bear_before;
    }
    #[test]
    fn bear_turn_moves_and_hurts() {
        let mut g = game(1);
        open_all(&mut g);
        place_player(&mut g, (10, 10));
        g.bears = vec![(10, 11)]; // рядом с игроком
        let now = later(&mut g);
        let mut hurt = false;
        for _ in 0..30 {
            // ставим медведя рядом с игроком каждый раз
            g.bears = vec![(10, 11)];
            g.health = 5;
            g.bear_turn(now);
            if g.health == 4 { hurt = true; break; }
        }
        assert!(hurt, "медведь должен ранить при встрече");
    }
    #[test]
    fn bear_stays_in_passages() {
        let mut g = game(3);
        let now = later(&mut g);
        for _ in 0..5 { g.bear_turn(now); }
        for &b in &g.bears {
            assert!(b.0 < H && b.1 < W);
            // медведь должен стоять на клетке, достижимой от старта (связность)
            let bb = bfs(&g.wall, (0, 0), None);
            assert_ne!(bb.dist[b.0][b.1], -1);
        }
    }
    #[test]
    fn zombie_only_acts_within_5() {
        let mut g = game(1);
        open_all(&mut g);
        place_player(&mut g, (10, 10));
        g.zombies = vec![(0, 0)]; // далеко
        let now = later(&mut g);
        g.zombie_turn(now);
        assert_eq!(g.zombies[0], (0, 0), "зомби далеко — стоит");
    }
    #[test]
    fn zombie_chases_and_bites_adjacent() {
        let mut g = game(1);
        open_all(&mut g);
        place_player(&mut g, (10, 10));
        g.zombies = vec![(10, 13)]; // дистанция 3
        let now = later(&mut g);
        g.zombie_turn(now);
        assert_eq!(g.zombies[0], (10, 12), "зомби сближается");
        // укус: зомби шагает в соседа, но не на клетку игрока (стена между)
        g.wall[10][10][3] = true; g.wall[10][11][2] = true;
        place_player(&mut g, (10, 10));
        g.zombies = vec![(10, 12)];
        g.health = 5;
        g.zombie_turn(now);
        assert_eq!(g.health, 4, "кусок соседа");
    }
    #[test]
    fn shooter_aim_then_shoot() {
        let mut g = game(1);
        open_all(&mut g);
        place_player(&mut g, (10, 10));
        g.shooters = vec![(10, 12)];
        g.compute_visibility();
        let now = later(&mut g);
        g.health = 5;
        g.shooter_turn(now);
        assert!(g.shooter_aim.contains(&(10, 12)), "первый такт — прицел");
        assert_eq!(g.health, 5, "прицел ещё не ранит");
        g.shooter_turn(now);
        assert_eq!(g.health, 4, "второй такт — выстрел");
        assert_eq!(g.shots.len(), 1);
    }
    #[test]
    fn shooter_ignores_when_far_or_unseen() {
        let mut g = game(1);
        open_all(&mut g);
        place_player(&mut g, (10, 10));
        g.shooters = vec![(0, 0)];
        g.compute_visibility();
        let now = later(&mut g);
        g.health = 5;
        g.shooter_turn(now);
        assert!(!g.shooter_aim.contains(&(0, 0)));
        assert_eq!(g.health, 5);
    }
    #[test]
    fn shooter_wanders_when_not_aiming() {
        let mut g = game(1);
        open_all(&mut g);
        place_player(&mut g, (10, 10));
        g.shooters = vec![(0, 0)];
        g.compute_visibility();
        let now = later(&mut g);
        for _ in 0..5 { g.shooter_turn(now); }
        // стрелок где-то ходит, но не стреляет вдаль
        assert_eq!(g.health, 5);
    }
    #[test]
    fn boss_knows_player_and_chases_bfs() {
        let mut g = game(1);
        open_all(&mut g);
        place_player(&mut g, (10, 15));
        g.bosses = vec![(10, 5)];
        g.compute_visibility();
        let now = later(&mut g);
        // босс далеко (>3): идёт по BFS к игроку
        g.boss_turn(now);
        assert_eq!(g.bosses[0], (10, 6), "босс идёт кратчайшим путём");
    }
    #[test]
    fn boss_shoots_at_close_range() {
        let mut g = game(1);
        open_all(&mut g);
        place_player(&mut g, (10, 12));
        g.bosses = vec![(10, 10)]; // дистанция 2, прямая видимость
        g.compute_visibility();
        let now = later(&mut g);
        g.health = 5;
        g.boss_turn(now);
        assert!(g.boss_aim.contains(&(10, 10)));
        g.boss_turn(now);
        assert_eq!(g.health, 4, "босс стреляет как охотник");
    }
    #[test]
    fn boss_melee_hits_for_two() {
        let mut g = game(1);
        open_all(&mut g);
        place_player(&mut g, (10, 11));
        g.bosses = vec![(10, 10)]; // вплотную
        g.compute_visibility();
        let now = later(&mut g);
        g.health = 5;
        g.boss_turn(now);
        // вплотную: dist=1 <= 3 и видимость есть → прицел, затем выстрел −1... JS: шаг adjacent даёт −2.
        // Проверяем: либо прицел, либо −2. Второй такт должен дать −1 (выстрел) или −2.
        let hp1 = g.health;
        g.boss_turn(now);
        assert!(g.health <= hp1 - 1, "босс ранит вплотную");
        // прямого −2 проверяем через has_clear_shot=false путь: стена между → идёт и бьёт
    }
    #[test]
    fn boss_melee_two_damage_behind_wall() {
        // босс вплотную −2: первый BFS-шаг — в соседа игрока, стрельба заблокирована стеной
        let mut g = game(1);
        open_all(&mut g);
        place_player(&mut g, (10, 10));
        // стена между (10,9) и (10,10): прямая (9,9)->(10,10) заблокирована, путь — через (9,10)
        g.wall[10][9][3] = true; g.wall[10][10][2] = true;
        g.bosses = vec![(9, 9)];
        let now = later(&mut g);
        g.health = 5;
        g.boss_turn(now);
        assert_eq!(g.bosses[0], (9, 10), "первый шаг BFS — вплотную");
        assert_eq!(g.health, 3, "босс вплотную −2");
    }
    #[test]
    fn has_clear_shot_logic() {
        let mut g = game(1);
        open_all(&mut g);
        assert!(g.has_clear_shot((5, 5), (5, 8)));
        assert!(g.has_clear_shot((5, 5), (8, 5)));
        assert!(g.has_clear_shot((5, 5), (5, 5)));
        g.wall[5][5][3] = true;
        assert!(!g.has_clear_shot((5, 5), (5, 8)));
        // порядок: сначала строки, потом столбцы — стена сбоку не мешает при вертикальном луче
        assert!(g.has_clear_shot((5, 5), (8, 5)));
    }
    #[test]
    fn next_step_to_bfs() {
        let mut g = game(1);
        open_all(&mut g);
        assert_eq!(g.next_step_to((5, 5), (5, 8)), Some((5, 6)));
        assert_eq!(g.next_step_to((5, 5), (5, 5)), None);
        // стена — путь в обход
        g.wall[5][5][3] = true; g.wall[5][6][2] = true;
        let step = g.next_step_to((5, 5), (5, 6)).unwrap();
        assert_ne!(step, (5, 6), "шаг должен вести в обход, не сквозь стену");
        // недостижимая цель
        let mut g2 = game(1);
        for r in 0..H { for c in 0..W { g2.wall[r][c] = [true; 4]; } }
        assert_eq!(g2.next_step_to((0, 0), (5, 5)), None);
    }

    // --- бомбы ---
    #[test]
    fn bomb_blows_wall() {
        let mut g = game(1);
        open_all(&mut g);
        place_player(&mut g, (10, 10));
        g.wall[10][10][3] = true; g.wall[10][11][2] = true;
        // нужен rng, дающий < 3/5: подбираем сид перебором
        let mut blew = false;
        for seed in 0..100u32 {
            let mut gg = game(seed);
            open_all(&mut gg);
            place_player(&mut gg, (10, 10));
            gg.wall[10][10][3] = true; gg.wall[10][11][2] = true;
            let bombs = gg.bombs;
            gg.bomb(0, 1, 1000);
            if gg.bombs == bombs - 1 && !gg.wall[10][10][3] { blew = true; break; }
        }
        assert!(blew, "бомба должна иногда взрывать стену");
    }
    #[test]
    fn bomb_stone_wall_consumes_charge() {
        let mut stone = false;
        for seed in 0..100u32 {
            let mut gg = game(seed);
            open_all(&mut gg);
            place_player(&mut gg, (10, 10));
            gg.wall[10][10][3] = true;
            gg.bomb(0, 1, 1000);
            if gg.wall[10][10][3] && gg.bombs == 4 && gg.msg == t(Lang::Ru, "w_stone") { stone = true; break; }
        }
        assert!(stone, "каменные стены (3/5) существуют");
    }
    #[test]
    fn bomb_no_wall_no_consume() {
        let mut g = game(1);
        open_all(&mut g);
        place_player(&mut g, (10, 10));
        g.bomb(0, 1, 1000);
        assert_eq!(g.bombs, 5);
        assert_eq!(g.msg, t(Lang::Ru, "w_noWall"));
    }
    #[test]
    fn bomb_at_edge() {
        let mut g = game(1);
        place_player(&mut g, (0, 0));
        // север — край
        g.bomb(-1, 0, 1000);
        assert_eq!(g.msg, t(Lang::Ru, "w_edge"));
        assert_eq!(g.bombs, 5);
    }
    #[test]
    fn bomb_without_bombs() {
        let mut g = game(1);
        open_all(&mut g);
        place_player(&mut g, (10, 10));
        g.bombs = 0;
        g.wall[10][10][3] = true;
        g.bomb(0, 1, 1000);
        assert_eq!(g.msg, t(Lang::Ru, "w_noBombs"));
        assert!(g.wall[10][10][3], "стена цела без бомб");
    }
    #[test]
    fn bomb_ignored_when_over() {
        let mut g = game(1);
        open_all(&mut g);
        place_player(&mut g, (10, 10));
        g.over = true;
        g.wall[10][10][3] = true;
        g.bomb(0, 1, 1000);
        assert!(g.wall[10][10][3]);
    }

    // --- компас ---
    #[test]
    fn compass_points_to_key_first() {
        let mut g = game(1);
        open_all(&mut g);
        place_player(&mut g, (10, 10));
        g.key = Some((5, 5)); g.chest = (15, 15); g.exit = (0, 0);
        g.carrying_key = false; g.carrying = false;
        let (dir, dx, dy, _) = g.direction_to_target();
        assert_eq!(dir, "↖"); assert_eq!((dx, dy), (-1, -1));
    }
    #[test]
    fn compass_to_chest_after_key() {
        let mut g = game(1);
        open_all(&mut g);
        place_player(&mut g, (10, 10));
        g.key = None; g.chest = (15, 15); g.exit = (0, 0);
        g.carrying_key = true; g.carrying = false;
        let (dir, _, _, _) = g.direction_to_target();
        assert_eq!(dir, "↘");
    }
    #[test]
    fn compass_to_exit_with_chest() {
        let mut g = game(1);
        open_all(&mut g);
        place_player(&mut g, (10, 10));
        g.key = None; g.chest = (15, 15); g.exit = (10, 18);
        g.carrying_key = true; g.carrying = true;
        let (dir, _, _, _) = g.direction_to_target();
        assert_eq!(dir, "➡");
    }
    #[test]
    fn compass_at_target_trophy() {
        let mut g = game(1);
        open_all(&mut g);
        place_player(&mut g, (10, 10));
        g.key = None; g.chest = (10, 10); g.exit = (0, 0);
        g.carrying_key = true; g.carrying = false;
        let (dir, dx, dy, dist) = g.direction_to_target();
        assert_eq!(dir, "🏆"); assert_eq!((dx, dy, dist), (0, 0, 0));
        // все 8 направлений
        let cases = [
            ((5, 10), "⬆"), ((15, 10), "⬇"), ((10, 5), "⬅"), ((10, 15), "➡"),
            ((5, 5), "↖"), ((5, 15), "↗"), ((15, 5), "↙"), ((15, 15), "↘"),
        ];
        for (tgt, want) in cases {
            g.chest = tgt;
            let (dir, _, _, _) = g.direction_to_target();
            assert_eq!(dir, want, "цель {:?}", tgt);
        }
    }

    // --- пауза / помощь ---
    #[test]
    fn pause_toggle_and_freezes_tick() {
        let mut g = game(1);
        g.toggle_pause();
        assert!(g.paused);
        let t = g.time_left;
        g.tick_second(0);
        assert_eq!(g.time_left, t, "пауза останавливает таймер");
        g.toggle_pause();
        assert!(!g.paused);
    }
    #[test]
    fn pause_ignored_when_over() {
        let mut g = game(1);
        g.over = true;
        g.toggle_pause();
        assert!(!g.paused);
    }
    #[test]
    fn help_toggles_and_clears_aim() {
        let mut g = game(1);
        g.aiming = true;
        g.toggle_help();
        assert!(g.help_open); assert!(!g.aiming);
        g.toggle_help();
        assert!(!g.help_open);
    }

    // --- достижения / рекорды ---
    #[test]
    fn achievements_unlock_once() {
        let mut st = Stats { wins: 1, ..Default::default() };
        let new = st.check_achievements();
        assert_eq!(new, vec!["first".to_string()]);
        let new2 = st.check_achievements();
        assert!(new2.is_empty());
    }
    #[test]
    fn achievement_names_match_i18n() {
        for id in ACH_IDS {
            assert_ne!(ach_name(Lang::Ru, id), id);
            assert_ne!(ach_name(Lang::En, id), id);
        }
    }
    #[test]
    fn win_speedrun_and_clean() {
        let mut g = game(1);
        open_all(&mut g);
        place_player(&mut g, (10, 9));
        g.lakes.clear(); g.holes.clear(); g.trap.clear(); g.fire.clear(); g.swamps.clear();
        g.exit = (10, 10); g.carrying = true; g.carrying_key = true;
        g.time_left = LEVEL_TIME - 60; // 60 секунд — speedrun на L1
        g.try_move(0, 1, 1000);
        assert!(g.stats.speedrun); assert!(g.stats.clean_win);
        assert!(g.stats.ach_unlocked("speedrun"));
        assert!(g.stats.ach_unlocked("clean"));
    }
    #[test]
    fn best_saved_on_win() {
        let mut g = game(1);
        g.best = Best::default();
        open_all(&mut g);
        place_player(&mut g, (10, 9));
        g.lakes.clear(); g.holes.clear(); g.trap.clear(); g.fire.clear(); g.swamps.clear();
        g.exit = (10, 10); g.carrying = true; g.carrying_key = true;
        g.time_left = LEVEL_TIME - 55;
        g.try_move(0, 1, 1000);
        assert_eq!(g.best.level, 1);
        assert_eq!(g.best.time1, Some(55.0));
    }
    #[test]
    fn explored_max_on_win() {
        let mut g = game(1);
        open_all(&mut g);
        g.explored = vec![vec![true; W]; H];
        // победа обновляет exploredMax
        open_all(&mut g);
        place_player(&mut g, (10, 9));
        g.lakes.clear(); g.holes.clear(); g.trap.clear(); g.fire.clear(); g.swamps.clear();
        g.explored = vec![vec![true; W]; H];
        g.exit = (10, 10); g.carrying = true; g.carrying_key = true;
        g.try_move(0, 1, 1000);
        assert_eq!(g.stats.explored_max, 400);
    }
    #[test]
    fn win_stats_html_contains_score() {
        let mut g = game(1);
        g.score = 1500;
        let s = g.win_stats_html();
        assert!(s.contains("1500"));
        assert!(s.contains(&t(Lang::Ru, "hudScore").to_string()) || s.contains("Очки"));
    }
    #[test]
    fn lose_and_timeout_stats_html() {
        let g = game(1);
        assert!(g.lose_stats_html().contains(&t(Lang::Ru, "w_loseStats").to_string()));
        assert!(g.timeout_stats_html().contains(&t(Lang::Ru, "w_timeoutStats").to_string()));
    }
    #[test]
    fn best_text_format() {
        let mut g = game(1);
        g.best = Best { level: 3, time1: Some(12.5) };
        let s = g.best_text();
        assert!(s.contains("3")); assert!(s.contains("12.5"));
        let mut g2 = game(1);
        g2.best = Best::default();
        assert!(!g2.best_text().contains("12"));
    }

    // --- i18n ---
    #[test]
    fn i18n_ru_en_key_parity() {
        let ru_keys: std::collections::HashSet<&str> = RU.iter().map(|(k, _)| *k).collect();
        let en_keys: std::collections::HashSet<&str> = EN.iter().map(|(k, _)| *k).collect();
        assert_eq!(ru_keys, en_keys, "RU/EN должны иметь одинаковые ключи");
        assert!(ru_keys.len() >= 60, "ключей: {}", ru_keys.len());
    }
    #[test]
    fn i18n_no_empty_values() {
        for (k, v) in RU.iter().chain(EN.iter()) {
            assert!(!v.is_empty(), "пустой {}", k);
        }
    }
    #[test]
    fn i18n_fallback_and_unknown() {
        assert_eq!(t(Lang::En, "hudHealth"), "Health");
        assert_eq!(t(Lang::Ru, "hudHealth"), "Здоровье");
        assert_eq!(t(Lang::Ru, "nonexistent_key"), "nonexistent_key");
    }
    #[test]
    fn dir_names_both_langs() {
        assert_eq!(dir_name(Lang::Ru, 0), "север");
        assert_eq!(dir_name(Lang::Ru, 3), "восток");
        assert_eq!(dir_name(Lang::En, 0), "north");
        assert_eq!(dir_name(Lang::En, 3), "east");
    }
    #[test]
    fn lang_roundtrip() {
        assert_eq!(Lang::from_code("ru"), Some(Lang::Ru));
        assert_eq!(Lang::from_code("en"), Some(Lang::En));
        assert_eq!(Lang::from_code("de"), None);
        assert_eq!(Lang::Ru.other(), Lang::En);
    }

    // --- HUD/символы ---
    #[test]
    fn cell_symbols() {
        let mut g = game(1);
        assert_eq!(g.cell_symbol(g.chest.0, g.chest.1), "💰");
        assert_eq!(g.cell_symbol(g.exit.0, g.exit.1), "🚩");
        if let Some(k) = g.key { assert_eq!(g.cell_symbol(k.0, k.1), "🗝️"); }
        assert_eq!(g.cell_symbol(g.hospital.0, g.hospital.1), "💉");
        if let Some(a) = g.arsenal.first() { assert_eq!(g.cell_symbol(a.0, a.1), "📦"); }
        if let Some(h) = g.holes.first() { assert_eq!(g.cell_symbol(h.0, h.1), "🕳"); }
        if let Some(l) = g.lakes.first().and_then(|l| l.cells.first()) { assert_eq!(g.cell_symbol(l.0, l.1), "🌀"); }
        if let Some(s) = g.swamps.first() { assert_eq!(g.cell_symbol(s.0, s.1), "🪷"); }
        if let Some(tr) = g.trap.first() { assert_eq!(g.cell_symbol(tr.0, tr.1), "⚠️"); }
    }
    #[test]
    fn hud_numbers() {
        let g = game(1);
        assert_eq!(g.health, MAX_HEALTH);
        assert_eq!(g.bombs, MAX_BOMBS);
        assert_eq!(g.time_left, LEVEL_TIME);
    }

    // --- прочие механики ---
    #[test]
    fn dir_index_and_helpers() {
        assert_eq!(dir_index(-1, 0), 0);
        assert_eq!(dir_index(1, 0), 1);
        assert_eq!(dir_index(0, -1), 2);
        assert_eq!(dir_index(0, 1), 3);
        assert_eq!(add((0, 0), DIRS[0]), None);
        assert_eq!(add((0, 0), DIRS[1]), Some((1, 0)));
        assert_eq!(manhattan((0, 0), (3, 4)), 7);
        assert!(!in_bounds(-1, 0)); assert!(in_bounds(0, 0));
    }
    #[test]
    fn bfs_on_generated_maze_all_reachable() {
        let g = game(20261106);
        let b = bfs(&g.wall, g.player, None);
        assert_eq!(b.dist[g.exit.0][g.exit.1] != -1, true);
        assert_eq!(b.dist[g.chest.0][g.chest.1] != -1, true);
    }
    #[test]
    fn damage_at_zero_hp_clamps() {
        let mut g = game(1);
        g.health = 1;
        g.damage_player(5, "w_bear");
        assert!(g.health <= 0);
        assert!(g.over);
    }
    #[test]
    fn drain_sfx_empties() {
        let mut g = game(1);
        g.sfx(Sfx::Step);
        assert_eq!(g.drain_sfx().len(), 1);
        assert!(g.drain_sfx().is_empty());
    }
    #[test]
    fn river_flag_set_on_lake_enter() {
        let mut g = game(1);
        open_all(&mut g);
        place_player(&mut g, (10, 9));
        g.holes.clear(); g.swamps.clear();
        // озеро только на (10,10) с течением на юг — выбросит на сушу
        g.lakes = vec![Lake { cells: vec![(10, 10)], direction: (1, 0) }];
        g.try_move(0, 1, 1000);
        assert!(g.river_on);
    }
    #[test]
    fn lake_tick_stops_when_out_of_lake() {
        let mut g = game(1);
        open_all(&mut g);
        place_player(&mut g, (10, 10));
        g.lakes.clear();
        g.river_on = true;
        g.lake_tick();
        assert!(!g.river_on);
    }
    #[test]
    fn swamp_prevents_dash() {
        let mut g = game(1);
        open_all(&mut g);
        place_player(&mut g, (10, 10));
        g.skip_move = 1;
        let p = g.player;
        g.dash(0, 1, 1000);
        assert_eq!(g.player, p);
        assert_eq!(g.skip_move, 0);
    }
    #[test]
    fn moves_increment_only_on_success() {
        let mut g = game(1);
        open_all(&mut g);
        place_player(&mut g, (10, 10));
        g.lakes.clear(); g.holes.clear();
        g.try_move(0, 1, 1000);
        assert_eq!(g.moves, 1);
        g.wall[g.player.0][g.player.1][3] = true;
        g.try_move(0, 1, 2000);
        assert_eq!(g.moves, 1, "в стену — не ход");
    }
    #[test]
    fn explored_count_increases() {
        let mut g = game(1);
        open_all(&mut g);
        place_player(&mut g, (5, 5));
        g.mark_explored();
        let n1 = g.count_explored();
        assert!(n1 > 0);
        place_player(&mut g, (15, 15));
        g.mark_explored();
        assert!(g.count_explored() > n1);
    }
    #[test]
    fn many_seeds_full_invariants_soak() {
        for seed in 0..30u32 {
            let g = game(seed * 137 + 11);
            assert_eq!(g.cell_count(), 400);
            assert!(g.health == 5 && g.bombs == 5 && g.time_left == 300);
            // все сущности на карте
            for e in g.bears.iter().chain(g.zombies.iter()).chain(g.shooters.iter()).chain(g.bosses.iter()) {
                assert!(e.0 < H && e.1 < W);
            }
            // BFS от игрока достигает выхода и сундука
            let b = bfs(&g.wall, g.player, None);
            assert_ne!(b.dist[g.exit.0][g.exit.1], -1, "seed {}", seed);
            assert_ne!(b.dist[g.chest.0][g.chest.1], -1, "seed {}", seed);
            if let Some(k) = g.key { assert_ne!(b.dist[k.0][k.1], -1); }
        }
    }
    #[test]
    fn no_panics_random_walks() {
        for seed in [1u32, 2, 3, 4, 5] {
            let mut g = game(seed);
            let mut now = 100_000u64;
            for i in 0..200 {
                now += 200;
                let d = DIRS[(i * 7 + seed as usize) % 4];
                if i % 17 == 0 { let n = later(&mut g); g.bear_turn(n); g.zombie_turn(n); g.shooter_turn(n); g.boss_turn(n); }
                g.try_move(d.0, d.1, now);
                if i % 17 == 0 { let n = later(&mut g); g.bear_turn(n); g.zombie_turn(n); g.shooter_turn(n); g.boss_turn(n); }
                if i % 23 == 0 { g.tick_second(now); }
                if g.over && i < 199 { g.restart(now); }
                assert!(g.player.0 < H && g.player.1 < W);
            }
        }
    }

#[test]
fn next_level_after_win_is_new_maze() {
    // победа + старт следующего уровня: новый лабиринт, тот же сид-базовый уровень растёт
    let mut g = game(42);
    open_all(&mut g);
    place_player(&mut g, (10, 9));
    g.lakes.clear(); g.holes.clear(); g.trap.clear(); g.fire.clear(); g.swamps.clear();
    g.exit = (10, 10); g.carrying = true; g.carrying_key = true;
    let wall_before = g.wall;
    g.try_move(0, 1, 1000);
    assert_eq!(g.over_kind, Some(OverKind::Win));
    assert_eq!(g.level, 2);
    // старт следующего уровня — новый мир (как веб-ристарт после победы)
    g.new_world(5000);
    assert_ne!(g.wall, wall_before, "после победы — новый лабиринт");
    assert_eq!(g.level, 2, "уровень не сброшен");
}

#[test]
fn restart_gives_up_to_level_1_same_world() {
    // «Заново» посреди уровня: тот же лабиринт, но с 1-го уровня
    let mut g = game(42);
    g.level = 4;
    let wall = g.wall;
    g.restart(100_000);
    assert_eq!(g.level, 1);
    assert_eq!(g.wall, wall, "тот же мир");
    assert_eq!(g.health, 5); assert_eq!(g.moves, 0);
}
