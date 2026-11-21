//! Чистая логика мода: типы ударов, планировщик, геометрия.
//! Этот файл не зависит от игры — его можно тестировать через `cargo test`.

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum StrikeKind {
    Railgun, // одна пробивная колонна вниз, до бедрока
    Nuke,    // сферический кратер
    Meteors, // россыпь мелких кратеров по кругу
}

pub const ALL_KINDS: [StrikeKind; 3] = [
    StrikeKind::Railgun,
    StrikeKind::Nuke,
    StrikeKind::Meteors,
];

impl StrikeKind {
    pub fn label(self) -> &'static str {
        match self {
            StrikeKind::Railgun => "Railgun (пробой до бедрока)",
            StrikeKind::Nuke => "Nuke (кратер-сфера)",
            StrikeKind::Meteors => "Meteors (метеоритный шторм)",
        }
    }
    pub fn next(self) -> StrikeKind {
        match self {
            StrikeKind::Railgun => StrikeKind::Nuke,
            StrikeKind::Nuke => StrikeKind::Meteors,
            StrikeKind::Meteors => StrikeKind::Railgun,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Strike {
    pub kind: StrikeKind,
    pub x: i64,
    pub y: i64,
    pub z: i64,
    pub due_tick: u64, // тик, когда ударить (20 тиков = 1 сек)
}

// Состояние игрока: выбранный удар и зафиксированная цель.
#[derive(Clone, Copy)]
pub struct PlayerState {
    pub selected: StrikeKind,
    pub aiming: Option<[i64; 3]>,
    // tick, до которого нужно держать правую кнопку, чтобы зафиксировать цель
    pub aim_started: Option<u64>,
}

impl Default for PlayerState {
    fn default() -> Self {
        PlayerState { selected: StrikeKind::Railgun, aiming: None, aim_started: None }
    }
}

thread_local! {
    pub static PENDING: std::cell::RefCell<Vec<Strike>> =
        const { std::cell::RefCell::new(Vec::new()) };
    pub static STATES: std::cell::RefCell<std::collections::HashMap<u64, PlayerState>> =
        const { std::cell::RefCell::new(std::collections::HashMap::new()) };
}

pub fn now_schedule(kind: StrikeKind, x: i64, y: i64, z: i64, now: u64, delay_ticks: u64) -> u64 {
    let strike = Strike { kind, x, y, z, due_tick: now + delay_ticks };
    let due = strike.due_tick;
    PENDING.with(|p| p.borrow_mut().push(strike));
    due
}

pub fn take_due(now: u64) -> Vec<Strike> {
    PENDING.with(|p| {
        let mut q = p.borrow_mut();
        let (due, rest): (Vec<_>, Vec<_>) = q.drain(..).partition(|s| s.due_tick <= now);
        *q = rest;
        due
    })
}

pub fn state_of(player: u64) -> PlayerState {
    STATES.with(|s| *s.borrow_mut().entry(player).or_default())
}

pub fn set_state(player: u64, st: PlayerState) {
    STATES.with(|s| { s.borrow_mut().insert(player, st); });
}

/// Все блоки внутри сферы с центром (cx, cy, cz).
pub fn sphere(cx: i64, cy: i64, cz: i64, r: i64) -> Vec<[i64; 3]> {
    let r2 = r * r;
    let mut out = Vec::new();
    for dy in -r..=r {
        let dy2 = dy * dy;
        for dx in -r..=r {
            let dx2 = dx * dx;
            if dx2 + dy2 > r2 { continue; }
            for dz in -r..=r {
                if dx2 + dy2 + dz * dz <= r2 {
                    out.push([cx + dx, cy + dy, cz + dz]);
                }
            }
        }
    }
    out
}

/// Вертикальная колонна: диск радиуса r, от высоты удара и вниз до 0 (бедрок).
pub fn column(cx: i64, cy: i64, cz: i64, r: i64) -> Vec<[i64; 3]> {
    let r2 = r * r;
    let mut out = Vec::new();
    for y in 0..=cy {
        for dx in -r..=r {
            for dz in -r..=r {
                if dx * dx + dz * dz <= r2 {
                    out.push([cx + dx, y, cz + dz]);
                }
            }
        }
    }
    out
}

/// Список блоков, которые вырежет удар. ВНИМАНИЕ: для больших ударов это
/// десятки тысяч блоков — в tick.rs их вырезают порциями, а не всё за тик.
pub fn blocks_for(strike: Strike) -> Vec<[i64; 3]> {
    match strike.kind {
        StrikeKind::Nuke => sphere(strike.x, strike.y, strike.z, 24),
        StrikeKind::Railgun => column(strike.x, strike.y, strike.z, 5),
        StrikeKind::Meteors => {
            let mut out = Vec::new();
            for i in 0..12 {
                let a = i as f64 * 0.52;
                let mx = strike.x + (a.sin() * 30.0) as i64;
                let mz = strike.z + (a.cos() * 30.0) as i64;
                out.extend(sphere(mx, strike.y, mz, 6));
            }
            out
        }
    }
}

/// Рейкаст взгляда: шагаем лучом от глаз игрока по направлению взгляда
/// до первого «твёрдого» блока (is_solid передаётся извне — из игрового хука).
pub fn raycast(
    eye: [f64; 3],
    dir: [f64; 3],
    max_dist: f64,
    is_solid: impl Fn(i64, i64, i64) -> bool,
) -> Option<[i64; 3]> {
    let len = (dir[0] * dir[0] + dir[1] * dir[1] + dir[2] * dir[2]).sqrt();
    if len < 1e-6 { return None; }
    let d = [dir[0] / len, dir[1] / len, dir[2] / len];
    let mut t = 0.0f64;
    while t <= max_dist {
        let px = (eye[0] + d[0] * t).floor() as i64;
        let py = (eye[1] + d[1] * t).floor() as i64;
        let pz = (eye[2] + d[2] * t).floor() as i64;
        if is_solid(px, py, pz) {
            return Some([px, py, pz]);
        }
        t += 0.5;
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crater_sphere_volume() {
        // объём шара радиуса 2 = 33 блока
        assert_eq!(sphere(0, 64, 0, 2).len(), 33);
    }

    #[test]
    fn column_reaches_bedrock() {
        let blocks = column(10, 70, 10, 2);
        assert!(blocks.contains(&[10, 0, 10])); // до бедрока включительно
        assert!(blocks.contains(&[10, 70, 10]));
        assert!(!blocks.contains(&[10, 71, 10]));
    }

    #[test]
    fn raycast_hits_first_solid() {
        let hit = raycast([0.5, 70.5, 0.5], [0.0, -1.0, 0.0], 100.0, |x, y, _z| y == 64 && x == 0);
        assert_eq!(hit, Some([0, 64, 0]));
    }

    #[test]
    fn kinds_cycle() {
        assert_eq!(StrikeKind::Railgun.next(), StrikeKind::Nuke);
        assert_eq!(StrikeKind::Meteors.next(), StrikeKind::Railgun);
    }
}
