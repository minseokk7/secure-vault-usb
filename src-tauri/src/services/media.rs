use crate::models::error::VaultError;
use crate::models::file::FileEntry;
use serde::{Deserialize, Serialize};

/// 미디어 서비스
/// 미디어 파일의 메타데이터 추출 및 스트리밍 기능을 제공합니다.
pub struct MediaService {
    /// 볼트 경로
    vault_path: String,
}

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
    /// MIME 타입
    pub mime_type: String,
}

/// 미디어 타입 열거형
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MediaType {
    Audio,
    Video,
}

/// 미디어 스트림 청크
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaChunk {
    /// 청크 데이터 (Base64 인코딩)
    pub data: String,
    /// 청크 크기
    pub size: usize,
    /// 전체 파일에서의 오프셋
    pub offset: usize,
    /// 마지막 청크 여부
    pub is_last: bool,
}

impl MediaService {
    /// 새로운 미디어 서비스 인스턴스를 생성합니다.
    /// 
    /// # 매개변수
    /// * `vault_path` - 볼트 디렉토리 경로
    /// 
    /// # 반환값
    /// * `Self` - 미디어 서비스 인스턴스
    pub fn new(vault_path: &str) -> Self {
        Self {
            vault_path: vault_path.to_string(),
        }
    }

    /// 미디어 파일의 메타데이터를 추출합니다.
    /// 
    /// # 매개변수
    /// * `file_entry` - 파일 엔트리
    /// * `file_data` - 파일 데이터 (메타데이터 추출용)
    /// 
    /// # 반환값
    /// * `Result<MediaMetadata, VaultError>` - 미디어 메타데이터 또는 에러
    pub fn extract_metadata(&self, file_entry: &FileEntry, _file_data: &[u8]) -> Result<MediaMetadata, VaultError> {
        let extension = self.get_file_extension(&file_entry.file_name);
        let media_type = self.determine_media_type(&extension);
        let mime_type = self.get_mime_type(&extension);

        // 기본 메타데이터 생성
        let metadata = MediaMetadata {
            title: self.extract_title_from_filename(&file_entry.file_name),
            artist: None,
            album: None,
            duration: None,
            bitrate: None,
            sample_rate: None,
            channels: None,
            media_type,
            file_size: file_entry.file_size,
            mime_type,
        };

        // 실제 메타데이터 추출 (향후 구현)
        // TODO: 실제 미디어 라이브러리를 사용하여 메타데이터 추출
        // 현재는 파일명 기반으로만 제목 추출
        
        Ok(metadata)
    }

    /// 미디어 파일이 지원되는지 확인합니다.
    /// 
    /// # 매개변수
    /// * `file_name` - 파일명
    /// 
    /// # 반환값
    /// * `bool` - 지원 여부
    pub fn is_supported(&self, file_name: &str) -> bool {
        let extension = self.get_file_extension(file_name);
        
        const SUPPORTED_EXTENSIONS: &[&str] = &[
            // 오디오 형식
            ".mp3", ".wav", ".ogg", ".aac", ".flac", ".m4a", ".wma", ".aiff", ".ape", ".opus",
            // 비디오 형식
            ".mp4", ".webm", ".avi", ".mov", ".mkv", ".flv", ".wmv", ".m4v", ".3gp"
        ];
        
        SUPPORTED_EXTENSIONS.contains(&extension.as_str())
    }

    /// 미디어 데이터를 청크 단위로 스트리밍합니다.
    /// 
    /// # 매개변수
    /// * `file_data` - 전체 파일 데이터
    /// * `offset` - 시작 오프셋
    /// * `chunk_size` - 청크 크기
    /// 
    /// # 반환값
    /// * `Result<MediaChunk, VaultError>` - 미디어 청크 또는 에러
    pub fn get_chunk(&self, file_data: &[u8], offset: usize, chunk_size: usize) -> Result<MediaChunk, VaultError> {
        if offset >= file_data.len() {
            return Err(VaultError::DatabaseError("오프셋이 파일 크기를 초과합니다.".to_string()));
        }

        let end = std::cmp::min(offset + chunk_size, file_data.len());
        let chunk_data = &file_data[offset..end];
        let is_last = end >= file_data.len();

        // Base64로 인코딩
        use base64::{Engine as _, engine::general_purpose};
        let encoded_data = general_purpose::STANDARD.encode(chunk_data);

        Ok(MediaChunk {
            data: encoded_data,
            size: chunk_data.len(),
            offset,
            is_last,
        })
    }

