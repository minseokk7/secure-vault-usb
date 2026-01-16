// 암호화 서비스
// 파일 암호화/복호화, 키 관리, 메모리 보안 등을 담당합니다.

use crate::models::{
    CryptoError, EncryptionAlgorithm, EncryptionMetadata, EncryptedData, 
    KeyDerivationParams, SecureMemory, SecureRandom,
};
use crate::SecureVaultResult;
use aes_gcm::{Aes256Gcm, Key, Nonce, aead::{Aead, KeyInit}};
use chacha20poly1305::{ChaCha20Poly1305, Key as ChaChaKey, Nonce as ChaChaNonce};
use sha2::{Sha256, Digest};
use pbkdf2::pbkdf2_hmac;
use uuid::Uuid;
use std::time::Instant;

/// 암호화 서비스
/// 파일 암호화/복호화와 키 관리를 담당합니다.
#[derive(Debug, Clone)]
pub struct CryptoService {
    /// 마스터 키 (메모리에서만 존재)
    master_key: Option<[u8; 32]>,
    
    /// 키 유도 매개변수
    kdf_params: KeyDerivationParams,
    
    /// 기본 암호화 알고리즘
    default_algorithm: EncryptionAlgorithm,
}

impl CryptoService {
    /// 새로운 암호화 서비스를 생성합니다.
    /// 
    /// # 반환값
    /// * `Self` - 초기화된 암호화 서비스
    pub fn new() -> Self {
        let salt = SecureRandom::generate_salt();
        
        Self {
            master_key: None,
            kdf_params: KeyDerivationParams::default_with_salt(salt.to_vec()),
            default_algorithm: EncryptionAlgorithm::default(),
        }
    }
    
    /// PIN으로부터 마스터 키를 유도합니다.
    /// 
    /// C# EncryptionService.DeriveKeyFromPin()과 동일한 로직을 사용합니다:
    /// - PBKDF2-HMAC-SHA256 알고리즘
    /// - 100,000회 반복 (C# 버전과 동일)
    /// - 32바이트 솔트 사용
    /// - 256비트(32바이트) 마스터 키 생성
    /// 
    /// # 매개변수
    /// * `pin` - 사용자 PIN (4-8자리 숫자)
    /// * `salt` - 32바이트 키 유도용 솔트
    /// 
    /// # 반환값
    /// * `SecureVaultResult<()>` - 키 유도 결과
    /// 
    /// # 오류
    /// * `CryptoError::InvalidPin` - PIN이 비어있거나 형식이 잘못됨
    /// * `CryptoError::InvalidSalt` - 솔트가 32바이트가 아님
    pub fn derive_master_key(&mut self, pin: &str, salt: &[u8]) -> SecureVaultResult<()> {
        // PIN 유효성 검사 (C# 버전과 동일)
        if pin.is_empty() {
            return Err(CryptoError::InvalidPin("PIN이 비어있습니다.".to_string()).into());
        }
        
        // 솔트 유효성 검사 (C# 버전과 동일: 32바이트)
        if salt.len() != 32 {
            return Err(CryptoError::InvalidSalt("솔트는 32바이트여야 합니다.".to_string()).into());
        }
        
        let mut key = [0u8; 32];
        
        // PBKDF2-HMAC-SHA256으로 키 유도 (C# 버전과 동일한 100,000회 반복)
        pbkdf2_hmac::<Sha256>(
            pin.as_bytes(),
            salt,
            100_000, // C# 버전과 동일한 반복 횟수
            &mut key
        );
        
        self.master_key = Some(key);
        
        log::info!("마스터 키가 성공적으로 유도되었습니다. (PBKDF2-SHA256, 100,000회 반복)");
        Ok(())
    }
    
    /// 현재 마스터 키를 반환합니다.
    /// 
    /// # 반환값
    /// * `Option<[u8; 32]>` - 마스터 키 (초기화되지 않은 경우 None)
    pub fn get_master_key(&self) -> Option<[u8; 32]> {
        self.master_key
    }
    
