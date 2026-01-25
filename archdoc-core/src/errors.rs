use thiserror::Error;

#[derive(Error, Debug)]
pub enum ArchDocError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Parse error in {file}:{line}: {message}")]
    ParseError {
        file: String,
        line: usize,
        message: String,
    },
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    #[error("Analysis error: {0}")]
    AnalysisError(String),
    
    #[error("Rendering error: {0}")]
    RenderingError(String),
    
    #[error("File consistency check failed: {0}")]
    ConsistencyError(String),
}