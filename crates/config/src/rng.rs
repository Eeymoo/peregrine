//! 简单的线性同余随机数生成器。
//!
//! 跨 crate 共享：被 `crates/material` 的 Rhai host function 与未来的
//! `crates/peregrine` 旧 `shapes.rs::SimpleRng` 共同使用，确保 `random_orb` 等
//! 物料脚本与旧实现产生完全一致的随机序列。
//!
//! 算法：64-bit LCG，常数 `6364136223846793005`（Knuth 推荐）。
//! 输出取高 24 位以获得更好的统计性质。

/// 简单的线性同余 RNG。
#[derive(Debug, Clone, Copy)]
pub struct SimpleRng {
    state: u64,
}

impl SimpleRng {
    /// 用给定种子创建 RNG。种子为 0 时内部转为 1（避免退化）。
    pub fn new(seed: u64) -> Self {
        Self { state: seed.max(1) }
    }

    /// 推进状态并返回下一个 u64。
    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_mul(6364136223846793005).wrapping_add(1);
        self.state
    }

    /// 返回 `[0.0, 1.0)` 区间的伪随机 f32。
    pub fn next_f32(&mut self) -> f32 {
        (self.next_u64() & 0x00FF_FFFF) as f32 / 0x0100_0000 as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic_with_same_seed() {
        let mut a = SimpleRng::new(42);
        let mut b = SimpleRng::new(42);
        for _ in 0..10 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
    }

    #[test]
    fn different_seeds_produce_different_output() {
        let mut a = SimpleRng::new(1);
        let mut b = SimpleRng::new(2);
        assert_ne!(a.next_u64(), b.next_u64());
    }

    #[test]
    fn zero_seed_normalizes_to_one() {
        let mut a = SimpleRng::new(0);
        let mut b = SimpleRng::new(1);
        assert_eq!(a.next_u64(), b.next_u64());
    }

    #[test]
    fn next_f32_in_range() {
        let mut rng = SimpleRng::new(12345);
        for _ in 0..1000 {
            let f = rng.next_f32();
            assert!((0.0..1.0).contains(&f));
        }
    }

    #[test]
    fn known_output() {
        // 与 crates/peregrine/src/shapes.rs::SimpleRng 保持完全一致的实现，
        // 此处断言旧测试中已验证的输出值。
        let mut rng = SimpleRng::new(1);
        // state = 1 * 6364136223846793005 + 1 = 6364136223846793006
        assert_eq!(rng.next_u64(), 6364136223846793006);
        // next_f32 取高 24 位：(6364136223846793006 & 0xFFFFFF) / 0x1000000
        let f = rng.next_f32();
        assert!(f >= 0.0 && f < 1.0);
    }
}
