// 파일 관련 데이터 모델
// 파일 엔트리, 메타데이터, 압축 정보 등을 정의합니다.

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use std::collections::HashMap;

/// 파일 엔트리
/// 볼트에 저장된 파일의 메타데이터를 나타냅니다.
/// C# FileMetadata와 완전 호환
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    /// 파일 고유 ID (C# Id 호환)
    pub id: Uuid,
    
    /// 볼트 내 파일명 (C# FileName 호환)
    pub file_name: String,
    
    /// 원본 파일명 (C# OriginalFileName 호환)
    pub original_file_name: String,
    
    /// 원본 파일 크기 (바이트) (C# FileSize 호환)
    pub file_size: u64,
    
    /// 파일 확장자 (C# FileExtension 호환)
    pub file_extension: String,
    
    /// MIME 타입 (C# MimeType 호환)
    pub mime_type: String,
    
    /// 파일 체크섬 (C# Checksum 호환)
    pub checksum: String,
    
    /// 파일 생성 일시 (C# CreatedDate 호환)
    pub created_date: DateTime<Utc>,
    
    /// 파일 수정 일시 (C# ModifiedDate 호환)
    pub modified_date: DateTime<Utc>,
    
    /// 마지막 접근 일시 (C# LastAccessDate 호환)
    pub last_access_date: DateTime<Utc>,
    
    /// 소속 폴더 ID (C# FolderId 호환)
    pub folder_id: Option<Uuid>,
    
    /// 암호화된 파일명 (C# EncryptedFileName 호환)
    pub encrypted_file_name: String,
    
    /// 암호화된 파일 크기 (C# EncryptedSize 호환)
    pub encrypted_size: u64,
    
    /// 압축 여부 (압축이 적용되었는지)
    pub is_compressed: bool,
    
    /// 압축된 크기 (압축 전 암호화된 크기, 압축되지 않은 경우 encrypted_size와 동일)
    pub compressed_size: u64,
    
    /// 압축률 (0.0 ~ 1.0, 압축되지 않은 경우 1.0)
    pub compression_ratio: f64,
    
    /// 파일 태그 목록 (C# Tags 호환)
    pub tags: Vec<String>,
    
    /// 파일 설명 (C# Description 호환)
    pub description: String,
    
    /// 파일 버전 (C# Version 호환)
    pub version: u32,
    
    /// 즐겨찾기 여부 (C# IsFavorite 호환)
    pub is_favorite: bool,
    
    /// 삭제 표시 여부 (C# IsDeleted 호환)
    pub is_deleted: bool,
    
    /// 삭제 표시 날짜 (C# DeletedDate 호환)
    pub deleted_date: Option<DateTime<Utc>>,
    
    /// 사용자 정의 속성 (C# CustomProperties 호환)
    pub custom_properties: HashMap<String, String>,
    
    /// 파일 접근 횟수 (C# AccessCount 호환)
    pub access_count: u32,
    
    /// 파일 보안 등급 (C# SecurityLevel 호환)
    pub security_level: FileSecurityLevel,
}

impl FileEntry {
    /// 새로운 파일 엔트리를 생성합니다.
    /// 
    /// # 매개변수
    /// * `file_name` - 볼트 내 파일명
    /// * `original_file_name` - 원본 파일명
    /// * `file_size` - 파일 크기
    /// * `file_extension` - 파일 확장자
    /// * `mime_type` - MIME 타입
    /// * `checksum` - 파일 체크섬
    /// * `folder_id` - 소속 폴더 ID
    /// * `encrypted_file_name` - 암호화된 파일명
    /// * `encrypted_size` - 암호화된 파일 크기
    /// 
    /// # 반환값
    /// * `Self` - 생성된 파일 엔트리
    pub fn new(
        file_name: String,
        original_file_name: String,
        file_size: u64,
        file_extension: String,
        mime_type: String,
        checksum: String,
        folder_id: Option<Uuid>,
        encrypted_file_name: String,
        encrypted_size: u64,
    ) -> Self {
        let now = Utc::now();
        
        Self {
            id: Uuid::new_v4(),
            file_name,
            original_file_name,
            file_size,
            file_extension,
            mime_type,
            checksum,
            created_date: now,
            modified_date: now,
            last_access_date: now,
            folder_id,
            encrypted_file_name,
            encrypted_size,
            is_compressed: false,
            compressed_size: encrypted_size,
            compression_ratio: 1.0,
            tags: Vec::new(),
            description: String::new(),
            version: 1,
            is_favorite: false,
            is_deleted: false,
            deleted_date: None,
            custom_properties: HashMap::new(),
            access_count: 0,
            security_level: FileSecurityLevel::Normal,
        }
    }

