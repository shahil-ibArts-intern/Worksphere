use sqlx::{Postgres, Transaction};

/// Extension trait for transaction utilities.
#[allow(async_fn_in_trait)]
pub trait TransactionExt {
    /// Commits the transaction, logging and returning the error if it fails.
    async fn commit_or_log(self) -> Result<(), sqlx::Error>;
}

impl TransactionExt for Transaction<'_, Postgres> {
    async fn commit_or_log(self) -> Result<(), sqlx::Error> {
        self.commit().await
    }
}
