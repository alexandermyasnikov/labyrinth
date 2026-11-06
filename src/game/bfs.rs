use super::*;

pub struct Bfs {
    pub dist: Vec<Vec<i32>>,
    pub prev: Vec<Vec<Option<Pos>>>,
}
pub fn bfs(wall: &[[[bool; 4]; W]; H], start: Pos, max_dist: Option<i32>) -> Bfs {
    let mut dist = vec![vec![-1i32; W]; H];
    let mut prev = vec![vec![None; W]; H];
    let mut q = vec![start];
    dist[start.0][start.1] = 0;
    let mut qi = 0;
    while qi < q.len() {
        let (r, c) = q[qi];
        qi += 1;
        if let Some(m) = max_dist {
            if dist[r][c] >= m {
                continue;
            }
        }
        for d in 0..4 {
            if wall[r][c][d] {
                continue;
            }
            let nr = r as i32 + DIRS[d].0;
            let nc = c as i32 + DIRS[d].1;
            if !in_bounds(nr, nc) {
                continue;
            }
            let (nr, nc) = (nr as usize, nc as usize);
            if dist[nr][nc] != -1 {
                continue;
            }
            dist[nr][nc] = dist[r][c] + 1;
            prev[nr][nc] = Some((r, c));
            q.push((nr, nc));
        }
    }
    Bfs { dist, prev }
}
