use async_trait::async_trait;

use crate::error::AppResult;
use crate::ids::{BoardId, OrgId, UserId};
use crate::types::board::{Board, BoardMember};

/// Repository trait for board data access.
#[async_trait]
pub trait BoardRepository: Send + Sync {
    /// Creates a new board.
    async fn create(&self, board: &Board) -> AppResult<Board>;

    /// Finds a board by ID.
    async fn find_by_id(&self, id: BoardId, org_id: OrgId) -> AppResult<Option<Board>>;

    /// Updates a board.
    async fn update(&self, board: &Board) -> AppResult<Board>;

    /// Soft-deletes a board.
    async fn soft_delete(&self, id: BoardId, org_id: OrgId) -> AppResult<()>;

    /// Lists boards in an organization.
    async fn list_by_org(&self, org_id: OrgId, page: i64, per_page: i64) -> AppResult<Vec<Board>>;

    /// Adds a member to a board.
    async fn add_member(&self, member: &BoardMember) -> AppResult<BoardMember>;

    /// Removes a member from a board.
    async fn remove_member(&self, board_id: BoardId, user_id: UserId) -> AppResult<()>;

    /// Lists members of a board.
    async fn list_members(&self, board_id: BoardId) -> AppResult<Vec<BoardMember>>;
}
