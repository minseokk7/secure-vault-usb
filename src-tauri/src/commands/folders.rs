use crate::models::folder::FolderEntry;
use crate::AppState;
use tauri::State;
use uuid::Uuid;
use std::sync::Mutex;

/// 테스트용 간단한 폴더 생성 커맨드
#[tauri::command]
pub async fn test_create_folder(name: String) -> Result<String, String> {
    log::info!("테스트 폴더 생성 요청: {}", name);
    
    if name.trim().is_empty() {
        return Err("폴더명이 비어있습니다.".to_string());
    }
    
    // 간단한 성공 응답
    Ok(format!("테스트 폴더 '{}' 생성 성공", name))
}

/// 새 폴더 생성 커맨드 (C# OnCreateFolder 포팅)
/// 
/// # 매개변수
/// * `name` - 폴더명
/// * `parentId` - 부모 폴더 ID (선택사항, None이면 루트)
/// * `state` - 애플리케이션 상태
/// 
/// # 반환값
/// * `Ok(FolderEntry)` - 생성된 폴더 정보
/// * `Err(String)` - 오류 메시지 (한국어)
#[tauri::command]
pub async fn create_folder(
    name: String,
    parent_id: Option<String>,
    state: State<'_, Mutex<AppState>>
) -> Result<FolderEntry, String> {
    log::info!("폴더 생성 요청: name={}, parent_id={:?}", name, parent_id);
    
    let app_state = state.lock().map_err(|e| {
        log::error!("상태 잠금 실패: {}", e);
        format!("상태 잠금 실패: {}", e)
    })?;
    
    let folder_service = &app_state.folder_service;
    
    // 부모 ID 변환
    let parent_uuid = if let Some(id_str) = parent_id {
        match Uuid::parse_str(&id_str) {
            Ok(uuid) => {
                log::info!("부모 폴더 ID 파싱 성공: {}", uuid);
                Some(uuid)
            },
            Err(e) => {
                log::error!("부모 폴더 ID 파싱 실패: {} -> {}", id_str, e);
                return Err("올바르지 않은 부모 폴더 ID 형식입니다.".to_string());
            }
        }
    } else {
        log::info!("루트 폴더에 생성");
        None
    };
    
    match folder_service.create_folder(&name, parent_uuid) {
        Ok(folder_entry) => {
            log::info!("폴더 생성 성공: {} (ID: {})", name, folder_entry.id);
            
            // 데이터베이스에 폴더 메타데이터 저장
            let database_service = app_state.database_service.lock()
                .map_err(|e| format!("데이터베이스 서비스 잠금 실패: {}", e))?;
            
            if let Err(e) = database_service.add_folder(&folder_entry) {
                log::error!("폴더 메타데이터 저장 실패: {}", e);
                // 메타데이터 저장 실패해도 폴더 생성은 성공으로 처리
            }
            
            Ok(folder_entry)
        },
        Err(e) => {
            log::error!("폴더 생성 실패: {}", e);
            Err(format!("폴더 생성 실패: {}", e))
        }
    }
}

