use crate::AppState;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::sync::Mutex;
use tauri::State;
use uuid::Uuid;

/// 미디어 메타데이터 구조체
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaMetadata {
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub duration: Option<f64>,
    pub bitrate: Option<u32>,
    pub sample_rate: Option<u32>,
    pub channels: Option<u16>,
    pub media_type: MediaType,
    pub file_size: u64,
    pub file_path: String, // 스트리밍을 위한 절대 경로
}

/// 미디어 타입 열거형
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MediaType {
    Audio,
    Video,
}

/// 미디어 파일 메타데이터 추출
#[tauri::command]
pub fn get_media_metadata(
    file_id: String,
    app_state: State<'_, Mutex<AppState>>,
) -> Result<MediaMetadata, String> {
    println!("미디어 메타데이터 추출 시작: file_id={}", file_id);

    let app_state = app_state
        .lock()
        .map_err(|e| format!("상태 잠금 실패: {}", e))?;
    let database_service = app_state
        .database_service
        .lock()
        .map_err(|e| format!("데이터베이스 서비스 잠금 실패: {}", e))?;

    // 파일 정보 가져오기
    let file_entry = database_service
        .get_file_metadata(&file_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "파일을 찾을 수 없습니다.".to_string())?;

    // 파일 확장자로 미디어 타입 판단
    let extension = get_file_extension(&file_entry.file_name);
    let media_type = determine_media_type(&extension);

    // 실제 파일 경로 계산 (절대 경로)
    let current_dir = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));

    // .securevault 폴더 위치 찾기 (현재 디렉토리 또는 상위 디렉토리)
    let mut vault_dir = current_dir.join(".securevault");
    if !vault_dir.exists() {
        if let Some(parent) = current_dir.parent() {
            let parent_vault = parent.join(".securevault");
            if parent_vault.exists() {
                vault_dir = parent_vault;
            }
        }
    }

    let file_path = vault_dir
        .join("files")
        .join(format!("{}.enc", file_entry.id));

    if !file_path.exists() {
        return Err(format!("실제 파일이 존재하지 않습니다: {:?}", file_path));
    }

    let file_path_str = file_path.to_string_lossy().to_string();

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
        file_path: file_path_str,
    };

    println!("미디어 메타데이터 추출 완료: {:?}", metadata);
    Ok(metadata)
}

/// 미디어 스트리밍 준비 (복호화 및 임시 파일 생성)
/// 미디어 스트리밍 준비 (복호화 및 임시 파일 생성)
#[tauri::command]
pub fn prepare_media_stream(
    file_id: String,
    app_state: State<'_, Mutex<AppState>>,
) -> Result<String, String> {
    println!("미디어 스트리밍 준비 요청: file_id={}", file_id);

    // 1. AppState 락 획득
    let app_state_guard = app_state
        .lock()
        .map_err(|e| format!("상태 잠금 실패: {}", e))?;

    // UUID 파싱
    let _file_uuid = Uuid::from_str(&file_id).map_err(|e| format!("잘못된 파일 ID 형식: {}", e))?;

    // 2. 파일 경로 계산
    let current_dir = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    let mut vault_dir = current_dir.join(".securevault");
    if !vault_dir.exists() {
        if let Some(parent) = current_dir.parent() {
            let parent_vault = parent.join(".securevault");
            if parent_vault.exists() {
                vault_dir = parent_vault;
            }
        }
    }

    let encrypted_file_path = vault_dir.join("files").join(format!("{}.enc", file_id));

    if !encrypted_file_path.exists() {
        return Err(format!(
            "암호화된 파일을 찾을 수 없습니다: {:?}",
            encrypted_file_path
        ));
    }

    // 4. 복호화 (청크 스트리밍 우선 시도, 실패 시 전체 파일 복호화 폴백)
    let decrypted_data = {
        let file_service = app_state_guard
            .file_service
            .lock()
            .map_err(|e| format!("파일 서비스 잠금 실패: {}", e))?;

        // 먼저 청크 스트리밍 복호화 시도 (개별 파일 업로드 형식)
        match file_service.decrypt_file_streaming_chunked(&encrypted_file_path) {
            Ok(data) => {
                log::info!("청크 스트리밍 복호화 성공");
                data
            }
            Err(e) => {
                log::info!("청크 복호화 실패, 전체 파일 복호화 시도: {}", e);

                // 폴백: 전체 파일을 읽어서 한 번에 복호화 (폴더 업로드 형식)
                let encrypted_data = std::fs::read(&encrypted_file_path)
                    .map_err(|e| format!("암호화된 파일 읽기 실패: {}", e))?;

                let master_key = file_service
                    .get_master_key()
                    .ok_or("마스터 키가 설정되지 않았습니다. (로그인 필요)")?;

                let crypto_service = crate::services::crypto::CryptoService::new();
                crypto_service
                    .decrypt_data_csharp_compatible(&encrypted_data, &master_key)
                    .map_err(|e2| format!("복호화 실패. 청크: {}, 전체: {}", e, e2))?
            }
        }
    };

    // 5. 파일 메타데이터에서 압축 여부 확인 및 압축 해제
    let final_data = {
        let database_service = app_state_guard
            .database_service
            .lock()
            .map_err(|e| format!("데이터베이스 서비스 잠금 실패: {}", e))?;

        if let Ok(Some(file_entry)) = database_service.get_file_metadata(&file_id) {
            if file_entry.is_compressed {
                log::info!("압축된 파일, 압축 해제 중...");
                let compression_service =
                    crate::services::compression::CompressionService::new_with_defaults();
                match compression_service.decompress_data(&decrypted_data) {
                    Ok(decompressed) => {
                        log::info!(
                            "압축 해제 완료: {} -> {} bytes",
                            decrypted_data.len(),
                            decompressed.len()
                        );
                        decompressed
                    }
                    Err(e) => {
                        log::warn!("압축 해제 실패, 원본 데이터 사용: {}", e);
                        decrypted_data
                    }
                }
            } else {
                decrypted_data
            }
        } else {
            decrypted_data
        }
    };

    // 6. 임시 파일 생성
    let temp_file_path = std::env::temp_dir().join(format!(
        "SecureVault_{}_{}",
        file_id,
        Uuid::new_v4().simple()
    ));

    std::fs::write(&temp_file_path, &final_data)
        .map_err(|e| format!("임시 파일 쓰기 실패: {}", e))?;

    let temp_path_str = temp_file_path.to_string_lossy().to_string();
    println!("미디어 스트리밍 준비 완료: {}", temp_path_str);
    Ok(temp_path_str)
}

