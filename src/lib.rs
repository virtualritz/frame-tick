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
//! - 24hz and 48hz, great for movie playback.
//!
//! - 6hz, 8hz and 12hz, great for animating on 4s, 3s and 2s.
//!
//! - 29.97hz, 59.94hz NTSC found in Japan, South Korea and the USA.
//!
//! - 30hz, 60hz, for internet video and TV in the USA.
//!
//! - 25hz and 50hz, for TV in the EU.
//!
//! - 72hz, for Oculus Quest 1.
//!
//! - 90hz for Quest 2, Rift and other headsets.
//!
//! - 120hz, 144hz and 240hz, for newer VR headesets and high frequency
//!   monitors.
//!
//! - And many more.
//!
//! # Examples
//!
//! ```
//! # use core::num::NonZeroU32;
//! use frame_tick::{FrameRateConversion, FramesPerSec, Tick};
//!
//! let tick = Tick::from_secs(1.0);
//!
//! /// A round trip is lossless.
//! assert_eq!(1.0, tick.to_secs());
//! /// One second at 120hz == frame № 120.
//! assert_eq!(120, tick.to_frames(FramesPerSec::new(120).unwrap()));
//! ```
//!
//! # Cargo features
#![doc = document_features::document_features!()]
#![cfg_attr(not(feature = "std"), no_std)]

use core::{
    convert::{AsMut, AsRef},
    num::{NonZeroU32, ParseIntError},
    ops::{Add, Div, Mul, Sub},
    str::FromStr,
};
#[cfg(feature = "facet")]
use facet::Facet;
#[cfg(all(feature = "std", doc))]
use std::time::Duration;

#[cfg(feature = "std")]
pub mod std_traits;

#[cfg(test)]
mod tests;

#[cfg(feature = "float_frame_rate")]
pub type FramesPerSecF32 = typed_floats::StrictlyPositiveFinite<f32>;
#[cfg(feature = "float_frame_rate")]
pub type FramesPerSecF64 = typed_floats::StrictlyPositiveFinite<f64>;

pub type FramesPerSec = NonZeroU32;

/// A frame rate represented as a fraction (numerator/denominator).
///
/// This allows exact representation of fractional frame rates like NTSC
/// (30000/1001 ≈ 29.97 fps).
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "facet", derive(Facet))]
#[cfg_attr(feature = "facet", facet(opaque))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FrameRate {
    /// Numerator (frames).
    num: NonZeroU32,
    /// Denominator (per seconds).
    den: NonZeroU32,
}

impl FrameRate {
    /// 24 fps - Film.
    pub const FILM: Self = Self {
        num: NonZeroU32::new(24).unwrap(),
        den: NonZeroU32::new(1).unwrap(),
    };
    /// 30 fps.
    pub const FPS_30: Self = Self {
        num: NonZeroU32::new(30).unwrap(),
        den: NonZeroU32::new(1).unwrap(),
    };
    /// 60 fps.
    pub const FPS_60: Self = Self {
        num: NonZeroU32::new(60).unwrap(),
        den: NonZeroU32::new(1).unwrap(),
    };
    /// 29.97 fps (30000/1001) - NTSC.
    pub const NTSC: Self = Self {
        num: NonZeroU32::new(30000).unwrap(),
        den: NonZeroU32::new(1001).unwrap(),
    };
    /// 23.976 fps (24000/1001) - Film on NTSC video.
    pub const NTSC_FILM: Self = Self {
        num: NonZeroU32::new(24000).unwrap(),
        den: NonZeroU32::new(1001).unwrap(),
    };
    /// 59.94 fps (60000/1001) - NTSC high frame rate.
    pub const NTSC_HIGH: Self = Self {
        num: NonZeroU32::new(60000).unwrap(),
        den: NonZeroU32::new(1001).unwrap(),
    };
    /// 25 fps - PAL.
    pub const PAL: Self = Self {
        num: NonZeroU32::new(25).unwrap(),
        den: NonZeroU32::new(1).unwrap(),
    };
    /// 50 fps - PAL high frame rate.
    pub const PAL_HIGH: Self = Self {
        num: NonZeroU32::new(50).unwrap(),
        den: NonZeroU32::new(1).unwrap(),
    };

    /// Create a new frame rate from numerator and denominator.
    ///
    /// # Example
    /// ```
    /// use frame_tick::FrameRate;
    ///
    /// // 29.97 fps (NTSC)
    /// let ntsc = FrameRate::new(30000, 1001).unwrap();
    /// ```
    #[inline]
    pub fn new(num: u32, den: u32) -> Option<Self> {
        Some(Self {
            num: NonZeroU32::new(num)?,
            den: NonZeroU32::new(den)?,
        })
    }