/// 폴더 삭제 커맨드 (C# OnDeleteFolderFromContext 포팅)
/// 
/// # 매개변수
/// * `folderId` - 삭제할 폴더 ID
/// * `recursive` - 하위 폴더와 파일도 함께 삭제할지 여부
/// * `state` - 애플리케이션 상태
/// 
/// # 반환값
/// * `Ok(())` - 삭제 성공
/// * `Err(String)` - 오류 메시지 (한국어)
#[tauri::command]
pub async fn delete_folder(
    folder_id: String,
    recursive: bool,
    state: State<'_, Mutex<AppState>>
) -> Result<(), String> {
    log::info!("폴더 삭제 요청: folder_id={}, recursive={}", folder_id, recursive);
    
    let folder_uuid = Uuid::parse_str(&folder_id)
        .map_err(|e| {
            log::error!("폴더 ID 파싱 실패: {} -> {}", folder_id, e);
            "올바르지 않은 폴더 ID 형식입니다.".to_string()
        })?;
    
    let app_state = state.lock().map_err(|e| {
        log::error!("상태 잠금 실패: {}", e);
        format!("상태 잠금 실패: {}", e)
    })?;
    
    // 먼저 데이터베이스에서 폴더 존재 확인
    let database_service = app_state.database_service.lock()
        .map_err(|e| {
            log::error!("데이터베이스 서비스 잠금 실패: {}", e);
            format!("데이터베이스 서비스 잠금 실패: {}", e)
        })?;
    
    // 폴더 존재 확인
    let folder_exists = match database_service.get_folder(&folder_uuid) {
        Ok(Some(_)) => true,
        Ok(None) => {
            log::warn!("삭제하려는 폴더를 찾을 수 없음: {}", folder_uuid);
            return Err("폴더를 찾을 수 없습니다.".to_string());
        },
        Err(e) => {
            log::error!("폴더 조회 실패: {}", e);
            return Err(format!("폴더 조회 실패: {}", e));
        }
    };
    
    if !folder_exists {
        return Err("폴더를 찾을 수 없습니다.".to_string());
    }
    
    // 하위 폴더 확인 (recursive가 false인 경우)
    if !recursive {
        let subfolder_count = database_service.count_subfolders(Some(folder_uuid))
            .map_err(|e| format!("하위 폴더 조회 실패: {}", e))?;
        
        if subfolder_count > 0 {
            return Err("폴더에 하위 폴더가 있습니다. 재귀 삭제를 사용하세요.".to_string());
        }
        
        // 폴더 내 파일 확인
        let file_count = database_service.count_files_in_folder(Some(folder_uuid))
            .map_err(|e| format!("폴더 내 파일 조회 실패: {}", e))?;
        
        if file_count > 0 {
            return Err("폴더에 파일이 있습니다. 재귀 삭제를 사용하세요.".to_string());
        }
    }
    
    // 재귀 삭제인 경우 하위 폴더들을 먼저 삭제
    if recursive {
        // 모든 폴더를 가져와서 하위 폴더 찾기
        let all_folders = database_service.get_all_folders()
            .map_err(|e| format!("폴더 목록 조회 실패: {}", e))?;
        
        let subfolders: Vec<_> = all_folders.into_iter()
            .filter(|folder| folder.parent_id == Some(folder_uuid))
            .collect();
        
        for subfolder in subfolders {
            // 재귀적으로 하위 폴더 삭제
            if let Err(e) = database_service.remove_folder(&subfolder.id) {
                log::error!("하위 폴더 삭제 실패: {} -> {}", subfolder.id, e);
            }
        }
        
        // 폴더 내 모든 파일 삭제
        let files = database_service.get_files_by_folder(Some(folder_uuid))
            .map_err(|e| format!("폴더 내 파일 조회 실패: {}", e))?;
        
        for file in files {
            if let Err(e) = database_service.remove_file(&file.id) {
                log::error!("폴더 내 파일 삭제 실패: {} -> {}", file.id, e);
            }
        }
    }
    
    // 데이터베이스에서 폴더 삭제
    database_service.remove_folder(&folder_uuid)
        .map_err(|e| {
            log::error!("폴더 메타데이터 삭제 실패: {}", e);
            format!("폴더 삭제 실패: {}", e)
        })?;
    
    // 메모리에서도 폴더 삭제 (FolderService는 현재 사용하지 않으므로 생략)
    
    log::info!("폴더 삭제 완료: ID={}", folder_uuid);
    Ok(())
}

