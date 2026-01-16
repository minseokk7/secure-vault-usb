// 볼트 관련 데이터 모델
// 볼트 설정, 구성 정보, 통계 등을 정의합니다.

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use std::path::PathBuf;

/// 압축 알고리즘
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CompressionAlgorithm {
    /// Zstandard (기본값, 빠르고 효율적)
    Zstd,
    /// LZ4 (매우 빠름, 낮은 압축률)
    Lz4,
    /// Deflate (호환성 좋음)
    Deflate,
    /// Brotli (높은 압축률)
    Brotli,
}

impl Default for CompressionAlgorithm {
    fn default() -> Self {
        Self::Zstd
    }
}

impl CompressionAlgorithm {
    /// 알고리즘의 한국어 이름을 반환합니다.
    /// 
    /// # 반환값
    /// * `&str` - 알고리즘 이름
    pub fn display_name(&self) -> &str {
        match self {
            Self::Zstd => "Zstandard (권장)",
            Self::Lz4 => "LZ4 (고속)",
            Self::Deflate => "Deflate (호환)",
            Self::Brotli => "Brotli (고압축)",
        }
    }
    
    /// 알고리즘의 특징을 반환합니다.
    /// 
    /// # 반환값
    /// * `&str` - 알고리즘 특징
    pub fn description(&self) -> &str {
        match self {
            Self::Zstd => "빠른 속도와 높은 압축률의 균형",
            Self::Lz4 => "매우 빠른 압축/해제 속도",
            Self::Deflate => "널리 지원되는 표준 알고리즘",
            Self::Brotli => "최고 수준의 압축률",
        }
    }
}

/// 볼트 설정 정보
/// 볼트의 전반적인 설정과 메타데이터를 저장합니다.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultConfig {
    /// 볼트 고유 ID
    pub id: Uuid,
    
    /// 볼트 이름
    pub name: String,
    
    /// 볼트 설명
    pub description: Option<String>,
    
    /// 볼트 버전
    pub version: String,
    
    /// 볼트 생성 일시
    pub created_at: DateTime<Utc>,
    
    /// 볼트 마지막 수정 일시
    pub updated_at: DateTime<Utc>,
    
    /// 마지막 접근 일시
    pub last_accessed_at: Option<DateTime<Utc>>,
    
    /// 볼트 경로
    pub vault_path: PathBuf,
    
    /// 압축 설정
    pub compression: CompressionConfig,
    
    /// 파일 이력 설정
    pub file_history: FileHistoryConfig,
    
    /// 보안 설정
    pub security: SecurityConfig,
    
    /// 백업 설정
    pub backup: BackupConfig,
    
    /// UI 설정
    pub ui: UiConfig,
    
    /// 볼트 상태
    pub status: VaultStatus,
    
    /// 볼트 크기 제한 (바이트, None이면 무제한)
    pub size_limit: Option<u64>,
    
    /// 현재 사용 중인 크기 (바이트)
    pub current_size: u64,
    
    /// 파일 개수 제한 (None이면 무제한)
    pub file_limit: Option<u32>,
    
    /// 현재 파일 개수
    pub current_file_count: u32,
}

impl VaultConfig {
    /// 새로운 볼트 설정을 생성합니다.
    /// 
    /// # 매개변수
    /// * `name` - 볼트 이름
    /// * `vault_path` - 볼트 경로
    /// 
    /// # 반환값
    /// * `Self` - 생성된 볼트 설정
    pub fn new(name: String, vault_path: PathBuf) -> Self {
        let now = Utc::now();
        
        Self {
            id: Uuid::new_v4(),
            name,
            description: None,
            version: env!("CARGO_PKG_VERSION").to_string(),
            created_at: now,
            updated_at: now,
            last_accessed_at: None,
            vault_path,
            compression: CompressionConfig::default(),
            file_history: FileHistoryConfig::default(),
            security: SecurityConfig::default(),
            backup: BackupConfig::default(),
            ui: UiConfig::default(),
            status: VaultStatus::Active,
            size_limit: None,
            current_size: 0,
            current_file_count: 0,
            file_limit: None,
        }
    }
    
    /// 볼트 사용률을 계산합니다 (0.0 ~ 1.0).
    /// 
    /// # 반환값
    /// * `f64` - 사용률 (제한이 없으면 0.0)
    pub fn usage_ratio(&self) -> f64 {
        if let Some(limit) = self.size_limit {
            if limit > 0 {
                return (self.current_size as f64) / (limit as f64);
            }
        }
        0.0
    }
    
    /// 남은 용량을 반환합니다 (바이트).
    /// 
    /// # 반환값
    /// * `Option<u64>` - 남은 용량 (제한이 없으면 None)
    pub fn remaining_space(&self) -> Option<u64> {
        self.size_limit.map(|limit| {
            if limit > self.current_size {
                limit - self.current_size
            } else {
                0
            }
        })
    }
    
