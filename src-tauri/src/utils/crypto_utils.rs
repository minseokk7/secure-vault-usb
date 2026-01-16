// 암호화 유틸리티 함수들
// 암호화와 관련된 공통 함수들을 제공합니다.

use sha2::{Sha256, Digest};
use rand::{RngCore, thread_rng};

/// SHA-256 해시를 계산합니다.
/// 
/// # 매개변수
/// * `data` - 해시할 데이터
/// 
/// # 반환값
/// * `String` - 16진수 해시 문자열
pub fn calculate_sha256(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    hex::encode(result)
}

/// 안전한 랜덤 바이트를 생성합니다.
/// 
/// # 매개변수
/// * `length` - 생성할 바이트 수
/// 
/// # 반환값
/// * `Vec<u8>` - 랜덤 바이트 배열
pub fn generate_random_bytes(length: usize) -> Vec<u8> {
    let mut bytes = vec![0u8; length];
    thread_rng().fill_bytes(&mut bytes);
    bytes
}

/// 바이트 배열을 16진수 문자열로 변환합니다.
/// 
/// # 매개변수
/// * `bytes` - 변환할 바이트 배열
/// 
/// # 반환값
/// * `String` - 16진수 문자열
pub fn bytes_to_hex(bytes: &[u8]) -> String {
    hex::encode(bytes)
}

/// 16진수 문자열을 바이트 배열로 변환합니다.
/// 
/// # 매개변수
/// * `hex_str` - 16진수 문자열
/// 
/// # 반환값
/// * `Result<Vec<u8>, hex::FromHexError>` - 변환된 바이트 배열
pub fn hex_to_bytes(hex_str: &str) -> Result<Vec<u8>, hex::FromHexError> {
    hex::decode(hex_str)
}

/// 메모리를 안전하게 지웁니다.
/// 
/// 컴파일러 최적화를 방지하여 실제로 메모리를 덮어씁니다.
/// 
/// # 매개변수
/// * `data` - 지울 데이터의 가변 참조
pub fn secure_zero_memory(data: &mut [u8]) {
    for byte in data.iter_mut() {
        unsafe {
            std::ptr::write_volatile(byte, 0);
        }
    }
}

/// 두 바이트 배열을 상수 시간에 비교합니다.
/// 
/// 타이밍 공격을 방지하기 위해 상수 시간 비교를 수행합니다.
/// 
/// # 매개변수
/// * `a` - 첫 번째 바이트 배열
/// * `b` - 두 번째 바이트 배열
/// 
/// # 반환값
/// * `bool` - 배열이 같으면 true
pub fn constant_time_compare(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    
    let mut result = 0u8;
    for (byte_a, byte_b) in a.iter().zip(b.iter()) {
        result |= byte_a ^ byte_b;
    }
    
    result == 0
}

/// 패스워드 강도를 평가합니다.
/// 
/// # 매개변수
/// * `password` - 평가할 패스워드
/// 
/// # 반환값
/// * `u8` - 강도 점수 (0-100)
pub fn evaluate_password_strength(password: &str) -> u8 {
    let mut score = 0u8;
    
    // 길이 점수 (최대 25점)
    let length_score = std::cmp::min(password.len() * 3, 25);
    score += length_score as u8;
    
    // 문자 종류 다양성 (각각 최대 15점)
    if password.chars().any(|c| c.is_ascii_lowercase()) {
        score += 15;
    }
    if password.chars().any(|c| c.is_ascii_uppercase()) {
        score += 15;
    }
    if password.chars().any(|c| c.is_ascii_digit()) {
        score += 15;
    }
    if password.chars().any(|c| c.is_ascii_punctuation()) {
        score += 15;
    }
    
    // 반복 패턴 감점
    if has_repeating_pattern(password) {
        score = score.saturating_sub(20);
    }
    
    // 일반적인 패스워드 감점
    if is_common_password(password) {
        score = score.saturating_sub(30);
    }
    
    std::cmp::min(score, 100)
}

/// 반복 패턴이 있는지 확인합니다.
/// 
/// # 매개변수
/// * `password` - 확인할 패스워드
/// 
/// # 반환값
/// * `bool` - 반복 패턴 존재 여부
fn has_repeating_pattern(password: &str) -> bool {
    let chars: Vec<char> = password.chars().collect();
    
    // 연속된 같은 문자 3개 이상
    for window in chars.windows(3) {
        if window[0] == window[1] && window[1] == window[2] {
            return true;
        }
    }
    
    // 연속된 숫자나 문자 (예: 123, abc)
    for window in chars.windows(3) {
        if let (Some(a), Some(b), Some(c)) = (
            window[0].to_digit(36),
            window[1].to_digit(36),
            window[2].to_digit(36),
        ) {
            if b == a + 1 && c == b + 1 {
                return true;
            }
        }
    }
    
    false
}

/// 일반적인 패스워드인지 확인합니다.
/// 
/// # 매개변수
/// * `password` - 확인할 패스워드
/// 
/// # 반환값
/// * `bool` - 일반적인 패스워드 여부
fn is_common_password(password: &str) -> bool {
    const COMMON_PASSWORDS: &[&str] = &[
        "password", "123456", "password123", "admin", "qwerty",
        "letmein", "welcome", "monkey", "1234567890", "abc123",
        "password1", "123456789", "welcome123", "admin123",
        "비밀번호", "1234", "0000", "1111", "2222", "3333",
    ];
    
    let lower_password = password.to_lowercase();
    COMMON_PASSWORDS.iter().any(|&common| lower_password == common)
}

/// 엔트로피를 계산합니다.
/// 
/// # 매개변수
/// * `data` - 엔트로피를 계산할 데이터
/// 
/// # 반환값
/// * `f64` - 엔트로피 값
pub fn calculate_entropy(data: &[u8]) -> f64 {
    use std::collections::HashMap;
    
    if data.is_empty() {
        return 0.0;
    }
    
    // 각 바이트의 빈도 계산
    let mut frequency = HashMap::new();
    for &byte in data {
        *frequency.entry(byte).or_insert(0) += 1;
    }
    
    // 엔트로피 계산
    let length = data.len() as f64;
    let mut entropy = 0.0;
    
    for count in frequency.values() {
        let probability = *count as f64 / length;
        entropy -= probability * probability.log2();
    }
    
    entropy
}