use crate::client::StorageClient;

/// Upload service for Neon Object Storage.
#[derive(Debug, Clone)]
pub struct UploadService {
    client: StorageClient,
}

impl UploadService {
    /// Creates a new upload service.
    pub fn new(client: StorageClient) -> Self {
        Self { client }
    }

    /// Returns the bucket name.
    pub fn bucket(&self) -> &str {
        self.client.bucket()
    }
}