    /// 파일 추가 가능 여부를 확인합니다.
    /// 
    /// # 매개변수
    /// * `file_size` - 추가할 파일 크기
    /// 
    /// # 반환값
    /// * `bool` - 추가 가능 여부
    pub fn can_add_file(&self, file_size: u64) -> bool {
        // 크기 제한 확인
        if let Some(limit) = self.size_limit {
            if self.current_size + file_size > limit {
                return false;
            }
        }
        
        // 파일 개수 제한 확인
        if let Some(limit) = self.file_limit {
            if self.current_file_count >= limit {
                return false;
            }
        }
        
        true
    }
    
    /// 마지막 접근 시간을 업데이트합니다.
    pub fn update_last_accessed(&mut self) {
        self.last_accessed_at = Some(Utc::now());
        self.updated_at = Utc::now();
    }
    
    /// 볼트 통계를 업데이트합니다.
    /// 
    /// # 매개변수
    /// * `size` - 현재 사용 크기
    /// * `file_count` - 현재 파일 개수
    pub fn update_stats(&mut self, size: u64, file_count: u32) {
        self.current_size = size;
        self.current_file_count = file_count;
        self.updated_at = Utc::now();
    }
}

/// 압축 설정
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionConfig {
    /// 압축 활성화 여부
    pub enabled: bool,
    
    /// 기본 압축 알고리즘
    pub default_algorithm: CompressionAlgorithm,
    
    /// 기본 압축 레벨 (0-9)
    pub default_level: u8,
    
    /// 자동 압축 임계값 (바이트, 이 크기 이상의 파일만 압축)
    pub auto_compress_threshold: u64,
    
    /// 압축 제외 확장자 목록
    pub exclude_extensions: Vec<String>,
    
    /// 이미 압축된 파일 재압축 여부
    pub recompress_compressed_files: bool,
}

impl Default for CompressionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            default_algorithm: CompressionAlgorithm::Zstd,
            default_level: 6,
            auto_compress_threshold: 1024 * 1024, // 1MB
            exclude_extensions: vec![
                "zip".to_string(),
                "rar".to_string(),
                "7z".to_string(),
                "gz".to_string(),
                "bz2".to_string(),
                "jpg".to_string(),
                "jpeg".to_string(),
                "png".to_string(),
                "mp3".to_string(),
                "mp4".to_string(),
                "avi".to_string(),
            ],
            recompress_compressed_files: false,
        }
    }
}

/// 파일 이력 설정
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileHistoryConfig {
    /// 파일 이력 추적 활성화 여부
    pub enabled: bool,
    
    /// 최대 보관할 버전 수 (0이면 무제한)
    pub max_versions: u32,
    
    /// 이력 보관 기간 (일, 0이면 무제한)
    pub retention_days: u32,
    
    /// 자동 정리 활성화 여부
    pub auto_cleanup: bool,
    
    /// 이력 압축 여부
    pub compress_history: bool,
    
    /// 중요 파일 이력 영구 보관 여부
    pub keep_important_files_forever: bool,
}

impl Default for FileHistoryConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_versions: 10,
            retention_days: 90,
            auto_cleanup: true,
            compress_history: true,
            keep_important_files_forever: false,
        }
    }
}

/// 보안 설정
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// PIN 복잡도 요구사항
    pub pin_complexity: PinComplexityRequirement,
    
    /// PIN 만료 기간 (일, 0이면 만료 없음)
    pub pin_expiry_days: u32,
    
    /// 자동 잠금 시간 (분, 0이면 자동 잠금 없음)
    pub auto_lock_minutes: u32,
    
    /// 브루트포스 방지 설정
    pub brute_force_protection: BruteForceConfig,
    
    /// 복구 키 활성화 여부
    pub recovery_key_enabled: bool,
    
    /// 이중 인증 활성화 여부 (향후 확장용)
    pub two_factor_enabled: bool,
    
    /// 메모리 보안 강화 여부
    pub enhanced_memory_security: bool,
    
    /// 로그 암호화 여부
    pub encrypt_logs: bool,
    
    /// 안전한 삭제 활성화 여부
    pub secure_delete: bool,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            pin_complexity: PinComplexityRequirement::Medium,
            pin_expiry_days: 0,
            auto_lock_minutes: 30,
            brute_force_protection: BruteForceConfig::default(),
            recovery_key_enabled: true,
            two_factor_enabled: false,
            enhanced_memory_security: true,
            encrypt_logs: true,
            secure_delete: true,
        }
    }
}

