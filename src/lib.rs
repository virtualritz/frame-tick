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
//! This makes it 'compatible' with lots of frame- and refresh rates without
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
//!
//! # Cargo features
#![doc = document_features::document_features!()]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod tests;

use core::{
    convert::{AsMut, AsRef},
    num::{NonZeroU32, ParseIntError},
    ops::{Add, Div, Mul, Sub},
    str::FromStr,
};
#[cfg(feature = "std")]
use std::{
    fmt::{Display, Error, Formatter},
    time::Duration,
};
#[cfg(feature = "float_frame_rate")]
pub use typed_floats::StrictlyPositiveFinite;

/// The number of ticks per second.
///
/// Use the `high_res` feature to configure this.
#[cfg(not(feature = "high_res"))]
pub const TICKS_PER_SECOND: i64 = 25_200;
/// The number of ticks per second.
///
/// Use the `high_res` feature to configure this.
#[cfg(feature = "high_res")]
pub const TICKS_PER_SECOND: i64 = 3_603_600;

/// Fixed-point representation of time where each second is divided into
/// [`TICKS_PER_SECOND`].
///
/// This type can also represent negative time as this is common in DCCs like a
/// video editor or animation system where this type would typically be used.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Tick(i64);

impl IntoIterator for Tick {
    type IntoIter = TickIter;
    type Item = i64;

    fn into_iter(self) -> Self::IntoIter {
        TickIter(self.0)
    }
}

/// An iterator over [`Tick`]s.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct TickIter(i64);

impl Iterator for TickIter {
    type Item = i64;

    fn next(&mut self) -> Option<Self::Item> {
        if i64::MAX == self.0 {
            None
        } else {
            let value = self.0;
            self.0 += 1;

            Some(value)
        }
    }
}

impl DoubleEndedIterator for TickIter {
    fn next_back(&mut self) -> Option<Self::Item> {
        if i64::MIN == self.0 {
            None
        } else {
            self.0 -= 1;
            Some(self.0)
        }
    }
}

/// An iterator over [`Tick`]s in reverse order.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct TickRevIter(i64);

impl Iterator for TickRevIter {
    type Item = i64;

    fn next(&mut self) -> Option<Self::Item> {
        let value = self.0;
        self.0 -= 1;
        if i64::MIN == self.0 {
            None
        } else {
            Some(value)
        }
    }
}

impl AsRef<i64> for Tick {
    fn as_ref(&self) -> &i64 {
        &self.0
    }
}

impl AsMut<i64> for Tick {
    fn as_mut(&mut self) -> &mut i64 {
        &mut self.0
    }
}

macro_rules! impl_tick_from {
    ($ty:ty) => {
        impl From<$ty> for Tick {
            fn from(value: $ty) -> Self {
                Self(value as _)
            }
        }
    };
}

macro_rules! impl_from_tick {
    ($ty:ty) => {
        impl From<Tick> for $ty {
            fn from(tick: Tick) -> Self {
                tick.0 as _
            }
        }
    };
}

impl_from_tick!(u8);
impl_from_tick!(u16);
impl_from_tick!(u32);
impl_from_tick!(u64);
impl_from_tick!(u128);
impl_from_tick!(usize);
impl_from_tick!(i8);
impl_from_tick!(i16);
impl_from_tick!(i32);
impl_from_tick!(i64);
impl_from_tick!(i128);
impl_from_tick!(isize);
impl_from_tick!(f32);

impl_from_tick!(f64);

impl_tick_from!(u8);
impl_tick_from!(u16);
impl_tick_from!(u32);
impl_tick_from!(i8);
impl_tick_from!(i16);
impl_tick_from!(i32);
impl_tick_from!(i64);

impl From<f32> for Tick {
    fn from(value: f32) -> Self {
        Self((value + 0.5) as _)
    }
}

impl From<f64> for Tick {
    fn from(value: f64) -> Self {
        Self((value + 0.5) as _)
    }
}

impl FromStr for Tick {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse::<i64>().map(Tick)
    }
}

impl Add for Tick {
    type Output = Tick;

    fn add(self, rhs: Self) -> Self::Output {
        Tick(self.0 + rhs.0)
    }
}

impl Sub for Tick {
    type Output = Tick;

    fn sub(self, rhs: Self) -> Self::Output {
        Tick(self.0 - rhs.0)
    }
}

// Multiplication is done with floating point numbers and rounded to the nearest
// tick.
impl Mul for Tick {
    type Output = Tick;

    fn mul(self, rhs: Self) -> Self::Output {
        let result = (self.0 as f64) * (rhs.0 as f64);
        let rounded = if result >= 0.0 {
            result + 0.5
        } else {
            result - 0.5
        };

        Tick(rounded as i64)
    }
}

// Division is done with floating point numbers and rounded to the nearest tick.
impl Div for Tick {
    type Output = Tick;

    fn div(self, rhs: Self) -> Self::Output {
        let result = (self.0 as f64) / (rhs.0 as f64);
        let rounded = if result >= 0.0 {
            result + 0.5
        } else {
            result - 0.5
        };

        Tick(rounded as i64)
    }
}

// Optional: Implement Display for better debugging
#[cfg(feature = "std")]
impl Display for Tick {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "Tick({})", self.0)
    }
}

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
        (self.0 as f64 * frame_rate.get() as f64 / TICKS_PER_SECOND as f64)
            .round() as _
    }

    fn from_frame(frame: i64, frame_rate: StrictlyPositiveFinite<f32>) -> Self {
        Self(
            (frame as f64 * TICKS_PER_SECOND as f64 / frame_rate.get() as f64)
                .round() as _,
        )
    }
}

#[cfg(feature = "float_frame_rate")]
impl FrameRateConversion<StrictlyPositiveFinite<f64>> for Tick {
    fn to_frame(self, frame_rate: StrictlyPositiveFinite<f64>) -> i64 {
        (self.0 as f64 * frame_rate.get() / TICKS_PER_SECOND as f64).round()
            as _
    }

    fn from_frame(frame: i64, frame_rate: StrictlyPositiveFinite<f64>) -> Self {
        Self(
            (frame as f64 * TICKS_PER_SECOND as f64 / frame_rate.get()).round()
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
