use std::{
    env,
    fs::{File, OpenOptions},
    io,
    path::{Path, PathBuf},
    sync::Mutex,
};

use thiserror::Error;
use tracing_subscriber::{EnvFilter, fmt, util::SubscriberInitExt};

pub const LOG_PATH_ENV: &str = "PADPROBE_LOG";

/// Initializes file logging when `PADPROBE_LOG` contains a destination path.
///
/// `RUST_LOG` controls filtering. When it is absent or invalid, PadProbe logs
/// its own debug-level lifecycle events.
///
/// # Errors
///
/// Returns an error if the destination cannot be opened or the global tracing
/// subscriber cannot be installed.
pub fn init() -> Result<Option<PathBuf>, LoggingError> {
    let Some(path) = env::var_os(LOG_PATH_ENV).filter(|path| !path.is_empty()) else {
        return Ok(None);
    };
    let path = PathBuf::from(path);
    let file = open_log_file(&path)?;
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("padprobe=debug"));

    fmt()
        .with_env_filter(filter)
        .with_ansi(false)
        .with_writer(Mutex::new(file))
        .finish()
        .try_init()?;

    Ok(Some(path))
}

fn open_log_file(path: &Path) -> io::Result<File> {
    OpenOptions::new().create(true).append(true).open(path)
}

#[derive(Debug, Error)]
pub enum LoggingError {
    #[error("could not open debug log: {0}")]
    Open(#[from] io::Error),
    #[error("could not initialize debug logging: {0}")]
    Initialize(#[from] tracing_subscriber::util::TryInitError),
}
