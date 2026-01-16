use crate::models::recovery::{RecoveryError, RecoveryKeyInfo, RecoveryVerificationResult};
use base64::{Engine as _, engine::general_purpose};
use rand::RngCore;
use sha2::{Digest, Sha256};
use std::sync::Mutex;

/// 복구 키 서비스
/// C# SecurityService의 복구 키 기능을 완전히 포팅
/// 256비트 Base64 복구 키 생성, 검증, 키 유도 기능 제공
#[derive(Debug)]
pub struct RecoveryService {
    /// 현재 복구 키 정보 (메모리에만 저장)
    current_recovery_key: Mutex<Option<RecoveryKeyInfo>>,
}

impl RecoveryService {
    /// 새로운 복구 키 서비스 생성
    pub fn new() -> Self {
        Self {
            current_recovery_key: Mutex::new(None),
        }
    }

    /// 복구 키 생성 (C# SecurityService와 동일한 256비트 Base64 키)
    /// 
    /// # 반환값
    /// * `Ok(String)` - Base64로 인코딩된 256비트 복구 키
    /// * `Err(RecoveryError)` - 키 생성 실패
    pub fn generate_recovery_key(&self) -> Result<String, RecoveryError> {
        // 256비트(32바이트) 랜덤 키 생성
        let mut key_bytes = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut key_bytes);

        // Base64로 인코딩 (C# Convert.ToBase64String과 동일)
        let recovery_key = general_purpose::STANDARD.encode(&key_bytes);

        // 복구 키 해시 생성 (저장용)
        let hash = self.hash_recovery_key_internal(&recovery_key)?;

        // 복구 키 정보 저장
        let recovery_info = RecoveryKeyInfo::new(recovery_key.clone(), hash);
        *self.current_recovery_key.lock().unwrap() = Some(recovery_info);

        // 메모리에서 원본 키 바이트 삭제 (보안)
        let mut key_bytes_mut = key_bytes;
        key_bytes_mut.fill(0);