    /// 32바이트 랜덤 솔트를 생성합니다.
    /// 
    /// C# EncryptionService.GenerateSalt()와 동일한 기능을 제공합니다.
    /// 
    /// # 반환값
    /// * `[u8; 32]` - 32바이트 랜덤 솔트
    pub fn generate_salt() -> [u8; 32] {
        SecureRandom::generate_salt()
    }
    
    /// 256비트 복구 키를 생성합니다.
    /// 
    /// C# EncryptionService.GenerateRecoveryKey()와 동일한 기능을 제공합니다.
    /// 
    /// # 반환값
    /// * `[u8; 32]` - 256비트 복구 키
    pub fn generate_recovery_key() -> [u8; 32] {
        let mut recovery_key = [0u8; 32];
        SecureRandom::fill_bytes(&mut recovery_key);
        recovery_key
    }
    
    /// 데이터를 C# 버전과 호환되는 형식으로 암호화합니다.
    /// 
    /// C# EncryptionService.EncryptData()와 동일한 형식을 사용합니다:
    /// - AES-256-GCM 알고리즘
    /// - 12바이트 IV (96비트, GCM 권장)
    /// - 16바이트 인증 태그 (128비트)
    /// - 결과 형식: IV + 암호문 + 인증태그
    /// 
    /// # 매개변수
    /// * `data` - 암호화할 원본 데이터
    /// * `key` - 256비트(32바이트) 암호화 키
    /// 
    /// # 반환값
    /// * `SecureVaultResult<Vec<u8>>` - 암호화된 데이터 (C# 호환 형식)
    /// 
    /// # 오류
    /// * `CryptoError::InvalidData` - 데이터가 비어있음
    /// * `CryptoError::InvalidKey` - 키가 32바이트가 아님
    /// * `CryptoError::EncryptionFailed` - 암호화 실패
    pub fn encrypt_data_csharp_compatible(&self, data: &[u8], key: &[u8]) -> SecureVaultResult<Vec<u8>> {
        // 데이터 유효성 검사 (C# 버전과 동일)
        if data.is_empty() {
            return Err(CryptoError::InvalidData("암호화할 데이터가 비어있습니다.".to_string()).into());
        }
        
        // 키 유효성 검사 (C# 버전과 동일: 32바이트)
        if key.len() != 32 {
            return Err(CryptoError::InvalidKey("키는 32바이트(256비트)여야 합니다.".to_string()).into());
        }
        
        let start_time = Instant::now();
        
        // C# 버전과 동일한 크기 사용
        const IV_SIZE: usize = 12;  // 96비트 IV (C# IvSize)
        const TAG_SIZE: usize = 16; // 128비트 인증 태그 (C# TagSize)
        
        // IV, 암호문, 인증 태그를 위한 배열 생성
        let mut iv = [0u8; IV_SIZE];
        
        // 랜덤 IV 생성 (C# RandomNumberGenerator.Fill과 동일)
        SecureRandom::fill_bytes(&mut iv);
        
        // AES-256-GCM 암호화 수행
        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
        let nonce = Nonce::from_slice(&iv);
        
        let ciphertext_with_tag = cipher.encrypt(nonce, data)
            .map_err(|_| CryptoError::EncryptionFailed)?;
        
        // 암호문과 태그 분리 (C# 버전과 동일한 방식)
        let ciphertext_len = ciphertext_with_tag.len() - TAG_SIZE;
        let ciphertext = &ciphertext_with_tag[..ciphertext_len];
        let computed_tag = &ciphertext_with_tag[ciphertext_len..];
        
        // C# 버전과 동일한 형식으로 결합: IV + 암호문 + 태그
        let mut result = Vec::with_capacity(IV_SIZE + ciphertext.len() + TAG_SIZE);
        result.extend_from_slice(&iv);
        result.extend_from_slice(ciphertext);
        result.extend_from_slice(computed_tag);
        
        let encryption_time = start_time.elapsed().as_millis() as u64;
        
        log::debug!("C# 호환 암호화 완료: {} bytes -> {} bytes ({}ms)", 
                   data.len(), result.len(), encryption_time);
        
        Ok(result)
    }
    