    /// 압축 정보와 함께 새로운 파일 엔트리를 생성합니다.
    /// 
    /// # 매개변수
    /// * `file_name` - 볼트 내 파일명
    /// * `original_file_name` - 원본 파일명
    /// * `file_size` - 원본 파일 크기
    /// * `file_extension` - 파일 확장자
    /// * `mime_type` - MIME 타입
    /// * `checksum` - 파일 체크섬
    /// * `folder_id` - 소속 폴더 ID
    /// * `encrypted_file_name` - 암호화된 파일명
    /// * `encrypted_size` - 최종 암호화된 파일 크기
    /// * `is_compressed` - 압축 여부
    /// * `compressed_size` - 압축된 크기 (암호화 전)
    /// * `compression_ratio` - 압축률
    /// 
    /// # 반환값
    /// * `Self` - 생성된 파일 엔트리
    pub fn new_with_compression(
        file_name: String,
        original_file_name: String,
        file_size: u64,
        file_extension: String,
        mime_type: String,
        checksum: String,
        folder_id: Option<Uuid>,
        encrypted_file_name: String,
        encrypted_size: u64,
        is_compressed: bool,
        compressed_size: u64,
        compression_ratio: f64,
    ) -> Self {
        let now = Utc::now();
        
        Self {
            id: Uuid::new_v4(),
            file_name,
            original_file_name,
            file_size,
            file_extension,
            mime_type,
            checksum,
            created_date: now,
            modified_date: now,
            last_access_date: now,
            folder_id,
            encrypted_file_name,
            encrypted_size,
            is_compressed,
            compressed_size,
            compression_ratio,
            tags: Vec::new(),
            description: String::new(),
            version: 1,
            is_favorite: false,
            is_deleted: false,
            deleted_date: None,
            custom_properties: HashMap::new(),
            access_count: 0,
            security_level: FileSecurityLevel::Normal,
        }
    }
    
    /// 파일의 압축률을 계산합니다.
    /// 
    /// # 반환값
    /// * `f64` - 압축률 (0.0 ~ 1.0)
    pub fn compression_ratio(&self) -> f64 {
        if self.file_size == 0 {
            return 0.0;
        }
        
        1.0 - (self.encrypted_size as f64 / self.file_size as f64)
    }
    
    /// 파일 크기를 사람이 읽기 쉬운 형태로 반환합니다.
    /// 
    /// # 반환값
    /// * `String` - 형식화된 파일 크기 (예: "1.5 MB")
    pub fn formatted_size(&self) -> String {
        format_file_size(self.file_size)
    }
    
    /// 암호화된 파일 크기를 사람이 읽기 쉬운 형태로 반환합니다.
    /// 
    /// # 반환값
    /// * `String` - 형식화된 암호화 크기
    pub fn formatted_encrypted_size(&self) -> String {
        format_file_size(self.encrypted_size)
    }
    
    /// 파일 타입 카테고리를 반환합니다.
    /// 
    /// # 반환값
    /// * `FileCategory` - 파일 카테고리
    pub fn category(&self) -> FileCategory {
        FileCategory::from_mime_type(&Some(self.mime_type.clone()))
    }
    
    /// 마지막 접근 시간을 업데이트합니다.
    pub fn update_last_accessed(&mut self) {
        self.last_access_date = Utc::now();
        self.access_count += 1;
    }
    
    /// 파일에 태그를 추가합니다.
    /// 
    /// # 매개변수
    /// * `tag` - 추가할 태그
    pub fn add_tag(&mut self, tag: String) {
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
        }
    }
    
    /// 파일에서 태그를 제거합니다.
    /// 
    /// # 매개변수
    /// * `tag` - 제거할 태그
    pub fn remove_tag(&mut self, tag: &str) {
        self.tags.retain(|t| t != tag);
    }
}