        Ok(recovery_key)
    }

    /// 복구 키를 SHA-256으로 해시화 (C# HashRecoveryKey 포팅)
    /// 
    /// # 매개변수
    /// * `recovery_key` - Base64 형식의 복구 키
    /// 
    /// # 반환값
    /// * `Ok(String)` - Base64로 인코딩된 SHA-256 해시값
    /// * `Err(RecoveryError)` - 해시 생성 실패
    pub fn hash_recovery_key(&self, recovery_key: &str) -> Result<String, RecoveryError> {
        self.hash_recovery_key_internal(recovery_key)
    }

    /// 내부 해시 함수 (C# SecurityService.HashRecoveryKey와 동일한 로직)
    fn hash_recovery_key_internal(&self, recovery_key: &str) -> Result<String, RecoveryError> {
        if recovery_key.is_empty() {
            return Err(RecoveryError::InvalidFormat("복구 키가 비어있습니다.".to_string()));
        }

        // Base64 디코딩 (C# Convert.FromBase64String과 동일)
        let key_bytes = general_purpose::STANDARD
            .decode(recovery_key)
            .map_err(|_| RecoveryError::InvalidFormat("올바르지 않은 Base64 형식의 복구 키입니다.".to_string()))?;

        // 32바이트(256비트) 검증
        if key_bytes.len() != 32 {
            return Err(RecoveryError::InvalidFormat("복구 키는 32바이트(256비트)여야 합니다.".to_string()));
        }

        // SHA-256 해시 계산 (C# SHA256.ComputeHash와 동일)
        let mut hasher = Sha256::new();
        hasher.update(&key_bytes);
        let hash = hasher.finalize();

        // Base64로 인코딩하여 반환
        Ok(general_purpose::STANDARD.encode(&hash))
    }

    /// 복구 키 검증 (C# VerifyRecoveryKey 포팅)
    /// 
    /// # 매개변수
    /// * `input_recovery_key` - 입력된 복구 키 (Base64 문자열)
    /// * `stored_hash` - 저장된 복구 키 해시값
    /// 
    /// # 반환값
    /// * `Ok(bool)` - 검증 결과 (true: 일치, false: 불일치)
    /// * `Err(RecoveryError)` - 검증 과정에서 오류 발생
    pub fn verify_recovery_key(&self, input_recovery_key: &str, stored_hash: &str) -> Result<bool, RecoveryError> {
        if input_recovery_key.is_empty() || stored_hash.is_empty() {
            return Ok(false);
        }

        // 입력된 복구 키의 해시 계산
        let input_hash = match self.hash_recovery_key_internal(input_recovery_key) {
            Ok(hash) => hash,
            Err(_) => return Ok(false), // 형식 오류 시 false 반환 (C# 버전과 동일)
        };

        // 해시 비교
        Ok(input_hash == stored_hash)
    }

    /// 복구 키로부터 마스터 키 유도 (C# DeriveKeyFromRecoveryKey 포팅)
    /// 
    /// # 매개변수
    /// * `recovery_key` - 복구 키 (Base64 문자열)
    /// * `salt` - 32바이트 솔트
    /// * `iterations` - PBKDF2 반복 횟수 (기본값: 100,000)
    /// 
    /// # 반환값
    /// * `Ok([u8; 32])` - 32바이트 마스터 키
    /// * `Err(RecoveryError)` - 키 유도 실패
    pub fn derive_key_from_recovery_key(
        &self,
        recovery_key: &str,
        salt: &[u8],
        iterations: Option<u32>,
    ) -> Result<[u8; 32], RecoveryError> {
        if recovery_key.is_empty() {
            return Err(RecoveryError::InvalidFormat("복구 키가 비어있습니다.".to_string()));
        }

        if salt.len() != 32 {
            return Err(RecoveryError::InvalidFormat("솔트는 32바이트여야 합니다.".to_string()));
        }

        // Base64 디코딩
        let key_bytes = general_purpose::STANDARD
            .decode(recovery_key)
            .map_err(|_| RecoveryError::InvalidFormat("올바르지 않은 Base64 형식의 복구 키입니다.".to_string()))?;

        if key_bytes.len() != 32 {
            return Err(RecoveryError::InvalidFormat("복구 키는 32바이트(256비트)여야 합니다.".to_string()));
        }

        // PBKDF2-HMAC-SHA256으로 키 유도 (C# Rfc2898DeriveBytes와 동일)
        let iterations = iterations.unwrap_or(100_000);
        
        use pbkdf2::pbkdf2_hmac;
        let mut master_key = [0u8; 32];
        pbkdf2_hmac::<sha2::Sha256>(&key_bytes, salt, iterations, &mut master_key);

        Ok(master_key)
    }

    /// 복구 키 검증 및 마스터 키 유도 (통합 함수)
    /// 
    /// # 매개변수
    /// * `input_recovery_key` - 입력된 복구 키
    /// * `stored_hash` - 저장된 복구 키 해시
    /// * `salt` - 솔트
    /// * `iterations` - PBKDF2 반복 횟수
    /// 
    /// # 반환값
    /// * `RecoveryVerificationResult` - 검증 결과 및 마스터 키
    pub fn verify_and_derive_key(
        &self,
        input_recovery_key: &str,
        stored_hash: &str,
        salt: &[u8],
        iterations: Option<u32>,
    ) -> RecoveryVerificationResult {
        // 복구 키 검증
        match self.verify_recovery_key(input_recovery_key, stored_hash) {
            Ok(true) => {
                // 검증 성공 시 마스터 키 유도
                match self.derive_key_from_recovery_key(input_recovery_key, salt, iterations) {
                    Ok(master_key) => RecoveryVerificationResult::success(master_key.to_vec()),
                    Err(e) => RecoveryVerificationResult::failure(format!("키 유도 실패: {}", e)),
                }
            }
            Ok(false) => RecoveryVerificationResult::failure("복구 키가 일치하지 않습니다.".to_string()),
            Err(e) => RecoveryVerificationResult::failure(format!("복구 키 검증 오류: {}", e)),
        }
    }

    /// 복구 키 형식 검증 (Base64 및 길이 확인)
    /// 
    /// # 매개변수
    /// * `recovery_key` - 검증할 복구 키
    /// 
    /// # 반환값
    /// * `Ok(())` - 형식이 올바름
    /// * `Err(RecoveryError)` - 형식 오류
    pub fn validate_recovery_key_format(&self, recovery_key: &str) -> Result<(), RecoveryError> {
        if recovery_key.is_empty() {
            return Err(RecoveryError::InvalidFormat("복구 키를 입력해주세요.".to_string()));
        }

        // Base64 디코딩 시도
        let key_bytes = general_purpose::STANDARD
            .decode(recovery_key)
            .map_err(|_| RecoveryError::InvalidFormat("올바르지 않은 Base64 형식입니다.".to_string()))?;

        // 길이 검증 (32바이트 = 256비트)
        if key_bytes.len() != 32 {
            return Err(RecoveryError::InvalidFormat("복구 키는 256비트(32바이트)여야 합니다.".to_string()));
        }

        Ok(())
    }

    /// 현재 복구 키 정보 조회
    pub fn get_current_recovery_key_info(&self) -> Option<RecoveryKeyInfo> {
        self.current_recovery_key.lock().unwrap().clone()
    }

    /// 복구 키 사용 표시 (한 번만 사용 가능)
    pub fn mark_recovery_key_used(&self) -> Result<(), RecoveryError> {
        let mut recovery_key = self.current_recovery_key.lock().unwrap();
        if let Some(ref mut info) = *recovery_key {
            if info.used {
                return Err(RecoveryError::VerificationFailed);
            }
            info.used = true;
            Ok(())
        } else {
            Err(RecoveryError::InternalError("복구 키 정보가 없습니다.".to_string()))
        }
    }

    /// 복구 키 정보 초기화 (보안을 위해)
    pub fn clear_recovery_key(&self) {
        *self.current_recovery_key.lock().unwrap() = None;
    }
}

