use crate::client::StorageClient;

/// Download service for Neon Object Storage.
#[derive(Debug, Clone)]
pub struct DownloadService {
    client: StorageClient,
}

impl DownloadService {
    /// Creates a new download service.
    pub fn new(client: StorageClient) -> Self {
        Self { client }
    }

    /// Returns the bucket name.
    pub fn bucket(&self) -> &str {
        self.client.bucket()
    }
}
