// 파일 유틸리티 함수들
// 파일 처리와 관련된 공통 함수들을 제공합니다.

use std::path::Path;
use crate::SecureVaultResult;

/// 파일 확장자를 추출합니다.
/// 
/// # 매개변수
/// * `file_path` - 파일 경로
/// 
/// # 반환값
/// * `Option<String>` - 파일 확장자 (소문자)
pub fn get_file_extension(file_path: &Path) -> Option<String> {
    file_path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_lowercase())
}

/// MIME 타입을 추정합니다.
/// 
/// # 매개변수
/// * `file_path` - 파일 경로
/// 
/// # 반환값
/// * `Option<String>` - 추정된 MIME 타입
pub fn guess_mime_type(file_path: &Path) -> Option<String> {
    let extension = get_file_extension(file_path)?;
    
    match extension.as_str() {
        // 텍스트 파일
        "txt" => Some("text/plain".to_string()),
        "md" => Some("text/markdown".to_string()),
        "json" => Some("application/json".to_string()),
        "xml" => Some("application/xml".to_string()),
        "html" | "htm" => Some("text/html".to_string()),
        "css" => Some("text/css".to_string()),
        "js" => Some("application/javascript".to_string()),
        
        // 이미지 파일
        "jpg" | "jpeg" => Some("image/jpeg".to_string()),
        "png" => Some("image/png".to_string()),
        "gif" => Some("image/gif".to_string()),
        "bmp" => Some("image/bmp".to_string()),
        "svg" => Some("image/svg+xml".to_string()),
        "webp" => Some("image/webp".to_string()),
        
        // 비디오 파일
        "mp4" => Some("video/mp4".to_string()),
        "avi" => Some("video/x-msvideo".to_string()),
        "mov" => Some("video/quicktime".to_string()),
        "wmv" => Some("video/x-ms-wmv".to_string()),
        "flv" => Some("video/x-flv".to_string()),
        "webm" => Some("video/webm".to_string()),
        
        // 오디오 파일
        "mp3" => Some("audio/mpeg".to_string()),
        "wav" => Some("audio/wav".to_string()),
        "flac" => Some("audio/flac".to_string()),
        "aac" => Some("audio/aac".to_string()),
        "ogg" => Some("audio/ogg".to_string()),
        
        // 문서 파일
        "pdf" => Some("application/pdf".to_string()),
        "doc" => Some("application/msword".to_string()),
        "docx" => Some("application/vnd.openxmlformats-officedocument.wordprocessingml.document".to_string()),
        "xls" => Some("application/vnd.ms-excel".to_string()),
        "xlsx" => Some("application/vnd.openxmlformats-officedocument.spreadsheetml.sheet".to_string()),
        "ppt" => Some("application/vnd.ms-powerpoint".to_string()),
        "pptx" => Some("application/vnd.openxmlformats-officedocument.presentationml.presentation".to_string()),
        
        // 압축 파일
        "zip" => Some("application/zip".to_string()),
        "rar" => Some("application/vnd.rar".to_string()),
        "7z" => Some("application/x-7z-compressed".to_string()),
        "tar" => Some("application/x-tar".to_string()),
        "gz" => Some("application/gzip".to_string()),
        
        _ => None,
    }
}

/// 파일 크기를 사람이 읽기 쉬운 형태로 변환합니다.
/// 
/// # 매개변수
/// * `size` - 파일 크기 (바이트)
/// 
/// # 반환값
/// * `String` - 형식화된 크기 문자열
pub fn format_file_size(size: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    const THRESHOLD: f64 = 1024.0;
    
    if size == 0 {
        return "0 B".to_string();
    }
    
    let mut size_f = size as f64;
    let mut unit_index = 0;
    
    while size_f >= THRESHOLD && unit_index < UNITS.len() - 1 {
        size_f /= THRESHOLD;
        unit_index += 1;
    }
    
    if unit_index == 0 {
        format!("{} {}", size, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size_f, UNITS[unit_index])
    }
}

/// 안전한 파일명을 생성합니다.
/// 
/// 파일명에서 위험한 문자들을 제거하고 안전한 형태로 변환합니다.
/// 
/// # 매개변수
/// * `filename` - 원본 파일명
/// 
/// # 반환값
/// * `String` - 안전한 파일명
pub fn sanitize_filename(filename: &str) -> String {
    // 위험한 문자들을 언더스코어로 대체
    let dangerous_chars = ['/', '\\', ':', '*', '?', '"', '<', '>', '|'];
    let mut safe_name = filename.to_string();
    
    for ch in dangerous_chars {
        safe_name = safe_name.replace(ch, "_");
    }
    
    // 앞뒤 공백 제거
    safe_name = safe_name.trim().to_string();
    
    // 빈 문자열이면 기본값 사용
    if safe_name.is_empty() {
        safe_name = "untitled".to_string();
    }
    
    // 길이 제한 (255자)
    if safe_name.len() > 255 {
        safe_name.truncate(255);
    }
    
    safe_name
}

/// 파일 경로가 유효한지 검증합니다.
/// 
/// # 매개변수
/// * `path` - 검증할 파일 경로
/// 
/// # 반환값
/// * `bool` - 유효성 여부
pub fn is_valid_file_path(path: &Path) -> bool {
    // 경로가 존재하는지 확인
    if !path.exists() {
        return false;
    }
    
    // 파일인지 확인 (디렉토리가 아닌)
    if !path.is_file() {
        return false;
    }
    
    // 읽기 권한이 있는지 확인
    match std::fs::metadata(path) {
        Ok(metadata) => !metadata.permissions().readonly(),
        Err(_) => false,
    }
}

/// 임시 파일 경로를 생성합니다.
/// 
/// # 매개변수
/// * `prefix` - 파일명 접두사
/// * `extension` - 파일 확장자
/// 
/// # 반환값
/// * `SecureVaultResult<std::path::PathBuf>` - 임시 파일 경로
pub fn create_temp_file_path(prefix: &str, extension: &str) -> SecureVaultResult<std::path::PathBuf> {
    use uuid::Uuid;
    
    let temp_dir = std::env::temp_dir();
    let unique_id = Uuid::new_v4();
    let filename = format!("{}_{}.{}", prefix, unique_id, extension);
    
    Ok(temp_dir.join(filename))
}

/// 디렉토리 크기를 계산합니다.
/// 
/// # 매개변수
/// * `dir_path` - 디렉토리 경로
/// 
/// # 반환값
/// * `SecureVaultResult<u64>` - 디렉토리 총 크기 (바이트)
pub fn calculate_directory_size(dir_path: &Path) -> SecureVaultResult<u64> {
    use walkdir::WalkDir;
    
    let mut total_size = 0u64;
    
    for entry in WalkDir::new(dir_path) {
        let entry = entry.map_err(|e| {
            std::io::Error::new(std::io::ErrorKind::Other, e)
        })?;
        
        if entry.file_type().is_file() {
            if let Ok(metadata) = entry.metadata() {
                total_size += metadata.len();
            }
        }
    }
    
    Ok(total_size)
}