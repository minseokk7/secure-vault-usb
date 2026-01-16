use crate::models::error::VaultError;
use crate::services::file::FileService;
use encoding_rs::{Encoding, UTF_8, EUC_KR, WINDOWS_1252};

/// 파일 뷰어 서비스
/// 볼트 내 파일을 안전하게 읽어서 뷰어에 제공합니다.
#[derive(Debug)]
pub struct ViewerService {
    file_service: FileService,
}

impl ViewerService {
    /// 새로운 뷰어 서비스 인스턴스를 생성합니다.
    pub fn new(file_service: FileService) -> Self {
        Self { file_service }
    }

    /// 파일 내용을 텍스트로 읽기
    /// 
    /// # 매개변수
    /// * `file_id` - 파일 ID
    /// 
    /// # 반환값
    /// * `Result<String, VaultError>` - 텍스트 내용 또는 에러
    pub fn get_text_content(&mut self, file_id: &str) -> Result<String, VaultError> {
        // 파일 데이터 읽기
        let file_data = self.file_service.get_file_content(file_id)?;
        
        // 파일 크기 제한 (10MB)
        const MAX_TEXT_SIZE: usize = 10 * 1024 * 1024;
        if file_data.len() > MAX_TEXT_SIZE {
            return Err(VaultError::FileTooLarge {
                size: file_data.len(),
                max_size: MAX_TEXT_SIZE,
            });
        }
        
        // 인코딩 감지 및 변환
        let (text, _encoding, _had_errors) = self.detect_encoding_and_decode(&file_data);
        
        Ok(text.into_owned())
    }
    
    /// 파일 내용을 바이너리로 읽기 (이미지/미디어용)
    /// 
    /// # 매개변수
    /// * `file_id` - 파일 ID
    /// 
    /// # 반환값
    /// * `Result<Vec<u8>, VaultError>` - 바이너리 데이터 또는 에러
    pub fn get_binary_content(&mut self, file_id: &str) -> Result<Vec<u8>, VaultError> {
        // 파일 크기 제한 (100MB)
        const MAX_BINARY_SIZE: usize = 100 * 1024 * 1024;
        
        let file_data = self.file_service.get_file_content(file_id)?;
        
        if file_data.len() > MAX_BINARY_SIZE {
            return Err(VaultError::FileTooLarge {
                size: file_data.len(),
                max_size: MAX_BINARY_SIZE,
            });
        }
        
        Ok(file_data)
    }
    
    /// 텍스트 파일 내용 저장
    /// 
    /// # 매개변수
    /// * `file_id` - 파일 ID
    /// * `content` - 저장할 텍스트 내용
    /// 
    /// # 반환값
    /// * `Result<(), VaultError>` - 성공 또는 에러
    pub fn save_text_content(&mut self, file_id: &str, content: &str) -> Result<(), VaultError> {
        // UTF-8로 인코딩
        let data = content.as_bytes().to_vec();
        
        // 파일 서비스를 통해 저장
        self.file_service.update_file_content(file_id, data)
    }
    
    /// 파일의 MIME 타입 감지
    /// 
    /// # 매개변수
    /// * `file_name` - 파일명
    /// * `data` - 파일 데이터 (선택사항, 더 정확한 감지를 위해)
    /// 
    /// # 반환값
    /// * `String` - MIME 타입
    pub fn detect_mime_type(&self, file_name: &str, data: Option<&[u8]>) -> String {
        // 파일 확장자 기반 MIME 타입 감지
        let guess = mime_guess::from_path(file_name);
        
        if let Some(mime) = guess.first() {
            return mime.to_string();
        }
        
        // 데이터 기반 감지 (매직 넘버)
        if let Some(data) = data {
            if let Some(mime) = self.detect_mime_from_data(data) {
                return mime;
            }
        }
        
        // 기본값
        "application/octet-stream".to_string()
    }
    
