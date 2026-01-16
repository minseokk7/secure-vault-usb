// SecureVault 에러 타입 정의
// 애플리케이션에서 발생할 수 있는 모든 에러를 정의합니다.

use thiserror::Error;
use std::time::Duration;

/// 인증 관련 에러
#[derive(Error, Debug)]
pub enum AuthError {
    #[error("PIN 형식이 올바르지 않습니다. 4-8자리 숫자를 입력해주세요.")]
    InvalidPinFormat,
    
    #[error("인증에 실패했습니다. PIN을 다시 확인해주세요.")]
    AuthenticationFailed,
    
    #[error("PIN이 설정되지 않았습니다.")]
    NoPinSet,
    
    #[error("해시 생성에 실패했습니다.")]
    HashingFailed,
    
    #[error("잘못된 해시 형식입니다.")]
    InvalidHash,
    
    #[error("솔트가 올바르지 않습니다. 32바이트여야 합니다.")]
    InvalidSalt,
    
    #[error("보안을 위해 {0:?} 후 다시 시도해주세요.")]
    BruteForceProtection(Duration),
    
    #[error("복구 키가 올바르지 않습니다.")]
    InvalidRecoveryKey,
    
    #[error("세션이 만료되었습니다. 다시 로그인해주세요.")]
    SessionExpired,
}

/// 암호화 관련 에러
#[derive(Error, Debug)]
pub enum CryptoError {
    #[error("마스터 키가 설정되지 않았습니다.")]
    NoMasterKey,
    
    #[error("암호화에 실패했습니다.")]
    EncryptionFailed,
    
    #[error("복호화에 실패했습니다.")]
    DecryptionFailed,
    
    #[error("키 유도에 실패했습니다.")]
    KeyDerivationFailed,
    
    #[error("잘못된 암호화 알고리즘입니다: {0}")]
    InvalidAlgorithm(String),
    
    #[error("암호화 메타데이터가 손상되었습니다.")]
    CorruptedMetadata,
    
    #[error("메모리 보안 작업에 실패했습니다.")]
    MemorySecurityFailed,
    
    // C# 호환성을 위한 추가 오류 타입들
    #[error("{0}")]
    InvalidPin(String),
    
    #[error("{0}")]
    InvalidSalt(String),
    
    #[error("{0}")]
    InvalidData(String),
    
    #[error("{0}")]
    InvalidKey(String),
}

/// 파일 관리 관련 에러
#[derive(Error, Debug)]
pub enum FileError {
    #[error("파일을 찾을 수 없습니다: {0}")]
    FileNotFound(String),
    
    #[error("파일 읽기에 실패했습니다: {0}")]
    ReadFailed(String),
    
    #[error("파일 쓰기에 실패했습니다: {0}")]
    WriteFailed(String),
    
    #[error("파일 삭제에 실패했습니다: {0}")]
    DeleteFailed(String),
    
    #[error("파일명이 올바르지 않습니다: {0}")]
    InvalidFileName(String),
    
    #[error("파일 크기가 제한을 초과했습니다. 최대 크기: {0}GB")]
    FileSizeExceeded(u64),
    
    #[error("지원하지 않는 파일 형식입니다: {0}")]
    UnsupportedFileType(String),
    
    #[error("파일이 이미 존재합니다: {0}")]
    FileAlreadyExists(String),
    
    #[error("파일 메타데이터 처리에 실패했습니다.")]
    MetadataError,
    
    #[error("임시 파일 생성에 실패했습니다.")]
    TempFileCreationFailed,
}

/// 폴더 관리 관련 에러
#[derive(Error, Debug)]
pub enum FolderError {
    #[error("폴더를 찾을 수 없습니다: {0}")]
    FolderNotFound(String),
    
    #[error("폴더 생성에 실패했습니다: {0}")]
    CreateFailed(String),
    
    #[error("폴더 삭제에 실패했습니다: {0}")]
    DeleteFailed(String),
    
    #[error("폴더명이 올바르지 않습니다: {0}")]
    InvalidFolderName(String),
    
    #[error("폴더가 이미 존재합니다: {0}")]
    FolderAlreadyExists(String),
    
    #[error("폴더가 비어있지 않습니다: {0}")]
    FolderNotEmpty(String),
    
    #[error("순환 참조가 발생했습니다.")]
    CircularReference,
}

/// 볼트 관리 관련 에러
#[derive(Error, Debug)]
pub enum VaultError {
    #[error("볼트가 초기화되지 않았습니다.")]
    NotInitialized,
    
    #[error("볼트가 이미 초기화되어 있습니다.")]
    AlreadyInitialized,
    
    #[error("볼트 설정이 올바르지 않습니다.")]
    InvalidConfiguration,
    
    #[error("볼트 설정 파일을 찾을 수 없습니다.")]
    ConfigNotFound,
    
    #[error("볼트 설정이 손상되었습니다.")]
    CorruptedConfig,
    
    #[error("접근이 거부되었습니다.")]
    AccessDenied,
    
    #[error("볼트 잠금에 실패했습니다.")]
    LockFailed,
    
