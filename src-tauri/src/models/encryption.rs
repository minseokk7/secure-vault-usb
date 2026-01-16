// 암호화 관련 데이터 모델
// 암호화 메타데이터와 알고리즘 정의를 담당합니다.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 지원하는 암호화 알고리즘
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum EncryptionAlgorithm {
    /// AES-256-GCM (기본값)
    /// 인증된 암호화를 제공하는 고성능 알고리즘
    AES256GCM,
    
    /// ChaCha20-Poly1305
    /// 모바일 환경에서 우수한 성능을 보이는 알고리즘
    ChaCha20Poly1305,
}

impl Default for EncryptionAlgorithm {
    fn default() -> Self {
        Self::AES256GCM
    }
}

impl EncryptionAlgorithm {
    /// 알고리즘의 키 크기를 반환합니다 (바이트 단위).
    /// 
    /// # 반환값
    /// * `usize` - 키 크기 (바이트)
    pub fn key_size(&self) -> usize {
        match self {
            Self::AES256GCM => 32,        // 256 bits
            Self::ChaCha20Poly1305 => 32, // 256 bits
        }
    }
    
    /// 알고리즘의 논스(nonce) 크기를 반환합니다 (바이트 단위).
    /// 
    /// # 반환값
    /// * `usize` - 논스 크기 (바이트)
    pub fn nonce_size(&self) -> usize {
        match self {
            Self::AES256GCM => 12,        // 96 bits
            Self::ChaCha20Poly1305 => 12, // 96 bits
        }
    }
    
    /// 알고리즘의 인증 태그 크기를 반환합니다 (바이트 단위).
    /// 
    /// # 반환값
    /// * `usize` - 태그 크기 (바이트)
    pub fn tag_size(&self) -> usize {
        match self {
            Self::AES256GCM => 16,        // 128 bits
            Self::ChaCha20Poly1305 => 16, // 128 bits
        }
    }
}

/// 암호화 메타데이터
/// 파일 암호화에 사용된 정보를 저장합니다.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionMetadata {
    /// 사용된 암호화 알고리즘
    pub algorithm: EncryptionAlgorithm,
    
    /// 초기화 벡터 (IV) 또는 논스
    /// 각 암호화 작업마다 고유한 값을 사용합니다.
    pub nonce: Vec<u8>,
    
    /// 인증 태그
    /// 데이터 무결성과 인증을 보장합니다.
    pub tag: Vec<u8>,
    
    /// 키 유도에 사용된 솔트
    /// 파일별로 고유한 키를 생성하기 위해 사용됩니다.
    pub salt: Vec<u8>,
    
    /// 키 유도 반복 횟수
    /// PBKDF2 등의 키 유도 함수에서 사용됩니다.
    pub iterations: u32,
    
    /// 암호화된 데이터의 해시값
    /// 데이터 무결성 검증에 사용됩니다.
    pub data_hash: Vec<u8>,
}

impl EncryptionMetadata {
    /// 새로운 암호화 메타데이터를 생성합니다.
    /// 
    /// # 매개변수
    /// * `algorithm` - 사용할 암호화 알고리즘
    /// * `nonce` - 초기화 벡터 또는 논스
    /// * `tag` - 인증 태그
    /// * `salt` - 키 유도용 솔트
    /// * `iterations` - 키 유도 반복 횟수
    /// * `data_hash` - 데이터 해시값
    /// 
    /// # 반환값
    /// * `Self` - 생성된 암호화 메타데이터
    pub fn new(
        algorithm: EncryptionAlgorithm,
        nonce: Vec<u8>,
        tag: Vec<u8>,
        salt: Vec<u8>,
        iterations: u32,
        data_hash: Vec<u8>,
    ) -> Self {
        Self {
            algorithm,
            nonce,
            tag,
            salt,
            iterations,
            data_hash,
        }
    }
    
