use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use std::collections::HashMap;
use std::fmt;

/// 폴더 관련 오류 타입
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FolderError {
    /// 잘못된 폴더명
    InvalidName(String),
    /// 중복된 폴더명
    DuplicateName(String),
    /// 폴더를 찾을 수 없음
    NotFound(String),
    /// 부모 폴더를 찾을 수 없음
    ParentNotFound(String),
    /// 폴더가 비어있지 않음
    NotEmpty(String),
    /// 권한 없음
    PermissionDenied(String),
    /// 내부 오류
    InternalError(String),
}

impl fmt::Display for FolderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FolderError::InvalidName(msg) => write!(f, "잘못된 폴더명: {}", msg),
            FolderError::DuplicateName(msg) => write!(f, "중복된 폴더명: {}", msg),
            FolderError::NotFound(msg) => write!(f, "폴더를 찾을 수 없음: {}", msg),
            FolderError::ParentNotFound(msg) => write!(f, "부모 폴더를 찾을 수 없음: {}", msg),
            FolderError::NotEmpty(msg) => write!(f, "폴더가 비어있지 않음: {}", msg),
            FolderError::PermissionDenied(msg) => write!(f, "권한 없음: {}", msg),
            FolderError::InternalError(msg) => write!(f, "내부 오류: {}", msg),
        }
    }
}

impl std::error::Error for FolderError {}

/// 폴더 엔트리 (C# FolderEntry 완전 포팅)
/// 볼트 내의 폴더 정보를 나타냅니다.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FolderEntry {
    /// 폴더 고유 ID (C# Guid.NewGuid().ToString() 호환)
    pub id: Uuid,
    
    /// 폴더명 (C# Name)
    pub name: String,
    
    /// 부모 폴더 ID (C# ParentFolderId, 루트는 None)
    pub parent_id: Option<Uuid>,
    
    /// 폴더 전체 경로 (C# Path, "/" 구분자)
    pub path: String,
    
    /// 폴더 생성 일시 (C# CreatedDate)
    pub created_at: DateTime<Utc>,
    
    /// 폴더 수정 일시 (C# ModifiedDate)
    pub modified_at: DateTime<Utc>,
    
    /// 폴더 상태
    pub status: FolderStatus,
    
    /// 하위 폴더 개수 (C# SubfolderCount)
    pub subfolder_count: u32,
    
    /// 하위 파일 개수 (C# FileCount)
    pub file_count: u32,
    
    /// 총 크기 (C# TotalSize, 하위 파일들의 합계)
    pub total_size: u64,
    
    /// 하위 폴더 ID 목록 (C# ChildFolderIds)
    pub child_folder_ids: Vec<Uuid>,
    
    /// 포함된 파일 ID 목록 (C# FileIds)
    pub file_ids: Vec<Uuid>,
    
    /// 하위 폴더 목록 (계층 구조용, 런타임에만 사용)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<FolderEntry>>,
}

impl FolderEntry {
    /// 새로운 폴더 엔트리를 생성합니다.
    /// 
    /// # 매개변수
    /// * `name` - 폴더명
    /// * `parent_id` - 부모 폴더 ID
    /// * `path` - 폴더 경로
    /// 
    /// # 반환값
    /// * `Self` - 생성된 폴더 엔트리
    pub fn new(name: String, parent_id: Option<Uuid>, path: String) -> Self {
        let now = Utc::now();
        
        Self {
            id: Uuid::new_v4(),
            name,
            parent_id,
            path,
            created_at: now,
            modified_at: now,
            status: FolderStatus::Active,
            subfolder_count: 0,
            file_count: 0,
            total_size: 0,
            child_folder_ids: Vec::new(),
            file_ids: Vec::new(),
            children: None,
        }
    }
    
    /// 폴더가 루트 폴더인지 확인합니다.
    /// 
    /// # 반환값
    /// * `bool` - 루트 폴더 여부
    pub fn is_root(&self) -> bool {
        self.parent_id.is_none()
    }
    
    /// 폴더가 비어있는지 확인합니다.
    /// 
    /// # 반환값
    /// * `bool` - 빈 폴더 여부
    pub fn is_empty(&self) -> bool {
        self.subfolder_count == 0 && self.file_count == 0
    }
    
    /// 폴더 통계를 업데이트합니다.
    /// 
    /// # 매개변수
    /// * `subfolder_count` - 하위 폴더 개수
    /// * `file_count` - 하위 파일 개수
    /// * `total_size` - 총 크기
    pub fn update_stats(&mut self, subfolder_count: u32, file_count: u32, total_size: u64) {
        self.subfolder_count = subfolder_count;
        self.file_count = file_count;
        self.total_size = total_size;
        self.modified_at = Utc::now();
    }
}

/// 폴더 상태
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum FolderStatus {
    /// 활성 상태 (정상)
    Active = 0,
    /// 삭제됨 (휴지통)
    Deleted = 1,
    /// 숨김
    Hidden = 2,
    /// 읽기 전용
    ReadOnly = 3,
}

impl From<i32> for FolderStatus {
    fn from(value: i32) -> Self {
        match value {
            0 => Self::Active,
            1 => Self::Deleted,
            2 => Self::Hidden,
            3 => Self::ReadOnly,
            _ => Self::Active,
        }
    }
}

impl FolderStatus {
    /// 상태의 한국어 설명을 반환합니다.
    /// 
    /// # 반환값
    /// * `&str` - 상태 설명
    pub fn description(&self) -> &str {
        match self {
            Self::Active => "정상",
            Self::Deleted => "삭제됨",
            Self::Hidden => "숨김",
            Self::ReadOnly => "읽기 전용",
        }
    }
}

