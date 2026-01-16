use tauri::State;
use crate::AppState;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

/// 미디어 메타데이터 구조체
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaMetadata {
    /// 제목
    pub title: Option<String>,
    /// 아티스트
    pub artist: Option<String>,
    /// 앨범
    pub album: Option<String>,
    /// 길이 (초)
    pub duration: Option<f64>,
    /// 비트레이트
    pub bitrate: Option<u32>,
    /// 샘플레이트
    pub sample_rate: Option<u32>,
    /// 채널 수
    pub channels: Option<u16>,
    /// 미디어 타입 (audio/video)
    pub media_type: MediaType,
    /// 파일 크기
    pub file_size: u64,
}

/// 미디어 타입 열거형
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MediaType {
    Audio,
    Video,
}

/// 미디어 파일 메타데이터 추출
/// 
/// # 매개변수
/// * `file_id` - 파일 ID
/// * `app_state` - 애플리케이션 상태
/// 
/// # 반환값
/// * `Result<MediaMetadata, String>` - 미디어 메타데이터 또는 에러 메시지
#[tauri::command]
pub fn get_media_metadata(
    file_id: String,
    app_state: State<'_, Mutex<AppState>>,
) -> Result<MediaMetadata, String> {
    println!("미디어 메타데이터 추출 시작: file_id={}", file_id);
    
    let app_state = app_state.lock().map_err(|e| format!("상태 잠금 실패: {}", e))?;
    let database_service = app_state.database_service.lock().map_err(|e| format!("데이터베이스 서비스 잠금 실패: {}", e))?;
    
    // 파일 정보 가져오기
    let file_entry = database_service.get_file_metadata(&file_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "파일을 찾을 수 없습니다.".to_string())?;
    
    // 파일 확장자로 미디어 타입 판단
    let extension = get_file_extension(&file_entry.file_name);
    let media_type = determine_media_type(&extension);
    
    // 기본 메타데이터 생성 (실제 구현에서는 파일 내용을 분석)
    let metadata = MediaMetadata {
        title: extract_title_from_filename(&file_entry.file_name),
        artist: None,
        album: None,
        duration: None,
        bitrate: None,
        sample_rate: None,
        channels: None,
        media_type,
        file_size: file_entry.file_size,
    };
    
    println!("미디어 메타데이터 추출 완료: {:?}", metadata);
    Ok(metadata)
}

/// 미디어 스트림 데이터 가져오기 (청크 단위)
/// 
/// # 매개변수
/// * `file_id` - 파일 ID
/// * `offset` - 시작 오프셋
/// * `size` - 읽을 크기
/// * `app_state` - 애플리케이션 상태
/// 
/// # 반환값
/// * `Result<String, String>` - Base64 인코딩된 미디어 데이터 또는 에러 메시지
#[tauri::command]
pub fn get_media_stream(
    file_id: String,
    offset: usize,
    size: usize,
    app_state: State<'_, Mutex<AppState>>,
) -> Result<String, String> {
    println!("미디어 스트림 요청: file_id={}, offset={}, size={}", file_id, offset, size);
    
    let app_state = app_state.lock().map_err(|e| format!("상태 잠금 실패: {}", e))?;
    let mut file_service = app_state.file_service.lock().map_err(|e| format!("파일 서비스 잠금 실패: {}", e))?;
    
    // 전체 파일 데이터 가져오기 (임시 구현)
    let data = file_service.get_file_content(&file_id).map_err(|e| e.to_string())?;
    
    // 요청된 범위의 데이터 추출
    let end = std::cmp::min(offset + size, data.len());
    if offset >= data.len() {
        return Ok(String::new());
    }
    
    let chunk = &data[offset..end];
    
    // Base64로 인코딩하여 반환
    use base64::{Engine as _, engine::general_purpose};
    let encoded = general_purpose::STANDARD.encode(chunk);
    
    println!("미디어 스트림 반환: chunk_size={}", chunk.len());
    Ok(encoded)
}