/// PIN 복잡도 요구사항
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PinComplexityRequirement {
    /// 낮음 (4자리 숫자)
    Low,
    /// 중간 (6자리 숫자)
    Medium,
    /// 높음 (8자리 이상, 특수문자 허용)
    High,
    /// 사용자 정의
    Custom {
        min_length: u8,
        max_length: u8,
        require_numbers: bool,
        require_letters: bool,
        require_special_chars: bool,
    },
}

impl PinComplexityRequirement {
    /// 복잡도 요구사항의 설명을 반환합니다.
    /// 
    /// # 반환값
    /// * `String` - 요구사항 설명
    pub fn description(&self) -> String {
        match self {
            Self::Low => "4자리 숫자".to_string(),
            Self::Medium => "6자리 숫자".to_string(),
            Self::High => "8자리 이상 (특수문자 허용)".to_string(),
            Self::Custom { min_length, max_length, require_numbers, require_letters, require_special_chars } => {
                let mut desc = format!("{}-{}자리", min_length, max_length);
                let mut requirements = Vec::new();
                
                if *require_numbers { requirements.push("숫자"); }
                if *require_letters { requirements.push("문자"); }
                if *require_special_chars { requirements.push("특수문자"); }
                
                if !requirements.is_empty() {
                    desc.push_str(&format!(" ({})", requirements.join(", ")));
                }
                
                desc
            }
        }
    }
}

/// 브루트포스 방지 설정
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BruteForceConfig {
    /// 최대 허용 실패 횟수
    pub max_attempts: u32,
    
    /// 기본 잠금 시간 (초)
    pub base_lockout_seconds: u64,
    
    /// 지수적 백오프 사용 여부
    pub exponential_backoff: bool,
    
    /// 최대 잠금 시간 (초)
    pub max_lockout_seconds: u64,
    
    /// IP 기반 차단 (향후 확장용)
    pub ip_blocking_enabled: bool,
}

impl Default for BruteForceConfig {
    fn default() -> Self {
        Self {
            max_attempts: 5,
            base_lockout_seconds: 30,
            exponential_backoff: true,
            max_lockout_seconds: 3600, // 1시간
            ip_blocking_enabled: false,
        }
    }
}

/// 백업 설정
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupConfig {
    /// 자동 백업 활성화 여부
    pub enabled: bool,
    
    /// 백업 간격 (시간)
    pub interval_hours: u32,
    
    /// 백업 보관 개수
    pub max_backups: u32,
    
    /// 백업 경로
    pub backup_path: Option<PathBuf>,
    
    /// 백업 압축 여부
    pub compress_backups: bool,
    
    /// 백업 암호화 여부
    pub encrypt_backups: bool,
    
    /// 증분 백업 사용 여부
    pub incremental_backup: bool,
}

impl Default for BackupConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            interval_hours: 24,
            max_backups: 7,
            backup_path: None,
            compress_backups: true,
            encrypt_backups: true,
            incremental_backup: true,
        }
    }
}

/// UI 설정
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    /// 테마 (다크/라이트)
    pub theme: UiTheme,
    
    /// 언어 설정
    pub language: String,
    
    /// 파일 목록 보기 모드
    pub default_view_mode: ViewMode,
    
    /// 기본 정렬 기준
    pub default_sort_by: String,
    
    /// 기본 정렬 순서
    pub default_sort_order: String,
    
    /// 미리보기 활성화 여부
    pub preview_enabled: bool,
    
    /// 썸네일 생성 여부
    pub generate_thumbnails: bool,
    
    /// 애니메이션 활성화 여부
    pub animations_enabled: bool,
    
    /// 알림 활성화 여부
    pub notifications_enabled: bool,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            theme: UiTheme::Auto,
            language: "ko".to_string(),
            default_view_mode: ViewMode::List,
            default_sort_by: "name".to_string(),
            default_sort_order: "asc".to_string(),
            preview_enabled: true,
            generate_thumbnails: true,
            animations_enabled: true,
            notifications_enabled: true,
        }
    }
}

/// UI 테마
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum UiTheme {
    /// 라이트 테마
    Light,
    /// 다크 테마
    Dark,
    /// 시스템 설정 따름
    Auto,
}

impl UiTheme {
    /// 테마의 한국어 이름을 반환합니다.
    /// 
    /// # 반환값
    /// * `&str` - 테마 이름
    pub fn display_name(&self) -> &str {
        match self {
            Self::Light => "라이트",
            Self::Dark => "다크",
            Self::Auto => "시스템 설정",
        }
    }
}

/// 파일 목록 보기 모드
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ViewMode {
    /// 목록 보기
    List,
    /// 격자 보기
    Grid,
    /// 상세 보기
    Details,
    /// 타일 보기
    Tiles,
}

impl ViewMode {
    /// 보기 모드의 한국어 이름을 반환합니다.
    /// 
    /// # 반환값
    /// * `&str` - 보기 모드 이름
    pub fn display_name(&self) -> &str {
        match self {
            Self::List => "목록",
            Self::Grid => "격자",
            Self::Details => "상세",
            Self::Tiles => "타일",
        }
    }
}

