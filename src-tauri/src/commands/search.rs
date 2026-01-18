use crate::models::{file::FileEntry, folder::FolderEntry};
use crate::AppState;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::State;

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResult {
    pub files: Vec<FileEntry>,
    pub folders: Vec<FolderEntry>,
}

/// 파일 및 폴더를 검색합니다.
///
/// # 매개변수
/// * `query` - 검색어
/// * `folder_id` - 검색할 폴더 ID (현재는 무시하고 전체 검색 수행)
/// * `state` - 애플리케이션 상태
///
/// # 반환값
/// * `Result<SearchResult, String>` - 검색 결과
#[tauri::command]
pub async fn search_files(
    query: String,
    _folder_id: Option<String>,
    state: State<'_, Mutex<AppState>>,
) -> Result<SearchResult, String> {
    let app_state = state.lock().map_err(|_| "상태 잠금 실패")?;
    let database_service = app_state
        .database_service
        .lock()
        .map_err(|_| "DB 서비스 잠금 실패")?;

    // 빈 검색어 처리
    if query.trim().is_empty() {
        return Ok(SearchResult {
            files: vec![],
            folders: vec![],
        });
    }

    let files = database_service
        .search_files(&query)
        .map_err(|e| format!("파일 검색 실패: {}", e))?;

    let folders = database_service
        .search_folders(&query)
        .map_err(|e| format!("폴더 검색 실패: {}", e))?;

    Ok(SearchResult { files, folders })
}

/// 파일 내용으로 검색합니다. (미구현)
#[tauri::command]
pub async fn search_content(
    query: String,
    file_types: Vec<String>,
    _state: State<'_, Mutex<AppState>>,
) -> Result<Vec<String>, String> {
    // TODO: 파일 내용 검색 구현
    log::debug!(
        "내용 검색 요청: query={}, file_types={:?}",
        query,
        file_types
    );
    Err("파일 내용 검색은 아직 지원되지 않습니다.".to_string())
}