/// 파일 보안 등급 (C# FileSecurityLevel 포팅)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum FileSecurityLevel {
    /// 일반 보안 등급
    Normal = 0,
    /// 높은 보안 등급 (추가 인증 필요)
    High = 1,
    /// 최고 보안 등급 (특별 권한 필요)
    Critical = 2,
}

impl Default for FileSecurityLevel {
    fn default() -> Self {
        Self::Normal
    }
}

impl From<i32> for FileSecurityLevel {
    fn from(value: i32) -> Self {
        match value {
            0 => Self::Normal,
            1 => Self::High,
            2 => Self::Critical,
            _ => Self::Normal,
        }
    }
}

impl FileSecurityLevel {
    /// 보안 등급의 한국어 설명을 반환합니다.
    /// 
    /// # 반환값
    /// * `&str` - 보안 등급 설명
    pub fn description(&self) -> &str {
        match self {
            Self::Normal => "일반",
            Self::High => "높음",
            Self::Critical => "최고",
        }
    }
}

/// 파일 카테고리
/// 파일 타입에 따른 분류를 제공합니다.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum FileCategory {
    /// 문서 파일
    Document,
    /// 이미지 파일
    Image,
    /// 비디오 파일
    Video,
    /// 오디오 파일
    Audio,
    /// 압축 파일
    Archive,
    /// 코드 파일
    Code,
    /// 기타
    Other,
}

impl FileCategory {
    /// MIME 타입으로부터 파일 카테고리를 결정합니다.
    /// 
    /// # 매개변수
    /// * `mime_type` - MIME 타입
    /// 
    /// # 반환값
    /// * `Self` - 파일 카테고리
    pub fn from_mime_type(mime_type: &Option<String>) -> Self {
        let mime = match mime_type {
            Some(m) => m.as_str(),
            None => return Self::Other,
        };
        
        if mime.starts_with("text/") || mime.contains("document") || mime.contains("pdf") {
            Self::Document
        } else if mime.starts_with("image/") {
            Self::Image
        } else if mime.starts_with("video/") {
            Self::Video
        } else if mime.starts_with("audio/") {
            Self::Audio
        } else if mime.contains("zip") || mime.contains("archive") || mime.contains("compressed") {
            Self::Archive
        } else if mime.contains("javascript") || mime.contains("json") || mime.contains("xml") {
            Self::Code
        } else {
            Self::Other
        }
    }
    
    /// 카테고리의 한국어 이름을 반환합니다.
    /// 
    /// # 반환값
    /// * `&str` - 카테고리 이름
    pub fn display_name(&self) -> &str {
        match self {
            Self::Document => "문서",
            Self::Image => "이미지",
            Self::Video => "비디오",
            Self::Audio => "오디오",
            Self::Archive => "압축 파일",
            Self::Code => "코드",
            Self::Other => "기타",
        }
    }
    
    /// 카테고리의 아이콘 이름을 반환합니다.
    /// 
    /// # 반환값
    /// * `&str` - 아이콘 이름
    pub fn icon_name(&self) -> &str {
        match self {
            Self::Document => "file-text",
            Self::Image => "image",
            Self::Video => "video",
            Self::Audio => "music",
            Self::Archive => "archive",
            Self::Code => "code",
            Self::Other => "file",
        }
    }
}

/// 파일 정렬 기준
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum FileSortBy {
    /// 이름순
    Name,
    /// 크기순
    Size,
    /// 생성일순
    CreatedAt,
    /// 수정일순
    ModifiedAt,
    /// 파일 타입순
    Type,
}

impl FileSortBy {
    /// 정렬 기준의 한국어 이름을 반환합니다.
    /// 
    /// # 반환값
    /// * `&str` - 정렬 기준 이름
    pub fn display_name(&self) -> &str {
        match self {
            Self::Name => "이름",
            Self::Size => "크기",
            Self::CreatedAt => "생성일",
            Self::ModifiedAt => "수정일",
            Self::Type => "파일 타입",
        }
    }
}

/// 파일 정렬 순서
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum FileSortOrder {
    /// 오름차순
    Ascending,
    /// 내림차순
    Descending,
}