    /// C# 버전과 호환되는 형식의 암호화된 데이터를 복호화합니다.
    /// 
    /// C# EncryptionService.DecryptData()와 동일한 형식을 처리합니다:
    /// - 입력 형식: IV + 암호문 + 인증태그
    /// - 12바이트 IV + 가변 길이 암호문 + 16바이트 태그
    /// - 인증 태그 자동 검증
    /// 
    /// # 매개변수
    /// * `encrypted_data` - 암호화된 데이터 (C# 호환 형식)
    /// * `key` - 256비트(32바이트) 복호화 키
    /// 
    /// # 반환값
    /// * `SecureVaultResult<Vec<u8>>` - 복호화된 원본 데이터
    /// 
    /// # 오류
    /// * `CryptoError::InvalidData` - 암호화된 데이터가 유효하지 않음
    /// * `CryptoError::InvalidKey` - 키가 32바이트가 아님
    /// * `CryptoError::DecryptionFailed` - 복호화 실패 (키 오류 또는 데이터 손상)
    pub fn decrypt_data_csharp_compatible(&self, encrypted_data: &[u8], key: &[u8]) -> SecureVaultResult<Vec<u8>> {
        const IV_SIZE: usize = 12;  // C# IvSize
        const TAG_SIZE: usize = 16; // C# TagSize
        
        // 데이터 유효성 검사 (C# 버전과 동일)
        if encrypted_data.len() < IV_SIZE + TAG_SIZE {
            return Err(CryptoError::InvalidData("암호화된 데이터가 유효하지 않습니다.".to_string()).into());
        }
        
        // 키 유효성 검사 (C# 버전과 동일)
        if key.len() != 32 {
            return Err(CryptoError::InvalidKey("키는 32바이트(256비트)여야 합니다.".to_string()).into());
        }
        
        let start_time = Instant::now();
        
        // IV, 암호문, 인증 태그 분리 (C# 버전과 동일한 방식)
        let iv = &encrypted_data[..IV_SIZE];
        let ciphertext_len = encrypted_data.len() - IV_SIZE - TAG_SIZE;
        let ciphertext = &encrypted_data[IV_SIZE..IV_SIZE + ciphertext_len];
        let tag = &encrypted_data[IV_SIZE + ciphertext_len..];
        
        // AES-256-GCM 복호화 수행
        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
        let nonce = Nonce::from_slice(iv);
        
        // 암호문과 태그를 결합하여 복호화
        let mut ciphertext_with_tag = Vec::with_capacity(ciphertext.len() + TAG_SIZE);
        ciphertext_with_tag.extend_from_slice(ciphertext);
        ciphertext_with_tag.extend_from_slice(tag);
        
        let plaintext = cipher.decrypt(nonce, ciphertext_with_tag.as_slice())
            .map_err(|_| CryptoError::DecryptionFailed)?;
        
        let decryption_time = start_time.elapsed().as_millis() as u64;
        
        log::debug!("C# 호환 복호화 완료: {} bytes -> {} bytes ({}ms)", 
                   encrypted_data.len(), plaintext.len(), decryption_time);
        
        Ok(plaintext)
    }
    /// 파일을 암호화합니다.
    /// 
    /// 각 파일마다 고유한 키를 사용하여 암호화합니다.
    /// 마스터 키에서 파일별 키를 유도하여 보안을 강화합니다.
    /// C# 버전과 호환되는 형식을 사용합니다.
    /// 
    /// # 매개변수
    /// * `data` - 암호화할 데이터
    /// * `file_id` - 파일 고유 ID
    /// 
    /// # 반환값
    /// * `SecureVaultResult<EncryptedData>` - 암호화된 데이터
    pub fn encrypt_file(&self, data: &[u8], file_id: &Uuid) -> SecureVaultResult<EncryptedData> {
        let start_time = Instant::now();
        
        // 마스터 키 확인
        let master_key = self.master_key.ok_or(CryptoError::NoMasterKey)?;
        
        // 파일별 고유 키 유도
        let file_key = self.derive_file_key(&master_key, file_id)?;
        
        // C# 호환 형식으로 암호화
        let encrypted_bytes = self.encrypt_data_csharp_compatible(data, &file_key)?;
        
        // 데이터 해시 계산
        let data_hash = self.calculate_data_hash(data);
        
        // 암호화 메타데이터 생성 (C# 호환)
        let metadata = EncryptionMetadata::new(
            EncryptionAlgorithm::AES256GCM, // C# 버전은 AES-256-GCM만 사용
            encrypted_bytes[..12].to_vec(), // IV (첫 12바이트)
            encrypted_bytes[encrypted_bytes.len()-16..].to_vec(), // 태그 (마지막 16바이트)
            SecureRandom::generate_salt().to_vec(),
            100_000, // C# 버전과 동일한 반복 횟수
            data_hash,
        );
        
        let encryption_time = start_time.elapsed().as_millis() as u64;
        
        let encrypted_data = EncryptedData::new(
            encrypted_bytes,
            metadata,
            data.len() as u64,
            encryption_time,
        );
        
        log::debug!("파일 암호화 완료 (C# 호환): {} bytes -> {} bytes ({}ms)", 
                   data.len(), encrypted_data.ciphertext.len(), encryption_time);
        
        Ok(encrypted_data)
    }
    