    /// 인코딩 감지 및 텍스트 변환
    /// 
    /// # 매개변수
    /// * `data` - 바이너리 데이터
    /// 
    /// # 반환값
    /// * `(String, &'static Encoding, bool)` - (텍스트, 인코딩, 에러 여부)
    fn detect_encoding_and_decode<'a>(&self, data: &'a [u8]) -> (std::borrow::Cow<'a, str>, &'static Encoding, bool) {
        // BOM 확인
        if data.len() >= 3 && data[0] == 0xEF && data[1] == 0xBB && data[2] == 0xBF {
            // UTF-8 BOM
            return UTF_8.decode(&data[3..]);
        }
        
        if data.len() >= 2 {
            if data[0] == 0xFF && data[1] == 0xFE {
                // UTF-16 LE BOM
                return encoding_rs::UTF_16LE.decode(&data[2..]);
            }
            if data[0] == 0xFE && data[1] == 0xFF {
                // UTF-16 BE BOM
                return encoding_rs::UTF_16BE.decode(&data[2..]);
            }
        }
        
        // UTF-8 검증 시도
        if let Ok(text) = std::str::from_utf8(data) {
            return (std::borrow::Cow::Borrowed(text), UTF_8, false);
        }
        
        // EUC-KR 시도 (한국어 지원)
        let (text, encoding, had_errors) = EUC_KR.decode(data);
        if !had_errors || self.is_likely_korean(&text) {
            return (text, encoding, had_errors);
        }
        
        // Windows-1252 시도 (서유럽 언어)
        let (text, encoding, had_errors) = WINDOWS_1252.decode(data);
        (text, encoding, had_errors)
    }
    
    /// 텍스트가 한국어일 가능성 확인
    /// 
    /// # 매개변수
    /// * `text` - 확인할 텍스트
    /// 
    /// # 반환값
    /// * `bool` - 한국어 가능성 여부
    fn is_likely_korean(&self, text: &str) -> bool {
        let korean_chars = text.chars()
            .filter(|c| {
                // 한글 음절 (가-힣)
                (*c >= '\u{AC00}' && *c <= '\u{D7AF}') ||
                // 한글 자모 (ㄱ-ㅎ, ㅏ-ㅣ)
                (*c >= '\u{3131}' && *c <= '\u{318E}')
            })
            .count();
        
        let total_chars = text.chars().filter(|c| !c.is_whitespace()).count();
        
        // 전체 문자의 10% 이상이 한글이면 한국어로 판단
        total_chars > 0 && (korean_chars as f64 / total_chars as f64) > 0.1
    }
    
    /// 데이터에서 MIME 타입 감지 (매직 넘버 기반)
    /// 
    /// # 매개변수
    /// * `data` - 파일 데이터
    /// 
    /// # 반환값
    /// * `Option<String>` - 감지된 MIME 타입
    fn detect_mime_from_data(&self, data: &[u8]) -> Option<String> {
        if data.len() < 4 {
            return None;
        }
        
        // 이미지 형식
        if data.starts_with(&[0xFF, 0xD8, 0xFF]) {
            return Some("image/jpeg".to_string());
        }
        if data.starts_with(&[0x89, 0x50, 0x4E, 0x47]) {
            return Some("image/png".to_string());
        }
        if data.starts_with(b"GIF8") {
            return Some("image/gif".to_string());
        }
        if data.starts_with(b"RIFF") && data.len() > 8 && &data[8..12] == b"WEBP" {
            return Some("image/webp".to_string());
        }
        
        // 오디오 형식
        if data.starts_with(b"ID3") || (data.len() > 2 && data[0] == 0xFF && (data[1] & 0xE0) == 0xE0) {
            return Some("audio/mpeg".to_string());
        }
        if data.starts_with(b"RIFF") && data.len() > 8 && &data[8..12] == b"WAVE" {
            return Some("audio/wav".to_string());
        }
        if data.starts_with(b"OggS") {
            return Some("audio/ogg".to_string());
        }
        if data.starts_with(b"fLaC") {
            return Some("audio/flac".to_string());
        }
        
        // 비디오 형식
        if data.len() > 8 && &data[4..8] == b"ftyp" {
            return Some("video/mp4".to_string());
        }
        if data.starts_with(&[0x1A, 0x45, 0xDF, 0xA3]) {
            return Some("video/webm".to_string());
        }
        
        // 텍스트 형식 (UTF-8 BOM 확인)
        if data.starts_with(&[0xEF, 0xBB, 0xBF]) {
            return Some("text/plain".to_string());
        }
        
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_korean_detection() {
        let service = ViewerService::new(FileService::new("test".into()));
        
        // 한국어 텍스트
        assert!(service.is_likely_korean("안녕하세요 테스트입니다"));
        assert!(service.is_likely_korean("ㄱㄴㄷㄹ"));
        
        // 영어 텍스트
        assert!(!service.is_likely_korean("Hello World"));
        assert!(!service.is_likely_korean("123456"));
    }
    
    #[test]
    fn test_mime_detection() {
        let service = ViewerService::new(FileService::new("test".into()));
        
        // JPEG 매직 넘버
        let jpeg_data = vec![0xFF, 0xD8, 0xFF, 0xE0];
        assert_eq!(
            service.detect_mime_from_data(&jpeg_data),
            Some("image/jpeg".to_string())
        );
        
        // PNG 매직 넘버
        let png_data = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
        assert_eq!(
            service.detect_mime_from_data(&png_data),
            Some("image/png".to_string())
        );
    }
}