/// 파일 필터 조건
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileFilter {
    /// 파일명 검색어
    pub name_contains: Option<String>,
    
    /// 파일 카테고리 필터
    pub categories: Vec<FileCategory>,
    
    /// 최소 파일 크기 (바이트)
    pub min_size: Option<u64>,
    
    /// 최대 파일 크기 (바이트)
    pub max_size: Option<u64>,
    
    /// 생성일 범위 (시작)
    pub created_after: Option<DateTime<Utc>>,
    
    /// 생성일 범위 (끝)
    pub created_before: Option<DateTime<Utc>>,
    
    /// 태그 필터
    pub tags: Vec<String>,
    
    /// 즐겨찾기만 표시
    pub favorites_only: bool,
    
    /// 삭제된 파일 제외
    pub exclude_deleted: bool,
}

impl Default for FileFilter {
    fn default() -> Self {
        Self {
            name_contains: None,
            categories: Vec::new(),
            min_size: None,
            max_size: None,
            created_after: None,
            created_before: None,
            tags: Vec::new(),
            favorites_only: false,
            exclude_deleted: true,
        }
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

/// 파일 해시를 계산합니다.
/// 
/// # 매개변수
/// * `data` - 파일 데이터
/// 
/// # 반환값
/// * `String` - SHA-256 해시 (16진수 문자열)
pub fn calculate_file_hash(data: &[u8]) -> String {
    use sha2::{Sha256, Digest};
    
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    
    hex::encode(result)
}

/// 큰 파일의 SHA-256 해시를 병렬로 계산합니다 (100MB 이상).
/// 
/// # 매개변수
/// * `data` - 해시를 계산할 데이터
/// 
/// # 반환값
/// * `String` - SHA-256 해시 (16진수 문자열)
pub fn calculate_file_hash_parallel(data: &[u8]) -> String {
    use sha2::{Sha256, Digest};
    use std::sync::{Arc, Mutex};
    use std::thread;
    
    // 작은 파일은 기존 방식 사용
    if data.len() < 100 * 1024 * 1024 { // 100MB 미만
        return calculate_file_hash(data);
    }
    
    // 병렬 처리용 청크 크기 (16MB)
    const PARALLEL_CHUNK_SIZE: usize = 16 * 1024 * 1024;
    let num_chunks = ((data.len() + PARALLEL_CHUNK_SIZE - 1) / PARALLEL_CHUNK_SIZE).max(1);
    let num_threads = std::cmp::min(num_chunks, num_cpus::get()).max(1);
    
    log::info!("병렬 해시 계산: {}MB, {} 청크, {} 스레드", 
              data.len() / (1024 * 1024), num_chunks, num_threads);
    
    // 병렬 해시 계산
    let chunk_hashes = Arc::new(Mutex::new(Vec::with_capacity(num_chunks)));
    let mut handles = Vec::new();
    
    for chunk_idx in 0..num_chunks {
        let start = chunk_idx * PARALLEL_CHUNK_SIZE;
        let end = std::cmp::min(start + PARALLEL_CHUNK_SIZE, data.len());
        let chunk_data = data[start..end].to_vec();
        
        let chunk_hashes_clone = Arc::clone(&chunk_hashes);
        
        let handle = thread::spawn(move || {
            let mut hasher = Sha256::new();
            hasher.update(&chunk_data);
            let chunk_hash = hasher.finalize();
            
            let mut hashes = chunk_hashes_clone.lock().unwrap();
            hashes.push((chunk_idx, chunk_hash.to_vec()));
        });
        
        handles.push(handle);
    }
    
    // 모든 스레드 완료 대기
    for handle in handles {
        handle.join().unwrap();
    }
    
    // 청크 해시들을 순서대로 정렬
    let mut chunk_hashes = chunk_hashes.lock().unwrap();
    chunk_hashes.sort_by_key(|(idx, _)| *idx);
    
    // 모든 청크 해시를 결합하여 최종 해시 계산
    let mut final_hasher = Sha256::new();
    for (_, chunk_hash) in chunk_hashes.iter() {
        final_hasher.update(chunk_hash);
    }
    let final_result = final_hasher.finalize();
    
    log::info!("병렬 해시 계산 완료: {} 청크 처리", num_chunks);
    hex::encode(final_result)
}