//! Хук native_player: отслеживаем ввод (ПКМ — прицеливание) и взгляд.
//! Плюс общий рейкаст-хелпер для цели.

use super::logic::*;
use super::{tick, AIM_HOLD_TICKS, ID};

/// Вызывается из тика для текущего игрока: если держится ПКМ с пушкой
/// в руке нужное число тиков — фиксируем цель по взгляду.
pub fn update_aim(game: &mut crate::game::Game<'_>, p: &crate::game::Player<'_>) {
    if !game.native_on(ID) { return; }
    let mut st = state_of(p.uuid());

    // TODO(api): «держится ли ПКМ» и «пушка ли в руке» берём из ввода/инвентаря.
    // В справочнике API ищите: input (mouse buttons) и held item / hotbar slot.
    let holding_railgun = super::content::is_railgun_held(p);
    let rmb_down = super::input::mouse_right_down(p);

    if holding_railgun && rmb_down {
        let now = tick();
        let started = st.aim_started.unwrap_or(now);
        st.aim_started = Some(started);
        if now - started >= AIM_HOLD_TICKS {
            if let Some(t) = look_target(game, p) {
                if st.aiming != Some(t) {
                    st.aiming = Some(t);
                    game.native_chat(&format!("Цель зафиксирована: ({}, {}, {})", t[0], t[1], t[2]));
                }
            }
        }
        set_state(p.uuid(), st);
    } else {
        // кнопку отпустили — начинаем замер прицеливания заново
        st.aim_started = None;
        set_state(p.uuid(), st);
    }
}

/// Точка, куда смотрит игрок: рейкаст от глаз по направлению взгляда.
/// is_solid — это заглушка-интеграция: реальную проверку блока берём из API мира.
pub fn look_target(
    game: &mut crate::game::Game<'_>,
    p: &crate::game::Player<'_>,
) -> Option<[i64; 3]> {
    let eye: [f64; 3] = [p.eye_x(), p.eye_y(), p.eye_z()];
    let dir: [f64; 3] = [p.look_x(), p.look_y(), p.look_z()];
    raycast(eye, dir, 200.0, |x, y, z| super::world_edit::is_solid(game, x, y, z))
}