impl Default for RecoveryService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_recovery_key() {
        let service = RecoveryService::new();
        let recovery_key = service.generate_recovery_key().unwrap();
        
        // Base64 형식 검증
        assert!(general_purpose::STANDARD.decode(&recovery_key).is_ok());
        
        // 길이 검증 (Base64로 인코딩된 32바이트는 44자)
        let decoded = general_purpose::STANDARD.decode(&recovery_key).unwrap();
        assert_eq!(decoded.len(), 32);
    }

    #[test]
    fn test_hash_recovery_key() {
        let service = RecoveryService::new();
        let recovery_key = service.generate_recovery_key().unwrap();
        let hash = service.hash_recovery_key(&recovery_key).unwrap();
        
        // 해시가 Base64 형식인지 확인
        assert!(general_purpose::STANDARD.decode(&hash).is_ok());
        
        // SHA-256 해시는 32바이트
        let decoded_hash = general_purpose::STANDARD.decode(&hash).unwrap();
        assert_eq!(decoded_hash.len(), 32);
    }

    #[test]
    fn test_verify_recovery_key() {
        let service = RecoveryService::new();
        let recovery_key = service.generate_recovery_key().unwrap();
        let hash = service.hash_recovery_key(&recovery_key).unwrap();
        
        // 올바른 복구 키 검증
        assert!(service.verify_recovery_key(&recovery_key, &hash).unwrap());
        
        // 잘못된 복구 키 검증
        let wrong_key = general_purpose::STANDARD.encode(&[0u8; 32]);
        assert!(!service.verify_recovery_key(&wrong_key, &hash).unwrap());
    }

    #[test]
    fn test_derive_key_from_recovery_key() {
        let service = RecoveryService::new();
        let recovery_key = service.generate_recovery_key().unwrap();
        let salt = [1u8; 32];
        
        let master_key = service.derive_key_from_recovery_key(&recovery_key, &salt, Some(1000)).unwrap();
        assert_eq!(master_key.len(), 32);
        
        // 같은 입력으로 같은 키가 나오는지 확인
        let master_key2 = service.derive_key_from_recovery_key(&recovery_key, &salt, Some(1000)).unwrap();
        assert_eq!(master_key, master_key2);
    }

    #[test]
    fn test_validate_recovery_key_format() {
        let service = RecoveryService::new();
        
        // 올바른 형식
        let valid_key = general_purpose::STANDARD.encode(&[0u8; 32]);
        assert!(service.validate_recovery_key_format(&valid_key).is_ok());
        
        // 잘못된 길이
        let invalid_key = general_purpose::STANDARD.encode(&[0u8; 16]);
        assert!(service.validate_recovery_key_format(&invalid_key).is_err());
        
        // 잘못된 Base64
        assert!(service.validate_recovery_key_format("invalid_base64!").is_err());
        
        // 빈 문자열
        assert!(service.validate_recovery_key_format("").is_err());
    }

    #[test]
    fn test_verify_and_derive_key() {
        let service = RecoveryService::new();
        let recovery_key = service.generate_recovery_key().unwrap();
        let hash = service.hash_recovery_key(&recovery_key).unwrap();
        let salt = [2u8; 32];
        
        let result = service.verify_and_derive_key(&recovery_key, &hash, &salt, Some(1000));
        assert!(result.is_valid);
        assert!(result.master_key.is_some());
        assert_eq!(result.master_key.unwrap().len(), 32);
    }
}