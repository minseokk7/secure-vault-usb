use crate::models::folder::{FolderEntry, FolderError, FolderTree, FolderStatus};
use std::collections::HashMap;
use std::sync::Mutex;
use uuid::Uuid;

/// 폴더 관리 서비스
/// C# FolderManager와 MainForm의 폴더 기능을 완전히 포팅
/// 계층적 폴더 구조 생성, 삭제, 이름 변경, 트리 관리 기능 제공
#[derive(Debug)]
pub struct FolderService {
    /// 폴더 목록 (ID -> FolderEntry)
    folders: Mutex<HashMap<Uuid, FolderEntry>>,
    /// 부모-자식 관계 맵 (부모 ID -> 자식 ID 목록)
    parent_child_map: Mutex<HashMap<Option<Uuid>, Vec<Uuid>>>,
}

impl FolderService {
    /// 새로운 폴더 서비스 생성
    pub fn new() -> Self {
        Self {
            folders: Mutex::new(HashMap::new()),
            parent_child_map: Mutex::new(HashMap::new()),
        }
    }

    /// 새 폴더 생성 (C# CreateFolderAsync 포팅)
    /// 
    /// # 매개변수
    /// * `name` - 폴더명
    /// * `parent_id` - 부모 폴더 ID (None이면 루트)
    /// 
    /// # 반환값
    /// * `Ok(FolderEntry)` - 생성된 폴더 엔트리
    /// * `Err(FolderError)` - 폴더 생성 실패
    pub fn create_folder(&self, name: &str, parent_id: Option<Uuid>) -> Result<FolderEntry, FolderError> {
        if name.trim().is_empty() {
            return Err(FolderError::InvalidName("폴더명이 유효하지 않습니다.".to_string()));
        }

        // 폴더명 유효성 검사 (C# IsValidFolderName 포팅)
        if !self.is_valid_folder_name(name) {
            return Err(FolderError::InvalidName("폴더명에 사용할 수 없는 문자가 포함되어 있습니다.".to_string()));
        }

        let mut folders = self.folders.lock().unwrap();
        let mut parent_child_map = self.parent_child_map.lock().unwrap();

        // 중복 폴더명 확인 (C# IsFolderNameExists 포팅)
        if self.is_folder_name_exists_internal(&folders, name, parent_id) {
            return Err(FolderError::DuplicateName(format!("'{}' 폴더가 이미 존재합니다.", name)));
        }

        // 부모 폴더 존재 확인
        if let Some(parent_id) = parent_id {
            if !folders.contains_key(&parent_id) {
                return Err(FolderError::ParentNotFound("부모 폴더를 찾을 수 없습니다.".to_string()));
            }
        }

        // 폴더 경로 계산
        let path = self.calculate_folder_path_internal(&folders, name, parent_id);

        // 새 폴더 엔트리 생성
        let folder_id = Uuid::new_v4();
        let mut folder_entry = FolderEntry::new(name.to_string(), parent_id, path);
        folder_entry.id = folder_id;

        // 폴더 추가
        folders.insert(folder_id, folder_entry.clone());

        // 부모-자식 관계 업데이트
        parent_child_map.entry(parent_id).or_insert_with(Vec::new).push(folder_id);

        // 부모 폴더의 하위 폴더 수 업데이트 (C# 로직 포팅)
        if let Some(parent_id) = parent_id {
            if let Some(parent_folder) = folders.get_mut(&parent_id) {
                parent_folder.subfolder_count += 1;
                parent_folder.modified_at = chrono::Utc::now();
                parent_folder.child_folder_ids.push(folder_id);
            }
        }

        log::info!("폴더 생성 완료: {} (ID: {})", name, folder_id);
        Ok(folder_entry)
    }