/// 폴더 트리 구조체 (C# TreeView 로직 기반)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FolderTree {
    /// 폴더 맵 (ID -> FolderEntry)
    pub folders: HashMap<Uuid, FolderEntry>,
    /// 부모-자식 관계 맵 (C# TreeNode 구조)
    pub children: HashMap<Option<Uuid>, Vec<Uuid>>,
    /// 루트 폴더 표시 (C# "볼트 루트" 노드)
    pub root_display_name: String,
}

impl FolderTree {
    /// 새로운 폴더 트리를 생성합니다.
    /// 
    /// # 반환값
    /// * `Self` - 생성된 폴더 트리
    pub fn new() -> Self {
        Self {
            folders: HashMap::new(),
            children: HashMap::new(),
            root_display_name: "볼트 루트".to_string(),
        }
    }
    
    /// 폴더를 조회합니다.
    /// 
    /// # 매개변수
    /// * `folder_id` - 폴더 ID
    /// 
    /// # 반환값
    /// * `Option<&FolderEntry>` - 폴더 엔트리
    pub fn get_folder(&self, folder_id: &Uuid) -> Option<&FolderEntry> {
        self.folders.get(folder_id)
    }
    
    /// 하위 폴더 목록을 조회합니다.
    /// 
    /// # 매개변수
    /// * `parent_id` - 부모 폴더 ID (None이면 루트)
    /// 
    /// # 반환값
    /// * `Vec<&FolderEntry>` - 하위 폴더 목록
    pub fn get_children(&self, parent_id: Option<Uuid>) -> Vec<&FolderEntry> {
        if let Some(child_ids) = self.children.get(&parent_id) {
            child_ids.iter()
                .filter_map(|id| self.folders.get(id))
                .filter(|folder| folder.status == FolderStatus::Active)
                .collect()
        } else {
            Vec::new()
        }
    }
    
    /// 폴더 경로로 폴더를 찾습니다.
    /// 
    /// # 매개변수
    /// * `path` - 폴더 경로 (예: "/문서/프로젝트")
    /// 
    /// # 반환값
    /// * `Option<&FolderEntry>` - 찾은 폴더
    pub fn find_by_path(&self, path: &str) -> Option<&FolderEntry> {
        if path == "/" {
            return None; // 루트는 실제 폴더가 아님
        }
        
        self.folders.values()
            .find(|folder| folder.path == path && folder.status == FolderStatus::Active)
    }
    
    /// 전체 폴더 개수를 반환합니다.
    /// 
    /// # 반환값
    /// * `usize` - 폴더 개수
    pub fn folder_count(&self) -> usize {
        self.folders.len()
    }
    
    /// 모든 활성 폴더를 반환합니다.
    /// 
    /// # 반환값
    /// * `Vec<&FolderEntry>` - 모든 활성 폴더 목록
    pub fn all_active_folders(&self) -> Vec<&FolderEntry> {
        self.folders.values()
            .filter(|folder| folder.status == FolderStatus::Active)
            .collect()
    }
}

impl Default for FolderTree {
    fn default() -> Self {
        Self::new()
    }
}

/// 폴더 정렬 기준
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum FolderSortBy {
    /// 이름순
    Name,
    /// 생성일순
    CreatedAt,
    /// 수정일순
    ModifiedAt,
    /// 크기순 (하위 파일 크기 합계)
    Size,
    /// 파일 개수순
    FileCount,
}

impl FolderSortBy {
    /// 정렬 기준의 한국어 이름을 반환합니다.
    /// 
    /// # 반환값
    /// * `&str` - 정렬 기준 이름
    pub fn display_name(&self) -> &str {
        match self {
            Self::Name => "이름",
            Self::CreatedAt => "생성일",
            Self::ModifiedAt => "수정일",
            Self::Size => "크기",
            Self::FileCount => "파일 개수",
        }
    }
}

/// 폴더 통계 정보
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FolderStats {
    /// 총 폴더 개수
    pub total_folders: u32,
    
    /// 총 파일 개수
    pub total_files: u32,
    
    /// 총 크기 (바이트)
    pub total_size: u64,
    
    /// 최대 깊이
    pub max_depth: u32,
    
    /// 평균 폴더당 파일 개수
    pub avg_files_per_folder: f64,
    
    /// 가장 큰 폴더 ID
    pub largest_folder_id: Option<Uuid>,
    
    /// 가장 큰 폴더 크기
    pub largest_folder_size: u64,
}

impl FolderStats {
    /// 폴더 트리로부터 통계를 계산합니다.
    /// 
    /// # 매개변수
    /// * `folder_tree` - 폴더 트리
    /// 
    /// # 반환값
    /// * `Self` - 계산된 통계
    pub fn calculate_from_tree(folder_tree: &FolderTree) -> Self {
        let folders = folder_tree.all_active_folders();
        let total_folders = folders.len() as u32;
        let total_files: u32 = folders.iter().map(|f| f.file_count).sum();
        let total_size: u64 = folders.iter().map(|f| f.total_size).sum();
        
        let (largest_folder_id, largest_folder_size) = folders.iter()
            .max_by_key(|f| f.total_size)
            .map(|f| (Some(f.id), f.total_size))
            .unwrap_or((None, 0));
        
        let avg_files_per_folder = if total_folders > 0 {
            total_files as f64 / total_folders as f64
        } else {
            0.0
        };
        
        // TODO: 최대 깊이 계산 구현
        let max_depth = 0;
        
        Self {
            total_folders,
            total_files,
            total_size,
            max_depth,
            avg_files_per_folder,
            largest_folder_id,
            largest_folder_size,
        }
    }
}