    /// Create an integer frame rate (e.g., 24 fps = 24/1).
    #[inline]
    pub fn from_int(fps: u32) -> Option<Self> {
        Self::new(fps, 1)
    }

    /// Get the numerator.
    #[inline]
    pub fn num(&self) -> u32 {
        self.num.get()
    }

    /// Get the denominator.
    #[inline]
    pub fn den(&self) -> u32 {
        self.den.get()
    }
}

impl From<NonZeroU32> for FrameRate {
    fn from(fps: NonZeroU32) -> Self {
        Self {
            num: fps,
            den: unsafe { NonZeroU32::new_unchecked(1) },
        }
    }
}

/// The number of ticks per second.
///
/// Use the `low_res` feature to configure this.
#[cfg(not(feature = "low_res"))]
pub const TICKS_PER_SECOND: i64 = 3_603_600;
/// The number of ticks per second.
///
/// Use the `low_res` feature to configure this.
#[cfg(feature = "low_res")]
pub const TICKS_PER_SECOND: i64 = 25_200;

/// Fixed-point representation of time where each second is divided into
/// [`TICKS_PER_SECOND`].
///
/// This type can also represent negative time as this is common in DCCs like a
/// video editor or animation system where this type would typically be used.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[cfg_attr(feature = "facet", derive(Facet))]
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
#[cfg_attr(feature = "facet", derive(Facet))]
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
#[cfg_attr(feature = "facet", derive(Facet))]
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

        impl From<&Tick> for $ty {
            fn from(tick: &Tick) -> Self {
                tick.0 as _
            }
        }
    };
}

impl_from_tick!(u64);
impl_from_tick!(u128);
impl_from_tick!(usize);
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

macro_rules! round {
    ($ty:ty, $value:expr) => {{
        let value = $value;
        #[cfg(feature = "std")]
        let value = value.round();
        #[cfg(not(feature = "std"))]
        let value = if value >= 0.0 {
            value + 0.5
        } else {
            value - 0.5
        };

        value as i64
    }};
}

impl From<f32> for Tick {
    fn from(value: f32) -> Self {
        Self(round!(f32, value))
    }
}

