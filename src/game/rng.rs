pub struct Rng(u32);
impl Rng {
    pub fn new(seed: u32) -> Self {
        Rng(seed)
    }
    pub fn next_f64(&mut self) -> f64 {
        let mut a = self.0;
        a = a.wrapping_add(0x6D2B79F5);
        self.0 = a;
        let mut t = (a ^ (a >> 15)).wrapping_mul(1 | a);
        t = t.wrapping_add((t ^ (t >> 7)).wrapping_mul(61 | t)) ^ t;
        ((t ^ (t >> 14)) as f64) / 4294967296.0
    }
    pub fn next_usize(&mut self, max: usize) -> usize {
        (self.next_f64() * max as f64) as usize
    }
    pub fn shuffle<T>(&mut self, a: &mut [T]) {
        for i in (1..a.len()).rev() {
            let j = self.next_usize(i + 1);
            a.swap(i, j);
        }
    }
}
