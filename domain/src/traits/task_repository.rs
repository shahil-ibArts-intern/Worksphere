use async_trait::async_trait;

use crate::error::AppResult;
use crate::ids::{BoardId, TaskId, UserId};
use crate::types::task::{Task, TaskAssignee, TaskComment};

/// Repository trait for task data access.
#[async_trait]
pub trait TaskRepository: Send + Sync {
    /// Creates a new task.
    async fn create(&self, task: &Task) -> AppResult<Task>;

    /// Finds a task by ID.
    async fn find_by_id(&self, id: TaskId) -> AppResult<Option<Task>>;

    /// Updates a task.
    async fn update(&self, task: &Task) -> AppResult<Task>;

    /// Soft-deletes a task.
    async fn soft_delete(&self, id: TaskId) -> AppResult<()>;

    /// Lists tasks on a board.
    async fn list_by_board(
        &self,
        board_id: BoardId,
        page: i64,
        per_page: i64,
    ) -> AppResult<Vec<Task>>;

    /// Lists tasks assigned to a user.
    async fn list_by_assignee(
        &self,
        assignee_id: UserId,
        page: i64,
        per_page: i64,
    ) -> AppResult<Vec<Task>>;

    /// Assigns a task to a user.
    async fn assign(&self, assignee: &TaskAssignee) -> AppResult<TaskAssignee>;

    /// Unassigns a user from a task.
    async fn unassign(&self, task_id: TaskId, user_id: UserId) -> AppResult<()>;

    /// Lists assignees of a task.
    async fn list_assignees(&self, task_id: TaskId) -> AppResult<Vec<TaskAssignee>>;

    /// Adds a comment to a task.
    async fn add_comment(&self, comment: &TaskComment) -> AppResult<TaskComment>;

    /// Lists comments on a task.
    async fn list_comments(
        &self,
        task_id: TaskId,
        page: i64,
        per_page: i64,
    ) -> AppResult<Vec<TaskComment>>;
}
