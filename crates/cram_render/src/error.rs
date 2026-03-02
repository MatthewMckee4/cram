use thiserror::Error;

#[derive(Debug, Error)]
pub enum RenderError {
    #[error("typst compile error: {0}")]
    Compile(String),
    #[error("no pages in document")]
    NoPages,
    #[error("png encode failed")]
    Encode,
}
