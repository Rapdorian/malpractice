use thiserror::Error;

#[derive(Error, Debug)]
pub enum GrrError {
    #[error("Window has not yet been created")]
    WindowNotInitialized,
}