    /// 폴더 삭제 (C# DeleteFolderAsync 포팅)
    /// 
    /// # 매개변수
    /// * `folder_id` - 삭제할 폴더 ID
    /// * `recursive` - 하위 폴더와 파일도 함께 삭제할지 여부
    /// 
    /// # 반환값
    /// * `Ok(())` - 삭제 성공
    /// * `Err(FolderError)` - 삭제 실패
    pub fn delete_folder(&self, folder_id: Uuid, recursive: bool) -> Result<(), FolderError> {
        let mut folders = self.folders.lock().unwrap();
        let mut parent_child_map = self.parent_child_map.lock().unwrap();

        // 폴더 존재 확인
        let folder = folders.get(&folder_id)
            .ok_or_else(|| FolderError::NotFound("폴더를 찾을 수 없습니다.".to_string()))?
            .clone();

        // 하위 폴더 확인 (C# 로직 포팅)
        let child_folders = parent_child_map.get(&Some(folder_id)).cloned().unwrap_or_default();
        if !child_folders.is_empty() && !recursive {
            return Err(FolderError::NotEmpty("폴더에 하위 폴더가 있습니다. 재귀 삭제를 사용하세요.".to_string()));
        }

        // 폴더 내 파일 확인
        if folder.file_count > 0 && !recursive {
            return Err(FolderError::NotEmpty("폴더에 파일이 있습니다. 재귀 삭제를 사용하세요.".to_string()));
        }

        // 재귀 삭제 (C# 로직 포팅)
        if recursive {
            // 하위 폴더들을 먼저 삭제 (깊이 우선 탐색)
            for child_id in child_folders {
                self.delete_folder_internal(&mut folders, &mut parent_child_map, child_id, true)?;
            }
        }

        // 폴더 삭제
        self.delete_folder_internal(&mut folders, &mut parent_child_map, folder_id, false)?;

        log::info!("폴더 삭제 완료: {} (ID: {})", folder.name, folder_id);
        Ok(())
    }

    /// 폴더 이름 변경 (C# OnRenameFolderFromContext 포팅)
    /// 
    /// # 매개변수
    /// * `folder_id` - 이름을 변경할 폴더 ID
    /// * `new_name` - 새 폴더명
    /// 
    /// # 반환값
    /// * `Ok(())` - 이름 변경 성공
    /// * `Err(FolderError)` - 이름 변경 실패
    pub fn rename_folder(&self, folder_id: Uuid, new_name: &str) -> Result<(), FolderError> {
        if new_name.trim().is_empty() {
            return Err(FolderError::InvalidName("폴더명이 유효하지 않습니다.".to_string()));
        }

        if !self.is_valid_folder_name(new_name) {
            return Err(FolderError::InvalidName("폴더명에 사용할 수 없는 문자가 포함되어 있습니다.".to_string()));
        }

        let mut folders = self.folders.lock().unwrap();

        // 폴더 존재 확인 및 정보 가져오기
        let (parent_id, old_name, old_path) = {
            let folder = folders.get(&folder_id)
                .ok_or_else(|| FolderError::NotFound("폴더를 찾을 수 없습니다.".to_string()))?;
            (folder.parent_id, folder.name.clone(), folder.path.clone())
        };

        // 중복 이름 확인
        if self.is_folder_name_exists_internal(&folders, new_name, parent_id) {
            return Err(FolderError::DuplicateName(format!("'{}' 폴더가 이미 존재합니다.", new_name)));
        }

        // 새 경로 계산
        let new_path = self.calculate_folder_path_internal(&folders, new_name, parent_id);

        // 폴더 이름 및 경로 업데이트
        let folder = folders.get_mut(&folder_id).unwrap();
        folder.name = new_name.to_string();
        folder.modified_at = chrono::Utc::now();
        folder.path = new_path.clone();

        // 하위 폴더들의 경로 업데이트 (C# 로직 포팅)
        self.update_subfolder_paths_internal(&mut folders, folder_id, &old_path, &new_path);

        log::info!("폴더 이름 변경 완료: {} -> {} (ID: {})", old_name, new_name, folder_id);
        Ok(())
    }

    /// 폴더 트리 구조 조회 (C# RefreshFolderTree 포팅)
    /// 
    /// # 반환값
    /// * `Ok(FolderTree)` - 계층적 폴더 구조
    /// * `Err(FolderError)` - 조회 실패
    pub fn get_folder_tree(&self) -> Result<FolderTree, FolderError> {
        let folders = self.folders.lock().unwrap();
        let parent_child_map = self.parent_child_map.lock().unwrap();

        let folder_tree = FolderTree {
            folders: folders.clone(),
            children: parent_child_map.clone(),
            root_display_name: "볼트 루트".to_string(),
        };

        Ok(folder_tree)
    }

