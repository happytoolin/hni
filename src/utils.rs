#[derive(Debug, thiserror::Error)]
pub enum NiError {
    #[error("Package manager detection failed")]
    DetectionFailure,
    
    #[error("Unsupported command for {agent}: {command}")]
    UnsupportedCommand {
        agent: String,
        command: String,
    },
    
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}