/// 볼트 상태
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum VaultStatus {
    /// 활성 상태
    Active,
    /// 잠금 상태
    Locked,
    /// 유지보수 모드
    Maintenance,
    /// 오류 상태
    Error,
    /// 백업 중
    Backing,
    /// 복원 중
    Restoring,
}

impl VaultStatus {
    /// 상태의 한국어 설명을 반환합니다.
    /// 
    /// # 반환값
    /// * `&str` - 상태 설명
    pub fn description(&self) -> &str {
        match self {
            Self::Active => "활성",
            Self::Locked => "잠금",
            Self::Maintenance => "유지보수",
            Self::Error => "오류",
            Self::Backing => "백업 중",
            Self::Restoring => "복원 중",
        }
    }
    
    /// 상태가 사용 가능한지 확인합니다.
    /// 
    /// # 반환값
    /// * `bool` - 사용 가능 여부
    pub fn is_available(&self) -> bool {
        matches!(self, Self::Active)
    }
}

/// 볼트 통계 정보
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultStats {
    /// 총 파일 개수
    pub total_files: u32,
    
    /// 총 폴더 개수
    pub total_folders: u32,
    
    /// 총 크기 (바이트)
    pub total_size: u64,
    
    /// 압축된 크기 (바이트)
    pub compressed_size: u64,
    
    /// 평균 압축률
    pub avg_compression_ratio: f64,
    
    /// 파일 타입별 통계
    pub file_type_stats: std::collections::HashMap<String, FileTypeStats>,
    
    /// 최근 활동 (최근 7일)
    pub recent_activity: RecentActivity,
    
    /// 통계 생성 일시
    pub generated_at: DateTime<Utc>,
}

/// 파일 타입별 통계
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileTypeStats {
    /// 파일 개수
    pub count: u32,
    
    /// 총 크기 (바이트)
    pub total_size: u64,
    
    /// 평균 크기 (바이트)
    pub avg_size: u64,
    
    /// 압축률
    pub compression_ratio: f64,
}

/// 최근 활동 통계
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentActivity {
    /// 추가된 파일 수
    pub files_added: u32,
    
    /// 삭제된 파일 수
    pub files_deleted: u32,
    
    /// 수정된 파일 수
    pub files_modified: u32,
    
    /// 접근한 파일 수
    pub files_accessed: u32,
    
    /// 총 접근 횟수
    pub total_accesses: u32,
}

impl VaultStats {
    /// 새로운 볼트 통계를 생성합니다.
    /// 
    /// # 반환값
    /// * `Self` - 초기화된 통계
    pub fn new() -> Self {
        Self {
            total_files: 0,
            total_folders: 0,
            total_size: 0,
            compressed_size: 0,
            avg_compression_ratio: 0.0,
            file_type_stats: std::collections::HashMap::new(),
            recent_activity: RecentActivity {
                files_added: 0,
                files_deleted: 0,
                files_modified: 0,
                files_accessed: 0,
                total_accesses: 0,
            },
            generated_at: Utc::now(),
        }
    }
    
    /// 전체 압축률을 계산합니다.
    /// 
    /// # 반환값
    /// * `f64` - 압축률 (0.0 ~ 1.0)
    pub fn overall_compression_ratio(&self) -> f64 {
        if self.total_size == 0 {
            return 0.0;
        }
        
        1.0 - (self.compressed_size as f64 / self.total_size as f64)
    }
    
    /// 절약된 공간을 반환합니다 (바이트).
    /// 
    /// # 반환값
    /// * `u64` - 절약된 공간
    pub fn space_saved(&self) -> u64 {
        if self.total_size > self.compressed_size {
            self.total_size - self.compressed_size
        } else {
            0
        }
    }
    
    /// 총 크기를 사람이 읽기 쉬운 형태로 반환합니다.
    /// 
    /// # 반환값
    /// * `String` - 형식화된 크기
    pub fn formatted_total_size(&self) -> String {
        crate::models::file::format_file_size(self.total_size)
    }
    
    /// 압축된 크기를 사람이 읽기 쉬운 형태로 반환합니다.
    /// 
    /// # 반환값
    /// * `String` - 형식화된 크기
    pub fn formatted_compressed_size(&self) -> String {
        crate::models::file::format_file_size(self.compressed_size)
    }
    
    /// 절약된 공간을 사람이 읽기 쉬운 형태로 반환합니다.
    /// 
    /// # 반환값
    /// * `String` - 형식화된 크기
    pub fn formatted_space_saved(&self) -> String {
        crate::models::file::format_file_size(self.space_saved())
    }
}

impl Default for VaultStats {
    fn default() -> Self {
        Self::new()
    }
}