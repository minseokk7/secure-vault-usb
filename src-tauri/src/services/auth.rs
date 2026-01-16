// 인증 서비스
// PIN 인증, 복구 키, 세션 관리 등의 인증 관련 기능을 제공합니다.
// C# SecurityService.cs를 완전히 포팅한 버전입니다.

use crate::models::{
    AuthError, AuthSession, AuthMethod, BruteForceProtection, PinInfo, PinComplexity,
    SimpleRecoveryKeyInfo, PinValidationResult, RecoveryKeyValidationResult, AuthState,
};
use crate::SecureVaultResult;
use sha2::{Sha256, Digest};
use pbkdf2::pbkdf2_hmac;
use base64::{Engine as _, engine::general_purpose};
use rand::{RngCore, rngs::OsRng};
use regex::Regex;
use uuid::Uuid;

/// 인증 서비스
/// C# SecurityService를 완전히 포팅한 Rust 버전
/// 사용자 인증과 세션 관리를 담당합니다.
#[derive(Debug)]
pub struct AuthService {
    /// PIN 정보
    pin_info: Option<PinInfo>,
    
    /// 복구 키 정보
    recovery_key_info: Option<SimpleRecoveryKeyInfo>,
    
    /// 현재 세션
    current_session: Option<AuthSession>,
    
    /// 브루트포스 방지 (C# 버전과 동일한 정책)
    brute_force_protection: BruteForceProtection,
    
    /// 인증 상태
    auth_state: AuthState,
}

impl AuthService {
    /// 새로운 인증 서비스를 생성합니다.
    /// 테스트용으로 PIN 1234가 미리 설정됩니다.
    /// 
    /// # 반환값
    /// * `Self` - 초기화된 인증 서비스
    pub fn new() -> Self {
        let mut service = Self {
            pin_info: None,
            recovery_key_info: None,
            current_session: None,
            brute_force_protection: BruteForceProtection::new(),
            auth_state: AuthState::Unauthenticated,
        };
        
        // 테스트용 PIN 1234 설정
        service.initialize_test_pin();
        
        service
    }
    
    /// 테스트용 PIN 1234를 초기화합니다.
    fn initialize_test_pin(&mut self) {
        // 32바이트 고정 솔트 (테스트용)
        let salt = [
            0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08,
            0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F, 0x10,
            0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18,
            0x19, 0x1A, 0x1B, 0x1C, 0x1D, 0x1E, 0x1F, 0x20,
        ];
        
        if let Ok(hashed_pin) = self.hash_pin("1234", &salt) {
            self.pin_info = Some(PinInfo {
                hash: hashed_pin,
                salt: salt.to_vec(),
            });
        }
    }
    
    /// PIN을 해시화합니다.
    /// C# SecurityService.HashPin()과 동일한 로직: SHA-256 + 솔트
    /// 
    /// # 매개변수
    /// * `pin` - 원본 PIN
    /// * `salt` - 32바이트 솔트
    /// 
    /// # 반환값
    /// * `SecureVaultResult<String>` - Base64로 인코딩된 해시값
    pub fn hash_pin(&self, pin: &str, salt: &[u8]) -> SecureVaultResult<String> {
        if pin.is_empty() {
            return Err(AuthError::InvalidPinFormat.into());
        }
        
        if salt.len() != 32 {
            return Err(AuthError::InvalidSalt.into());
        }
        
        let mut hasher = Sha256::new();
        
        // PIN + 솔트 결합 (C# 버전과 동일한 순서)
        hasher.update(pin.as_bytes());
        hasher.update(salt);
        
        let hash = hasher.finalize();
        Ok(general_purpose::STANDARD.encode(&hash))
    }
    
    /// PIN 형식을 검증합니다.
    /// C# SecurityService.ValidatePinFormat()과 동일한 로직: 4-8자리 숫자
    /// 
    /// # 매개변수
    /// * `pin` - 검증할 PIN
    /// 
    /// # 반환값
    /// * `SecureVaultResult<()>` - 검증 결과
    pub fn validate_pin_format(&self, pin: &str) -> SecureVaultResult<()> {
        if pin.is_empty() {
            return Err(AuthError::InvalidPinFormat.into());
        }
        
        // C# 버전과 동일한 정규식: 4-8자리 숫자
        let pin_regex = Regex::new(r"^\d{4,8}$").unwrap();
        if !pin_regex.is_match(pin) {
            return Err(AuthError::InvalidPinFormat.into());
        }
        
        Ok(())
    }
    
