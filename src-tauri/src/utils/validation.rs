// 검증 유틸리티 함수들
// 입력값 검증과 관련된 공통 함수들을 제공합니다.

use std::path::Path;
use regex::Regex;

/// 파일명이 유효한지 검증합니다.
/// 
/// # 매개변수
/// * `filename` - 검증할 파일명
/// 
/// # 반환값
/// * `bool` - 유효성 여부
pub fn is_valid_filename(filename: &str) -> bool {
    // 빈 문자열 체크
    if filename.is_empty() || filename.trim().is_empty() {
        return false;
    }
    
    // 길이 체크 (255자 제한)
    if filename.len() > 255 {
        return false;
    }
    
    // 위험한 문자 체크
    let dangerous_chars = ['/', '\\', ':', '*', '?', '"', '<', '>', '|', '\0'];
    if filename.chars().any(|c| dangerous_chars.contains(&c)) {
        return false;
    }
    
    // 예약된 이름 체크 (Windows)
    let reserved_names = [
        "CON", "PRN", "AUX", "NUL",
        "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8", "COM9",
        "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
    ];
    
    let name_without_ext = filename.split('.').next().unwrap_or("").to_uppercase();
    if reserved_names.contains(&name_without_ext.as_str()) {
        return false;
    }
    
    // 점으로만 구성된 이름 체크
    if filename.chars().all(|c| c == '.') {
        return false;
    }
    
    true
}

/// 폴더명이 유효한지 검증합니다.
/// 
/// # 매개변수
/// * `foldername` - 검증할 폴더명
/// 
/// # 반환값
/// * `bool` - 유효성 여부
pub fn is_valid_foldername(foldername: &str) -> bool {
    // 파일명과 동일한 규칙 적용
    is_valid_filename(foldername)
}

/// PIN 형식이 유효한지 검증합니다.
/// 
/// # 매개변수
/// * `pin` - 검증할 PIN
/// * `min_length` - 최소 길이
/// * `max_length` - 최대 길이
/// * `allow_letters` - 문자 허용 여부
/// * `allow_special` - 특수문자 허용 여부
/// 
/// # 반환값
/// * `bool` - 유효성 여부
pub fn is_valid_pin(
    pin: &str,
    min_length: usize,
    max_length: usize,
    allow_letters: bool,
    allow_special: bool,
) -> bool {
    // 길이 체크
    if pin.len() < min_length || pin.len() > max_length {
        return false;
    }
    
    // 문자 종류 체크
    for ch in pin.chars() {
        if ch.is_ascii_digit() {
            continue; // 숫자는 항상 허용
        } else if allow_letters && ch.is_ascii_alphabetic() {
            continue; // 문자 허용 시
        } else if allow_special && ch.is_ascii_punctuation() {
            continue; // 특수문자 허용 시
        } else {
            return false; // 허용되지 않는 문자
        }
    }
    
    // 최소 숫자 개수 체크 (PIN의 절반 이상은 숫자여야 함)
    let digit_count = pin.chars().filter(|c| c.is_ascii_digit()).count();
    if digit_count < pin.len() / 2 {
        return false;
    }
    
    true
}

/// 이메일 주소가 유효한지 검증합니다.
/// 
/// # 매개변수
/// * `email` - 검증할 이메일 주소
/// 
/// # 반환값
/// * `bool` - 유효성 여부
pub fn is_valid_email(email: &str) -> bool {
    // 간단한 이메일 정규식 (RFC 5322 완전 준수는 아님)
    let email_regex = Regex::new(
        r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$"
    ).unwrap();
    
    email_regex.is_match(email)
}

/// URL이 유효한지 검증합니다.
/// 
/// # 매개변수
/// * `url` - 검증할 URL
/// 
/// # 반환값
/// * `bool` - 유효성 여부
pub fn is_valid_url(url: &str) -> bool {
    // HTTP/HTTPS URL 정규식
    let url_regex = Regex::new(
        r"^https?://[a-zA-Z0-9.-]+(?:\.[a-zA-Z]{2,})?(?:/[^\s]*)?$"
    ).unwrap();
    
    url_regex.is_match(url)
}

