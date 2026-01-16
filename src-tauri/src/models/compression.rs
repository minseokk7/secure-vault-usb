use serde::{Deserialize, Serialize};
use std::fmt;

/// 압축 레벨 열거형
/// C# CompressionLevel과 호환
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum CompressionLevel {
    /// 빠른 압축 (낮은 압축률, 빠른 속도)
    Fast,
    /// 보통 압축 (균형)
    Normal,
    /// 최대 압축 (높은 압축률, 느린 속도)
    Maximum,
}

impl Default for CompressionLevel {
    fn default() -> Self {
        CompressionLevel::Normal
    }
}

impl fmt::Display for CompressionLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CompressionLevel::Fast => write!(f, "빠름"),
            CompressionLevel::Normal => write!(f, "보통"),
            CompressionLevel::Maximum => write!(f, "최대"),
        }
    }
}

impl From<u8> for CompressionLevel {
    fn from(value: u8) -> Self {
        match value {
            0 => CompressionLevel::Fast,
            1 => CompressionLevel::Normal,
            2 => CompressionLevel::Maximum,
            _ => CompressionLevel::Normal,
        }
    }
}

impl From<CompressionLevel> for u8 {
    fn from(level: CompressionLevel) -> Self {
        match level {
            CompressionLevel::Fast => 0,
            CompressionLevel::Normal => 1,
            CompressionLevel::Maximum => 2,
        }
    }
}

/// 압축 결과 정보
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionResult {
    /// 원본 크기 (바이트)
    pub original_size: u64,
    /// 압축된 크기 (바이트)
    pub compressed_size: u64,
    /// 압축률 (0.0 ~ 1.0)
    pub compression_ratio: f64,
    /// 압축에 걸린 시간 (밀리초)
    pub compression_time_ms: u64,
    /// 사용된 압축 레벨
    pub compression_level: CompressionLevel,
}

impl CompressionResult {
    /// 새로운 압축 결과를 생성합니다.
    /// 
    /// # 매개변수
    /// * `original_size` - 원본 크기
    /// * `compressed_size` - 압축된 크기
    /// * `compression_time_ms` - 압축 시간 (밀리초)
    /// * `compression_level` - 압축 레벨
    /// 
    /// # 반환값
    /// * `Self` - 압축 결과
    pub fn new(
        original_size: u64,
        compressed_size: u64,
        compression_time_ms: u64,
        compression_level: CompressionLevel,
    ) -> Self {
        let compression_ratio = if original_size > 0 {
            compressed_size as f64 / original_size as f64
        } else {
            1.0
        };

        Self {
            original_size,
            compressed_size,
            compression_ratio,
            compression_time_ms,
            compression_level,
        }
    }

    /// 압축률을 백분율로 반환합니다.
    /// 
    /// # 반환값
    /// * `f64` - 압축률 백분율 (0.0 ~ 100.0)
    pub fn compression_ratio_percent(&self) -> f64 {
        self.compression_ratio * 100.0
    }

    /// 절약된 공간을 바이트로 반환합니다.
    /// 
    /// # 반환값
    /// * `u64` - 절약된 바이트 수
    pub fn space_saved(&self) -> u64 {
        if self.original_size > self.compressed_size {
            self.original_size - self.compressed_size
        } else {
            0
        }
    }

    /// 절약된 공간을 백분율로 반환합니다.
    /// 
    /// # 반환값
    /// * `f64` - 절약된 공간 백분율 (0.0 ~ 100.0)
    pub fn space_saved_percent(&self) -> f64 {
        if self.original_size > 0 {
            (self.space_saved() as f64 / self.original_size as f64) * 100.0
        } else {
            0.0
        }
    }
}

/// 압축 설정
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionSettings {
    /// 압축 활성화 여부
    pub enabled: bool,
    /// 압축 레벨
    pub level: CompressionLevel,
    /// 압축 임계값 (이 크기 이상의 파일만 압축, 바이트)
    pub threshold_bytes: u64,
    /// 압축 제외 확장자 목록
    pub excluded_extensions: Vec<String>,
}