    /// 메타데이터의 유효성을 검증합니다.
    /// 
    /// # 반환값
    /// * `bool` - 유효성 검증 결과
    pub fn is_valid(&self) -> bool {
        // 논스 크기 검증
        if self.nonce.len() != self.algorithm.nonce_size() {
            return false;
        }
        
        // 태그 크기 검증
        if self.tag.len() != self.algorithm.tag_size() {
            return false;
        }
        
        // 솔트 크기 검증 (최소 16바이트)
        if self.salt.len() < 16 {
            return false;
        }
        
        // 반복 횟수 검증 (최소 10,000회)
        if self.iterations < 10_000 {
            return false;
        }
        
        // 해시 크기 검증 (SHA-256: 32바이트)
        if self.data_hash.len() != 32 {
            return false;
        }
        
        true
    }
}

/// 암호화된 데이터 구조체
/// 암호화 작업의 결과를 담습니다.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedData {
    /// 암호화된 데이터
    pub ciphertext: Vec<u8>,
    
    /// 암호화 메타데이터
    pub metadata: EncryptionMetadata,
    
    /// 원본 데이터 크기
    pub original_size: u64,
    
    /// 암호화 시간 (성능 측정용)
    pub encryption_time_ms: u64,
}

impl EncryptedData {
    /// 새로운 암호화된 데이터를 생성합니다.
    /// 
    /// # 매개변수
    /// * `ciphertext` - 암호화된 데이터
    /// * `metadata` - 암호화 메타데이터
    /// * `original_size` - 원본 데이터 크기
    /// * `encryption_time_ms` - 암호화 소요 시간 (밀리초)
    /// 
    /// # 반환값
    /// * `Self` - 생성된 암호화 데이터
    pub fn new(
        ciphertext: Vec<u8>,
        metadata: EncryptionMetadata,
        original_size: u64,
        encryption_time_ms: u64,
    ) -> Self {
        Self {
            ciphertext,
            metadata,
            original_size,
            encryption_time_ms,
        }
    }
    
    /// 압축률을 계산합니다.
    /// 
    /// # 반환값
    /// * `f64` - 압축률 (0.0 ~ 1.0)
    pub fn compression_ratio(&self) -> f64 {
        if self.original_size == 0 {
            return 0.0;
        }
        
        let encrypted_size = self.ciphertext.len() as u64;
        1.0 - (encrypted_size as f64 / self.original_size as f64)
    }
    
    /// 암호화 효율성을 계산합니다 (MB/s).
    /// 
    /// # 반환값
    /// * `f64` - 암호화 속도 (MB/s)
    pub fn encryption_speed_mbps(&self) -> f64 {
        if self.encryption_time_ms == 0 {
            return 0.0;
        }
        
        let mb_size = self.original_size as f64 / (1024.0 * 1024.0);
        let seconds = self.encryption_time_ms as f64 / 1000.0;
        
        mb_size / seconds
    }
}

/// 키 유도 매개변수
/// PBKDF2 등의 키 유도 함수에서 사용되는 매개변수입니다.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyDerivationParams {
    /// 솔트 값
    pub salt: Vec<u8>,
    
    /// 반복 횟수
    pub iterations: u32,
    
    /// 출력 키 길이 (바이트)
    pub key_length: usize,
    
    /// 해시 알고리즘 (예: "SHA256")
    pub hash_algorithm: String,
}

impl KeyDerivationParams {
    /// 기본 키 유도 매개변수를 생성합니다.
    /// 
    /// # 매개변수
    /// * `salt` - 솔트 값
    /// 
    /// # 반환값
    /// * `Self` - 기본 키 유도 매개변수
    pub fn default_with_salt(salt: Vec<u8>) -> Self {
        Self {
            salt,
            iterations: 100_000,      // OWASP 권장값
            key_length: 32,           // 256 bits
            hash_algorithm: "SHA256".to_string(),
        }
    }
    
    /// 고성능 키 유도 매개변수를 생성합니다.
    /// 
    /// 성능이 중요한 상황에서 사용하며, 보안 수준을 약간 낮춥니다.
    /// 
    /// # 매개변수
    /// * `salt` - 솔트 값
    /// 
    /// # 반환값
    /// * `Self` - 고성능 키 유도 매개변수
    pub fn fast_with_salt(salt: Vec<u8>) -> Self {
        Self {
            salt,
            iterations: 50_000,       // 성능 우선
            key_length: 32,
            hash_algorithm: "SHA256".to_string(),
        }
    }
    