    #[error("볼트 잠금 해제에 실패했습니다.")]
    UnlockFailed,
    
    #[error("볼트 백업에 실패했습니다.")]
    BackupFailed,
    
    #[error("볼트 복원에 실패했습니다.")]
    RestoreFailed,
    
    #[error("볼트 용량이 부족합니다. 사용 가능한 공간: {0}MB")]
    InsufficientSpace(u64),
    
    #[error("파일 크기가 제한을 초과했습니다. 현재: {size}바이트, 최대: {max_size}바이트")]
    FileTooLarge { size: usize, max_size: usize },
    
    #[error("데이터베이스 오류: {0}")]
    DatabaseError(String),
}

/// 데이터베이스 관련 에러
#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("데이터베이스 연결에 실패했습니다: {0}")]
    ConnectionFailed(String),
    
    #[error("데이터베이스 쿼리 실행에 실패했습니다: {0}")]
    QueryFailed(String),
    
    #[error("데이터베이스 마이그레이션에 실패했습니다: {0}")]
    MigrationFailed(String),
    
    #[error("데이터베이스 트랜잭션에 실패했습니다: {0}")]
    TransactionFailed(String),
    
    #[error("데이터베이스 무결성 검사에 실패했습니다.")]
    IntegrityCheckFailed,
    
    #[error("데이터베이스가 잠겨있습니다.")]
    DatabaseLocked,
}

/// 압축 관련 에러
#[derive(Error, Debug)]
pub enum CompressionError {
    #[error("압축에 실패했습니다: {0}")]
    CompressionFailed(String),
    
    #[error("압축 해제에 실패했습니다: {0}")]
    DecompressionFailed(String),
    
    #[error("지원하지 않는 압축 알고리즘입니다: {0}")]
    UnsupportedAlgorithm(String),
    
    #[error("압축 레벨이 올바르지 않습니다: {0}")]
    InvalidCompressionLevel(u8),
}