/// 폴더 이름 변경 커맨드 (C# OnRenameFolderFromContext 포팅)
/// 
/// # 매개변수
/// * `folderId` - 이름을 변경할 폴더 ID
/// * `newName` - 새 폴더명
/// * `state` - 애플리케이션 상태
/// 
/// # 반환값
/// * `Ok(())` - 이름 변경 성공
/// * `Err(String)` - 오류 메시지 (한국어)
#[tauri::command]
pub async fn rename_folder(
    folder_id: String,
    new_name: String,
    state: State<'_, Mutex<AppState>>
) -> Result<(), String> {
    log::info!("폴더 이름 변경 요청: folder_id={}, new_name={}", folder_id, new_name);
    
    let folder_uuid = Uuid::parse_str(&folder_id)
        .map_err(|e| {
            log::error!("폴더 ID 파싱 실패: {} -> {}", folder_id, e);
            "올바르지 않은 폴더 ID 형식입니다.".to_string()
        })?;
    
    // 새 폴더명 유효성 검사
    let trimmed_name = new_name.trim();
    if trimmed_name.is_empty() {
        log::error!("폴더명이 비어있습니다");
        return Err("폴더명이 비어있습니다.".to_string());
    }
    
    // 폴더명에 허용되지 않는 문자 검사
    let invalid_chars = ['<', '>', ':', '"', '|', '?', '*', '/', '\\'];
    if trimmed_name.chars().any(|c| invalid_chars.contains(&c)) {
        log::error!("폴더명에 허용되지 않는 문자가 포함되어 있습니다: {}", trimmed_name);
        return Err("폴더명에 다음 문자는 사용할 수 없습니다: < > : \" | ? * / \\".to_string());
    }
    
    let app_state = state.lock().map_err(|e| {
        log::error!("상태 잠금 실패: {}", e);
        format!("상태 잠금 실패: {}", e)
    })?;
    
    let database_service = app_state.database_service.lock()
        .map_err(|e| {
            log::error!("데이터베이스 서비스 잠금 실패: {}", e);
            format!("데이터베이스 서비스 잠금 실패: {}", e)
        })?;
    
    // 폴더 존재 확인
    let mut folder_entry = match database_service.get_folder(&folder_uuid) {
        Ok(Some(folder)) => folder,
        Ok(None) => {
            log::error!("폴더를 찾을 수 없습니다: {}", folder_uuid);
            return Err("폴더를 찾을 수 없습니다.".to_string());
        },
        Err(e) => {
            log::error!("폴더 조회 실패: {}", e);
            return Err(format!("폴더 조회 실패: {}", e));
        }
    };
    
    // 같은 부모 폴더 내에서 중복 이름 검사
    let all_folders = database_service.get_all_folders()
        .map_err(|e| {
            log::error!("폴더 목록 조회 실패: {}", e);
            format!("폴더 목록 조회 실패: {}", e)
        })?;
    
    // 현재 폴더를 제외하고 같은 부모를 가진 폴더들 중에서 같은 이름이 있는지 확인
    for existing_folder in all_folders {
        if existing_folder.id != folder_uuid 
            && existing_folder.parent_id == folder_entry.parent_id 
            && existing_folder.name.eq_ignore_ascii_case(trimmed_name) {
            log::error!("같은 위치에 동일한 이름의 폴더가 이미 존재합니다: {}", trimmed_name);
            return Err("같은 위치에 동일한 이름의 폴더가 이미 존재합니다.".to_string());
        }
    }
    
    // 폴더명 업데이트
    let old_name = folder_entry.name.clone();
    folder_entry.name = trimmed_name.to_string();
    folder_entry.modified_at = chrono::Utc::now();
    
    // 데이터베이스에서 폴더 정보 업데이트
    if let Err(e) = database_service.update_folder(&folder_entry) {
        log::error!("폴더 정보 업데이트 실패: {}", e);
        return Err(format!("폴더 정보 업데이트 실패: {}", e));
    }
    
    log::info!("폴더 이름 변경 완료: {} -> {} (ID: {})", old_name, trimmed_name, folder_uuid);
    Ok(())
}

