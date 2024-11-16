mod pb;

use std::fmt;

use chrono::{DateTime, Utc};
use prost_types::Timestamp;

pub use pb::*;

pub fn convert_to_utc_time(ts: Timestamp) -> DateTime<Utc> {
    DateTime::<Utc>::from_naive_utc_and_offset(
        DateTime::<Utc>::from_timestamp(ts.seconds, ts.nanos.try_into().unwrap())
            .unwrap()
            .naive_utc(),
        Utc,
    )
}

pub fn convert_to_timestamp(dt: DateTime<Utc>) -> Timestamp {
    Timestamp {
        seconds: dt.timestamp(),
        nanos: dt.timestamp_subsec_nanos().try_into().unwrap(),
    }
}

impl fmt::Display for ReservationStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                ReservationStatus::Pending => "pending",
                ReservationStatus::Blocked => "blocked",
                ReservationStatus::Confirmed => "confirmed",
                ReservationStatus::Unknown => "unknown",
            }
        )
    }
}