/// 미디어 스트림 데이터 가져오기 (청크 단위)
#[tauri::command]
pub fn get_media_stream(
    file_id: String,
    offset: usize,
    size: usize,
    app_state: State<'_, Mutex<AppState>>,
) -> Result<String, String> {
    println!(
        "미디어 스트림 요청: file_id={}, offset={}, size={}",
        file_id, offset, size
    );

    let app_state = app_state
        .lock()
        .map_err(|e| format!("상태 잠금 실패: {}", e))?;
    let mut file_service = app_state
        .file_service
        .lock()
        .map_err(|e| format!("파일 서비스 잠금 실패: {}", e))?;

    // 전체 파일 데이터 가져오기 (임시 구현)
    let data = file_service
        .get_file_content(&file_id)
        .map_err(|e| e.to_string())?;

    // 요청된 범위의 데이터 추출
    let end = std::cmp::min(offset + size, data.len());
    if offset >= data.len() {
        return Ok(String::new());
    }

    let chunk = &data[offset..end];

    // Base64로 인코딩하여 반환
    use base64::{engine::general_purpose, Engine as _};
    let encoded = general_purpose::STANDARD.encode(chunk);

    println!("미디어 스트림 반환: chunk_size={}", chunk.len());
    Ok(encoded)
}

/// 전체 미디어 파일 데이터 가져오기 (작은 파일용)
#[tauri::command]
pub fn get_full_media_data(
    file_id: String,
    app_state: State<'_, Mutex<AppState>>,
) -> Result<String, String> {
    println!("전체 미디어 데이터 요청: file_id={}", file_id);

    let app_state = app_state
        .lock()
        .map_err(|e| format!("상태 잠금 실패: {}", e))?;
    let database_service = app_state
        .database_service
        .lock()
        .map_err(|e| format!("데이터베이스 서비스 잠금 실패: {}", e))?;

    // 파일 크기 확인 (10MB 제한)
    let file_entry = database_service
        .get_file_metadata(&file_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "파일을 찾을 수 없습니다.".to_string())?;
    const MAX_SIZE: u64 = 500 * 1024 * 1024; // 500MB

    if file_entry.file_size > MAX_SIZE {
        return Err(format!(
            "파일이 너무 큽니다. 최대 {}MB까지 지원됩니다.",
            MAX_SIZE / 1024 / 1024
        ));
    }

    // 파일 서비스에서 전체 파일 데이터 가져오기
    drop(database_service); // 락 해제
    let mut file_service = app_state
        .file_service
        .lock()
        .map_err(|e| format!("파일 서비스 잠금 실패: {}", e))?;

    // 전체 파일 데이터 가져오기
    let data = file_service
        .get_file_content(&file_id)
        .map_err(|e| e.to_string())?;

    // Base64로 인코딩하여 반환
    use base64::{engine::general_purpose, Engine as _};
    let encoded = general_purpose::STANDARD.encode(data);

    println!("전체 미디어 데이터 반환: size={}", file_entry.file_size);
    Ok(encoded)
}

/// 미디어 파일 지원 여부 확인
#[tauri::command]
pub fn is_media_file_supported(file_name: String) -> bool {
    let extension = get_file_extension(&file_name);

    const SUPPORTED_EXTENSIONS: &[&str] = &[
        // 오디오 형식
        ".mp3", ".wav", ".ogg", ".aac", ".flac", ".m4a", ".wma", ".aiff", ".ape", ".opus",
        // 비디오 형식
        ".mp4", ".webm", ".avi", ".mov", ".mkv", ".flv", ".wmv", ".m4v", ".3gp",
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
        ".mp3", ".wav", ".ogg", ".aac", ".flac", ".m4a", ".wma", ".aiff", ".ape", ".opus",
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
        assert_eq!(
            extract_title_from_filename("My Song.mp3"),
            Some("My Song".to_string())
        );
        assert_eq!(
            extract_title_from_filename("video"),
            Some("video".to_string())
        );
        assert_eq!(extract_title_from_filename(""), None);
    }
}
