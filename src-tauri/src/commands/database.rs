use crate::models::{
    file::FileEntry,
    folder::FolderEntry,
};
use crate::AppState;
use tauri::State;
use uuid::Uuid;
use std::sync::Mutex;

/// 데이터베이스 초기화 커맨드
/// 
/// # 매개변수
/// * `vault_path` - 볼트 경로
/// * `state` - 애플리케이션 상태
/// 
/// # 반환값
/// * `Ok(())` - 초기화 성공
/// * `Err(String)` - 오류 메시지 (한국어)
#[tauri::command]
pub async fn initialize_database(
    vault_path: String,
    state: State<'_, Mutex<AppState>>
) -> Result<(), String> {
    let app_state = state.lock().unwrap();
    let mut db_service = app_state.database_service.lock().unwrap();
    
    db_service
        .initialize(&vault_path)
        .map_err(|e| format!("데이터베이스 초기화 실패: {}", e))
}

/// 파일 메타데이터 추가 커맨드
/// 
/// # 매개변수
/// * `file_entry` - 파일 엔트리
/// * `state` - 애플리케이션 상태
/// 
/// # 반환값
/// * `Ok(())` - 추가 성공
/// * `Err(String)` - 오류 메시지 (한국어)
#[tauri::command]
pub async fn add_file_metadata(
    file_entry: FileEntry,
    state: State<'_, Mutex<AppState>>
) -> Result<(), String> {
    let app_state = state.lock().unwrap();
    let db_service = app_state.database_service.lock().unwrap();
    
    db_service
        .add_file(&file_entry)
        .map_err(|e| format!("파일 메타데이터 추가 실패: {}", e))
}

/// 파일 메타데이터 조회 커맨드
/// 
/// # 매개변수
/// * `file_id` - 파일 ID
/// * `state` - 애플리케이션 상태
/// 
/// # 반환값
/// * `Ok(Option<FileEntry>)` - 파일 엔트리 (없으면 None)
/// * `Err(String)` - 오류 메시지 (한국어)
#[tauri::command]
pub async fn get_file_metadata(
    file_id: String,
    state: State<'_, Mutex<AppState>>
) -> Result<Option<FileEntry>, String> {
    let app_state = state.lock().unwrap();
    let db_service = app_state.database_service.lock().unwrap();
    
    let file_uuid = Uuid::parse_str(&file_id)
        .map_err(|_| "올바르지 않은 파일 ID 형식입니다.".to_string())?;
    
    db_service
        .get_file(&file_uuid)
        .map_err(|e| format!("파일 메타데이터 조회 실패: {}", e))
}

/// 폴더별 파일 목록 조회 커맨드
/// 
/// # 매개변수
/// * `folder_id` - 폴더 ID (선택사항, None이면 루트)
/// * `state` - 애플리케이션 상태
/// 
/// # 반환값
/// * `Ok(Vec<FileEntry>)` - 파일 목록
/// * `Err(String)` - 오류 메시지 (한국어)
#[tauri::command]
pub async fn get_files_by_folder(
    folder_id: Option<String>,
    state: State<'_, Mutex<AppState>>
) -> Result<Vec<FileEntry>, String> {
    let app_state = state.lock().unwrap();
    let db_service = app_state.database_service.lock().unwrap();
    
    // 폴더 ID 변환
    let folder_uuid = if let Some(id_str) = folder_id {
        Some(Uuid::parse_str(&id_str)
            .map_err(|_| "올바르지 않은 폴더 ID 형식입니다.".to_string())?)
    } else {
        None
    };
    
    db_service
        .get_files_by_folder(folder_uuid)
        .map_err(|e| format!("파일 목록 조회 실패: {}", e))
}

/// 파일 메타데이터 삭제 커맨드
/// 
/// # 매개변수
/// * `file_id` - 파일 ID
/// * `state` - 애플리케이션 상태
/// 
/// # 반환값
/// * `Ok(())` - 삭제 성공
/// * `Err(String)` - 오류 메시지 (한국어)
#[tauri::command]
pub async fn remove_file_metadata(
    file_id: String,
    state: State<'_, Mutex<AppState>>
) -> Result<(), String> {
    let app_state = state.lock().unwrap();
    let db_service = app_state.database_service.lock().unwrap();
    
    let file_uuid = Uuid::parse_str(&file_id)
        .map_err(|_| "올바르지 않은 파일 ID 형식입니다.".to_string())?;
    
    db_service
        .remove_file(&file_uuid)
        .map_err(|e| format!("파일 메타데이터 삭제 실패: {}", e))
}

