# `frame-tick`

<!-- cargo-rdme start -->

Fixed-point representation of time where each second is divided into
3,603,600 `Tick`s (or 25,200, if the `cargo` feature `low_res` is set).

This crate was inspired by [this article](https://iquilezles.org/articles/ticks/)
from [Inigo Quilez](https://iquilezles.org/). Please refer to this for a
more detailed explanation.
> *Note that the default for `TICKS_PER_SECOND`, 3,603,600, is the Least
> Common Multiple of all numbers in the list given in the article as well as
> 11 and 13, which are needed for NTSC.*

This makes it 'compatible' with lots of frame- and refresh rates without
ever mapping outside of or repeating a frame. That is: without strobing.

In particular, a `Tick` can represent exactly:

* 24hz and 48hz, great for movie playback.

* 6hz, 8hz and 12hz, great for animating on 4s, 3s and 2s.

* 29.97hz, 59.94hz NTSC found in Japan, South Korea and the USA.

* 30hz, 60hz, for internet video and TV in the USA.

* 25hz and 50hz, for TV in the EU.

* 72hz, for Oculus Quest 1.

* 90hz for Quest 2, Rift and other headsets.

* 120hz, 144hz and 240hz, for newer VR headesets and high frequency
  monitors.

* And many more.

## Examples

```rust
use frame_tick::{FrameRate, FrameRateConversion, Tick};

let tick = Tick::from_secs(1.0);

/// A round trip is lossless.
assert_eq!(1.0, tick.to_secs());
/// One second at 120hz == frame № 120.
assert_eq!(120, tick.to_frame(FrameRate::new(120).unwrap()));
```

## Cargo features

<!-- cargo-rdme end -->

## License

Apache-2.0 OR BSD-3-Clause OR MIT OR Zlib at your discretion.
