use chrono::Utc;
use sqlx::PgPool;
use tracing::instrument;
use uuid::Uuid;

use domain::error::{AppError, AppResult};
use domain::ids::{BoardId, TaskId, UserId};
use domain::types::task::{Task, TaskPriority, TaskStatus};

/// Service for task-related business logic.
/// NOTE: Uses runtime queries instead of compile-time checked queries.
/// TODO: Convert to compile-time checked queries when database is available.
pub struct TaskService {
    pool: PgPool,
}

impl TaskService {
    /// Creates a new task service.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Creates a new task on a board.
    #[instrument(skip(self))]
    pub async fn create_task(
        &self,
        board_id: BoardId,
        title: &str,
        description: Option<&str>,
        priority: TaskPriority,
        created_by: UserId,
    ) -> AppResult<Task> {
        let task_id = TaskId::new();
        let _now = Utc::now();

        let task = sqlx::query_as::<_, Task>(
            r#"
            INSERT INTO tasks (id, board_id, org_id, title, description, status, priority, position, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, 0, $8)
            RETURNING id, board_id, title, description, status, priority, assignee_id, due_date, position, created_by, created_at, updated_at, deleted_at
            "#,
        )
        .bind(task_id.as_uuid())
        .bind(board_id.as_uuid())
        .bind(Uuid::nil())
        .bind(title)
        .bind(description)
        .bind(TaskStatus::Todo as domain::types::task::TaskStatus)
        .bind(priority as domain::types::task::TaskPriority)
        .bind(created_by.as_uuid())
        .fetch_one(&self.pool)
        .await?;

        Ok(task)
    }

    /// Gets a task by ID.
    #[instrument(skip(self))]
    pub async fn get_task(&self, task_id: TaskId) -> AppResult<Task> {
        let task = sqlx::query_as::<_, Task>(
            r#"
            SELECT id, board_id, title, description, status, priority, assignee_id, due_date, position, created_by, created_at, updated_at, deleted_at
            FROM tasks
            WHERE id = $1 AND deleted_at IS NULL
            "#,
        )
        .bind(task_id.as_uuid())
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| AppError::NotFound {
            resource: "Task".to_string(),
            id: task_id.to_string(),
        })?;

        Ok(task)
    }
}