/// 폴더 메타데이터 추가 커맨드
/// 
/// # 매개변수
/// * `folder_entry` - 폴더 엔트리
/// * `state` - 애플리케이션 상태
/// 
/// # 반환값
/// * `Ok(())` - 추가 성공
/// * `Err(String)` - 오류 메시지 (한국어)
#[tauri::command]
pub async fn add_folder_metadata(
    folder_entry: FolderEntry,
    state: State<'_, Mutex<AppState>>
) -> Result<(), String> {
    let app_state = state.lock().unwrap();
    let db_service = app_state.database_service.lock().unwrap();
    
    db_service
        .add_folder(&folder_entry)
        .map_err(|e| format!("폴더 메타데이터 추가 실패: {}", e))
}

/// 폴더 메타데이터 조회 커맨드
/// 
/// # 매개변수
/// * `folder_id` - 폴더 ID
/// * `state` - 애플리케이션 상태
/// 
/// # 반환값
/// * `Ok(Option<FolderEntry>)` - 폴더 엔트리 (없으면 None)
/// * `Err(String)` - 오류 메시지 (한국어)
#[tauri::command]
pub async fn get_folder_metadata(
    folder_id: String,
    state: State<'_, Mutex<AppState>>
) -> Result<Option<FolderEntry>, String> {
    let app_state = state.lock().unwrap();
    let db_service = app_state.database_service.lock().unwrap();
    
    let folder_uuid = Uuid::parse_str(&folder_id)
        .map_err(|_| "올바르지 않은 폴더 ID 형식입니다.".to_string())?;
    
    db_service
        .get_folder(&folder_uuid)
        .map_err(|e| format!("폴더 메타데이터 조회 실패: {}", e))
}

/// 모든 폴더 메타데이터 조회 커맨드
/// 
/// # 매개변수
/// * `state` - 애플리케이션 상태
/// 
/// # 반환값
/// * `Ok(Vec<FolderEntry>)` - 폴더 목록
/// * `Err(String)` - 오류 메시지 (한국어)
#[tauri::command]
pub async fn get_all_folders_metadata(
    state: State<'_, Mutex<AppState>>
) -> Result<Vec<FolderEntry>, String> {
    let app_state = state.lock().unwrap();
    let db_service = app_state.database_service.lock().unwrap();
    
    db_service
        .get_all_folders()
        .map_err(|e| format!("폴더 목록 조회 실패: {}", e))
}

/// 폴더 메타데이터 업데이트 커맨드
/// 
/// # 매개변수
/// * `folder_entry` - 폴더 엔트리
/// * `state` - 애플리케이션 상태
/// 
/// # 반환값
/// * `Ok(())` - 업데이트 성공
/// * `Err(String)` - 오류 메시지 (한국어)
#[tauri::command]
pub async fn update_folder_metadata(
    folder_entry: FolderEntry,
    state: State<'_, Mutex<AppState>>
) -> Result<(), String> {
    let app_state = state.lock().unwrap();
    let db_service = app_state.database_service.lock().unwrap();
    
    db_service
        .update_folder(&folder_entry)
        .map_err(|e| format!("폴더 메타데이터 업데이트 실패: {}", e))
}

/// 폴더 메타데이터 삭제 커맨드
/// 
/// # 매개변수
/// * `folder_id` - 폴더 ID
/// * `state` - 애플리케이션 상태
/// 
/// # 반환값
/// * `Ok(())` - 삭제 성공
/// * `Err(String)` - 오류 메시지 (한국어)
#[tauri::command]
pub async fn remove_folder_metadata(
    folder_id: String,
    state: State<'_, Mutex<AppState>>
) -> Result<(), String> {
    let app_state = state.lock().unwrap();
    let db_service = app_state.database_service.lock().unwrap();
    
    let folder_uuid = Uuid::parse_str(&folder_id)
        .map_err(|_| "올바르지 않은 폴더 ID 형식입니다.".to_string())?;
    
    db_service
        .remove_folder(&folder_uuid)
        .map_err(|e| format!("폴더 메타데이터 삭제 실패: {}", e))
}