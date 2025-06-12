use crate::Tick;
use std::{
    fmt::{Display, Error, Formatter},
    time::Duration,
};

impl Display for Tick {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "Tick({})", self.0)
    }
}

impl From<Duration> for Tick {
    fn from(duration: Duration) -> Self {
        let secs = duration.as_secs_f64();
        Self::from_secs(secs)
    }
}

impl From<Tick> for Duration {
    fn from(tick: Tick) -> Self {
        Duration::from_secs_f64(tick.to_secs())
    }
}