    /// 고보안 키 유도 매개변수를 생성합니다.
    /// 
    /// 보안이 최우선인 상황에서 사용하며, 성능을 희생합니다.
    /// 
    /// # 매개변수
    /// * `salt` - 솔트 값
    /// 
    /// # 반환값
    /// * `Self` - 고보안 키 유도 매개변수
    pub fn secure_with_salt(salt: Vec<u8>) -> Self {
        Self {
            salt,
            iterations: 200_000,      // 보안 우선
            key_length: 32,
            hash_algorithm: "SHA256".to_string(),
        }
    }
}

/// 메모리 보안 유틸리티
/// 민감한 데이터를 메모리에서 안전하게 제거하는 기능을 제공합니다.
pub struct SecureMemory;

impl SecureMemory {
    /// 바이트 배열을 안전하게 지웁니다.
    /// 
    /// 컴파일러 최적화를 방지하여 메모리에서 실제로 데이터를 제거합니다.
    /// 
    /// # 매개변수
    /// * `data` - 지울 데이터의 가변 참조
    pub fn clear_bytes(data: &mut [u8]) {
        // volatile write를 사용하여 컴파일러 최적화 방지
        for byte in data.iter_mut() {
            unsafe {
                std::ptr::write_volatile(byte, 0);
            }
        }
    }
    
    /// 벡터를 안전하게 지웁니다.
    /// 
    /// # 매개변수
    /// * `data` - 지울 벡터의 가변 참조
    pub fn clear_vec(data: &mut Vec<u8>) {
        Self::clear_bytes(data.as_mut_slice());
        data.clear();
        data.shrink_to_fit();
    }
    
    /// 문자열을 안전하게 지웁니다.
    /// 
    /// # 매개변수
    /// * `data` - 지울 문자열의 가변 참조
    pub fn clear_string(data: &mut String) {
        unsafe {
            let bytes = data.as_bytes_mut();
            Self::clear_bytes(bytes);
        }
        data.clear();
        data.shrink_to_fit();
    }
}

/// 보안 랜덤 생성기
/// 암호학적으로 안전한 랜덤 값을 생성합니다.
pub struct SecureRandom;

impl SecureRandom {
    /// 지정된 크기의 랜덤 바이트를 생성합니다.
    /// 
    /// # 매개변수
    /// * `size` - 생성할 바이트 수
    /// 
    /// # 반환값
    /// * `Vec<u8>` - 생성된 랜덤 바이트
    pub fn generate_bytes(size: usize) -> Vec<u8> {
        use rand::RngCore;
        let mut rng = rand::thread_rng();
        let mut bytes = vec![0u8; size];
        rng.fill_bytes(&mut bytes);
        bytes
    }
    
    /// 바이트 배열을 랜덤 값으로 채웁니다.
    /// 
    /// C# RandomNumberGenerator.Fill()과 동일한 기능을 제공합니다.
    /// 
    /// # 매개변수
    /// * `buffer` - 채울 바이트 배열의 가변 참조
    pub fn fill_bytes(buffer: &mut [u8]) {
        use rand::RngCore;
        let mut rng = rand::thread_rng();
        rng.fill_bytes(buffer);
    }
    
    /// 암호화용 솔트를 생성합니다.
    /// 
    /// C# EncryptionService.GenerateSalt()와 동일한 기능을 제공합니다.
    /// 
    /// # 반환값
    /// * `[u8; 32]` - 32바이트 솔트
    pub fn generate_salt() -> [u8; 32] {
        let mut salt = [0u8; 32];
        Self::fill_bytes(&mut salt);
        salt
    }
    
    /// 논스를 생성합니다.
    /// 
    /// # 매개변수
    /// * `algorithm` - 암호화 알고리즘
    /// 
    /// # 반환값
    /// * `Vec<u8>` - 알고리즘에 맞는 크기의 논스
    pub fn generate_nonce(algorithm: &EncryptionAlgorithm) -> Vec<u8> {
        Self::generate_bytes(algorithm.nonce_size())
    }
    
    /// UUID를 생성합니다.
    /// 
    /// # 반환값
    /// * `Uuid` - 생성된 UUID
    pub fn generate_uuid() -> Uuid {
        Uuid::new_v4()
    }
}