/// 폴더 트리 조회 커맨드 (C# RefreshFolderTree 포팅)
/// 
/// # 매개변수
/// * `state` - 애플리케이션 상태
/// 
/// # 반환값
/// * `Ok(Vec<FolderEntry>)` - 폴더 목록 (계층 구조 포함)
/// * `Err(String)` - 오류 메시지 (한국어)
#[tauri::command]
pub async fn get_folder_tree(
    state: State<'_, Mutex<AppState>>
) -> Result<Vec<FolderEntry>, String> {
    let app_state = state.lock().map_err(|e| format!("상태 잠금 실패: {}", e))?;
    
    // 데이터베이스에서 모든 폴더 로드
    let database_service = app_state.database_service.lock()
        .map_err(|e| format!("데이터베이스 서비스 잠금 실패: {}", e))?;
    
    match database_service.get_all_folders() {
        Ok(all_folders) => {
            log::info!("폴더 트리 조회 완료: {} 개 폴더", all_folders.len());
            
            // 계층 구조로 변환
            let folder_tree = build_folder_tree(all_folders);
            Ok(folder_tree)
        },
        Err(e) => {
            log::error!("폴더 트리 조회 실패: {}", e);
            
            // 실패 시 빈 배열 반환
            Ok(Vec::new())
        }
    }
}

/// 폴더 목록을 계층 구조로 변환합니다.
/// 
/// # 매개변수
/// * `folders` - 평면 폴더 목록
/// 
/// # 반환값
/// * `Vec<FolderEntry>` - 계층 구조 폴더 목록
fn build_folder_tree(mut folders: Vec<FolderEntry>) -> Vec<FolderEntry> {
    use std::collections::HashMap;
    
    // 폴더 ID를 키로 하는 맵 생성
    let mut folder_map: HashMap<uuid::Uuid, FolderEntry> = HashMap::new();
    let mut root_folders = Vec::new();
    
    // 모든 폴더를 맵에 저장
    for folder in folders.drain(..) {
        folder_map.insert(folder.id, folder);
    }
    
    // 부모-자식 관계 설정
    let folder_ids: Vec<uuid::Uuid> = folder_map.keys().cloned().collect();
    
    for folder_id in folder_ids {
        if let Some(folder) = folder_map.remove(&folder_id) {
            if let Some(parent_id) = folder.parent_id {
                // 부모 폴더가 있는 경우
                if let Some(parent_folder) = folder_map.get_mut(&parent_id) {
                    if parent_folder.children.is_none() {
                        parent_folder.children = Some(Vec::new());
                    }
                    parent_folder.children.as_mut().unwrap().push(folder);
                } else {
                    // 부모 폴더를 찾을 수 없는 경우 루트로 처리
                    root_folders.push(folder);
                }
            } else {
                // 루트 폴더
                root_folders.push(folder);
            }
        }
    }
    
    // 남은 폴더들을 루트에 추가 (부모가 이미 처리된 경우)
    for (_, folder) in folder_map {
        root_folders.push(folder);
    }
    
    // 이름순으로 정렬
    root_folders.sort_by(|a, b| a.name.cmp(&b.name));
    
    // 각 폴더의 자식들도 정렬
    fn sort_children(folder: &mut FolderEntry) {
        if let Some(ref mut children) = folder.children {
            children.sort_by(|a, b| a.name.cmp(&b.name));
            for child in children {
                sort_children(child);
            }
        }
    }
    
    for folder in &mut root_folders {
        sort_children(folder);
    }
    
    root_folders
}