    /// PIN을 설정합니다.
    /// 
    /// # 매개변수
    /// * `pin` - 설정할 PIN
    /// * `complexity` - PIN 복잡도 레벨
    /// 
    /// # 반환값
    /// * `SecureVaultResult<()>` - 설정 결과
    pub fn set_pin(&mut self, pin: &str, complexity: PinComplexity) -> SecureVaultResult<()> {
        // PIN 형식 검증
        self.validate_pin_format(pin)?;
        
        // 32바이트 솔트 생성 (C# 버전과 동일)
        let mut salt = [0u8; 32];
        OsRng.fill_bytes(&mut salt);
        
        // PIN 해시 생성
        let hash = self.hash_pin(pin, &salt)?;
        
        // PIN 정보 저장
        self.pin_info = Some(PinInfo::new(
            hash,
            salt.to_vec(),
            complexity,
        ));
        
        log::info!("PIN이 성공적으로 설정되었습니다.");
        Ok(())
    }
    
    /// PIN이 일치하는지 검증합니다.
    /// C# SecurityService.VerifyPin()과 동일한 로직
    /// 
    /// # 매개변수
    /// * `input_pin` - 입력된 PIN
    /// * `stored_hash` - 저장된 해시값
    /// * `salt` - 솔트
    /// 
    /// # 반환값
    /// * `bool` - 검증 결과
    pub fn verify_pin_hash(&self, input_pin: &str, stored_hash: &str, salt: &[u8]) -> bool {
        if input_pin.is_empty() || stored_hash.is_empty() {
            return false;
        }
        
        match self.hash_pin(input_pin, salt) {
            Ok(input_hash) => input_hash == stored_hash,
            Err(_) => false,
        }
    }
    
    /// PIN을 검증합니다.
    /// 
    /// # 매개변수
    /// * `pin` - 검증할 PIN
    /// 
    /// # 반환값
    /// * `SecureVaultResult<PinValidationResult>` - 검증 결과
    pub fn verify_pin(&mut self, pin: &str) -> SecureVaultResult<PinValidationResult> {
        // 브루트포스 방지 체크 (C# SecurityService.CanAttemptLogin()과 동일)
        if self.brute_force_protection.is_currently_locked() {
            if let Some(remaining) = self.brute_force_protection.remaining_lockout_seconds() {
                return Ok(PinValidationResult::AccountLocked(remaining));
            }
        }
        
        // PIN 정보 확인
        let pin_info = self.pin_info.as_ref()
            .ok_or(AuthError::NoPinSet)?;
        
        // PIN 만료 확인
        if pin_info.is_expired() {
            return Ok(PinValidationResult::Expired);
        }
        
        // PIN 형식 검증
        if let Err(_) = self.validate_pin_format(pin) {
            return Ok(PinValidationResult::InvalidFormat);
        }
        
        // 해시 검증
        if self.verify_pin_hash(pin, &pin_info.hash, &pin_info.salt) {
            // 인증 성공 (C# SecurityService.ClearFailedLogins()와 동일)
            self.brute_force_protection.record_success();
            self.auth_state = AuthState::Authenticated(AuthMethod::Pin);
            
            // 세션 생성
            self.create_session(AuthMethod::Pin, 3600)?; // 1시간 세션
            
            log::info!("PIN 인증이 성공했습니다.");
            Ok(PinValidationResult::Valid)
        } else {
            // 인증 실패 (C# SecurityService.RecordFailedLogin()과 동일)
            self.brute_force_protection.record_failure();
            log::warn!("PIN 인증이 실패했습니다.");
            Ok(PinValidationResult::Invalid)
        }
    }
    
