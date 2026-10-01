use async_trait::async_trait;
use sqlx::PgPool;

use domain::error::{AppError, AppResult};
use domain::ids::{BoardId, OrgId, UserId};
use domain::traits::BoardRepository;
use domain::types::board::{Board, BoardMember};

/// SQLx implementation of the BoardRepository trait.
/// NOTE: Uses runtime queries instead of compile-time checked queries.
/// TODO: Convert to compile-time checked queries when database is available.
#[derive(Clone)]
pub struct SqlxBoardRepository {
    pool: PgPool,
}

impl SqlxBoardRepository {
    /// Creates a new repository instance.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl BoardRepository for SqlxBoardRepository {
    async fn create(&self, board: &Board) -> AppResult<Board> {
        let result = sqlx::query_as::<_, Board>(
            r#"
            INSERT INTO boards (id, org_id, name, description, visibility, created_by)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, org_id, name, description, visibility, created_by, created_at, updated_at, deleted_at
            "#,
        )
        .bind(board.id.as_uuid())
        .bind(board.org_id.as_uuid())
        .bind(&board.name)
        .bind(&board.description)
        .bind(board.visibility as domain::types::board::BoardVisibility)
        .bind(board.created_by.as_uuid())
        .fetch_one(&self.pool)
        .await?;

        Ok(result)
    }

    async fn find_by_id(&self, id: BoardId, org_id: OrgId) -> AppResult<Option<Board>> {
        let result = sqlx::query_as::<_, Board>(
            r#"
            SELECT id, org_id, name, description, visibility, created_by, created_at, updated_at, deleted_at
            FROM boards
            WHERE id = $1 AND org_id = $2 AND deleted_at IS NULL
            "#,
        )
        .bind(id.as_uuid())
        .bind(org_id.as_uuid())
        .fetch_optional(&self.pool)
        .await?;

        Ok(result)
    }

    async fn update(&self, board: &Board) -> AppResult<Board> {
        let result = sqlx::query_as::<_, Board>(
            r#"
            UPDATE boards
            SET name = $3, description = $4, visibility = $5
            WHERE id = $1 AND org_id = $2 AND deleted_at IS NULL
            RETURNING id, org_id, name, description, visibility, created_by, created_at, updated_at, deleted_at
            "#,
        )
        .bind(board.id.as_uuid())
        .bind(board.org_id.as_uuid())
        .bind(&board.name)
        .bind(&board.description)
        .bind(board.visibility as domain::types::board::BoardVisibility)
        .fetch_one(&self.pool)
        .await?;

        Ok(result)
    }

    async fn soft_delete(&self, id: BoardId, org_id: OrgId) -> AppResult<()> {
        let result = sqlx::query(
            r#"
            UPDATE boards SET deleted_at = NOW() WHERE id = $1 AND org_id = $2 AND deleted_at IS NULL
            "#,
        )
        .bind(id.as_uuid())
        .bind(org_id.as_uuid())
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound {
                resource: "Board".to_string(),
                id: id.to_string(),
            });
        }

        Ok(())
    }

    async fn list_by_org(&self, org_id: OrgId, page: i64, per_page: i64) -> AppResult<Vec<Board>> {
        let offset = (page - 1) * per_page;
        let result = sqlx::query_as::<_, Board>(
            r#"
            SELECT id, org_id, name, description, visibility, created_by, created_at, updated_at, deleted_at
            FROM boards
            WHERE org_id = $1 AND deleted_at IS NULL
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(org_id.as_uuid())
        .bind(per_page)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(result)
    }

    async fn add_member(&self, member: &BoardMember) -> AppResult<BoardMember> {
        let result = sqlx::query_as::<_, BoardMember>(
            r#"
            INSERT INTO board_members (id, board_id, user_id, org_id, can_edit, joined_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, board_id, user_id, org_id, can_edit, joined_at, created_at, updated_at
            "#,
        )
        .bind(member.id)
        .bind(member.board_id.as_uuid())
        .bind(member.user_id.as_uuid())
        .bind(member.org_id.as_uuid())
        .bind(member.can_edit)
        .bind(member.joined_at)
        .fetch_one(&self.pool)
        .await?;

        Ok(result)
    }

    async fn remove_member(&self, board_id: BoardId, user_id: UserId) -> AppResult<()> {
        let result = sqlx::query(
            r#"
            DELETE FROM board_members WHERE board_id = $1 AND user_id = $2
            "#,
        )
        .bind(board_id.as_uuid())
        .bind(user_id.as_uuid())
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound {
                resource: "BoardMember".to_string(),
                id: format!("board_id={}, user_id={}", board_id, user_id),
            });
        }

        Ok(())
    }

    async fn list_members(&self, board_id: BoardId) -> AppResult<Vec<BoardMember>> {
        let result = sqlx::query_as::<_, BoardMember>(
            r#"
            SELECT id, board_id, user_id, org_id, can_edit, joined_at, created_at, updated_at
            FROM board_members
            WHERE board_id = $1
            ORDER BY joined_at ASC
            "#,
        )
        .bind(board_id.as_uuid())
        .fetch_all(&self.pool)
        .await?;

        Ok(result)
    }
}