    /// 파일을 복호화합니다.
    /// 
    /// C# 버전과 호환되는 형식의 암호화된 데이터를 복호화합니다.
    /// 
    /// # 매개변수
    /// * `encrypted_data` - 암호화된 데이터
    /// * `file_id` - 파일 고유 ID
    /// 
    /// # 반환값
    /// * `SecureVaultResult<Vec<u8>>` - 복호화된 데이터
    pub fn decrypt_file(&self, encrypted_data: &EncryptedData, file_id: &Uuid) -> SecureVaultResult<Vec<u8>> {
        let start_time = Instant::now();
        
        // 마스터 키 확인
        let master_key = self.master_key.ok_or(CryptoError::NoMasterKey)?;
        
        // 메타데이터 유효성 검증
        if !encrypted_data.metadata.is_valid() {
            return Err(CryptoError::CorruptedMetadata.into());
        }
        
        // 파일별 고유 키 유도
        let file_key = self.derive_file_key(&master_key, file_id)?;
        
        // C# 호환 형식으로 복호화
        let plaintext = self.decrypt_data_csharp_compatible(&encrypted_data.ciphertext, &file_key)?;
        
        // 데이터 무결성 검증
        let calculated_hash = self.calculate_data_hash(&plaintext);
        if calculated_hash != encrypted_data.metadata.data_hash {
            return Err(CryptoError::CorruptedMetadata.into());
        }
        
        let decryption_time = start_time.elapsed().as_millis() as u64;
        
        log::debug!("파일 복호화 완료 (C# 호환): {} bytes -> {} bytes ({}ms)", 
                   encrypted_data.ciphertext.len(), plaintext.len(), decryption_time);
        
        Ok(plaintext)
    }
    
