use super::*;
use std::collections::HashSet;

impl Game {
    pub(crate) fn gen_walls(&mut self) -> [[[bool; 4]; W]; H] {
        let mut w = [[[true; 4]; W]; H];
        for r in 0..H {
            for c in 0..W {
                w[r][c][1] = r == H - 1 || self.rng.next_f64() < 0.30;
                w[r][c][3] = c == W - 1 || self.rng.next_f64() < 0.30;
                if r > 0 {
                    w[r][c][0] = w[r - 1][c][1];
                }
                if c > 0 {
                    w[r][c][2] = w[r][c - 1][3];
                }
            }
        }
        w
    }
    pub fn cell_count(&self) -> usize {
        let b = bfs(&self.wall, (0, 0), None);
        b.dist
            .iter()
            .flat_map(|r| r.iter())
            .filter(|&&d| d != -1)
            .count()
    }
    pub fn reachable(&self) -> Vec<Vec<bool>> {
        let b = bfs(&self.wall, (0, 0), None);
        b.dist
            .iter()
            .map(|row| row.iter().map(|&d| d != -1).collect())
            .collect()
    }
    pub(crate) fn ensure_connected(&mut self) {
        for _ in 0..600 {
            if self.cell_count() == H * W {
                return;
            }
            let mark = self.reachable();
            'outer: for r in 0..H {
                for c in 0..W {
                    if mark[r][c] {
                        continue;
                    }
                    for &d in &DIRS {
                        let nr = r as i32 + d.0;
                        let nc = c as i32 + d.1;
                        if !in_bounds(nr, nc) || !mark[nr as usize][nc as usize] {
                            continue;
                        }
                        let di = dir_index(d.0, d.1);
                        self.wall[r][c][di] = false;
                        self.wall[nr as usize][nc as usize][di ^ 1] = false;
                        break 'outer;
                    }
                }
            }
        }
    }
    pub(crate) fn gen_lakes(&mut self) -> Vec<Lake> {
        let mut used = vec![vec![false; W]; H];
        let mut res: Vec<Lake> = vec![];
        let n_lakes = 2 + self.rng.next_usize(2);
        let mut attempts = 0;
        while res.len() < n_lakes && attempts < 200 {
            attempts += 1;
            let size = 1 + self.rng.next_usize(10);
            let sr = self.rng.next_usize(H);
            let sc = self.rng.next_usize(W);
            if used[sr][sc] {
                continue;
            }
            let mut cluster = vec![(sr, sc)];
            used[sr][sc] = true;
            let mut grown = 1;
            while grown < size {
                let pool = cluster[self.rng.next_usize(cluster.len())];
                let dd = DIRS[self.rng.next_usize(4)];
                let nr = pool.0 as i32 + dd.0;
                let nc = pool.1 as i32 + dd.1;
                if !in_bounds(nr, nc) || used[nr as usize][nc as usize] {
                    continue;
                }
                used[nr as usize][nc as usize] = true;
                cluster.push((nr as usize, nc as usize));
                grown += 1;
            }
            let direction = DIRS[self.rng.next_usize(4)];
            res.push(Lake {
                cells: cluster,
                direction,
            });
        }
        if res.is_empty() {
            res.push(Lake {
                cells: vec![(1, 1)],
                direction: (0, 1),
            });
        }
        res
    }
    pub(crate) fn ensure_lake_open(&mut self) {
        let lake_set: HashSet<Pos> = self
            .lakes
            .iter()
            .flat_map(|l| l.cells.iter().copied())
            .collect();
        let cells: Vec<Pos> = self
            .lakes
            .iter()
            .flat_map(|l| l.cells.iter().copied())
            .collect();
        for (r, c) in cells {
            for d in 0..4 {
                if let Some(n) = add((r, c), DIRS[d]) {
                    if lake_set.contains(&n) {
                        self.wall[r][c][d] = false;
                        self.wall[n.0][n.1][d ^ 1] = false;
                    }
                }
            }
        }
    }
    pub(crate) fn gen_swamps(&mut self) -> Vec<Pos> {
        let lake_set: HashSet<Pos> = self
            .lakes
            .iter()
            .flat_map(|l| l.cells.iter().copied())
            .collect();
        let mut used = vec![vec![false; W]; H];
        for &(rr, cc) in &lake_set {
            used[rr][cc] = true;
        }
        let mut res: Vec<Pos> = vec![];
        let n_swamp = self.rng.next_usize(2) + 1;
        let mut attempts = 0;
        while res.len() < n_swamp && attempts < 200 {
            attempts += 1;
            let size = self.rng.next_usize(4) + 2;
            let sr = self.rng.next_usize(H);
            let sc = self.rng.next_usize(W);
            if used[sr][sc] {
                continue;
            }
            let mut cluster = vec![(sr, sc)];
            used[sr][sc] = true;
            let mut grown = 1;
            while grown < size {
                let pool = cluster[self.rng.next_usize(cluster.len())];
                let dd = DIRS[self.rng.next_usize(4)];
                let nr = pool.0 as i32 + dd.0;
                let nc = pool.1 as i32 + dd.1;
                if !in_bounds(nr, nc) || used[nr as usize][nc as usize] {
                    continue;
                }
                used[nr as usize][nc as usize] = true;
                cluster.push((nr as usize, nc as usize));
                grown += 1;
            }
            res.extend(cluster);
        }
        res
    }
}