/// 하위 폴더 목록 조회 커맨드
/// 
/// # 매개변수
/// * `parent_id` - 부모 폴더 ID (선택사항, None이면 루트)
/// * `state` - 애플리케이션 상태
/// 
/// # 반환값
/// * `Ok(Vec<FolderEntry>)` - 하위 폴더 목록
/// * `Err(String)` - 오류 메시지 (한국어)
#[tauri::command]
pub async fn get_subfolders(
    parent_id: Option<String>,
    state: State<'_, Mutex<AppState>>
) -> Result<Vec<FolderEntry>, String> {
    let app_state = state.lock().map_err(|e| format!("상태 잠금 실패: {}", e))?;
    
    // 부모 ID 변환
    let parent_uuid = if let Some(id_str) = parent_id {
        Some(Uuid::parse_str(&id_str)
            .map_err(|_| "올바르지 않은 부모 폴더 ID 형식입니다.".to_string())?)
    } else {
        None
    };
    
    // 데이터베이스에서 모든 폴더를 가져와서 필터링
    let database_service = app_state.database_service.lock()
        .map_err(|e| format!("데이터베이스 서비스 잠금 실패: {}", e))?;
    
    match database_service.get_all_folders() {
        Ok(all_folders) => {
            // 부모 ID가 일치하는 폴더들만 필터링
            let subfolders: Vec<FolderEntry> = all_folders
                .into_iter()
                .filter(|folder| folder.parent_id == parent_uuid)
                .collect();
            
            log::info!("하위 폴더 조회 완료: parent_id={:?}, count={}", parent_uuid, subfolders.len());
            Ok(subfolders)
        },
        Err(e) => {
            log::error!("하위 폴더 조회 실패: {}", e);
            
            // 실패 시 폴더 서비스에서 조회
            let folder_service = &app_state.folder_service;
            folder_service
                .get_subfolders(parent_uuid)
                .map_err(|e| format!("하위 폴더 조회 실패: {}", e))
        }
    }
}

/// 폴더 정보 조회 커맨드
/// 
/// # 매개변수
/// * `folder_id` - 폴더 ID
/// * `state` - 애플리케이션 상태
/// 
/// # 반환값
/// * `Ok(Option<FolderEntry>)` - 폴더 정보 (없으면 None)
/// * `Err(String)` - 오류 메시지 (한국어)
#[tauri::command]
pub async fn get_folder(
    folder_id: String,
    state: State<'_, Mutex<AppState>>
) -> Result<Option<FolderEntry>, String> {
    let app_state = state.lock().map_err(|e| format!("상태 잠금 실패: {}", e))?;
    let folder_service = &app_state.folder_service;
    
    let folder_uuid = Uuid::parse_str(&folder_id)
        .map_err(|_| "올바르지 않은 폴더 ID 형식입니다.".to_string())?;
    
    Ok(folder_service.get_folder(folder_uuid))
}

/// 폴더 경로 조회 커맨드 (C# GetFolderPath 포팅)
/// 
/// # 매개변수
/// * `folder_id` - 폴더 ID
/// * `state` - 애플리케이션 상태
/// 
/// # 반환값
/// * `Ok(String)` - 폴더 전체 경로
/// * `Err(String)` - 오류 메시지 (한국어)
#[tauri::command]
pub async fn get_folder_path(
    folder_id: String,
    state: State<'_, Mutex<AppState>>
) -> Result<String, String> {
    let app_state = state.lock().map_err(|e| format!("상태 잠금 실패: {}", e))?;
    let folder_service = &app_state.folder_service;
    
    let folder_uuid = Uuid::parse_str(&folder_id)
        .map_err(|_| "올바르지 않은 폴더 ID 형식입니다.".to_string())?;
    
    folder_service
        .get_folder_path(folder_uuid)
        .map_err(|e| format!("폴더 경로 조회 실패: {}", e))
}

/// 현재 선택된 폴더 ID 조회 커맨드 (C# GetCurrentFolderId 포팅)
/// 
/// # 매개변수
/// * `selected_path` - 선택된 경로
/// * `state` - 애플리케이션 상태
/// 
/// # 반환값
/// * `Ok(Option<String>)` - 폴더 ID (없으면 None)
/// * `Err(String)` - 오류 메시지 (한국어)
#[tauri::command]
pub async fn get_current_folder_id(
    selected_path: String,
    state: State<'_, Mutex<AppState>>
) -> Result<Option<String>, String> {
    let app_state = state.lock().map_err(|e| format!("상태 잠금 실패: {}", e))?;
    let folder_service = &app_state.folder_service;
    
    let folder_id = folder_service.get_current_folder_id(&selected_path);
    Ok(folder_id.map(|id| id.to_string()))
}