    /// 스트리밍 방식으로 대용량 파일을 암호화합니다.
    /// 
    /// 메모리 사용량을 제한하면서 대용량 파일을 처리할 수 있습니다.
    /// 
    /// # 매개변수
    /// * `input_data` - 입력 데이터 (청크 단위)
    /// * `file_id` - 파일 고유 ID
    /// * `chunk_index` - 청크 인덱스
    /// 
    /// # 반환값
    /// * `SecureVaultResult<Vec<u8>>` - 암호화된 청크
    pub fn encrypt_chunk(&self, input_data: &[u8], file_id: &Uuid, chunk_index: u32) -> SecureVaultResult<Vec<u8>> {
        // 마스터 키 확인
        let master_key = self.master_key.ok_or(CryptoError::NoMasterKey)?;
        
        // 청크별 고유 키 유도 (파일 ID + 청크 인덱스)
        let mut chunk_id_bytes = file_id.as_bytes().to_vec();
        chunk_id_bytes.extend_from_slice(&chunk_index.to_le_bytes());
        
        let chunk_key = self.derive_chunk_key(&master_key, &chunk_id_bytes)?;
        
        // 논스 생성 (청크별 고유)
        let nonce_bytes = SecureRandom::generate_nonce(&self.default_algorithm);
        
        // 암호화
        let (mut ciphertext, tag) = match self.default_algorithm {
            EncryptionAlgorithm::AES256GCM => {
                self.encrypt_with_aes256gcm(&chunk_key, &nonce_bytes, input_data)?
            }
            EncryptionAlgorithm::ChaCha20Poly1305 => {
                self.encrypt_with_chacha20poly1305(&chunk_key, &nonce_bytes, input_data)?
            }
        };
        
        // 결과 조합: nonce + tag + ciphertext
        let mut result = Vec::with_capacity(nonce_bytes.len() + tag.len() + ciphertext.len());
        result.extend_from_slice(&nonce_bytes);
        result.extend_from_slice(&tag);
        result.append(&mut ciphertext);
        
        Ok(result)
    }
    
    /// 스트리밍 방식으로 암호화된 청크를 복호화합니다.
    /// 
    /// # 매개변수
    /// * `encrypted_chunk` - 암호화된 청크
    /// * `file_id` - 파일 고유 ID
    /// * `chunk_index` - 청크 인덱스
    /// 
    /// # 반환값
    /// * `SecureVaultResult<Vec<u8>>` - 복호화된 청크
    pub fn decrypt_chunk(&self, encrypted_chunk: &[u8], file_id: &Uuid, chunk_index: u32) -> SecureVaultResult<Vec<u8>> {
        // 마스터 키 확인
        let master_key = self.master_key.ok_or(CryptoError::NoMasterKey)?;
        
        // 청크별 고유 키 유도
        let mut chunk_id_bytes = file_id.as_bytes().to_vec();
        chunk_id_bytes.extend_from_slice(&chunk_index.to_le_bytes());
        
        let chunk_key = self.derive_chunk_key(&master_key, &chunk_id_bytes)?;
        
        // 논스, 태그, 암호문 분리
        let nonce_size = self.default_algorithm.nonce_size();
        let tag_size = self.default_algorithm.tag_size();
        
        if encrypted_chunk.len() < nonce_size + tag_size {
            return Err(CryptoError::CorruptedMetadata.into());
        }
        
        let nonce = &encrypted_chunk[..nonce_size];
        let tag = &encrypted_chunk[nonce_size..nonce_size + tag_size];
        let ciphertext = &encrypted_chunk[nonce_size + tag_size..];
        
        // 복호화
        let plaintext = match self.default_algorithm {
            EncryptionAlgorithm::AES256GCM => {
                self.decrypt_with_aes256gcm(&chunk_key, nonce, ciphertext, tag)?
            }
            EncryptionAlgorithm::ChaCha20Poly1305 => {
                self.decrypt_with_chacha20poly1305(&chunk_key, nonce, ciphertext, tag)?
            }
        };
        
        Ok(plaintext)
    }
    
    /// 메모리에서 민감한 데이터를 안전하게 제거합니다.
    /// 
    /// 애플리케이션 종료 시나 로그아웃 시 호출하여
    /// 메모리에 남아있는 키 정보를 안전하게 삭제합니다.
    pub fn clear_sensitive_data(&mut self) {
        if let Some(ref mut key) = self.master_key {
            SecureMemory::clear_bytes(key);
        }
        self.master_key = None;
        
        // KDF 매개변수의 솔트도 클리어
        SecureMemory::clear_vec(&mut self.kdf_params.salt);
        
        log::info!("민감한 데이터가 메모리에서 안전하게 제거되었습니다.");
    }
    