    /// 특정 폴더의 하위 폴더 목록 조회 (C# GetSubfolders 포팅)
    /// 
    /// # 매개변수
    /// * `parent_id` - 부모 폴더 ID (None이면 루트)
    /// 
    /// # 반환값
    /// * `Ok(Vec<FolderEntry>)` - 하위 폴더 목록
    /// * `Err(FolderError)` - 조회 실패
    pub fn get_subfolders(&self, parent_id: Option<Uuid>) -> Result<Vec<FolderEntry>, FolderError> {
        let folders = self.folders.lock().unwrap();
        let parent_child_map = self.parent_child_map.lock().unwrap();

        let child_ids = parent_child_map.get(&parent_id).cloned().unwrap_or_default();
        let mut subfolders = Vec::new();

        for child_id in child_ids {
            if let Some(folder) = folders.get(&child_id).cloned() {
                if folder.status == FolderStatus::Active {
                    // TODO: 실시간 폴더 크기 및 개수 계산은 데이터베이스 서비스와 연동 후 구현
                    // 현재는 기본값 사용
                    subfolders.push(folder);
                }
            }
        }

        // 이름순으로 정렬
        subfolders.sort_by(|a, b| a.name.cmp(&b.name));

        Ok(subfolders)
    }

    /// 폴더 경로 계산 (C# 경로 로직 포팅)
    /// 
    /// # 매개변수
    /// * `folder_id` - 폴더 ID
    /// 
    /// # 반환값
    /// * `Ok(String)` - 폴더 전체 경로
    /// * `Err(FolderError)` - 경로 계산 실패
    pub fn get_folder_path(&self, folder_id: Uuid) -> Result<String, FolderError> {
        let folders = self.folders.lock().unwrap();
        
        let folder = folders.get(&folder_id)
            .ok_or_else(|| FolderError::NotFound("폴더를 찾을 수 없습니다.".to_string()))?;

        Ok(folder.path.clone())
    }

    /// 현재 선택된 폴더 ID 가져오기 (C# GetCurrentFolderId 포팅)
    /// 
    /// # 매개변수
    /// * `selected_path` - 선택된 경로
    /// 
    /// # 반환값
    /// * `Some(Uuid)` - 폴더 ID
    /// * `None` - 루트 또는 찾을 수 없음
    pub fn get_current_folder_id(&self, selected_path: &str) -> Option<Uuid> {
        if selected_path.is_empty() || selected_path == "/" {
            return None; // 루트
        }

        let folders = self.folders.lock().unwrap();
        
        for (id, folder) in folders.iter() {
            if folder.path == selected_path && folder.status == FolderStatus::Active {
                return Some(*id);
            }
        }

        None
    }

    /// 폴더 정보 조회
    /// 
    /// # 매개변수
    /// * `folder_id` - 폴더 ID
    /// 
    /// # 반환값
    /// * `Some(FolderEntry)` - 폴더 정보
    /// * `None` - 폴더를 찾을 수 없음
    pub fn get_folder(&self, folder_id: Uuid) -> Option<FolderEntry> {
        let folders = self.folders.lock().unwrap();
        folders.get(&folder_id).cloned()
    }

    /// 모든 활성 폴더 목록 조회
    /// 
    /// # 반환값
    /// * `Vec<FolderEntry>` - 활성 폴더 목록
    pub fn get_all_folders(&self) -> Vec<FolderEntry> {
        let folders = self.folders.lock().unwrap();
        folders.values()
            .filter(|folder| folder.status == FolderStatus::Active)
            .cloned()
            .collect()
    }

