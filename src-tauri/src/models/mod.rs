// SecureVault 데이터 모델 모듈
// 애플리케이션에서 사용되는 모든 데이터 구조를 정의합니다.

pub mod error;
pub mod vault;
pub mod file;
pub mod folder;
// pub mod auth;
pub mod auth_simple;
pub mod encryption;
pub mod recovery;
pub mod compression;

// 모델들을 재내보내기 (모호한 재내보내기 방지)
pub use error::{VaultError, DatabaseError, CryptoError, FileError, AuthError};
pub use vault::*;
pub use file::*;
pub use folder::{FolderEntry, FolderTree, FolderStatus};
pub use auth_simple::*;
pub use encryption::*;
pub use recovery::{RecoveryError, RecoveryKeyInfo, RecoveryVerificationResult};
pub use compression::{CompressionLevel, CompressionResult};