use super::*;

impl Game {
    pub fn compute_visibility(&mut self) {
        let b = bfs(&self.wall, self.player, Some(FOG_RADIUS));
        self.vis = b
            .dist
            .iter()
            .map(|row| row.iter().map(|&d| d != -1).collect())
            .collect();
    }
    pub fn visible(&self, r: usize, c: usize) -> bool {
        self.vis[r][c]
    }
    pub fn mark_explored(&mut self) {
        for r in 0..H {
            for c in 0..W {
                if self.vis[r][c] {
                    self.explored[r][c] = true;
                }
            }
        }
    }
    pub fn count_explored(&self) -> u32 {
        self.explored
            .iter()
            .flat_map(|r| r.iter())
            .filter(|&&v| v)
            .count() as u32
    }
    pub fn lake_of(&self, p: Pos) -> Option<&Lake> {
        self.lakes.iter().find(|l| l.cells.contains(&p))
    }
    pub fn in_lake(&self, p: Pos) -> bool {
        self.lake_of(p).is_some()
    }
}
