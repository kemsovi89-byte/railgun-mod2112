//! Хук native_tick: выполняем дозревшие удары и обновляем прицеливание.

use super::logic::take_due;
use super::player::update_aim;
use super::tick;
use super::ID;

/// Порция блоков, вырезаемая за один тик (чтобы не фризить игру).
pub const BLOCKS_PER_TICK: usize = 4096;

thread_local! {
    static QUEUE: std::cell::RefCell<std::collections::VecDeque<[i64; 3]>> =
        const { std::cell::RefCell::new(std::collections::VecDeque::new()) };
}

pub fn railgun_tick(game: &mut crate::game::Game<'_>) {
    if !game.native_on(ID) { return; }
    let now = tick();

    // 1. Прицеливание текущего игрока (вызывается для каждого игрока в реальном моде).
    //    В одиночной игре native_player доступен здесь же.
    // TODO(api): если API даёт перебор всех игроков — делайте цикл по ним.

    // 2. Новые дозревшие удары -> раскладываем их блоки в очередь.
    let due = take_due(now);
    if !due.is_empty() {
        for strike in due {
            game.native_chat(&format!(
                "Орбитальный удар! ({}, {}, {})",
                strike.x, strike.y, strike.z
            ));
            for block in super::logic::blocks_for(strike) {
                QUEUE.with(|q| q.borrow_mut().push_back(block));
            }
        }
    }

    // 3. Вырезаем порцию блоков из очереди.
    QUEUE.with(|q| {
        let mut q = q.borrow_mut();
        for _ in 0..BLOCKS_PER_TICK {
            match q.pop_front() {
                Some([x, y, z]) => super::world_edit::set_air(game, x, y, z),
                None => break,
            }
        }
    });
}
