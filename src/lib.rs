//! Fixed-point representation of time where each second is divided into
//! 3,603,600 `Tick`s (or 25,200, if the `cargo` feature `low_res` is set).
//!
//! This crate was inspired by [this article](https://iquilezles.org/articles/ticks/)
//! from [Inigo Quilez](https://iquilezles.org/). Please refer to this for a
//! more detailed explanation.
//! > *Note that the default for `TICKS_PER_SECOND`, 3,603,600, is the Least
//! > Common Multiple of all numbers in the list given in the article as well as
//! > 11 and 13, which are needed for NTSC.*
//!
//! This makes is 'compatible' with lots of frame- and refresh rates without
//! ever mapping outside of or repeating a frame. That is: without strobing.
//!
//! In particular, a `Tick` can represent exactly:
//!
//! * 24hz and 48hz, great for movie playback.
//!
//! * 6hz, 8hz and 12hz, great for animating on 4s, 3s and 2s.
//!
//! * 29.97hz, 59.94hz NTSC found in Japan, South Korea and the USA.
//!
//! * 30hz, 60hz, for internet video and TV in the USA.
//!
//! * 25hz and 50hz, for TV in the EU.
//!
//! * 72hz, for Oculus Quest 1.
//!
//! * 90hz for Quest 2, Rift and other headsets.
//!
//! * 120hz, 144hz and 240hz, for newer VR headesets and high frequency
//!   monitors.
//!
//! * And many more.
//!
//! # Examples
//!
//! ```
//! # use core::num::NonZeroU32;
//! use tick::{FrameRateConversion, Tick};
//!
//! let tick = Tick::from_secs(1.0);
//!
//! /// A round trip is lossless.
//! assert_eq!(1.0, tick.to_secs());
//! /// One second at 120hz == frame № 120.
//! assert_eq!(120, tick.to_frame(NonZeroU32::new(120).unwrap()));
//! ```
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod tests;

use core::num::NonZeroU32;
use derive_more as dm;
#[cfg(feature = "std")]
use std::time::Duration;
#[cfg(feature = "float_frame_rate")]
pub use typed_floats::StrictlyPositiveFinite;

/// Fixed-point representation of time where each second is divided into
/// [`TICKS_PER_SECOND`].
///
/// This type can also represent negative time as this is common in DCCs like a
/// video editor or animation system where this type would typically be used.
#[derive(
    Copy,
    Clone,
    Debug,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Hash,
    dm::Display,
    dm::From,
    dm::Into,
    dm::FromStr,
    dm::AsRef,
    dm::AsMut,
    dm::Mul,
    dm::Div,
    dm::Add,
    dm::Sub,
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Tick(i64);

/// The number of ticks per second.
///
/// Use the [`low_res`] feature to configure this.
#[cfg(not(feature = "low_res"))]
pub const TICKS_PER_SECOND: i64 = 3603600;
/// The number of ticks per second.
///
/// Use the [`low_res`] feature to configure this.
#[cfg(feature = "low_res")]
pub const TICKS_PER_SECOND: i64 = 25200;

impl Tick {
    pub fn new(value: i64) -> Self {
        Self(value)
    }

    /// Create ticks from seconds.
    pub fn from_secs(secs: f64) -> Self {
        Self((secs * TICKS_PER_SECOND as f64) as i64)
    }

    /// Convert ticks to seconds.
    pub fn to_secs(&self) -> f64 {
        self.0 as f64 / TICKS_PER_SECOND as f64
    }
}

/// Conversion to/from specified frame rates.
pub trait FrameRateConversion<T> {
    fn to_frame(self, frame_rate: T) -> i64;
    fn from_frame(frame: i64, frame_rate: T) -> Self;
}

impl FrameRateConversion<NonZeroU32> for Tick {
    /// Convert ticks to frame number at the specified integer frame rate.
    fn to_frame(self, frame_rate: NonZeroU32) -> i64 {
        (self.0 as i128 * frame_rate.get() as i128 / TICKS_PER_SECOND as i128)
            as _
    }

    /// Convert frame number to ticks at the specified integer frame rate.
    fn from_frame(frame: i64, frame_rate: NonZeroU32) -> Self {
        Self(
            (frame as i128 * TICKS_PER_SECOND as i128
                / frame_rate.get() as i128) as _,
        )
    }
}

#[cfg(feature = "float_frame_rate")]
impl FrameRateConversion<StrictlyPositiveFinite<f32>> for Tick {
    fn to_frame(self, frame_rate: StrictlyPositiveFinite<f32>) -> i64 {
        (self.0 as f64 * frame_rate.get() as f64 / TICKS_PER_SECOND as f64
            + 0.5) as _
    }

    fn from_frame(frame: i64, frame_rate: StrictlyPositiveFinite<f32>) -> Self {
        Self(
            (frame as f64 * TICKS_PER_SECOND as f64 / frame_rate.get() as f64
                + 0.5) as _,
        )
    }
}

#[cfg(feature = "float_frame_rate")]
impl FrameRateConversion<StrictlyPositiveFinite<f64>> for Tick {
    fn to_frame(self, frame_rate: StrictlyPositiveFinite<f64>) -> i64 {
        (self.0 as f64 * frame_rate.get() / TICKS_PER_SECOND as f64 + 0.5) as _
    }

    fn from_frame(frame: i64, frame_rate: StrictlyPositiveFinite<f64>) -> Self {
        Self(
            (frame as f64 * TICKS_PER_SECOND as f64 / frame_rate.get() + 0.5)
                as _,
        )
    }
}

#[cfg(feature = "std")]
impl From<Duration> for Tick {
    fn from(duration: Duration) -> Self {
        let secs = duration.as_secs_f64();
        Self::from_secs(secs)
    }
}

#[cfg(feature = "std")]
impl From<Tick> for Duration {
    fn from(tick: Tick) -> Self {
        Duration::from_secs_f64(tick.to_secs())
    }
}