    /// 폴더 통계 업데이트
    /// 
    /// # 매개변수
    /// * `folder_id` - 폴더 ID
    /// * `file_count_delta` - 파일 수 변화량
    /// * `size_delta` - 크기 변화량
    /// 
    /// # 반환값
    /// * `Ok(())` - 업데이트 성공
    /// * `Err(FolderError)` - 업데이트 실패
    pub fn update_folder_stats(&self, folder_id: Uuid, file_count_delta: i32, size_delta: i64) -> Result<(), FolderError> {
        let mut folders = self.folders.lock().unwrap();
        
        let folder = folders.get_mut(&folder_id)
            .ok_or_else(|| FolderError::NotFound("폴더를 찾을 수 없습니다.".to_string()))?;

        // 파일 수 업데이트
        if file_count_delta < 0 && folder.file_count < (-file_count_delta) as u32 {
            folder.file_count = 0;
        } else {
            folder.file_count = (folder.file_count as i32 + file_count_delta).max(0) as u32;
        }

        // 크기 업데이트
        if size_delta < 0 && folder.total_size < (-size_delta) as u64 {
            folder.total_size = 0;
        } else {
            folder.total_size = (folder.total_size as i64 + size_delta).max(0) as u64;
        }

        folder.modified_at = chrono::Utc::now();

        Ok(())
    }

    // === 내부 헬퍼 메서드들 ===

    /// 폴더명 유효성 검사 (C# IsValidFolderName 포팅)
    fn is_valid_folder_name(&self, name: &str) -> bool {
        // Windows 파일명 제한 문자 확인
        let invalid_chars = ['<', '>', ':', '"', '|', '?', '*', '/', '\\'];
        
        if name.chars().any(|c| invalid_chars.contains(&c)) {
            return false;
        }

        // 예약된 이름 확인
        let reserved_names = [
            "CON", "PRN", "AUX", "NUL",
            "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8", "COM9",
            "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9"
        ];

        let upper_name = name.to_uppercase();
        if reserved_names.contains(&upper_name.as_str()) {
            return false;
        }

        // 길이 제한 (255자)
        if name.len() > 255 {
            return false;
        }

        // 공백으로만 구성된 이름 금지
        if name.trim().is_empty() {
            return false;
        }

        true
    }

    /// 중복 폴더명 확인 (C# IsFolderNameExists 포팅)
    fn is_folder_name_exists_internal(&self, folders: &HashMap<Uuid, FolderEntry>, name: &str, parent_id: Option<Uuid>) -> bool {
        folders.values().any(|folder| {
            folder.name.eq_ignore_ascii_case(name) && 
            folder.parent_id == parent_id && 
            folder.status == FolderStatus::Active
        })
    }

    /// 폴더 경로 계산 (내부용)
    fn calculate_folder_path_internal(&self, folders: &HashMap<Uuid, FolderEntry>, name: &str, parent_id: Option<Uuid>) -> String {
        if let Some(parent_id) = parent_id {
            if let Some(parent_folder) = folders.get(&parent_id) {
                if parent_folder.path == "/" {
                    format!("/{}", name)
                } else {
                    format!("{}/{}", parent_folder.path, name)
                }
            } else {
                format!("/{}", name)
            }
        } else {
            format!("/{}", name)
        }
    }

    /// 하위 폴더들의 경로 업데이트 (재귀)
    fn update_subfolder_paths_internal(&self, folders: &mut HashMap<Uuid, FolderEntry>, folder_id: Uuid, old_path: &str, new_path: &str) {
        let parent_child_map = self.parent_child_map.lock().unwrap();
        
        if let Some(child_ids) = parent_child_map.get(&Some(folder_id)) {
            for child_id in child_ids {
                if let Some(child_folder) = folders.get_mut(child_id) {
                    if child_folder.path.starts_with(old_path) {
                        child_folder.path = child_folder.path.replace(old_path, new_path);
                        child_folder.modified_at = chrono::Utc::now();
                        
                        // 재귀적으로 하위 폴더들도 업데이트
                        self.update_subfolder_paths_internal(folders, *child_id, old_path, new_path);
                    }
                }
            }
        }
    }

