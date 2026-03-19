use sqlx::SqlitePool;

#[derive(Debug, Clone, sqlx::FromRow, serde::Serialize)]
pub struct ClassDefinition {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub capacity: i64,
    pub duration_minutes: i64,
    pub is_active: bool,
    pub created_by: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, sqlx::FromRow, serde::Serialize)]
pub struct ClassSchedule {
    pub id: i64,
    pub class_definition_id: i64,
    pub instructor_id: Option<i64>,
    pub starts_at: String,
    pub ends_at: String,
    pub is_cancelled: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, sqlx::FromRow, serde::Serialize)]
pub struct ScheduleView {
    pub id: i64,
    pub class_name: String,
    pub description: String,
    pub instructor_name: Option<String>,
    pub starts_at: String,
    pub ends_at: String,
    pub capacity: i64,
    pub is_cancelled: bool,
    pub booked_count: i64,
}

impl ClassDefinition {
    pub async fn list_active(pool: &SqlitePool) -> Result<Vec<ClassDefinition>, sqlx::Error> {
        sqlx::query_as::<_, ClassDefinition>(
            "SELECT id, name, description, capacity, duration_minutes, is_active, created_by, created_at, updated_at
             FROM class_definitions
             WHERE is_active = 1
             ORDER BY name",
        )
        .fetch_all(pool)
        .await
    }

    pub async fn list_all(pool: &SqlitePool) -> Result<Vec<ClassDefinition>, sqlx::Error> {
        sqlx::query_as::<_, ClassDefinition>(
            "SELECT id, name, description, capacity, duration_minutes, is_active, created_by, created_at, updated_at
             FROM class_definitions
             ORDER BY name",
        )
        .fetch_all(pool)
        .await
    }

    pub async fn find_by_id(pool: &SqlitePool, id: i64) -> Result<Option<ClassDefinition>, sqlx::Error> {
        sqlx::query_as::<_, ClassDefinition>(
            "SELECT id, name, description, capacity, duration_minutes, is_active, created_by, created_at, updated_at
             FROM class_definitions
             WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(pool)
        .await
    }

    pub async fn create(
        pool: &SqlitePool,
        name: &str,
        description: &str,
        capacity: i64,
        duration_minutes: i64,
        created_by: i64,
    ) -> Result<ClassDefinition, sqlx::Error> {
        sqlx::query_as::<_, ClassDefinition>(
            "INSERT INTO class_definitions (name, description, capacity, duration_minutes, is_active, created_by, created_at, updated_at)
             VALUES (?, ?, ?, ?, 1, ?, datetime('now'), datetime('now'))
             RETURNING id, name, description, capacity, duration_minutes, is_active, created_by, created_at, updated_at",
        )
        .bind(name)
        .bind(description)
        .bind(capacity)
        .bind(duration_minutes)
        .bind(created_by)
        .fetch_one(pool)
        .await
    }

    pub async fn update(
        pool: &SqlitePool,
        id: i64,
        name: &str,
        description: &str,
        capacity: i64,
        duration_minutes: i64,
    ) -> Result<ClassDefinition, sqlx::Error> {
        sqlx::query_as::<_, ClassDefinition>(
            "UPDATE class_definitions
             SET name = ?, description = ?, capacity = ?, duration_minutes = ?, updated_at = datetime('now')
             WHERE id = ?
             RETURNING id, name, description, capacity, duration_minutes, is_active, created_by, created_at, updated_at",
        )
        .bind(name)
        .bind(description)
        .bind(capacity)
        .bind(duration_minutes)
        .bind(id)
        .fetch_one(pool)
        .await
    }

    pub async fn delete(pool: &SqlitePool, id: i64) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE class_definitions SET is_active = 0, updated_at = datetime('now') WHERE id = ?",
        )
        .bind(id)
        .execute(pool)
        .await?;
        Ok(())
    }
}

impl ClassSchedule {
    pub async fn list_upcoming(pool: &SqlitePool) -> Result<Vec<ScheduleView>, sqlx::Error> {
        sqlx::query_as::<_, ScheduleView>(
            "SELECT
                cs.id,
                cd.name AS class_name,
                cd.description,
                u.display_name AS instructor_name,
                cs.starts_at,
                cs.ends_at,
                cd.capacity,
                cs.is_cancelled,
                COALESCE((SELECT COUNT(*) FROM bookings b WHERE b.class_schedule_id = cs.id AND b.status = 'confirmed'), 0) AS booked_count
             FROM class_schedules cs
             JOIN class_definitions cd ON cd.id = cs.class_definition_id
             LEFT JOIN users u ON u.id = cs.instructor_id
             WHERE cs.starts_at >= datetime('now')
             ORDER BY cs.starts_at",
        )
        .fetch_all(pool)
        .await
    }

    pub async fn find_by_id(pool: &SqlitePool, id: i64) -> Result<Option<ClassSchedule>, sqlx::Error> {
        sqlx::query_as::<_, ClassSchedule>(
            "SELECT id, class_definition_id, instructor_id, starts_at, ends_at, is_cancelled, created_at, updated_at
             FROM class_schedules
             WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(pool)
        .await
    }

    pub async fn create(
        pool: &SqlitePool,
        class_definition_id: i64,
        instructor_id: Option<i64>,
        starts_at: &str,
        ends_at: &str,
    ) -> Result<ClassSchedule, sqlx::Error> {
        sqlx::query_as::<_, ClassSchedule>(
            "INSERT INTO class_schedules (class_definition_id, instructor_id, starts_at, ends_at, is_cancelled, created_at, updated_at)
             VALUES (?, ?, ?, ?, 0, datetime('now'), datetime('now'))
             RETURNING id, class_definition_id, instructor_id, starts_at, ends_at, is_cancelled, created_at, updated_at",
        )
        .bind(class_definition_id)
        .bind(instructor_id)
        .bind(starts_at)
        .bind(ends_at)
        .fetch_one(pool)
        .await
    }

    pub async fn cancel(pool: &SqlitePool, id: i64) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE class_schedules SET is_cancelled = 1, updated_at = datetime('now') WHERE id = ?",
        )
        .bind(id)
        .execute(pool)
        .await?;
        Ok(())
    }

    pub async fn list_for_class(pool: &SqlitePool, class_definition_id: i64) -> Result<Vec<ClassSchedule>, sqlx::Error> {
        sqlx::query_as::<_, ClassSchedule>(
            "SELECT id, class_definition_id, instructor_id, starts_at, ends_at, is_cancelled, created_at, updated_at
             FROM class_schedules
             WHERE class_definition_id = ?
             ORDER BY starts_at",
        )
        .bind(class_definition_id)
        .fetch_all(pool)
        .await
    }
}
