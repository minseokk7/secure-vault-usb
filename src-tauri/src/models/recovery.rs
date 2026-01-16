use serde::{Deserialize, Serialize};
use std::fmt;

/// 복구 키 관련 오류 타입
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecoveryError {
    /// 잘못된 복구 키 형식
    InvalidFormat(String),
    /// 복구 키 검증 실패
    VerificationFailed,
    /// 키 유도 실패
    KeyDerivationFailed(String),
    /// 암호화 오류
    CryptoError(String),
    /// 내부 오류
    InternalError(String),
}

impl fmt::Display for RecoveryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RecoveryError::InvalidFormat(msg) => write!(f, "복구 키 형식 오류: {}", msg),
            RecoveryError::VerificationFailed => write!(f, "복구 키 검증에 실패했습니다"),
            RecoveryError::KeyDerivationFailed(msg) => write!(f, "키 유도 실패: {}", msg),
            RecoveryError::CryptoError(msg) => write!(f, "암호화 오류: {}", msg),
            RecoveryError::InternalError(msg) => write!(f, "내부 오류: {}", msg),
        }
    }
}

impl std::error::Error for RecoveryError {}

/// 복구 키 정보 구조체
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryKeyInfo {
    /// Base64로 인코딩된 복구 키 (256비트)
    pub key: String,
    /// SHA-256 해시값 (저장용)
    pub hash: String,
    /// 생성 일시
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// 사용 여부 (한 번만 사용 가능)
    pub used: bool,
}

impl RecoveryKeyInfo {
    /// 새로운 복구 키 정보 생성
    pub fn new(key: String, hash: String) -> Self {
        Self {
            key,
            hash,
            created_at: chrono::Utc::now(),
            used: false,
        }
    }
}

/// 복구 키 검증 결과
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryVerificationResult {
    /// 검증 성공 여부
    pub is_valid: bool,
    /// 오류 메시지 (검증 실패 시)
    pub error_message: Option<String>,
    /// 유도된 마스터 키 (검증 성공 시)
    pub master_key: Option<Vec<u8>>,
}

impl RecoveryVerificationResult {
    /// 성공 결과 생성
    pub fn success(master_key: Vec<u8>) -> Self {
        Self {
            is_valid: true,
            error_message: None,
            master_key: Some(master_key),
        }
    }

    /// 실패 결과 생성
    pub fn failure(error_message: String) -> Self {
        Self {
            is_valid: false,
            error_message: Some(error_message),
            master_key: None,
        }
    }
}