    /// 폴더 삭제 (내부용)
    fn delete_folder_internal(&self, folders: &mut HashMap<Uuid, FolderEntry>, parent_child_map: &mut HashMap<Option<Uuid>, Vec<Uuid>>, folder_id: Uuid, _is_recursive: bool) -> Result<(), FolderError> {
        // 폴더 정보 가져오기
        let folder = folders.get(&folder_id)
            .ok_or_else(|| FolderError::NotFound("폴더를 찾을 수 없습니다.".to_string()))?
            .clone();

        // 부모 폴더에서 자식 목록 제거
        if let Some(parent_children) = parent_child_map.get_mut(&folder.parent_id) {
            parent_children.retain(|&id| id != folder_id);
        }

        // 자식 폴더 목록 제거
        parent_child_map.remove(&Some(folder_id));

        // 부모 폴더의 하위 폴더 수 업데이트
        if let Some(parent_id) = folder.parent_id {
            if let Some(parent_folder) = folders.get_mut(&parent_id) {
                parent_folder.subfolder_count = parent_folder.subfolder_count.saturating_sub(1);
                parent_folder.modified_at = chrono::Utc::now();
                parent_folder.child_folder_ids.retain(|&id| id != folder_id);
            }
        }

        // 폴더 제거
        folders.remove(&folder_id);

        Ok(())
    }
}



impl Default for FolderService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_folder() {
        // 폴더 서비스 생성
        let service = FolderService::new();
        
        // 루트 폴더 생성
        let folder = service.create_folder("테스트폴더", None).unwrap();
        assert_eq!(folder.name, "테스트폴더");
        assert_eq!(folder.parent_id, None);
        assert_eq!(folder.path, "/테스트폴더");
        assert_eq!(folder.status, FolderStatus::Active);
    }

    #[test]
    fn test_create_subfolder() {
        // 폴더 서비스 생성
        let service = FolderService::new();
        
        // 부모 폴더 생성
        let parent = service.create_folder("부모폴더", None).unwrap();
        
        // 하위 폴더 생성
        let child = service.create_folder("자식폴더", Some(parent.id)).unwrap();
        assert_eq!(child.name, "자식폴더");
        assert_eq!(child.parent_id, Some(parent.id));
        assert_eq!(child.path, "/부모폴더/자식폴더");
    }

    #[test]
    fn test_duplicate_folder_name() {
        // 폴더 서비스 생성
        let service = FolderService::new();
        
        // 첫 번째 폴더 생성
        service.create_folder("중복폴더", None).unwrap();
        
        // 같은 이름으로 두 번째 폴더 생성 시도
        let result = service.create_folder("중복폴더", None);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), FolderError::DuplicateName(_)));
    }

    #[test]
    fn test_invalid_folder_name() {
        // 폴더 서비스 생성
        let service = FolderService::new();
        
        // 잘못된 문자가 포함된 폴더명
        let result = service.create_folder("잘못된<폴더>명", None);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), FolderError::InvalidName(_)));
        
        // 빈 폴더명
        let result = service.create_folder("", None);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), FolderError::InvalidName(_)));
    }

    #[test]
    fn test_rename_folder() {
        // 폴더 서비스 생성
        let service = FolderService::new();
        
        // 폴더 생성
        let folder = service.create_folder("원래이름", None).unwrap();
        
        // 폴더 이름 변경
        service.rename_folder(folder.id, "새이름").unwrap();
        
        // 변경된 이름 확인
        let updated_folder = service.get_folder(folder.id).unwrap();
        assert_eq!(updated_folder.name, "새이름");
        assert_eq!(updated_folder.path, "/새이름");
    }

    #[test]
    fn test_delete_folder() {
        // 폴더 서비스 생성
        let service = FolderService::new();
        
        // 폴더 생성
        let folder = service.create_folder("삭제할폴더", None).unwrap();
        
        // 폴더 삭제
        service.delete_folder(folder.id, false).unwrap();
        
        // 삭제 확인
        assert!(service.get_folder(folder.id).is_none());
    }

    #[test]
    fn test_get_folder_tree() {
        // 폴더 서비스 생성
        let service = FolderService::new();
        
        // 여러 폴더 생성
        let root1 = service.create_folder("루트1", None).unwrap();
        let root2 = service.create_folder("루트2", None).unwrap();
        let child1 = service.create_folder("자식1", Some(root1.id)).unwrap();
        
        // 폴더 트리 조회
        let tree = service.get_folder_tree().unwrap();
        
        // 트리 구조 확인
        assert_eq!(tree.folders.len(), 3);
        assert!(tree.folders.contains_key(&root1.id));
        assert!(tree.folders.contains_key(&root2.id));
        assert!(tree.folders.contains_key(&child1.id));
    }
}