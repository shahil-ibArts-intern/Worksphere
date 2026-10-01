use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use domain::error::{AppError, AppResult};
use domain::ids::{BoardId, TaskId, UserId};
use domain::traits::TaskRepository;
use domain::types::task::{Task, TaskAssignee, TaskComment};

/// SQLx implementation of the TaskRepository trait.
/// NOTE: Uses runtime queries instead of compile-time checked queries.
/// TODO: Convert to compile-time checked queries when database is available.
#[derive(Clone)]
pub struct SqlxTaskRepository {
    pool: PgPool,
}

impl SqlxTaskRepository {
    /// Creates a new repository instance.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TaskRepository for SqlxTaskRepository {
    async fn create(&self, task: &Task) -> AppResult<Task> {
        let result = sqlx::query_as::<_, Task>(
            r#"
            INSERT INTO tasks (id, board_id, org_id, title, description, status, priority, assignee_id, due_date, position, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            RETURNING id, board_id, title, description, status, priority, assignee_id, due_date, position, created_by, created_at, updated_at, deleted_at
            "#,
        )
        .bind(task.id.as_uuid())
        .bind(task.board_id.as_uuid())
        .bind(Uuid::nil()) // org_id - should be derived from board
        .bind(&task.title)
        .bind(&task.description)
        .bind(task.status as domain::types::task::TaskStatus)
        .bind(task.priority as domain::types::task::TaskPriority)
        .bind(task.assignee_id.map(|id| id.as_uuid()))
        .bind(task.due_date)
        .bind(task.position)
        .bind(task.created_by.as_uuid())
        .fetch_one(&self.pool)
        .await?;

        Ok(result)
    }

    async fn find_by_id(&self, id: TaskId) -> AppResult<Option<Task>> {
        let result = sqlx::query_as::<_, Task>(
            r#"
            SELECT id, board_id, title, description, status, priority, assignee_id, due_date, position, created_by, created_at, updated_at, deleted_at
            FROM tasks
            WHERE id = $1 AND deleted_at IS NULL
            "#,
        )
        .bind(id.as_uuid())
        .fetch_optional(&self.pool)
        .await?;

        Ok(result)
    }

    async fn update(&self, task: &Task) -> AppResult<Task> {
        let result = sqlx::query_as::<_, Task>(
            r#"
            UPDATE tasks
            SET title = $2, description = $3, status = $4, priority = $5, assignee_id = $6, due_date = $7, position = $8
            WHERE id = $1 AND deleted_at IS NULL
            RETURNING id, board_id, title, description, status, priority, assignee_id, due_date, position, created_by, created_at, updated_at, deleted_at
            "#,
        )
        .bind(task.id.as_uuid())
        .bind(&task.title)
        .bind(&task.description)
        .bind(task.status as domain::types::task::TaskStatus)
        .bind(task.priority as domain::types::task::TaskPriority)
        .bind(task.assignee_id.map(|id| id.as_uuid()))
        .bind(task.due_date)
        .bind(task.position)
        .fetch_one(&self.pool)
        .await?;

        Ok(result)
    }

    async fn soft_delete(&self, id: TaskId) -> AppResult<()> {
        let result = sqlx::query(
            r#"
            UPDATE tasks SET deleted_at = NOW() WHERE id = $1 AND deleted_at IS NULL
            "#,
        )
        .bind(id.as_uuid())
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound {
                resource: "Task".to_string(),
                id: id.to_string(),
            });
        }

        Ok(())
    }

    async fn list_by_board(
        &self,
        board_id: BoardId,
        page: i64,
        per_page: i64,
    ) -> AppResult<Vec<Task>> {
        let offset = (page - 1) * per_page;
        let result = sqlx::query_as::<_, Task>(
            r#"
            SELECT id, board_id, title, description, status, priority, assignee_id, due_date, position, created_by, created_at, updated_at, deleted_at
            FROM tasks
            WHERE board_id = $1 AND deleted_at IS NULL
            ORDER BY position ASC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(board_id.as_uuid())
        .bind(per_page)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(result)
    }

    async fn list_by_assignee(
        &self,
        assignee_id: UserId,
        page: i64,
        per_page: i64,
    ) -> AppResult<Vec<Task>> {
        let offset = (page - 1) * per_page;
        let result = sqlx::query_as::<_, Task>(
            r#"
            SELECT id, board_id, title, description, status, priority, assignee_id, due_date, position, created_by, created_at, updated_at, deleted_at
            FROM tasks
            WHERE assignee_id = $1 AND deleted_at IS NULL
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(assignee_id.as_uuid())
        .bind(per_page)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(result)
    }

    async fn assign(&self, assignee: &TaskAssignee) -> AppResult<TaskAssignee> {
        let result = sqlx::query_as::<_, TaskAssignee>(
            r#"
            INSERT INTO task_assignees (id, task_id, user_id, assigned_at, assigned_by)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, task_id, user_id, assigned_at, assigned_by
            "#,
        )
        .bind(assignee.id)
        .bind(assignee.task_id.as_uuid())
        .bind(assignee.user_id.as_uuid())
        .bind(assignee.assigned_at)
        .bind(assignee.assigned_by.as_uuid())
        .fetch_one(&self.pool)
        .await?;

        Ok(result)
    }

    async fn unassign(&self, task_id: TaskId, user_id: UserId) -> AppResult<()> {
        let result = sqlx::query(
            r#"
            DELETE FROM task_assignees WHERE task_id = $1 AND user_id = $2
            "#,
        )
        .bind(task_id.as_uuid())
        .bind(user_id.as_uuid())
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound {
                resource: "TaskAssignee".to_string(),
                id: format!("task_id={}, user_id={}", task_id, user_id),
            });
        }

        Ok(())
    }

    async fn list_assignees(&self, task_id: TaskId) -> AppResult<Vec<TaskAssignee>> {
        let result = sqlx::query_as::<_, TaskAssignee>(
            r#"
            SELECT id, task_id, user_id, assigned_at, assigned_by
            FROM task_assignees
            WHERE task_id = $1
            ORDER BY assigned_at ASC
            "#,
        )
        .bind(task_id.as_uuid())
        .fetch_all(&self.pool)
        .await?;

        Ok(result)
    }

    async fn add_comment(&self, _comment: &TaskComment) -> AppResult<TaskComment> {
        // Task comments table not in initial migration - stub implementation
        Err(AppError::Internal(
            "Task comments not yet implemented".to_string(),
        ))
    }

    async fn list_comments(
        &self,
        _task_id: TaskId,
        _page: i64,
        _per_page: i64,
    ) -> AppResult<Vec<TaskComment>> {
        // Task comments table not in initial migration - stub implementation
        Ok(vec![])
    }
}
