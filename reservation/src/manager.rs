use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::postgres::types::PgRange;

use crate::{error::ReservationError, ReservationId, ReservationManager, Rsvp};

#[async_trait]
impl Rsvp for ReservationManager {
    async fn reserve(
        &self,
        mut rsvp: abi::Reservation,
    ) -> Result<abi::Reservation, ReservationError> {
        if rsvp.start_time.is_none() || rsvp.end_time.is_none() {
            return Err(ReservationError::InvalidTime);
        }

        let start_time = abi::convert_to_utc_time(rsvp.start_time.unwrap());
        let end_time = abi::convert_to_utc_time(rsvp.end_time.unwrap());

        if start_time <= end_time {
            return Err(ReservationError::InvalidTime);
        }

        let timespan: PgRange<DateTime<Utc>> = (start_time..end_time).into();

        // let status

        // generate a insert sql for the reservation
        // execute the sql
        let id: i64 = sqlx::query_scalar(
                "INSERT INTO reservations (user_id, resource_id, timespan, note, status) VALUES ($1, $2, $3, $4, $5::rsvp.reservation_status) RETURNING id")
            .bind(rsvp.user_id.clone())
            .bind(rsvp.resource_id.clone())
            .bind(timespan)
            .bind(rsvp.note.clone())
            .bind(rsvp.status)
            .fetch_one(&self.pool)
            .await?;

        rsvp.id = id.to_string();
        Ok(rsvp)
    }

    async fn change_status(
        &self,
        _id: ReservationId,
    ) -> Result<abi::Reservation, ReservationError> {
        todo!()
    }

    async fn update_note(
        &self,
        _id: ReservationId,
        _note: String,
    ) -> Result<abi::Reservation, ReservationError> {
        todo!()
    }

    async fn delete(&self, _id: ReservationId) -> Result<abi::Reservation, ReservationError> {
        todo!()
    }

    async fn get(&self, _id: ReservationId) -> Result<abi::Reservation, ReservationError> {
        todo!()
    }

    async fn query(
        &self,
        _query: abi::ReservationQuery,
    ) -> Result<Vec<abi::Reservation>, ReservationError> {
        todo!()
    }
}
