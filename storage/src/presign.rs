use crate::client::StorageClient;

/// Pre-signed URL generation service for Neon Object Storage.
#[derive(Debug, Clone)]
pub struct PresignService {
    client: StorageClient,
}

impl PresignService {
    /// Creates a new presign service.
    pub fn new(client: StorageClient) -> Self {
        Self { client }
    }

    /// Returns the bucket name.
    pub fn bucket(&self) -> &str {
        self.client.bucket()
    }
}
