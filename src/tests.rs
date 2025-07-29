use super::*;
#[cfg(feature = "std")]
use std::time::Duration;

#[test]
#[cfg(feature = "std")]
fn test_from_duration() {
    let duration = Duration::from_secs_f64(2.5);
    let tick = Tick::from(duration);

    // 2.5 seconds * 3,603,600 ticks/second = 9,009,000 ticks.
    assert_eq!(tick.0, (2.5 * TICKS_PER_SECOND as f64) as _);

    // Verify round-trip conversion.
    let back_to_duration = Duration::from(tick);
    assert!((back_to_duration.as_secs_f64() - 2.5).abs() < 1e-10);
}

#[test]
#[cfg(feature = "std")]
fn test_duration_zero() {
    let duration = Duration::ZERO;
    let tick = Tick::from(duration);
    assert_eq!(tick.0, 0);
}

#[test]
#[cfg(feature = "std")]
fn test_subsecond_precision() {
    // Test with a duration that has nanosecond precision.
    //  1.5 seconds.
    let duration = Duration::new(1, 500_000_000);
    let tick = Tick::from(duration);

    // 1.5 seconds * 3,603,600 ticks/second = 5,405,400 ticks,
    assert_eq!(tick.0, (1.5 * TICKS_PER_SECOND as f64) as _);
}

#[test]
fn test_frame_conversions() {
    // Test 60 FPS,
    let fps_60 = NonZeroU32::new(60).unwrap();

    // 1 second should be 60 frames,
    let one_second_ticks = Tick::from_secs(1.0);
    assert_eq!(one_second_ticks.to_frame(fps_60), 60);

    // 60 frames should be 1 second worth of ticks,
    let sixty_frames = Tick::from_frame(60, fps_60);
    assert_eq!(sixty_frames.0, TICKS_PER_SECOND as i64);

    // Test round-trip conversion,
    assert_eq!(sixty_frames.to_frame(fps_60), 60);
}

#[test]
fn test_frame_conversions_various_rates() {
    let test_cases = [
        // 24 FPS: 1 second = 24 frames,
        (24, 1.0, 24),
        // 30 FPS: 2 seconds = 60 frames,
        (30, 2.0, 60),
        // 120 FPS: 0.5 seconds = 60 frames,
        (120, 0.5, 60),
        // 25 FPS: 1 second = 25 frames,
        (25, 1.0, 25),
    ];

    for (fps, seconds, expected_frames) in test_cases {
        let frame_rate = NonZeroU32::new(fps).unwrap();
        let ticks = Tick::from_secs(seconds);
        let frames = ticks.to_frame(frame_rate);

        assert_eq!(
            frames, expected_frames,
            "Failed for {}fps, {}s: expected {}, got {}",
            fps, seconds, expected_frames, frames
        );

        // Test round-trip
        let back_to_ticks = Tick::from_frame(frames, frame_rate);
        assert_eq!(
            back_to_ticks, ticks,
            "Round-trip failed for {}fps, frame {}",
            fps, frames
        );
    }
}

#[test]
fn test_frame_edge_cases() {
    let fps_60 = NonZeroU32::new(60).unwrap();

    // Test zero.
    let zero_ticks = Tick::new(0);
    assert_eq!(zero_ticks.to_frame(fps_60), 0);
    assert_eq!(Tick::from_frame(0, fps_60), zero_ticks);

    // Test negative values.
    // -1 second.
    let negative_ticks = Tick::new(-TICKS_PER_SECOND as i64);
    assert_eq!(negative_ticks.to_frame(fps_60), -60);
    assert_eq!(Tick::from_frame(-60, fps_60), negative_ticks);
}

#[test]
fn test_high_precision_frame_rates() {
    // Test with NTSC frame rate (29.97 fps).
    // We use the NonZeroU32 `to_frame`-variant so we'll test with 2997 (29.97 *
    // 100) and treat it as if it's 29.97 by scaling appropriately.
    let fps_2997 = NonZeroU32::new(2997).unwrap();

    // For 100 seconds at 29.97fps = 2997 frames.
    let hundred_seconds = Tick::from_secs(100.0);
    let frames = hundred_seconds.to_frame(fps_2997);
    // This should be close to 2997 * 100 = 299700 frames.
    assert_eq!(frames, 299700);

    // Test round-trip
    let back_to_ticks = Tick::from_frame(frames, fps_2997);
    assert_eq!(back_to_ticks, hundred_seconds);
}

#[test]
fn test_ops() {
    let ticks = Tick::from_secs(1.0);

    // Test addition.
    assert_eq!(ticks + ticks, Tick::from_secs(2.0));
    assert_eq!(ticks + Tick::from_secs(0.5), Tick::from_secs(1.5));

    // Test subtraction.
    assert_eq!(ticks - ticks, Tick::from_secs(0.0));
    assert_eq!(ticks - Tick::from_secs(0.5), Tick::from_secs(0.5));

    // Test multiplication.
    assert_eq!(ticks * 2.0, Tick::from_secs(2.0));
    assert_eq!(ticks * 0.5, Tick::from_secs(0.5));

    // Test division.
    assert_eq!(ticks / 2.0, Tick::from_secs(0.5));
    assert_eq!(ticks / 0.5, Tick::from_secs(2.0));
}