/// 통합 에러 타입
/// 모든 에러를 하나의 타입으로 통합하여 처리를 단순화합니다.
#[derive(Error, Debug)]
pub enum SecureVaultError {
    #[error("인증 오류: {0}")]
    Auth(#[from] AuthError),
    
    #[error("암호화 오류: {0}")]
    Crypto(#[from] CryptoError),
    
    #[error("파일 오류: {0}")]
    File(#[from] FileError),
    
    #[error("폴더 오류: {0}")]
    Folder(#[from] FolderError),
    
    #[error("볼트 오류: {0}")]
    Vault(#[from] VaultError),
    
    #[error("데이터베이스 오류: {0}")]
    Database(#[from] DatabaseError),
    
    #[error("압축 오류: {0}")]
    Compression(#[from] CompressionError),
    
    #[error("I/O 오류: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("JSON 직렬화 오류: {0}")]
    Json(#[from] serde_json::Error),
    
    #[error("UUID 파싱 오류: {0}")]
    Uuid(#[from] uuid::Error),
    
    #[error("시간 처리 오류: {0}")]
    Time(#[from] chrono::ParseError),
    
    #[error("알 수 없는 오류: {0}")]
    Unknown(String),
}

/// 결과 타입 별칭
/// 애플리케이션 전반에서 사용할 표준 Result 타입입니다.
pub type SecureVaultResult<T> = Result<T, SecureVaultError>;

impl SecureVaultError {
    /// 사용자 친화적인 에러 메시지를 반환합니다.
    /// 
    /// 기술적인 세부사항을 숨기고 사용자가 이해하기 쉬운
    /// 메시지를 제공합니다.
    /// 
    /// # 반환값
    /// * `String` - 사용자 친화적인 에러 메시지
    pub fn user_friendly_message(&self) -> String {
        match self {
            SecureVaultError::Auth(AuthError::InvalidPinFormat) => {
                "PIN은 4-8자리 숫자로 입력해주세요.".to_string()
            }
            SecureVaultError::Auth(AuthError::AuthenticationFailed) => {
                "PIN이 올바르지 않습니다. 다시 확인해주세요.".to_string()
            }
            SecureVaultError::Auth(AuthError::BruteForceProtection(duration)) => {
                format!("보안을 위해 {}초 후 다시 시도해주세요.", duration.as_secs())
            }
            SecureVaultError::File(FileError::FileNotFound(_)) => {
                "파일을 찾을 수 없습니다.".to_string()
            }
            SecureVaultError::File(FileError::FileSizeExceeded(max_size)) => {
                format!("파일 크기가 너무 큽니다. 최대 {}GB까지 지원됩니다.", max_size)
            }
            SecureVaultError::Vault(VaultError::InsufficientSpace(available)) => {
                format!("저장 공간이 부족합니다. 사용 가능한 공간: {}MB", available)
            }
            _ => "작업을 완료할 수 없습니다. 잠시 후 다시 시도해주세요.".to_string()
        }
    }
    
    /// 에러의 심각도를 반환합니다.
    /// 
    /// # 반환값
    /// * `ErrorSeverity` - 에러의 심각도 레벨
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            SecureVaultError::Auth(AuthError::BruteForceProtection(_)) => ErrorSeverity::Warning,
            SecureVaultError::Crypto(_) => ErrorSeverity::Critical,
            SecureVaultError::Database(DatabaseError::IntegrityCheckFailed) => ErrorSeverity::Critical,
            SecureVaultError::Vault(VaultError::CorruptedConfig) => ErrorSeverity::Critical,
            SecureVaultError::File(FileError::FileNotFound(_)) => ErrorSeverity::Info,
            _ => ErrorSeverity::Error,
        }
    }
}

/// 에러 심각도 레벨
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorSeverity {
    /// 정보성 메시지 (파일 없음 등)
    Info,
    /// 경고 (브루트포스 방지 등)
    Warning,
    /// 일반 에러 (파일 읽기 실패 등)
    Error,
    /// 심각한 에러 (암호화 실패, 데이터 손상 등)
    Critical,
}
// VaultError에 대한 From trait 구현들
// 다른 에러 타입들을 VaultError로 변환할 수 있도록 합니다.

impl From<AuthError> for VaultError {
    fn from(err: AuthError) -> Self {
        VaultError::DatabaseError(format!("인증 오류: {}", err))
    }
}

impl From<CryptoError> for VaultError {
    fn from(err: CryptoError) -> Self {
        VaultError::DatabaseError(format!("암호화 오류: {}", err))
    }
}

impl From<FileError> for VaultError {
    fn from(err: FileError) -> Self {
        VaultError::DatabaseError(format!("파일 오류: {}", err))
    }
}

impl From<FolderError> for VaultError {
    fn from(err: FolderError) -> Self {
        VaultError::DatabaseError(format!("폴더 오류: {}", err))
    }
}

impl From<DatabaseError> for VaultError {
    fn from(err: DatabaseError) -> Self {
        VaultError::DatabaseError(format!("데이터베이스 오류: {}", err))
    }
}

impl From<CompressionError> for VaultError {
    fn from(err: CompressionError) -> Self {
        VaultError::DatabaseError(format!("압축 오류: {}", err))
    }
}

impl From<std::io::Error> for VaultError {
    fn from(err: std::io::Error) -> Self {
        VaultError::DatabaseError(format!("I/O 오류: {}", err))
    }
}

impl From<serde_json::Error> for VaultError {
    fn from(err: serde_json::Error) -> Self {
        VaultError::DatabaseError(format!("JSON 직렬화 오류: {}", err))
    }
}

impl From<uuid::Error> for VaultError {
    fn from(err: uuid::Error) -> Self {
        VaultError::DatabaseError(format!("UUID 파싱 오류: {}", err))
    }
}

impl From<chrono::ParseError> for VaultError {
    fn from(err: chrono::ParseError) -> Self {
        VaultError::DatabaseError(format!("시간 처리 오류: {}", err))
    }
}

// VaultError에 user_friendly_message 메서드 추가
impl VaultError {
    /// 사용자 친화적인 에러 메시지를 반환합니다.
    /// 
    /// 기술적인 세부사항을 숨기고 사용자가 이해하기 쉬운
    /// 메시지를 제공합니다.
    /// 
    /// # 반환값
    /// * `String` - 사용자 친화적인 에러 메시지
    pub fn user_friendly_message(&self) -> String {
        match self {
            VaultError::NotInitialized => {
                "볼트가 초기화되지 않았습니다. 먼저 볼트를 설정해주세요.".to_string()
            }
            VaultError::AlreadyInitialized => {
                "볼트가 이미 초기화되어 있습니다.".to_string()
            }
            VaultError::InvalidConfiguration => {
                "볼트 설정이 올바르지 않습니다. 설정을 확인해주세요.".to_string()
            }
            VaultError::CorruptedConfig => {
                "볼트 설정 파일이 손상되었습니다. 복구가 필요합니다.".to_string()
            }
            VaultError::AccessDenied => {
                "접근이 거부되었습니다. 권한을 확인해주세요.".to_string()
            }
            VaultError::LockFailed => {
                "볼트 잠금에 실패했습니다.".to_string()
            }
            VaultError::UnlockFailed => {
                "볼트 잠금 해제에 실패했습니다.".to_string()
            }
            VaultError::BackupFailed => {
                "볼트 백업에 실패했습니다.".to_string()
            }
            VaultError::RestoreFailed => {
                "볼트 복원에 실패했습니다.".to_string()
            }
            VaultError::InsufficientSpace(available) => {
                format!("저장 공간이 부족합니다. 사용 가능한 공간: {}MB", available)
            }
            VaultError::FileTooLarge { size, max_size } => {
                format!("파일 크기가 너무 큽니다. 현재: {}MB, 최대: {}MB", 
                       size / (1024 * 1024), max_size / (1024 * 1024))
            }
            VaultError::DatabaseError(msg) => {
                format!("데이터베이스 오류가 발생했습니다: {}", msg)
            }
            VaultError::ConfigNotFound => {
                "볼트 설정 파일을 찾을 수 없습니다. 볼트를 다시 초기화해주세요.".to_string()
            }
        }
    }
}