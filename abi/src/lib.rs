mod pb;

use chrono::{DateTime, Utc};
pub use pb::*;
use prost_types::Timestamp;

pub fn convert_to_utc_time(ts: Timestamp) -> DateTime<Utc> {
    DateTime::<Utc>::from_naive_utc_and_offset(
        DateTime::<Utc>::from_timestamp(ts.seconds, ts.nanos.try_into().unwrap())
            .unwrap()
            .naive_utc(),
        Utc,
    )
}
