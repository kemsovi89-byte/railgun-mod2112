//! Хук native_command: команды мода.
//!
//! /railgun give        — выдать пушку (предмет контент-мода railgun_designator)
//! /railgun select      — переключить тип удара (пока без колеса — по кругу)
//! /railgun select <1|2|3> — выбрать удар напрямую
//! /railgun fire        — выстрел по зафиксированной цели (или по точке взгляда)
//! /strike now          — быстрый удар по своей позиции (для теста)

use super::logic::*;
use super::{tick, AIM_HOLD_TICKS, DEFAULT_DELAY_TICKS, ID};

pub fn railgun_command(game: &mut crate::game::Game<'_>, command: &str, ran: &mut bool) {
    let Some(args) = command.strip_prefix("railgun") else { return; };
    if !game.native_on(ID) || !(args.is_empty() || args.starts_with(' ')) { return; }
    *ran = true;

    let args = args.trim();
    let mut parts = args.split_whitespace();

    match parts.next() {
        None | Some("help") => {
            game.native_chat("/railgun give | select [1-3] | fire");
            game.native_chat("/strike now — удар по своей позиции");
        }
        Some("give") => {
            // TODO(api): заменить на точное имя функции выдачи предмета из справочника:
            // https://playhorizon.dev/docs — раздел Content (предметы контент-модов).
            game.native_chat("Выдан: Орбитальная Рельсовая Пушка (id: railgun:railgun_designator)");
            game.native_chat("  ^ если предмет не появился — см. README, раздел «Точка выдачи предмета»");
        }
        Some("select") => {
            let Some(p) = game.native_player() else {
                game.native_chat("Зайдите в мир, чтобы выбирать удар");
                return;
            };
            let mut st = state_of(p.uuid());
            st.selected = match parts.next() {
                Some("1") => StrikeKind::Railgun,
                Some("2") => StrikeKind::Nuke,
                Some("3") => StrikeKind::Meteors,
                _ => st.selected.next(),
            };
            set_state(p.uuid(), st);
            game.native_chat(&format!("Выбран удар: {}", st.selected.label()));
        }
        Some("fire") => {
            fire_from_state(game);
        }
        Some(_) => {
            game.native_chat("Неизвестная подкоманда. /railgun help");
        }
    }
}

pub fn strike_command(game: &mut crate::game::Game<'_>, command: &str, ran: &mut bool) {
    let Some(args) = command.strip_prefix("strike") else { return; };
    if !game.native_on(ID) || !(args.is_empty() || args.starts_with(' ')) { return; }
    *ran = true;
    let args = args.trim();

    if args == "now" || args.is_empty() {
        let Some(p) = game.native_player() else {
            game.native_chat("Зайдите в мир, чтобы ударить");
            return;
        };
        let now = tick();
        let due = now_schedule(StrikeKind::Nuke, p.x() as i64, p.y() as i64, p.z() as i64, now, DEFAULT_DELAY_TICKS);
        game.native_chat(&format!("Удар засечён, упадёт через 30 сек (тик {due}) — бегите!"));
    } else {
        game.native_chat("Использование: /strike [now]");
    }
}

/// Выстрел: берём зафиксированную цель из состояния игрока.
/// Если цели нет — пробуем рейкаст взгляда прямо сейчас.
pub fn fire_from_state(game: &mut crate::game::Game<'_>) {
    let Some(p) = game.native_player() else {
        game.native_chat("Зайдите в мир, чтобы стрелять");
        return;
    };
    let st = state_of(p.uuid());
    let target = match st.aiming {
        Some(t) => t,
        None => match super::player::look_target(game, p) {
            Some(t) => t,
            None => { game.native_chat("Нет цели: зажмите ПКМ с пушкой в руке, чтобы прицелиться"); return; }
        },
    };
    let now = tick();
    let (kind, x, y, z) = (st.selected, target[0], target[1], target[2]);
    let due = now_schedule(kind, x, y, z, now, DEFAULT_DELAY_TICKS);
    game.native_chat(&format!("{}! Цель ({x}, {y}, {z}), удар через 30 сек", kind.label()));
    // Сброс цели после выстрела
    let mut st2 = state_of(p.uuid());
    st2.aiming = None;
    st2.aim_started = None;
    set_state(p.uuid(), st2);
    let _ = AIM_HOLD_TICKS;
}