/// 파일 경로가 안전한지 검증합니다.
/// 
/// 경로 순회 공격(Path Traversal)을 방지합니다.
/// 
/// # 매개변수
/// * `path` - 검증할 파일 경로
/// 
/// # 반환값
/// * `bool` - 안전성 여부
pub fn is_safe_path(path: &Path) -> bool {
    let path_str = path.to_string_lossy();
    
    // 위험한 패턴 체크
    let dangerous_patterns = [
        "..", // 상위 디렉토리 접근
        "~",  // 홈 디렉토리 접근
        "/etc", "/proc", "/sys", // 시스템 디렉토리 (Linux)
        "C:\\Windows", "C:\\System32", // 시스템 디렉토리 (Windows)
    ];
    
    for pattern in &dangerous_patterns {
        if path_str.contains(pattern) {
            return false;
        }
    }
    
    // 절대 경로 체크 (상대 경로만 허용)
    if path.is_absolute() {
        return false;
    }
    
    true
}

/// 문자열이 유효한 UUID인지 검증합니다.
/// 
/// # 매개변수
/// * `uuid_str` - 검증할 UUID 문자열
/// 
/// # 반환값
/// * `bool` - 유효성 여부
pub fn is_valid_uuid(uuid_str: &str) -> bool {
    use uuid::Uuid;
    Uuid::parse_str(uuid_str).is_ok()
}

/// 포트 번호가 유효한지 검증합니다.
/// 
/// # 매개변수
/// * `port` - 검증할 포트 번호
/// 
/// # 반환값
/// * `bool` - 유효성 여부
pub fn is_valid_port(port: u16) -> bool {
    // 1-65535 범위의 포트만 유효
    // 0번 포트는 예약됨
    port > 0
}

/// IP 주소가 유효한지 검증합니다.
/// 
/// # 매개변수
/// * `ip` - 검증할 IP 주소
/// 
/// # 반환값
/// * `bool` - 유효성 여부
pub fn is_valid_ip_address(ip: &str) -> bool {
    use std::net::IpAddr;
    ip.parse::<IpAddr>().is_ok()
}

/// 파일 크기가 제한 내에 있는지 검증합니다.
/// 
/// # 매개변수
/// * `size` - 파일 크기 (바이트)
/// * `max_size` - 최대 허용 크기 (바이트)
/// 
/// # 반환값
/// * `bool` - 유효성 여부
pub fn is_valid_file_size(size: u64, max_size: u64) -> bool {
    size <= max_size && size > 0
}

/// 문자열에 SQL 인젝션 패턴이 있는지 검사합니다.
/// 
/// # 매개변수
/// * `input` - 검사할 입력 문자열
/// 
/// # 반환값
/// * `bool` - 안전하면 true, 위험하면 false
pub fn is_safe_sql_input(input: &str) -> bool {
    let input_lower = input.to_lowercase();
    
    // 위험한 SQL 키워드 체크
    let dangerous_keywords = [
        "select", "insert", "update", "delete", "drop", "create",
        "alter", "exec", "execute", "union", "script", "javascript",
        "vbscript", "onload", "onerror", "onclick",
    ];
    
    for keyword in &dangerous_keywords {
        if input_lower.contains(keyword) {
            return false;
        }
    }
    
    // 위험한 문자 체크
    let dangerous_chars = ['\'', '"', ';', '-', '/', '*', '%'];
    if input.chars().any(|c| dangerous_chars.contains(&c)) {
        return false;
    }
    
    true
}

/// 입력 문자열을 HTML 이스케이프합니다.
/// 
/// # 매개변수
/// * `input` - 이스케이프할 문자열
/// 
/// # 반환값
/// * `String` - 이스케이프된 문자열
pub fn html_escape(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
        .replace('/', "&#x2F;")
}

/// 입력 문자열의 길이가 유효한지 검증합니다.
/// 
/// # 매개변수
/// * `input` - 검증할 문자열
/// * `min_length` - 최소 길이
/// * `max_length` - 최대 길이
/// 
/// # 반환값
/// * `bool` - 유효성 여부
pub fn is_valid_string_length(input: &str, min_length: usize, max_length: usize) -> bool {
    let length = input.chars().count(); // 유니코드 문자 수 기준
    length >= min_length && length <= max_length
}