/// 모든 폴더 목록 조회 커맨드
/// 
/// # 매개변수
/// * `state` - 애플리케이션 상태
/// 
/// # 반환값
/// * `Ok(Vec<FolderEntry>)` - 모든 활성 폴더 목록
/// * `Err(String)` - 오류 메시지 (한국어)
#[tauri::command]
pub async fn get_all_folders(
    state: State<'_, Mutex<AppState>>
) -> Result<Vec<FolderEntry>, String> {
    let app_state = state.lock().map_err(|e| format!("상태 잠금 실패: {}", e))?;
    let folder_service = &app_state.folder_service;
    Ok(folder_service.get_all_folders())
}

/// 폴더 통계 업데이트 커맨드
/// 
/// # 매개변수
/// * `folder_id` - 폴더 ID
/// * `file_count_delta` - 파일 수 변화량
/// * `size_delta` - 크기 변화량
/// * `state` - 애플리케이션 상태
/// 
/// # 반환값
/// * `Ok(())` - 업데이트 성공
/// * `Err(String)` - 오류 메시지 (한국어)
#[tauri::command]
pub async fn update_folder_stats(
    folder_id: String,
    file_count_delta: i32,
    size_delta: i64,
    state: State<'_, Mutex<AppState>>
) -> Result<(), String> {
    let app_state = state.lock().map_err(|e| format!("상태 잠금 실패: {}", e))?;
    let folder_service = &app_state.folder_service;
    
    let folder_uuid = Uuid::parse_str(&folder_id)
        .map_err(|_| "올바르지 않은 폴더 ID 형식입니다.".to_string())?;
    
    folder_service
        .update_folder_stats(folder_uuid, file_count_delta, size_delta)
        .map_err(|e| format!("폴더 통계 업데이트 실패: {}", e))
}

/// 폴더의 실시간 크기와 파일 개수를 계산합니다
/// 
/// # 매개변수
/// * `folder_id` - 폴더 ID (None이면 루트)
/// * `state` - 애플리케이션 상태
/// 
/// # 반환값
/// * `Ok(FolderStats)` - 폴더 통계 정보
/// * `Err(String)` - 오류 메시지 (한국어)
#[tauri::command]
pub async fn calculate_folder_stats(
    folder_id: Option<String>,
    state: State<'_, Mutex<AppState>>
) -> Result<FolderStats, String> {
    let app_state = state.lock().map_err(|e| format!("상태 잠금 실패: {}", e))?;
    let database_service = &app_state.database_service;
    
    // 폴더 ID 변환
    let folder_uuid = if let Some(id_str) = folder_id {
        Some(Uuid::parse_str(&id_str)
            .map_err(|_| "올바르지 않은 폴더 ID 형식입니다.".to_string())?)
    } else {
        None
    };
    
    // 데이터베이스 서비스 락 획득
    let db_service = database_service.lock().map_err(|e| format!("데이터베이스 서비스 잠금 실패: {}", e))?;
    
    // 실시간 크기 계산
    let total_size = db_service
        .calculate_folder_size(folder_uuid)
        .map_err(|e| format!("폴더 크기 계산 실패: {}", e))?;
    
    // 실시간 파일 개수 계산
    let file_count = db_service
        .count_files_in_folder(folder_uuid)
        .map_err(|e| format!("파일 개수 계산 실패: {}", e))?;
    
    // 하위 폴더 개수 계산
    let subfolder_count = db_service
        .count_subfolders(folder_uuid)
        .map_err(|e| format!("하위 폴더 개수 계산 실패: {}", e))?;
    
    Ok(FolderStats {
        total_size,
        file_count,
        subfolder_count,
    })
}

/// 폴더 통계 정보 구조체
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct FolderStats {
    /// 총 크기 (바이트)
    pub total_size: u64,
    /// 파일 개수
    pub file_count: u32,
    /// 하위 폴더 개수
    pub subfolder_count: u32,
}