    /// 마스터 키가 설정되어 있는지 확인합니다.
    /// 
    /// # 반환값
    /// * `bool` - 마스터 키 설정 여부
    pub fn has_master_key(&self) -> bool {
        self.master_key.is_some()
    }
    
    /// 기본 암호화 알고리즘을 설정합니다.
    /// 
    /// # 매개변수
    /// * `algorithm` - 설정할 알고리즘
    pub fn set_default_algorithm(&mut self, algorithm: EncryptionAlgorithm) {
        log::info!("기본 암호화 알고리즘이 {:?}로 변경되었습니다.", algorithm);
        self.default_algorithm = algorithm;
    }
    
    /// 현재 기본 암호화 알고리즘을 반환합니다.
    /// 
    /// # 반환값
    /// * `&EncryptionAlgorithm` - 현재 알고리즘
    pub fn get_default_algorithm(&self) -> &EncryptionAlgorithm {
        &self.default_algorithm
    }
    
    /// 파일별 고유 키를 유도합니다.
    /// 
    /// # 매개변수
    /// * `master_key` - 마스터 키
    /// * `file_id` - 파일 고유 ID
    /// 
    /// # 반환값
    /// * `SecureVaultResult<[u8; 32]>` - 유도된 파일 키
    fn derive_file_key(&self, master_key: &[u8; 32], file_id: &Uuid) -> SecureVaultResult<[u8; 32]> {
        let mut file_key = [0u8; 32];
        let file_id_bytes = file_id.as_bytes();
        
        // PBKDF2-HMAC-SHA256으로 파일별 키 유도
        pbkdf2_hmac::<Sha256>(
            master_key,
            file_id_bytes,
            10_000, // 파일 키는 상대적으로 적은 반복 횟수 사용
            &mut file_key
        );
        
        Ok(file_key)
    }
    
    /// 청크별 고유 키를 유도합니다.
    /// 
    /// # 매개변수
    /// * `master_key` - 마스터 키
    /// * `chunk_id` - 청크 식별자
    /// 
    /// # 반환값
    /// * `SecureVaultResult<[u8; 32]>` - 유도된 청크 키
    fn derive_chunk_key(&self, master_key: &[u8; 32], chunk_id: &[u8]) -> SecureVaultResult<[u8; 32]> {
        let mut chunk_key = [0u8; 32];
        
        // PBKDF2-HMAC-SHA256으로 청크별 키 유도
        pbkdf2_hmac::<Sha256>(
            master_key,
            chunk_id,
            5_000, // 청크 키는 더 적은 반복 횟수 사용 (성능 고려)
            &mut chunk_key
        );
        
        Ok(chunk_key)
    }
    
    /// AES-256-GCM으로 암호화합니다.
    /// 
    /// # 매개변수
    /// * `key` - 암호화 키
    /// * `nonce` - 논스
    /// * `data` - 암호화할 데이터
    /// 
    /// # 반환값
    /// * `SecureVaultResult<(Vec<u8>, Vec<u8>)>` - (암호문, 인증 태그)
    fn encrypt_with_aes256gcm(&self, key: &[u8; 32], nonce: &[u8], data: &[u8]) -> SecureVaultResult<(Vec<u8>, Vec<u8>)> {
        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
        let nonce = Nonce::from_slice(nonce);
        
        let ciphertext = cipher.encrypt(nonce, data)
            .map_err(|_| CryptoError::EncryptionFailed)?;
        
        // AES-GCM은 인증 태그가 암호문에 포함되어 있음
        let tag_size = self.default_algorithm.tag_size();
        let (ciphertext_only, tag) = ciphertext.split_at(ciphertext.len() - tag_size);
        
        Ok((ciphertext_only.to_vec(), tag.to_vec()))
    }
    
