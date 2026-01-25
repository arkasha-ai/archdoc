//! ArchDoc Core Library
//!
//! This crate provides the core functionality for analyzing Python projects
//! and generating architecture documentation.

// Public modules
pub mod errors;
pub mod config;
pub mod model;
pub mod scanner;
pub mod python_analyzer;
pub mod renderer;
pub mod writer;
pub mod cache;

// Re-export commonly used types
pub use errors::ArchDocError;
pub use config::Config;
pub use model::ProjectModel;


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
