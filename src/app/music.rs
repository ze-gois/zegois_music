pub(super) fn semitone_for_pitch_class_near(pitch_class: i32, previous: i32) -> i32 {
    (-24_i32..=24)
        .filter(|candidate| (69_i32 + candidate).rem_euclid(12) == pitch_class.rem_euclid(12))
        .min_by_key(|candidate| (candidate - previous).abs())
        .unwrap_or(previous)
}

pub(super) fn graph_walk_melody() -> Vec<i32> {
    let intervals = [7, 4, -3, 7, -5, 3, -4, 7, 4, -3, -7, 5, 3, -4, 7, -5];
    let mut melody = Vec::with_capacity(32);
    let mut current = 0_i32;
    melody.push(current);

    for interval in intervals.into_iter().cycle().take(31) {
        current += interval;
        let pitch_class = (69_i32 + current).rem_euclid(12);
        current = semitone_for_pitch_class_near(pitch_class, current.clamp(-12, 12));
        melody.push(current);
    }

    melody
}