/// 전체 미디어 파일 데이터 가져오기 (작은 파일용)
/// 
/// # 매개변수
/// * `file_id` - 파일 ID
/// * `app_state` - 애플리케이션 상태
/// 
/// # 반환값
/// * `Result<String, String>` - Base64 인코딩된 전체 미디어 데이터 또는 에러 메시지
#[tauri::command]
pub fn get_full_media_data(
    file_id: String,
    app_state: State<'_, Mutex<AppState>>,
) -> Result<String, String> {
    println!("전체 미디어 데이터 요청: file_id={}", file_id);
    
    let app_state = app_state.lock().map_err(|e| format!("상태 잠금 실패: {}", e))?;
    let database_service = app_state.database_service.lock().map_err(|e| format!("데이터베이스 서비스 잠금 실패: {}", e))?;
    
    // 파일 크기 확인 (10MB 제한)
    let file_entry = database_service.get_file_metadata(&file_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "파일을 찾을 수 없습니다.".to_string())?;
    const MAX_SIZE: u64 = 10 * 1024 * 1024; // 10MB
    
    if file_entry.file_size > MAX_SIZE {
        return Err(format!("파일이 너무 큽니다. 최대 {}MB까지 지원됩니다.", MAX_SIZE / 1024 / 1024));
    }
    
    // 파일 서비스에서 전체 파일 데이터 가져오기
    drop(database_service); // 락 해제
    let mut file_service = app_state.file_service.lock().map_err(|e| format!("파일 서비스 잠금 실패: {}", e))?;
    
    // 전체 파일 데이터 가져오기
    let data = file_service.get_file_content(&file_id).map_err(|e| e.to_string())?;
    
    // Base64로 인코딩하여 반환
    use base64::{Engine as _, engine::general_purpose};
    let encoded = general_purpose::STANDARD.encode(data);
    
    println!("전체 미디어 데이터 반환: size={}", file_entry.file_size);
    Ok(encoded)
}

/// 미디어 파일 지원 여부 확인
/// 
/// # 매개변수
/// * `file_name` - 파일명
/// 
/// # 반환값
/// * `bool` - 지원 여부
#[tauri::command]
pub fn is_media_file_supported(file_name: String) -> bool {
    let extension = get_file_extension(&file_name);
    
    const SUPPORTED_EXTENSIONS: &[&str] = &[
        // 오디오 형식
        ".mp3", ".wav", ".ogg", ".aac", ".flac", ".m4a", ".wma", ".aiff", ".ape", ".opus",
        // 비디오 형식
        ".mp4", ".webm", ".avi", ".mov", ".mkv", ".flv", ".wmv", ".m4v", ".3gp"
    ];
    
    SUPPORTED_EXTENSIONS.contains(&extension.as_str())
}

/// 파일 확장자 추출
fn get_file_extension(file_name: &str) -> String {
    if let Some(pos) = file_name.rfind('.') {
        file_name[pos..].to_lowercase()
    } else {
        String::new()
    }
}

/// 미디어 타입 판단
fn determine_media_type(extension: &str) -> MediaType {
    const AUDIO_EXTENSIONS: &[&str] = &[
        ".mp3", ".wav", ".ogg", ".aac", ".flac", ".m4a", ".wma", ".aiff", ".ape", ".opus"
    ];
    
    if AUDIO_EXTENSIONS.contains(&extension) {
        MediaType::Audio
    } else {
        MediaType::Video
    }
}

/// 파일명에서 제목 추출
fn extract_title_from_filename(file_name: &str) -> Option<String> {
    // 확장자 제거
    let name_without_ext = if let Some(pos) = file_name.rfind('.') {
        &file_name[..pos]
    } else {
        file_name
    };
    
    // 빈 문자열이 아니면 제목으로 사용
    if !name_without_ext.is_empty() {
        Some(name_without_ext.to_string())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_file_extension() {
        assert_eq!(get_file_extension("music.mp3"), ".mp3");
        assert_eq!(get_file_extension("video.MP4"), ".mp4");
        assert_eq!(get_file_extension("noext"), "");
    }
    
    #[test]
    fn test_media_type_determination() {
        assert!(matches!(determine_media_type(".mp3"), MediaType::Audio));
        assert!(matches!(determine_media_type(".mp4"), MediaType::Video));
        assert!(matches!(determine_media_type(".flac"), MediaType::Audio));
    }
    
    #[test]
    fn test_media_file_support() {
        assert!(is_media_file_supported("music.mp3".to_string()));
        assert!(is_media_file_supported("video.mp4".to_string()));
        assert!(!is_media_file_supported("document.txt".to_string()));
    }
    
    #[test]
    fn test_title_extraction() {
        assert_eq!(extract_title_from_filename("My Song.mp3"), Some("My Song".to_string()));
        assert_eq!(extract_title_from_filename("video"), Some("video".to_string()));
        assert_eq!(extract_title_from_filename(""), None);
    }
}