    /// AES-256-GCM으로 복호화합니다.
    /// 
    /// # 매개변수
    /// * `key` - 복호화 키
    /// * `nonce` - 논스
    /// * `ciphertext` - 암호문
    /// * `tag` - 인증 태그
    /// 
    /// # 반환값
    /// * `SecureVaultResult<Vec<u8>>` - 복호화된 데이터
    fn decrypt_with_aes256gcm(&self, key: &[u8; 32], nonce: &[u8], ciphertext: &[u8], tag: &[u8]) -> SecureVaultResult<Vec<u8>> {
        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
        let nonce = Nonce::from_slice(nonce);
        
        // 암호문과 태그를 결합
        let mut ciphertext_with_tag = ciphertext.to_vec();
        ciphertext_with_tag.extend_from_slice(tag);
        
        let plaintext = cipher.decrypt(nonce, ciphertext_with_tag.as_slice())
            .map_err(|_| CryptoError::DecryptionFailed)?;
        
        Ok(plaintext)
    }
    
    /// ChaCha20-Poly1305로 암호화합니다.
    /// 
    /// # 매개변수
    /// * `key` - 암호화 키
    /// * `nonce` - 논스
    /// * `data` - 암호화할 데이터
    /// 
    /// # 반환값
    /// * `SecureVaultResult<(Vec<u8>, Vec<u8>)>` - (암호문, 인증 태그)
    fn encrypt_with_chacha20poly1305(&self, key: &[u8; 32], nonce: &[u8], data: &[u8]) -> SecureVaultResult<(Vec<u8>, Vec<u8>)> {
        let cipher = ChaCha20Poly1305::new(ChaChaKey::from_slice(key));
        let nonce = ChaChaNonce::from_slice(nonce);
        
        let ciphertext = cipher.encrypt(nonce, data)
            .map_err(|_| CryptoError::EncryptionFailed)?;
        
        // ChaCha20-Poly1305도 인증 태그가 암호문에 포함되어 있음
        let tag_size = self.default_algorithm.tag_size();
        let (ciphertext_only, tag) = ciphertext.split_at(ciphertext.len() - tag_size);
        
        Ok((ciphertext_only.to_vec(), tag.to_vec()))
    }
    
    /// ChaCha20-Poly1305로 복호화합니다.
    /// 
    /// # 매개변수
    /// * `key` - 복호화 키
    /// * `nonce` - 논스
    /// * `ciphertext` - 암호문
    /// * `tag` - 인증 태그
    /// 
    /// # 반환값
    /// * `SecureVaultResult<Vec<u8>>` - 복호화된 데이터
    fn decrypt_with_chacha20poly1305(&self, key: &[u8; 32], nonce: &[u8], ciphertext: &[u8], tag: &[u8]) -> SecureVaultResult<Vec<u8>> {
        let cipher = ChaCha20Poly1305::new(ChaChaKey::from_slice(key));
        let nonce = ChaChaNonce::from_slice(nonce);
        
        // 암호문과 태그를 결합
        let mut ciphertext_with_tag = ciphertext.to_vec();
        ciphertext_with_tag.extend_from_slice(tag);
        
        let plaintext = cipher.decrypt(nonce, ciphertext_with_tag.as_slice())
            .map_err(|_| CryptoError::DecryptionFailed)?;
        
        Ok(plaintext)
    }
    
    /// 데이터의 SHA-256 해시를 계산합니다.
    /// 
    /// # 매개변수
    /// * `data` - 해시할 데이터
    /// 
    /// # 반환값
    /// * `Vec<u8>` - SHA-256 해시
    fn calculate_data_hash(&self, data: &[u8]) -> Vec<u8> {
        let mut hasher = Sha256::new();
        hasher.update(data);
        hasher.finalize().to_vec()
    }
}

impl Default for CryptoService {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for CryptoService {
    /// 서비스 소멸 시 민감한 데이터를 자동으로 정리합니다.
    fn drop(&mut self) {
        self.clear_sensitive_data();
    }
}