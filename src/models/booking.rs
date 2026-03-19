use sqlx::SqlitePool;

#[derive(Debug, Clone, sqlx::FromRow, serde::Serialize)]
pub struct Booking {
    pub id: i64,
    pub class_schedule_id: i64,
    pub user_id: i64,
    pub status: String,
    pub booked_at: String,
    pub cancelled_at: Option<String>,
}

#[derive(Debug, Clone, sqlx::FromRow, serde::Serialize)]
pub struct BookingView {
    pub id: i64,
    pub class_name: String,
    pub starts_at: String,
    pub ends_at: String,
    pub instructor_name: Option<String>,
    pub status: String,
    pub booked_at: String,
}

#[derive(Debug, Clone, sqlx::FromRow, serde::Serialize)]
pub struct BookingWithUser {
    pub id: i64,
    pub class_schedule_id: i64,
    pub user_id: i64,
    pub user_name: String,
    pub user_email: String,
    pub status: String,
    pub booked_at: String,
    pub cancelled_at: Option<String>,
}

impl Booking {
    pub async fn create_booking(
        pool: &SqlitePool,
        class_schedule_id: i64,
        user_id: i64,
    ) -> Result<Booking, sqlx::Error> {
        sqlx::query_as::<_, Booking>(
            "INSERT INTO bookings (class_schedule_id, user_id, status, booked_at)
             SELECT ?, ?, 'confirmed', datetime('now')
             WHERE NOT EXISTS (
                 SELECT 1 FROM bookings
                 WHERE class_schedule_id = ? AND user_id = ? AND status = 'confirmed'
             )
             RETURNING id, class_schedule_id, user_id, status, booked_at, cancelled_at",
        )
        .bind(class_schedule_id)
        .bind(user_id)
        .bind(class_schedule_id)
        .bind(user_id)
        .fetch_one(pool)
        .await
    }

    pub async fn cancel_booking(pool: &SqlitePool, id: i64, user_id: i64) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE bookings
             SET status = 'cancelled', cancelled_at = datetime('now')
             WHERE id = ? AND user_id = ? AND status = 'confirmed'",
        )
        .bind(id)
        .bind(user_id)
        .execute(pool)
        .await?;
        Ok(())
    }

    pub async fn list_for_user(pool: &SqlitePool, user_id: i64) -> Result<Vec<BookingView>, sqlx::Error> {
        sqlx::query_as::<_, BookingView>(
            "SELECT
                b.id,
                cd.name AS class_name,
                cs.starts_at,
                cs.ends_at,
                u.display_name AS instructor_name,
                b.status,
                b.booked_at
             FROM bookings b
             JOIN class_schedules cs ON cs.id = b.class_schedule_id
             JOIN class_definitions cd ON cd.id = cs.class_definition_id
             LEFT JOIN users u ON u.id = cs.instructor_id
             WHERE b.user_id = ?
             ORDER BY cs.starts_at DESC",
        )
        .bind(user_id)
        .fetch_all(pool)
        .await
    }

    pub async fn list_for_schedule(pool: &SqlitePool, class_schedule_id: i64) -> Result<Vec<BookingWithUser>, sqlx::Error> {
        sqlx::query_as::<_, BookingWithUser>(
            "SELECT
                b.id,
                b.class_schedule_id,
                b.user_id,
                u.display_name AS user_name,
                u.email AS user_email,
                b.status,
                b.booked_at,
                b.cancelled_at
             FROM bookings b
             JOIN users u ON u.id = b.user_id
             WHERE b.class_schedule_id = ?
             ORDER BY b.booked_at",
        )
        .bind(class_schedule_id)
        .fetch_all(pool)
        .await
    }

    pub async fn count_for_schedule(pool: &SqlitePool, class_schedule_id: i64) -> Result<i64, sqlx::Error> {
        let row: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM bookings WHERE class_schedule_id = ? AND status = 'confirmed'",
        )
        .bind(class_schedule_id)
        .fetch_one(pool)
        .await?;
        Ok(row.0)
    }

    pub async fn is_user_booked(
        pool: &SqlitePool,
        class_schedule_id: i64,
        user_id: i64,
    ) -> Result<bool, sqlx::Error> {
        let row: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM bookings WHERE class_schedule_id = ? AND user_id = ? AND status = 'confirmed'",
        )
        .bind(class_schedule_id)
        .bind(user_id)
        .fetch_one(pool)
        .await?;
        Ok(row.0 > 0)
    }

    pub async fn find_by_user_and_schedule(
        pool: &SqlitePool,
        user_id: i64,
        class_schedule_id: i64,
    ) -> Result<Option<Booking>, sqlx::Error> {
        sqlx::query_as::<_, Booking>(
            "SELECT id, class_schedule_id, user_id, status, booked_at, cancelled_at
             FROM bookings
             WHERE user_id = ? AND class_schedule_id = ? AND status = 'confirmed'",
        )
        .bind(user_id)
        .bind(class_schedule_id)
        .fetch_optional(pool)
        .await
    }
}