    /// PIN으로부터 마스터 키를 유도합니다.
    /// C# SecurityService.DeriveKey()와 동일한 로직: PBKDF2-HMAC-SHA256
    /// 
    /// # 매개변수
    /// * `pin` - PIN
    /// * `salt` - 솔트
    /// * `iterations` - 반복 횟수 (기본값: 100,000)
    /// 
    /// # 반환값
    /// * `SecureVaultResult<Vec<u8>>` - 32바이트 마스터 키
    pub fn derive_key_from_pin(&self, pin: &str, salt: &[u8], iterations: u32) -> SecureVaultResult<Vec<u8>> {
        if pin.is_empty() {
            return Err(AuthError::InvalidPinFormat.into());
        }
        
        if salt.len() != 32 {
            return Err(AuthError::InvalidSalt.into());
        }
        
        let mut key = [0u8; 32]; // 256비트 키
        pbkdf2_hmac::<Sha256>(pin.as_bytes(), salt, iterations, &mut key);
        
        Ok(key.to_vec())
    }
    
    /// 복구 키를 해시화합니다.
    /// C# SecurityService.HashRecoveryKey()와 동일한 로직
    /// 
    /// # 매개변수
    /// * `recovery_key` - 복구 키 (Base64 문자열)
    /// 
    /// # 반환값
    /// * `SecureVaultResult<String>` - Base64로 인코딩된 해시값
    pub fn hash_recovery_key(&self, recovery_key: &str) -> SecureVaultResult<String> {
        if recovery_key.is_empty() {
            return Err(AuthError::InvalidRecoveryKey.into());
        }
        
        // Base64 디코딩
        let key_bytes = general_purpose::STANDARD.decode(recovery_key)
            .map_err(|_| AuthError::InvalidRecoveryKey)?;
        
        if key_bytes.len() != 32 {
            return Err(AuthError::InvalidRecoveryKey.into());
        }
        
        // SHA-256 해시
        let mut hasher = Sha256::new();
        hasher.update(&key_bytes);
        let hash = hasher.finalize();
        
        Ok(general_purpose::STANDARD.encode(&hash))
    }
    
    /// 복구 키가 일치하는지 검증합니다.
    /// C# SecurityService.VerifyRecoveryKey()와 동일한 로직
    /// 
    /// # 매개변수
    /// * `input_recovery_key` - 입력된 복구 키 (Base64 문자열)
    /// * `stored_hash` - 저장된 복구 키 해시값
    /// 
    /// # 반환값
    /// * `bool` - 검증 결과
    pub fn verify_recovery_key_hash(&self, input_recovery_key: &str, stored_hash: &str) -> bool {
        if input_recovery_key.is_empty() || stored_hash.is_empty() {
            return false;
        }
        
        match self.hash_recovery_key(input_recovery_key) {
            Ok(input_hash) => input_hash == stored_hash,
            Err(_) => false,
        }
    }
    
    /// 복구 키를 생성합니다.
    /// C# 버전과 동일: 32바이트 랜덤 키를 Base64로 인코딩
    /// 
    /// # 반환값
    /// * `SecureVaultResult<String>` - 생성된 복구 키 (Base64 문자열)
    pub fn generate_recovery_key(&mut self) -> SecureVaultResult<String> {
        // 32바이트 랜덤 키 생성 (C# 버전과 동일)
        let mut key_bytes = [0u8; 32];
        OsRng.fill_bytes(&mut key_bytes);
        
        let recovery_key = general_purpose::STANDARD.encode(&key_bytes);
        
        // 복구 키 해시 생성
        let hash = self.hash_recovery_key(&recovery_key)?;
        
        // 복구 키 정보 저장
        self.recovery_key_info = Some(SimpleRecoveryKeyInfo::new(hash));
        
        log::info!("복구 키가 생성되었습니다.");
        Ok(recovery_key)
    }
    
