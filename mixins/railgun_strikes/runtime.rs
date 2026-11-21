//! Точка входа бандла: счётчик тиков и общие вспомогательные вещи.
//! Сюда же попадает логика из logic.rs.

pub mod logic;

pub use logic::*;

pub const ID: &str = "railgun_strikes";
pub const DEFAULT_DELAY_TICKS: u64 = 20 * 30; // 30 секунд до удара
pub const AIM_HOLD_TICKS: u64 = 15;           // сколько тиков держать ПКМ для фиксации цели

thread_local! {
    static TICK: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
}

pub fn tick() -> u64 {
    TICK.with(|t| { let n = t.get() + 1; t.set(n); n })
}
