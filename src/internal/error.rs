use thiserror::Error;
use std::io; // Import std::io

/// Unified error type for the Tonitru library.
#[derive(Error, Debug)]
pub enum Error {
    /// Placeholder error
    #[error("Placeholder Error: {0}")]
    PlaceholderError(String),

    /// Error related to data encoding/decoding.
    #[error("Codec Error: {0}")]
    CodecError(String),

    /// Error related to the network protocol (QUIC).
    #[error("Protocol Error: {0}")]
    ProtocolError(String),

    /// Error related to compression/decompression.
    #[error("Compression Error: {0}")]
    CompressionError(String),

    /// Error related to encryption/decryption.
    #[error("Encryption Error: {0}")]
    EncryptionError(String),

    /// Error related to schema management or validation.
    #[error("Schema Error: {0}")]
    SchemaError(String),

    /// Error related to predicate evaluation.
    #[error("Predicate Error: {0}")]
    PredicateError(String),

    /// Error related to indexing.
    #[error("Index Error: {0}")]
    IndexError(String),

    /// Error related to WASM execution.
    #[error("Wasm Error: {0}")]
    WasmError(String),

    /// Error related to internal utilities or distributed components.
    #[error("Internal Error: {0}")]
    InternalError(String),

    // TODO: Add more specific error types as modules are implemented
}

/// A specialized `Result` type for Tonitru operations.
pub type Result<T> = std::result::Result<T, Error>;

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        // Convert std::io::Error to a CodecError, as byteorder errors are codec-related
        Error::CodecError(format!("IO Error during codec operation: {}", err))
    }
}

/*
异常 (Panic) 处理策略:

对于可恢复的错误，我们使用 Result<T, Error> 进行处理和传播。
对于不可恢复的错误，例如编程错误、断言失败、资源耗尽等，我们允许 panic 发生。
在关键边界（如 FFI 接口，如果未来需要）或顶层应用中，可以考虑捕获 panic，进行清理或记录日志，防止整个程序崩溃。
在库内部，通常不捕获 panic，而是让其传播，这符合 Rust 的惯用法，表示一个不可恢复的状态。
*/