impl From<f64> for Tick {
    fn from(value: f64) -> Self {
        Self(round!(f64, value))
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

macro_rules! impl_mul_div_float {
    ($ty:ty) => {
        impl Mul<$ty> for Tick {
            type Output = Self;

            fn mul(self, rhs: $ty) -> Self::Output {
                let value = self.0 as $ty * rhs;
                Self(round!($ty, value))
            }
        }

        impl Div<$ty> for Tick {
            type Output = Self;

            fn div(self, rhs: $ty) -> Self::Output {
                let value = self.0 as $ty / rhs;
                Self(round!($ty, value))
            }
        }
    };
}

impl_mul_div_float!(f32);
impl_mul_div_float!(f64);

macro_rules! impl_mul_div_int {
    ($ty:ty) => {
        impl Mul<$ty> for Tick {
            type Output = Self;

            fn mul(self, rhs: $ty) -> Self::Output {
                Self((self.0 as $ty * rhs) as _)
            }
        }

        impl Div<$ty> for Tick {
            type Output = Self;

            fn div(self, rhs: $ty) -> Self::Output {
                Self((self.0 as $ty / rhs) as _)
            }
        }
    };
}

impl_mul_div_int!(u8);
impl_mul_div_int!(u16);
impl_mul_div_int!(u32);
impl_mul_div_int!(u64);
impl_mul_div_int!(u128);
impl_mul_div_int!(usize);
impl_mul_div_int!(i8);
impl_mul_div_int!(i16);
impl_mul_div_int!(i32);
impl_mul_div_int!(i64);
impl_mul_div_int!(i128);
impl_mul_div_int!(isize);

impl Tick {
    #[inline]
    pub fn new(value: i64) -> Self {
        Self(value)
    }

    /// Create ticks from seconds.
    #[inline]
    pub fn from_secs(secs: f64) -> Self {
        Self((secs * TICKS_PER_SECOND as f64) as i64)
    }

    /// Convert ticks to seconds.
    #[inline]
    pub fn to_secs(&self) -> f64 {
        self.0 as f64 / TICKS_PER_SECOND as f64
    }

    /// Linearly interpolate between two ticks.
    #[inline]
    pub fn lerp(self, other: Self, t: f64) -> Self {
        let t = t.clamp(0.0, 1.0);
        Self(round!(f64, self.0 as f64 * (1.0 - t) + other.0 as f64 * t))
    }

    /// Convert ticks to timecode (hours, minutes, seconds, frames) at the
    /// given frame rate.
    ///
    /// Returns `(hours, minutes, seconds, frames)`.
    #[inline]
    pub fn to_timecode(self, frame_rate: FrameRate) -> (i64, i64, i64, i64) {
        // Calculate total frames using exact frame rate (with rounding):
        // total_frames = ticks * (num/den) / TICKS_PER_SECOND
        let divisor = TICKS_PER_SECOND as i128 * frame_rate.den() as i128;
        let total_frames = ((self.0 as i128 * frame_rate.num() as i128
            + divisor / 2)
            / divisor) as i64;

        // Nominal fps for h:m:s:f display (ceiling of actual fps).
        let nominal_fps = (frame_rate.num() as i64 + frame_rate.den() as i64
            - 1)
            / frame_rate.den() as i64;

        let frames = total_frames % nominal_fps;
        let total_seconds = total_frames / nominal_fps;
        let seconds = total_seconds % 60;
        let total_minutes = total_seconds / 60;
        let minutes = total_minutes % 60;
        let hours = total_minutes / 60;

        (hours, minutes, seconds, frames)
    }

    /// Create ticks from timecode (hours, minutes, seconds, frames) at the
    /// given frame rate.
    #[inline]
    pub fn from_timecode(
        hours: i64,
        minutes: i64,
        seconds: i64,
        frames: i64,
        frame_rate: FrameRate,
    ) -> Self {
        // Nominal fps for h:m:s:f display (ceiling of actual fps).
        let nominal_fps = (frame_rate.num() as i64 + frame_rate.den() as i64
            - 1)
            / frame_rate.den() as i64;

        let total_frames = hours * 3600 * nominal_fps
            + minutes * 60 * nominal_fps
            + seconds * nominal_fps
            + frames;

        // Convert frames to ticks using exact frame rate:
        // ticks = total_frames * TICKS_PER_SECOND / (num/den)
        //       = total_frames * TICKS_PER_SECOND * den / num
        Self(
            (total_frames as i128
                * TICKS_PER_SECOND as i128
                * frame_rate.den() as i128
                / frame_rate.num() as i128) as i64,
        )
    }
}

/// Conversion to/from specified frame rates.
pub trait FrameRateConversion<T> {
    fn to_frames(self, frame_rate: T) -> i64;
    fn from_frames(frames: i64, frame_rate: T) -> Self;
}

impl FrameRateConversion<FramesPerSec> for Tick {
    /// Convert ticks to frame number at the specified integer frame rate.
    fn to_frames(self, frame_rate: FramesPerSec) -> i64 {
        (self.0 as i128 * frame_rate.get() as i128 / TICKS_PER_SECOND as i128)
            as _
    }

    /// Convert frame number to ticks at the specified integer frame rate.
    fn from_frames(frames: i64, frame_rate: FramesPerSec) -> Self {
        Self(
            (frames as i128 * TICKS_PER_SECOND as i128
                / frame_rate.get() as i128) as _,
        )
    }
}

#[cfg(feature = "float_frame_rate")]
impl FrameRateConversion<FramesPerSecF32> for Tick {
    /// Convert ticks to frame number at the specified floating point frame
    /// rate.
    fn to_frames(self, frame_rate: FramesPerSecF32) -> i64 {
        (self.0 as f64 * frame_rate.get() as f64 / TICKS_PER_SECOND as f64)
            .round() as _
    }

    /// Convert frame number to ticks at the specified floating point frame
    /// rate.
    fn from_frames(frames: i64, frame_rate: FramesPerSecF32) -> Self {
        Self(
            (frames as f64 * TICKS_PER_SECOND as f64 / frame_rate.get() as f64)
                .round() as _,
        )
    }
}

#[cfg(feature = "float_frame_rate")]
impl FrameRateConversion<FramesPerSecF64> for Tick {
    /// Convert ticks to frame number at the specified floating point frame
    /// rate.
    fn to_frames(self, frame_rate: FramesPerSecF64) -> i64 {
        (self.0 as f64 * frame_rate.get() / TICKS_PER_SECOND as f64).round()
            as _
    }

    /// Convert frame number to ticks at the specified floating point frame
    /// rate.
    fn from_frames(frames: i64, frame_rate: FramesPerSecF64) -> Self {
        Self(
            (frames as f64 * TICKS_PER_SECOND as f64 / frame_rate.get()).round()
                as _,
        )
    }
}
