/// Whiteboard error types stub for Wave 1.
#[derive(Debug)]
pub struct WhiteboardError;

impl std::fmt::Display for WhiteboardError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Whiteboard error")
    }
}

impl std::error::Error for WhiteboardError {}