impl Default for CompressionSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            level: CompressionLevel::Normal,
            threshold_bytes: 1024, // 1KB 이상만 압축
            excluded_extensions: vec![
                // 이미 압축된 형식들
                "zip".to_string(),
                "rar".to_string(),
                "7z".to_string(),
                "gz".to_string(),
                "bz2".to_string(),
                "xz".to_string(),
                // 이미 압축된 미디어 형식들
                "jpg".to_string(),
                "jpeg".to_string(),
                "png".to_string(),
                "gif".to_string(),
                "mp3".to_string(),
                "mp4".to_string(),
                "avi".to_string(),
                "mkv".to_string(),
                "webm".to_string(),
                "ogg".to_string(),
                "flac".to_string(),
            ],
        }
    }
}

impl CompressionSettings {
    /// 파일이 압축 대상인지 확인합니다.
    /// 
    /// # 매개변수
    /// * `file_size` - 파일 크기 (바이트)
    /// * `file_extension` - 파일 확장자
    /// 
    /// # 반환값
    /// * `bool` - 압축 대상 여부
    pub fn should_compress(&self, file_size: u64, file_extension: &str) -> bool {
        if !self.enabled {
            return false;
        }

        if file_size < self.threshold_bytes {
            return false;
        }

        let extension_lower = file_extension.to_lowercase();
        !self.excluded_extensions.contains(&extension_lower)
    }
}

/// 압축 에러 타입
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompressionError {
    /// 압축 실패
    CompressionFailed(String),
    /// 압축 해제 실패
    DecompressionFailed(String),
    /// 잘못된 압축 데이터
    InvalidCompressedData,
    /// 지원하지 않는 압축 레벨
    UnsupportedCompressionLevel,
    /// 입출력 오류
    IoError(String),
    /// 메모리 부족
    OutOfMemory,
}

impl fmt::Display for CompressionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CompressionError::CompressionFailed(msg) => write!(f, "압축 실패: {}", msg),
            CompressionError::DecompressionFailed(msg) => write!(f, "압축 해제 실패: {}", msg),
            CompressionError::InvalidCompressedData => write!(f, "잘못된 압축 데이터입니다."),
            CompressionError::UnsupportedCompressionLevel => write!(f, "지원하지 않는 압축 레벨입니다."),
            CompressionError::IoError(msg) => write!(f, "입출력 오류: {}", msg),
            CompressionError::OutOfMemory => write!(f, "메모리가 부족합니다."),
        }
    }
}

impl std::error::Error for CompressionError {}

impl From<std::io::Error> for CompressionError {
    fn from(error: std::io::Error) -> Self {
        CompressionError::IoError(error.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compression_level_conversion() {
        // u8에서 CompressionLevel로 변환 테스트
        assert_eq!(CompressionLevel::from(0), CompressionLevel::Fast);
        assert_eq!(CompressionLevel::from(1), CompressionLevel::Normal);
        assert_eq!(CompressionLevel::from(2), CompressionLevel::Maximum);
        assert_eq!(CompressionLevel::from(99), CompressionLevel::Normal); // 기본값

        // CompressionLevel에서 u8로 변환 테스트
        assert_eq!(u8::from(CompressionLevel::Fast), 0);
        assert_eq!(u8::from(CompressionLevel::Normal), 1);
        assert_eq!(u8::from(CompressionLevel::Maximum), 2);
    }

    #[test]
    fn test_compression_result() {
        let result = CompressionResult::new(1000, 600, 50, CompressionLevel::Normal);
        
        assert_eq!(result.original_size, 1000);
        assert_eq!(result.compressed_size, 600);
        assert_eq!(result.compression_ratio, 0.6);
        assert_eq!(result.compression_ratio_percent(), 60.0);
        assert_eq!(result.space_saved(), 400);
        assert_eq!(result.space_saved_percent(), 40.0);
    }

    #[test]
    fn test_compression_settings_should_compress() {
        let settings = CompressionSettings::default();
        
        // 크기가 임계값보다 작은 경우
        assert!(!settings.should_compress(500, "txt"));
        
        // 크기가 임계값보다 크지만 제외된 확장자인 경우
        assert!(!settings.should_compress(2000, "jpg"));
        assert!(!settings.should_compress(2000, "zip"));
        
        // 압축 대상인 경우
        assert!(settings.should_compress(2000, "txt"));
        assert!(settings.should_compress(2000, "doc"));
        
        // 압축이 비활성화된 경우
        let mut disabled_settings = settings.clone();
        disabled_settings.enabled = false;
        assert!(!disabled_settings.should_compress(2000, "txt"));
    }
}