    /// 복구 키를 검증합니다.
    /// 
    /// # 매개변수
    /// * `recovery_key` - 검증할 복구 키
    /// 
    /// # 반환값
    /// * `SecureVaultResult<RecoveryKeyValidationResult>` - 검증 결과
    pub fn verify_recovery_key(&mut self, recovery_key: &str) -> SecureVaultResult<RecoveryKeyValidationResult> {
        // 복구 키 정보 확인
        let recovery_info = self.recovery_key_info.as_ref()
            .ok_or(AuthError::InvalidRecoveryKey)?;
        
        // 비활성화 상태 확인
        if !recovery_info.is_active {
            return Ok(RecoveryKeyValidationResult::Deactivated);
        }
        
        // 복구 키 형식 검증 (Base64 32바이트)
        if let Err(_) = general_purpose::STANDARD.decode(recovery_key) {
            return Ok(RecoveryKeyValidationResult::InvalidFormat);
        }
        
        // 해시 값을 미리 복사
        let stored_hash = recovery_info.hash.clone();
        
        // 해시 검증
        if self.verify_recovery_key_hash(recovery_key, &stored_hash) {
            // 인증 성공 - 이제 가변 참조 사용
            if let Some(recovery_info_mut) = self.recovery_key_info.as_mut() {
                recovery_info_mut.record_usage();
            }
            self.auth_state = AuthState::Authenticated(AuthMethod::RecoveryKey);
            
            // 세션 생성
            self.create_session(AuthMethod::RecoveryKey, 1800)?; // 30분 세션
            
            log::info!("복구 키 인증이 성공했습니다.");
            Ok(RecoveryKeyValidationResult::Valid)
        } else {
            log::warn!("복구 키 인증이 실패했습니다.");
            Ok(RecoveryKeyValidationResult::Invalid)
        }
    }
    
    /// 복구 키로부터 마스터 키를 유도합니다.
    /// C# SecurityService.DeriveKeyFromRecoveryKey()와 동일한 로직
    /// 
    /// # 매개변수
    /// * `recovery_key` - 복구 키 (Base64 문자열)
    /// * `salt` - 솔트
    /// * `iterations` - 반복 횟수 (기본값: 100,000)
    /// 
    /// # 반환값
    /// * `SecureVaultResult<Vec<u8>>` - 32바이트 마스터 키
    pub fn derive_key_from_recovery_key(&self, recovery_key: &str, salt: &[u8], iterations: u32) -> SecureVaultResult<Vec<u8>> {
        if recovery_key.is_empty() {
            return Err(AuthError::InvalidRecoveryKey.into());
        }
        
        if salt.len() != 32 {
            return Err(AuthError::InvalidSalt.into());
        }
        
        // Base64 디코딩
        let key_bytes = general_purpose::STANDARD.decode(recovery_key)
            .map_err(|_| AuthError::InvalidRecoveryKey)?;
        
        if key_bytes.len() != 32 {
            return Err(AuthError::InvalidRecoveryKey.into());
        }
        
        let mut master_key = [0u8; 32]; // 256비트 키
        pbkdf2_hmac::<Sha256>(&key_bytes, salt, iterations, &mut master_key);
        
        Ok(master_key.to_vec())
    }
    
    /// 세션을 생성합니다.
    /// 
    /// # 매개변수
    /// * `auth_method` - 인증 방법
    /// * `timeout_seconds` - 세션 만료 시간 (초)
    /// 
    /// # 반환값
    /// * `SecureVaultResult<Uuid>` - 생성된 세션 ID
    pub fn create_session(&mut self, auth_method: AuthMethod, timeout_seconds: u64) -> SecureVaultResult<Uuid> {
        let session = AuthSession::new(auth_method, timeout_seconds);
        let session_id = session.id;
        
        self.current_session = Some(session);
        
        log::info!("새 세션이 생성되었습니다: {}", session_id);
        Ok(session_id)
    }
    
    /// 현재 세션을 확인합니다.
    /// 
    /// # 반환값
    /// * `bool` - 유효한 세션 여부
    pub fn is_session_valid(&mut self) -> bool {
        if let Some(session) = &mut self.current_session {
            if !session.is_expired() {
                session.refresh_activity();
                return true;
            } else {
                // 세션 만료
                self.auth_state = AuthState::SessionExpired;
                self.current_session = None;
                log::info!("세션이 만료되었습니다.");
            }
        }
        
        false
    }
    
    /// 로그아웃합니다.
    /// 
    /// # 반환값
    /// * `SecureVaultResult<()>` - 로그아웃 결과
    pub fn logout(&mut self) -> SecureVaultResult<()> {
        if let Some(session) = &mut self.current_session {
            session.terminate();
            log::info!("세션이 종료되었습니다: {}", session.id);
        }
        
        self.current_session = None;
        self.auth_state = AuthState::Unauthenticated;
        
        log::info!("로그아웃이 완료되었습니다.");
        Ok(())
    }
    
    /// 현재 인증 상태를 반환합니다.
    /// 
    /// # 반환값
    /// * `&AuthState` - 인증 상태
    pub fn get_auth_state(&self) -> &AuthState {
        &self.auth_state
    }
    