    /// 작은 미디어 파일의 전체 데이터를 반환합니다.
    /// 
    /// # 매개변수
    /// * `file_data` - 파일 데이터
    /// * `max_size` - 최대 허용 크기 (바이트)
    /// 
    /// # 반환값
    /// * `Result<String, VaultError>` - Base64 인코딩된 데이터 또는 에러
    pub fn get_full_data(&self, file_data: &[u8], max_size: usize) -> Result<String, VaultError> {
        if file_data.len() > max_size {
            return Err(VaultError::DatabaseError(
                format!("파일이 너무 큽니다. 최대 {}MB까지 지원됩니다.", max_size / 1024 / 1024)
            ));
        }

        // Base64로 인코딩
        use base64::{Engine as _, engine::general_purpose};
        Ok(general_purpose::STANDARD.encode(file_data))
    }

    /// 파일 확장자를 추출합니다.
    fn get_file_extension(&self, file_name: &str) -> String {
        if let Some(pos) = file_name.rfind('.') {
            file_name[pos..].to_lowercase()
        } else {
            String::new()
        }
    }

    /// 미디어 타입을 판단합니다.
    fn determine_media_type(&self, extension: &str) -> MediaType {
        const AUDIO_EXTENSIONS: &[&str] = &[
            ".mp3", ".wav", ".ogg", ".aac", ".flac", ".m4a", ".wma", ".aiff", ".ape", ".opus"
        ];
        
        if AUDIO_EXTENSIONS.contains(&extension) {
            MediaType::Audio
        } else {
            MediaType::Video
        }
    }

    /// MIME 타입을 반환합니다.
    fn get_mime_type(&self, extension: &str) -> String {
        match extension {
            // 오디오 형식
            ".mp3" => "audio/mpeg",
            ".wav" => "audio/wav",
            ".ogg" => "audio/ogg",
            ".aac" => "audio/aac",
            ".flac" => "audio/flac",
            ".m4a" => "audio/mp4",
            ".wma" => "audio/x-ms-wma",
            ".aiff" => "audio/aiff",
            ".ape" => "audio/ape",
            ".opus" => "audio/opus",
            
            // 비디오 형식
            ".mp4" => "video/mp4",
            ".webm" => "video/webm",
            ".avi" => "video/x-msvideo",
            ".mov" => "video/quicktime",
            ".mkv" => "video/x-matroska",
            ".flv" => "video/x-flv",
            ".wmv" => "video/x-ms-wmv",
            ".m4v" => "video/mp4",
            ".3gp" => "video/3gpp",
            
            _ => "application/octet-stream",
        }.to_string()
    }

    /// 파일명에서 제목을 추출합니다.
    fn extract_title_from_filename(&self, file_name: &str) -> Option<String> {
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_media_service_creation() {
        let service = MediaService::new("/test/vault");
        assert_eq!(service.vault_path, "/test/vault");
    }

    #[test]
    fn test_file_extension_extraction() {
        let service = MediaService::new("/test");
        assert_eq!(service.get_file_extension("music.mp3"), ".mp3");
        assert_eq!(service.get_file_extension("video.MP4"), ".mp4");
        assert_eq!(service.get_file_extension("noext"), "");
    }

    #[test]
    fn test_media_type_determination() {
        let service = MediaService::new("/test");
        assert!(matches!(service.determine_media_type(".mp3"), MediaType::Audio));
        assert!(matches!(service.determine_media_type(".mp4"), MediaType::Video));
        assert!(matches!(service.determine_media_type(".flac"), MediaType::Audio));
    }

    #[test]
    fn test_mime_type_detection() {
        let service = MediaService::new("/test");
        assert_eq!(service.get_mime_type(".mp3"), "audio/mpeg");
        assert_eq!(service.get_mime_type(".mp4"), "video/mp4");
        assert_eq!(service.get_mime_type(".unknown"), "application/octet-stream");
    }

    #[test]
    fn test_media_file_support() {
        let service = MediaService::new("/test");
        assert!(service.is_supported("music.mp3"));
        assert!(service.is_supported("video.mp4"));
        assert!(!service.is_supported("document.txt"));
    }

    #[test]
    fn test_title_extraction() {
        let service = MediaService::new("/test");
        assert_eq!(service.extract_title_from_filename("My Song.mp3"), Some("My Song".to_string()));
        assert_eq!(service.extract_title_from_filename("video"), Some("video".to_string()));
        assert_eq!(service.extract_title_from_filename(""), None);
    }

    #[test]
    fn test_chunk_creation() {
        let service = MediaService::new("/test");
        let data = b"Hello, World! This is test data for chunking.";
        
        let chunk = service.get_chunk(data, 0, 10).unwrap();
        assert_eq!(chunk.offset, 0);
        assert_eq!(chunk.size, 10);
        assert!(!chunk.is_last);
        
        let last_chunk = service.get_chunk(data, 40, 10).unwrap();
        assert!(last_chunk.is_last);
    }

    #[test]
    fn test_full_data_encoding() {
        let service = MediaService::new("/test");
        let data = b"Hello, World!";
        
        let encoded = service.get_full_data(data, 1024).unwrap();
        assert!(!encoded.is_empty());
        
        // 너무 큰 파일 테스트
        let result = service.get_full_data(data, 5);
        assert!(result.is_err());
    }
}