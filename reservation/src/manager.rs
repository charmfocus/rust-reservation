use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{postgres::types::PgRange, types::Uuid};

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

        let timespan: PgRange<DateTime<Utc>> = (start_time..end_time).into();

        let status = abi::ReservationStatus::try_from(rsvp.status)
            .unwrap_or(abi::ReservationStatus::Pending);

        // generate a insert sql for the reservation
        // execute the sql
        let id: Uuid = sqlx::query_scalar(
                "INSERT INTO rsvp.reservations (user_id, resource_id, timespan, note, status) VALUES ($1, $2, $3, $4, $5::rsvp.reservation_status) RETURNING id")
            .bind(rsvp.user_id.clone())
            .bind(rsvp.resource_id.clone())
            .bind(timespan)
            .bind(rsvp.note.clone())
            .bind(status.to_string())
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

impl ReservationManager {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }
}

#[cfg(test)]
mod tests {
    use abi::convert_to_timestamp;
    use chrono::FixedOffset;

    use super::*;

    #[sqlx_database_tester::test(pool(variable = "migrated_pool", migrations = "../migrations"))]
    async fn reserve_should_work_for_valid_window() {
        let start: DateTime<FixedOffset> = "2024-11-15T00:00:00+08:00".parse().unwrap();
        let end: DateTime<FixedOffset> = "2024-11-16T12:00:00+08:00".parse().unwrap();
        let manager = ReservationManager::new(migrated_pool.clone());
        let rsvp = abi::Reservation {
            id: "".to_string(),
            user_id: "wiki".to_string(),
            resource_id: "ocean-view-room-713".to_string(),
            start_time: Some(convert_to_timestamp(start.with_timezone(&Utc))),
            end_time: Some(convert_to_timestamp(end.with_timezone(&Utc))),
            note: "I'll arrive at 3pm. Please help to upgrade to execuitive room if possible."
                .to_string(),
            status: abi::ReservationStatus::Pending as i32,
        };
        let rsvp = manager.reserve(rsvp).await.unwrap();
        assert_eq!(rsvp.id.len(), 36);
    }
}