    /// 현재 세션 정보를 반환합니다.
    /// 
    /// # 반환값
    /// * `Option<&AuthSession>` - 세션 정보
    pub fn get_current_session(&self) -> Option<&AuthSession> {
        self.current_session.as_ref()
    }
    
    /// PIN이 설정되어 있는지 확인합니다.
    /// 
    /// # 반환값
    /// * `bool` - PIN 설정 여부
    pub fn has_pin(&self) -> bool {
        self.pin_info.is_some()
    }
    
    /// 복구 키가 설정되어 있는지 확인합니다.
    /// 
    /// # 반환값
    /// * `bool` - 복구 키 설정 여부
    pub fn has_recovery_key(&self) -> bool {
        self.recovery_key_info.is_some()
    }
    
    /// 브루트포스 공격 방지 상태를 확인합니다.
    /// C# SecurityService.CanAttemptLogin()과 동일한 로직
    /// 
    /// # 반환값
    /// * `bool` - 로그인 시도 가능 여부
    pub fn can_attempt_login(&self) -> bool {
        !self.brute_force_protection.is_currently_locked()
    }
    
    /// 남은 대기 시간을 반환합니다.
    /// C# SecurityService.GetRemainingWaitTime()과 동일한 로직 (분 단위를 초 단위로 변환)
    /// 
    /// # 반환값
    /// * `u64` - 남은 대기 시간 (초)
    pub fn get_remaining_wait_time_seconds(&self) -> u64 {
        self.brute_force_protection.remaining_lockout_seconds().unwrap_or(0)
    }
}

impl Default for AuthService {
    fn default() -> Self {
        Self::new()
    }
}

impl AuthService {
    /// 브루트포스 방지 상태를 반환합니다.
    /// 
    /// # 반환값
    /// * `&BruteForceProtection` - 브루트포스 방지 정보
    pub fn get_brute_force_protection(&self) -> &BruteForceProtection {
        &self.brute_force_protection
    }
    
    /// PIN 정보를 반환합니다.
    /// 
    /// # 반환값
    /// * `Option<&PinInfo>` - PIN 정보
    pub fn get_pin_info(&self) -> Option<&PinInfo> {
        self.pin_info.as_ref()
    }
    
    /// 복구 키 정보를 반환합니다.
    /// 
    /// # 반환값
    /// * `Option<&SimpleRecoveryKeyInfo>` - 복구 키 정보
    pub fn get_recovery_key_info(&self) -> Option<&SimpleRecoveryKeyInfo> {
        self.recovery_key_info.as_ref()
    }
    
    /// 세션 남은 시간을 반환합니다 (초).
    /// 
    /// # 반환값
    /// * `Option<u64>` - 남은 시간 (세션이 없으면 None)
    pub fn get_session_remaining_time(&self) -> Option<u64> {
        self.current_session.as_ref().map(|s| s.remaining_time_seconds())
    }
    
    /// 복구 키를 비활성화합니다.
    /// 
    /// # 반환값
    /// * `SecureVaultResult<()>` - 비활성화 결과
    pub fn deactivate_recovery_key(&mut self) -> SecureVaultResult<()> {
        if let Some(recovery_info) = &mut self.recovery_key_info {
            recovery_info.deactivate();
            log::info!("복구 키가 비활성화되었습니다.");
            Ok(())
        } else {
            Err(AuthError::InvalidRecoveryKey.into())
        }
    }
    
    /// PIN을 변경합니다.
    /// 
    /// # 매개변수
    /// * `old_pin` - 기존 PIN
    /// * `new_pin` - 새 PIN
    /// * `complexity` - 새 PIN 복잡도
    /// 
    /// # 반환값
    /// * `SecureVaultResult<()>` - 변경 결과
    pub fn change_pin(&mut self, old_pin: &str, new_pin: &str, complexity: PinComplexity) -> SecureVaultResult<()> {
        // 기존 PIN 검증
        match self.verify_pin(old_pin)? {
            PinValidationResult::Valid => {
                // 새 PIN 설정
                self.set_pin(new_pin, complexity)?;
                log::info!("PIN이 성공적으로 변경되었습니다.");
                Ok(())
            }
            _ => Err(AuthError::AuthenticationFailed.into())
        }
    }
}