use crate::models::file::FileEntry;
use crate::AppState;
use std::collections::HashMap;
use std::io::Read;
use std::sync::Mutex;
use tauri::State;

/// 청크 업로드 세션 정보
#[derive(Debug, Clone)]
struct UploadSession {
    /// 세션 ID
    pub _session_id: String,
    /// 파일명
    pub file_name: String,
    /// 전체 파일 크기
    pub _file_size: u64,
    /// 대상 폴더 ID
    pub folder_id: Option<uuid::Uuid>,
    /// 임시 디렉토리 경로
    pub temp_dir: std::path::PathBuf,
    /// 생성 시간
    pub _created_at: chrono::DateTime<chrono::Utc>,
}

/// 전역 업로드 세션 관리자
static UPLOAD_SESSIONS: std::sync::LazyLock<Mutex<HashMap<String, UploadSession>>> =
    std::sync::LazyLock::new(|| Mutex::new(HashMap::new()));

/// 폴더별 파일 목록을 조회합니다.
///
/// # 매개변수
/// * `folder_id` - 폴더 ID (None이면 루트)
/// * `state` - 애플리케이션 상태
///
/// # 반환값
/// * `Result<Vec<FileEntry>, String>` - 파일 목록
#[tauri::command]
pub async fn get_files_in_folder(
    folder_id: Option<String>,
    state: State<'_, Mutex<AppState>>,
) -> Result<Vec<FileEntry>, String> {
    log::info!("파일 목록 조회 요청: folder_id={:?}", folder_id);

    let folder_uuid = if let Some(id_str) = folder_id {
        match uuid::Uuid::parse_str(&id_str) {
            Ok(uuid) => {
                log::info!("폴더 ID 파싱 성공: {}", uuid);
                Some(uuid)
            }
            Err(e) => {
                log::error!("폴더 ID 파싱 실패: {} -> {}", id_str, e);
                return Err(format!("잘못된 폴더 ID 형식: {}", e));
            }
        }
    } else {
        log::info!("루트 폴더 파일 목록 조회");
        None
    };

    let app_state = state.lock().map_err(|e| {
        log::error!("상태 잠금 실패: {}", e);
        format!("상태 잠금 실패: {}", e)
    })?;

    let database_service = app_state.database_service.lock().map_err(|e| {
        log::error!("데이터베이스 서비스 잠금 실패: {}", e);
        format!("데이터베이스 서비스 잠금 실패: {}", e)
    })?;

    match database_service.get_files_by_folder(folder_uuid) {
        Ok(files) => {
            log::info!("파일 목록 조회 완료: {} 개 파일", files.len());
            Ok(files)
        }
        Err(e) => {
            log::error!("파일 목록 조회 실패: {}", e);
            // 실패 시 빈 배열 반환
            Ok(Vec::new())
        }
    }
}

/// 파일을 볼트에 추가합니다.
///
/// # 매개변수
/// * `filePath` - 원본 파일 경로
/// * `fileName` - 볼트 내 파일명
/// * `folderId` - 폴더 ID (None이면 루트)
/// * `state` - 애플리케이션 상태
///
/// # 반환값
/// * `Result<FileEntry, String>` - 생성된 파일 엔트리
#[tauri::command]
pub async fn add_file_to_vault(
    file_path: String,
    file_name: Option<String>,
    folder_id: Option<String>,
    state: State<'_, Mutex<AppState>>,
) -> Result<FileEntry, String> {
    use std::fs;
    use std::path::Path;

    log::info!(
        "파일 추가 요청: file_path={}, file_name={:?}, folder_id={:?}",
        file_path,
        file_name,
        folder_id
    );

    // 폴더 ID 변환 및 검증
    let folder_uuid = if let Some(id_str) = &folder_id {
        log::info!("폴더 ID 문자열 파싱 시도: {}", id_str);
        match uuid::Uuid::parse_str(id_str) {
            Ok(uuid) => {
                log::info!("폴더 ID 파싱 성공: {}", uuid);
                Some(uuid)
            }
            Err(e) => {
                log::error!("폴더 ID 파싱 실패: {} -> {}", id_str, e);
                return Err("잘못된 폴더 ID 형식입니다.".to_string());
            }
        }
    } else {
        log::info!("루트 폴더에 파일 추가 (folderId가 None)");
        None
    };

    // 파일 존재 확인
    let source_path = Path::new(&file_path);
    if !source_path.exists() {
        log::error!("파일이 존재하지 않습니다: {}", file_path);
        return Err("파일이 존재하지 않습니다.".to_string());
    }

    // 파일명 결정 (FileService::add_file에 필요)
    let actual_file_name = file_name.unwrap_or_else(|| {
        source_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown_file")
            .to_string()
    });

    // 중복된 파일명 처리
    let actual_file_name = get_unique_filename(&state, folder_uuid, &actual_file_name)
        .map_err(|e| format!("파일명 중복 처리 실패: {}", e))?;

    // 4. FileService를 사용하여 파일 추가 (암호화 및 DB 저장 포함)
    // MutexGuard는 Send가 아니므로 await 지점을 넘어갈 수 없습니다.
    // 따라서 FileService를 복제(Clone)하여 사용합니다. (FileService는 Clone을 derive하고 내부적으로 Arc 등을 사용하여 상태를 공유함)
    // 2. 파일 서비스 복제 (Lock 최소화)
    let file_service_clone = {
        let app_state = state.lock().map_err(|e| {
            log::error!("상태 잠금 실패: {}", e);
            format!("상태 잠금 실패: {}", e)
        })?;
        let file_service_guard = app_state.file_service.lock().map_err(|e| {
            log::error!("파일 서비스 잠금 실패: {}", e);
            format!("파일 서비스 잠금 실패: {}", e)
        })?;
        file_service_guard.clone()
    };
    let mut file_service = file_service_clone;

    let file_entry = file_service
        .add_file(&file_path, &actual_file_name, folder_uuid)
        .await
        .map_err(|e| {
            log::error!("파일 추가 실패: {}", e);
            format!("파일 추가 실패: {}", e)
        })?;

    log::info!(
        "파일 추가 완료: {} (ID: {})",
        actual_file_name,
        file_entry.id
    );

    Ok(file_entry)
}

/// 파일을 볼트에서 삭제합니다.
///
/// # 매개변수
/// * `fileId` - 파일 ID
/// * `state` - 애플리케이션 상태
///
/// # 반환값
/// * `Result<(), String>` - 삭제 결과
#[tauri::command]
pub async fn delete_file_from_vault(
    file_id: String,
    state: State<'_, Mutex<AppState>>,
) -> Result<(), String> {
    log::info!("파일 삭제 요청: file_id={}", file_id);

    // 파일 ID 파싱
    let file_uuid = match uuid::Uuid::parse_str(&file_id) {
        Ok(uuid) => {
            log::info!("파일 ID 파싱 성공: {}", uuid);
            uuid
        }
        Err(e) => {
            log::error!("파일 ID 파싱 실패: {} -> {}", file_id, e);
            return Err("잘못된 파일 ID 형식입니다.".to_string());
        }
    };

    let app_state = state.lock().map_err(|e| {
        log::error!("상태 잠금 실패: {}", e);
        format!("상태 잠금 실패: {}", e)
    })?;

    let database_service = app_state.database_service.lock().map_err(|e| {
        log::error!("데이터베이스 서비스 잠금 실패: {}", e);
        format!("데이터베이스 서비스 잠금 실패: {}", e)
    })?;

    // 파일 존재 확인
    let file_entry = match database_service.get_file(&file_uuid) {
        Ok(Some(file)) => file,
        Ok(None) => {
            log::error!("파일을 찾을 수 없습니다: {}", file_uuid);
            return Err("파일을 찾을 수 없습니다.".to_string());
        }
        Err(e) => {
            log::error!("파일 조회 실패: {}", e);
            return Err(format!("파일 조회 실패: {}", e));
        }
    };

    // TODO: 실제 암호화된 파일 삭제 (파일 시스템에서)
    // let encrypted_file_path = format!(".securevault/data/files/{}", file_entry.encrypted_file_name);
    // std::fs::remove_file(encrypted_file_path).map_err(|e| format!("파일 삭제 실패: {}", e))?;

    // 데이터베이스에서 파일 메타데이터 삭제
    if let Err(e) = database_service.remove_file(&file_uuid) {
        log::error!("파일 메타데이터 삭제 실패: {}", e);
        return Err(format!("파일 메타데이터 삭제 실패: {}", e));
    }

    log::info!(
        "파일 삭제 완료: {} (ID: {})",
        file_entry.file_name,
        file_uuid
    );
    Ok(())
}

/// 파일 이름을 변경합니다.
///
/// # 매개변수
/// * `fileId` - 파일 ID
/// * `newName` - 새로운 파일명
/// * `state` - 애플리케이션 상태
///
/// # 반환값
/// * `Result<(), String>` - 이름 변경 결과
#[tauri::command]
pub async fn rename_file_in_vault(
    file_id: String,
    new_name: String,
    state: State<'_, Mutex<AppState>>,
) -> Result<(), String> {
    log::info!(
        "파일 이름 변경 요청: file_id={}, new_name={}",
        file_id,
        new_name
    );

    // 파일 ID 파싱
    let file_uuid = match uuid::Uuid::parse_str(&file_id) {
        Ok(uuid) => {
            log::info!("파일 ID 파싱 성공: {}", uuid);
            uuid
        }
        Err(e) => {
            log::error!("파일 ID 파싱 실패: {} -> {}", file_id, e);
            return Err("잘못된 파일 ID 형식입니다.".to_string());
        }
    };

    // 새 파일명 유효성 검사
    let trimmed_name = new_name.trim();
    if trimmed_name.is_empty() {
        log::error!("파일명이 비어있습니다");
        return Err("파일명이 비어있습니다.".to_string());
    }

    // 파일명에 허용되지 않는 문자 검사
    let invalid_chars = ['<', '>', ':', '"', '|', '?', '*', '/', '\\'];
    if trimmed_name.chars().any(|c| invalid_chars.contains(&c)) {
        log::error!(
            "파일명에 허용되지 않는 문자가 포함되어 있습니다: {}",
            trimmed_name
        );
        return Err("파일명에 다음 문자는 사용할 수 없습니다: < > : \" | ? * / \\".to_string());
    }

    let app_state = state.lock().map_err(|e| {
        log::error!("상태 잠금 실패: {}", e);
        format!("상태 잠금 실패: {}", e)
    })?;

    let database_service = app_state.database_service.lock().map_err(|e| {
        log::error!("데이터베이스 서비스 잠금 실패: {}", e);
        format!("데이터베이스 서비스 잠금 실패: {}", e)
    })?;

    // 파일 존재 확인
    let mut file_entry = match database_service.get_file(&file_uuid) {
        Ok(Some(file)) => file,
        Ok(None) => {
            log::error!("파일을 찾을 수 없습니다: {}", file_uuid);
            return Err("파일을 찾을 수 없습니다.".to_string());
        }
        Err(e) => {
            log::error!("파일 조회 실패: {}", e);
            return Err(format!("파일 조회 실패: {}", e));
        }
    };

    // 같은 폴더 내에서 중복 이름 검사
    let existing_files = database_service
        .get_files_by_folder(file_entry.folder_id)
        .map_err(|e| {
            log::error!("폴더 내 파일 목록 조회 실패: {}", e);
            format!("폴더 내 파일 목록 조회 실패: {}", e)
        })?;

    // 현재 파일을 제외하고 같은 이름의 파일이 있는지 확인
    for existing_file in existing_files {
        if existing_file.id != file_uuid
            && existing_file.file_name.eq_ignore_ascii_case(trimmed_name)
        {
            log::error!(
                "같은 폴더에 동일한 이름의 파일이 이미 존재합니다: {}",
                trimmed_name
            );
            return Err("같은 폴더에 동일한 이름의 파일이 이미 존재합니다.".to_string());
        }
    }

    // 파일명 업데이트
    let old_name = file_entry.file_name.clone();
    file_entry.file_name = trimmed_name.to_string();
    file_entry.original_file_name = trimmed_name.to_string();

    // 파일 확장자 업데이트
    let new_extension = std::path::Path::new(trimmed_name)
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("")
        .to_string();
    file_entry.file_extension = new_extension.clone();

    // MIME 타입 업데이트 (확장자 변경에 따라)
    let new_mime_type = match new_extension.to_lowercase().as_str() {
        // 텍스트 파일
        "txt" => "text/plain",
        "md" => "text/markdown",
        "json" => "application/json",
        "xml" => "application/xml",
        "csv" => "text/csv",
        "html" => "text/html",
        "css" => "text/css",
        "js" => "application/javascript",
        "py" => "text/x-python",
        "cs" => "text/x-csharp",
        "java" => "text/x-java-source",
        "cpp" => "text/x-c++src",
        "sql" => "application/sql",
        "yaml" | "yml" => "application/x-yaml",
        "ini" => "text/plain",
        "log" => "text/plain",

        // 이미지 파일
        "jpg" | "jpeg" => "image/jpeg",
        "png" => "image/png",
        "gif" => "image/gif",
        "bmp" => "image/bmp",
        "webp" => "image/webp",
        "svg" => "image/svg+xml",
        "ico" => "image/x-icon",

        // 오디오 파일
        "mp3" => "audio/mpeg",
        "wav" => "audio/wav",
        "ogg" => "audio/ogg",
        "m4a" => "audio/mp4",
        "flac" => "audio/flac",

        // 비디오 파일
        "mp4" => "video/mp4",
        "avi" => "video/x-msvideo",
        "mov" => "video/quicktime",
        "wmv" => "video/x-ms-wmv",
        "webm" => "video/webm",

        // 문서 파일
        "pdf" => "application/pdf",
        "doc" => "application/msword",
        "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        "xls" => "application/vnd.ms-excel",
        "xlsx" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        "ppt" => "application/vnd.ms-powerpoint",
        "pptx" => "application/vnd.openxmlformats-officedocument.presentationml.presentation",

        // 압축 파일
        "zip" => "application/zip",
        "rar" => "application/vnd.rar",
        "7z" => "application/x-7z-compressed",
        "tar" => "application/x-tar",
        "gz" => "application/gzip",

        // 기본값
        _ => "application/octet-stream",
    }
    .to_string();

    let old_mime_type = file_entry.mime_type.clone();
    file_entry.mime_type = new_mime_type.clone();

    log::info!(
        "파일 MIME 타입 업데이트: {} -> {} (확장자: {})",
        old_mime_type,
        new_mime_type,
        new_extension
    );

    // 수정 날짜 업데이트
    file_entry.modified_date = chrono::Utc::now();

    // 데이터베이스에서 파일 정보 업데이트
    if let Err(e) = database_service.update_file(&file_entry) {
        log::error!("파일 정보 업데이트 실패: {}", e);
        return Err(format!("파일 정보 업데이트 실패: {}", e));
    }

    log::info!(
        "파일 이름 변경 완료: {} -> {} (ID: {})",
        old_name,
        trimmed_name,
        file_uuid
    );
    Ok(())
}

/// 파일을 임시 위치에 추출합니다.
///
/// # 매개변수
/// * `file_id` - 파일 ID
/// * `state` - 애플리케이션 상태
///
/// # 반환값
/// * `Result<String, String>` - 임시 파일 경로
#[tauri::command]
pub async fn extract_file_from_vault(
    _file_id: String,
    _state: State<'_, Mutex<AppState>>,
) -> Result<String, String> {
    // TODO: 파일 서비스 구현 후 활성화
    Err("파일 추출 기능이 아직 구현되지 않았습니다.".to_string())
}

/// 파일을 볼트 외부로 내보냅니다.
///
/// # 매개변수
/// * `file_id` - 파일 ID
/// * `export_path` - 내보낼 경로
/// * `state` - 애플리케이션 상태
///
/// # 반환값
/// * `Result<(), String>` - 내보내기 결과
#[tauri::command]
pub async fn export_file_from_vault(
    file_id: String,
    export_path: String,
    state: State<'_, Mutex<AppState>>,
) -> Result<(), String> {
    use std::path::Path;

    log::info!(
        "파일 내보내기 요청: file_id={}, export_path={}",
        file_id,
        export_path
    );

    // 파일 ID 파싱
    let file_uuid = match uuid::Uuid::parse_str(&file_id) {
        Ok(uuid) => {
            log::info!("파일 ID 파싱 성공: {}", uuid);
            uuid
        }
        Err(e) => {
            log::error!("파일 ID 파싱 실패: {} -> {}", file_id, e);
            return Err("잘못된 파일 ID 형식입니다.".to_string());
        }
    };

    // 파일 메타데이터 조회 (스코프 분리)
    let file_entry = {
        let app_state = state.lock().map_err(|e| {
            log::error!("상태 잠금 실패: {}", e);
            format!("상태 잠금 실패: {}", e)
        })?;

        let database_service = app_state.database_service.lock().map_err(|e| {
            log::error!("데이터베이스 서비스 잠금 실패: {}", e);
            format!("데이터베이스 서비스 잠금 실패: {}", e)
        })?;

        // 파일 메타데이터 조회
        match database_service.get_file(&file_uuid) {
            Ok(Some(file)) => file,
            Ok(None) => {
                log::error!("파일을 찾을 수 없습니다: {}", file_uuid);
                return Err("파일을 찾을 수 없습니다.".to_string());
            }
            Err(e) => {
                log::error!("파일 조회 실패: {}", e);
                return Err(format!("파일 조회 실패: {}", e));
            }
        }
    };

    log::info!(
        "파일 메타데이터 조회 완료: {} (원본명: {})",
        file_entry.file_name,
        file_entry.original_file_name
    );

    // 내보낼 경로 검증
    let export_path_obj = Path::new(&export_path);

    // 부모 디렉토리가 존재하는지 확인
    if let Some(parent_dir) = export_path_obj.parent() {
        if !parent_dir.exists() {
            log::error!("대상 디렉토리가 존재하지 않습니다: {:?}", parent_dir);
            return Err("대상 디렉토리가 존재하지 않습니다.".to_string());
        }
    }

    // 파일이 이미 존재하는지 확인
    if export_path_obj.exists() {
        log::warn!("대상 파일이 이미 존재합니다: {}", export_path);
        return Err("대상 파일이 이미 존재합니다. 다른 이름을 선택해주세요.".to_string());
    }

    // 파일 서비스를 복사하여 await 포인트에서 사용
    let file_service_clone = {
        let app_state = state.lock().map_err(|e| {
            log::error!("상태 잠금 실패: {}", e);
            format!("상태 잠금 실패: {}", e)
        })?;

        // 파일 서비스를 복사
        let file_service_guard = app_state.file_service.lock().map_err(|e| {
            log::error!("파일 서비스 잠금 실패: {}", e);
            format!("파일 서비스 잠금 실패: {}", e)
        })?;

        // Clone을 사용하여 복사
        file_service_guard.clone()
    };
    let mut file_service_copy = file_service_clone;

    // 실제 파일 내보내기 실행
    file_service_copy
        .export_file(&file_uuid, &export_path)
        .await
        .map_err(|e| {
            log::error!("파일 내보내기 실패: {}", e);
            format!("파일 내보내기 실패: {}", e)
        })?;

    log::info!("파일 내보내기 완료: {} -> {}", file_uuid, export_path);
    Ok(())
}

/// 새 파일을 생성하고 볼트에 추가합니다.
///
/// # 매개변수
/// * `folder_id` - 폴더 ID (None이면 루트)
/// * `file_name` - 파일명
/// * `content` - 파일 내용
/// * `state` - 애플리케이션 상태
///
/// # 반환값
/// * `Result<FileEntry, String>` - 생성된 파일 엔트리
#[tauri::command]
pub async fn create_new_file_in_vault(
    folder_id: Option<String>,
    file_name: String,
    content: String,
    state: State<'_, Mutex<AppState>>,
) -> Result<FileEntry, String> {
    log::info!(
        "새 파일 생성 요청: file_name={}, folder_id={:?}",
        file_name,
        folder_id
    );

    // 폴더 ID 변환 및 검증
    let folder_uuid = if let Some(id_str) = &folder_id {
        log::info!("폴더 ID 문자열 파싱 시도: {}", id_str);
        match uuid::Uuid::parse_str(id_str) {
            Ok(uuid) => {
                log::info!("폴더 ID 파싱 성공: {}", uuid);
                Some(uuid)
            }
            Err(e) => {
                log::error!("폴더 ID 파싱 실패: {} -> {}", id_str, e);
                return Err("잘못된 폴더 ID 형식입니다.".to_string());
            }
        }
    } else {
        log::info!("루트 폴더에 파일 생성 (folder_id가 None)");
        None
    };

    // 파일명 유효성 검사
    if file_name.trim().is_empty() {
        log::error!("파일명이 비어있습니다");
        return Err("파일명이 비어있습니다.".to_string());
    }

    // 중복된 파일명 처리
    let file_name = get_unique_filename(&state, folder_uuid, &file_name)
        .map_err(|e| format!("파일명 중복 처리 실패: {}", e))?;

    log::info!(
        "새 파일 생성 정보: name={}, content_length={}",
        file_name,
        content.len()
    );

    // 임시 파일 생성하여 내용 작성
    use std::io::Write;
    use tempfile::NamedTempFile;

    let mut temp_file = NamedTempFile::new().map_err(|e| {
        log::error!("임시 파일 생성 실패: {}", e);
        format!("임시 파일 생성 실패: {}", e)
    })?;

    // 파일 내용 작성
    temp_file.write_all(content.as_bytes()).map_err(|e| {
        log::error!("파일 내용 작성 실패: {}", e);
        format!("파일 내용 작성 실패: {}", e)
    })?;

    // 파일 엔트리 생성
    let file_id = uuid::Uuid::new_v4();
    let file_extension = std::path::Path::new(&file_name)
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("")
        .to_string();

    // MIME 타입 추정
    let mime_type = match file_extension.to_lowercase().as_str() {
        "txt" => "text/plain",
        "md" => "text/markdown",
        "json" => "application/json",
        "xml" => "application/xml",
        "csv" => "text/csv",
        "html" => "text/html",
        "css" => "text/css",
        "js" => "application/javascript",
        "py" => "text/x-python",
        "cs" => "text/x-csharp",
        "java" => "text/x-java-source",
        "cpp" => "text/x-c++src",
        "sql" => "application/sql",
        "yaml" | "yml" => "application/x-yaml",
        "ini" => "text/plain",
        "log" => "text/plain",
        _ => "text/plain",
    }
    .to_string();

    // 암호화 수행
    let (encrypted_data, encrypted_size) = {
        let app_state = state.lock().map_err(|e| format!("상태 잠금 실패: {}", e))?;

        // 마스터 키 확인 및 가져오기
        let master_key = if let Some(key) = app_state.crypto_service.get_master_key() {
            key
        } else {
            return Err("마스터 키가 설정되지 않았습니다. (로그인 필요)".to_string());
        };

        // 직접 C# 호환 암호화 (마스터 키 사용) - FileService와 일치시킴
        let encrypted_bytes = app_state
            .crypto_service
            .encrypt_data_csharp_compatible(content.as_bytes(), &master_key)
            .map_err(|e| format!("파일 암호화 실패: {}", e))?;

        let size = encrypted_bytes.len() as u64;
        (encrypted_bytes, size)
    };

    // 파일 엔트리 생성
    let mut file_entry = crate::models::file::FileEntry::new(
        file_name.clone(),
        file_name.clone(),
        content.len() as u64,
        file_extension,
        mime_type,
        "".to_string(), // TODO: 체크섬 계산
        folder_uuid,
        format!("{}.enc", file_id), // UUID + .enc 확장자 사용
        encrypted_size,
    );
    file_entry.id = file_id; // 암호화에 사용된 ID로 설정

    // 볼트 디렉토리 초기화 확인
    let vault_dir = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    let securevault_dir = vault_dir.join(".securevault");
    let files_dir = securevault_dir.join("files");

    // 디렉토리 생성
    if !securevault_dir.exists() {
        std::fs::create_dir_all(&securevault_dir)
            .map_err(|e| format!("볼트 디렉토리 생성 실패: {}", e))?;
    }
    if !files_dir.exists() {
        std::fs::create_dir_all(&files_dir)
            .map_err(|e| format!("파일 디렉토리 생성 실패: {}", e))?;
    }

    // 파일 내용을 암호화된 파일로 저장
    let encrypted_file_path = files_dir.join(&file_entry.encrypted_file_name);
    std::fs::write(&encrypted_file_path, &encrypted_data)
        .map_err(|e| format!("파일 저장 실패: {}", e))?;

    // 데이터베이스에 메타데이터 저장
    {
        let app_state = state.lock().map_err(|e| {
            log::error!("상태 잠금 실패: {}", e);
            format!("상태 잠금 실패: {}", e)
        })?;

        let database_service = app_state.database_service.lock().map_err(|e| {
            log::error!("데이터베이스 서비스 잠금 실패: {}", e);
            format!("데이터베이스 서비스 잠금 실패: {}", e)
        })?;

        // 데이터베이스가 초기화되지 않았다면 초기화
        if let Err(_) = database_service.add_file(&file_entry) {
            log::info!("데이터베이스 초기화 시도");
            drop(database_service); // 락 해제

            // 볼트 초기화
            crate::initialize_vault_directory_simple()
                .map_err(|e| format!("볼트 초기화 실패: {}", e))?;

            // 다시 시도
            let database_service = app_state
                .database_service
                .lock()
                .map_err(|e| format!("데이터베이스 서비스 재잠금 실패: {}", e))?;

            database_service
                .add_file(&file_entry)
                .map_err(|e| format!("파일 메타데이터 저장 실패: {}", e))?;
        }
    }

    log::info!("새 파일 생성 완료: {} (ID: {})", file_name, file_entry.id);
    Ok(file_entry)
}

/// 바이너리 파일을 생성하고 볼트에 추가합니다.
///
/// # 매개변수
/// * `folder_id` - 폴더 ID (None이면 루트)
/// * `file_name` - 파일명
/// * `content` - 파일 내용 (base64 인코딩된 바이너리 데이터)
/// * `state` - 애플리케이션 상태
///
/// # 반환값
/// * `Result<FileEntry, String>` - 생성된 파일 엔트리
#[tauri::command]
pub async fn create_binary_file_in_vault(
    folder_id: Option<String>,
    file_name: String,
    content: String, // base64 인코딩된 바이너리 데이터
    state: State<'_, Mutex<AppState>>,
) -> Result<FileEntry, String> {
    use base64::{engine::general_purpose, Engine as _};
    use std::fs;
    use std::io::Write;
    use tempfile::NamedTempFile;

    log::info!(
        "바이너리 파일 생성 요청: file_name={}, folder_id={:?}",
        file_name,
        folder_id
    );

    // 폴더 ID 변환 및 검증
    let folder_uuid = if let Some(id_str) = &folder_id {
        log::info!("폴더 ID 문자열 파싱 시도: {}", id_str);
        match uuid::Uuid::parse_str(id_str) {
            Ok(uuid) => {
                log::info!("폴더 ID 파싱 성공: {}", uuid);
                Some(uuid)
            }
            Err(e) => {
                log::error!("폴더 ID 파싱 실패: {} -> {}", id_str, e);
                return Err("잘못된 폴더 ID 형식입니다.".to_string());
            }
        }
    } else {
        log::info!("루트 폴더에 바이너리 파일 생성 (folder_id가 None)");
        None
    };

    // 파일명 유효성 검사
    if file_name.trim().is_empty() {
        log::error!("파일명이 비어있습니다");
        return Err("파일명이 비어있습니다.".to_string());
    }

    // 중복된 파일명 처리
    let file_name = get_unique_filename(&state, folder_uuid, &file_name)
        .map_err(|e| format!("파일명 중복 처리 실패: {}", e))?;

    // base64 디코딩
    let binary_data = general_purpose::STANDARD.decode(&content).map_err(|e| {
        log::error!("base64 디코딩 실패: {}", e);
        format!("파일 데이터 디코딩 실패: {}", e)
    })?;

    // 파일 크기
    let original_size = binary_data.len() as u64;

    // 파일 확장자 추출
    let extension = std::path::Path::new(&file_name)
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("")
        .to_string();

    // MIME 타입 추정 (확장된 버전)
    let mime_type = match extension.to_lowercase().as_str() {
        // 텍스트 파일
        "txt" => "text/plain",
        "md" => "text/markdown",
        "json" => "application/json",
        "xml" => "application/xml",
        "csv" => "text/csv",
        "html" => "text/html",
        "css" => "text/css",
        "js" => "application/javascript",
        "py" => "text/x-python",
        "cs" => "text/x-csharp",
        "java" => "text/x-java-source",
        "cpp" => "text/x-c++src",
        "sql" => "application/sql",
        "yaml" | "yml" => "application/x-yaml",
        "ini" => "text/plain",
        "log" => "text/plain",

        // 이미지 파일
        "jpg" | "jpeg" => "image/jpeg",
        "png" => "image/png",
        "gif" => "image/gif",
        "bmp" => "image/bmp",
        "webp" => "image/webp",
        "svg" => "image/svg+xml",
        "ico" => "image/x-icon",

        // 오디오 파일
        "mp3" => "audio/mpeg",
        "wav" => "audio/wav",
        "ogg" => "audio/ogg",
        "m4a" => "audio/mp4",
        "flac" => "audio/flac",

        // 비디오 파일
        "mp4" => "video/mp4",
        "avi" => "video/x-msvideo",
        "mov" => "video/quicktime",
        "wmv" => "video/x-ms-wmv",
        "webm" => "video/webm",

        // 문서 파일
        "pdf" => "application/pdf",
        "doc" => "application/msword",
        "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        "xls" => "application/vnd.ms-excel",
        "xlsx" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        "ppt" => "application/vnd.ms-powerpoint",
        "pptx" => "application/vnd.openxmlformats-officedocument.presentationml.presentation",

        // 압축 파일
        "zip" => "application/zip",
        "rar" => "application/vnd.rar",
        "7z" => "application/x-7z-compressed",
        "tar" => "application/x-tar",
        "gz" => "application/gzip",

        // 기본값
        _ => "application/octet-stream",
    }
    .to_string();

    let file_id = uuid::Uuid::new_v4();

    // 암호화 수행
    let (encrypted_data, encrypted_size) = {
        let app_state = state.lock().map_err(|e| format!("상태 잠금 실패: {}", e))?;

        // 마스터 키 확인
        if !app_state.crypto_service.has_master_key() {
            return Err("마스터 키가 설정되지 않았습니다. (로그인 필요)".to_string());
        }

        let encrypted = app_state
            .crypto_service
            .encrypt_file(&binary_data, &file_id)
            .map_err(|e| format!("파일 암호화 실패: {}", e))?;

        let size = encrypted.ciphertext.len() as u64;
        (encrypted.ciphertext, size)
    };

    // 파일 엔트리 생성
    let mut file_entry = FileEntry::new(
        file_name.clone(),
        file_name.clone(),
        original_size,
        extension,
        mime_type,
        "".to_string(), // TODO: 체크섬 계산
        folder_uuid,
        format!("{}", file_id), // 확장자 없이 UUID 사용
        encrypted_size,
    );
    file_entry.id = file_id; // 암호화에 사용된 ID로 설정 (중요)

    // 데이터베이스에 파일 메타데이터 저장
    {
        let app_state = state.lock().map_err(|e| {
            log::error!("상태 잠금 실패: {}", e);
            format!("상태 잠금 실패: {}", e)
        })?;

        let database_service = app_state.database_service.lock().map_err(|e| {
            log::error!("데이터베이스 서비스 잠금 실패: {}", e);
            format!("데이터베이스 서비스 잠금 실패: {}", e)
        })?;

        // 파일 메타데이터를 데이터베이스에 저장
        if let Err(e) = database_service.add_file(&file_entry) {
            log::error!("파일 메타데이터 저장 실패: {}", e);
            return Err(format!("파일 메타데이터 저장 실패: {}", e));
        }
    }

    // 볼트 디렉토리 및 파일 저장
    let vault_dir = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    let securevault_dir = vault_dir.join(".securevault");
    let files_dir = securevault_dir.join("files");

    if !securevault_dir.exists() {
        std::fs::create_dir_all(&securevault_dir)
            .map_err(|e| format!("볼트 디렉토리 생성 실패: {}", e))?;
    }
    if !files_dir.exists() {
        std::fs::create_dir_all(&files_dir)
            .map_err(|e| format!("파일 디렉토리 생성 실패: {}", e))?;
    }

    let encrypted_file_path = files_dir.join(&file_entry.encrypted_file_name);
    std::fs::write(&encrypted_file_path, &encrypted_data)
        .map_err(|e| format!("파일 저장 실패: {}", e))?;

    log::info!(
        "바이너리 파일 생성 완료: {} (ID: {})",
        file_name,
        file_entry.id
    );
    Ok(file_entry)
}

/// 파일 내용을 조회합니다.
///
/// # 매개변수
/// 파일 내용을 가져옵니다 (복호화 및 압축 해제 포함).
///
/// # 매개변수
/// * `file_id` - 파일 ID
/// * `state` - 애플리케이션 상태
///
/// # 반환값
/// * `Result<Vec<u8>, String>` - 파일 내용 (원본 데이터)
#[tauri::command]
pub fn get_file_content(
    file_id: String,
    state: State<'_, Mutex<AppState>>,
) -> Result<Vec<u8>, String> {
    let app_state = state.lock().map_err(|e| format!("상태 잠금 실패: {}", e))?;

    // 파일 메타데이터 조회
    let db_service = app_state
        .database_service
        .lock()
        .map_err(|e| format!("DB 잠금 실패: {}", e))?;
    let file_metadata = db_service
        .get_file_metadata(&file_id)
        .map_err(|e| format!("파일 메타데이터 조회 실패: {}", e))?;

    if file_metadata.is_none() {
        return Err("파일을 찾을 수 없습니다.".to_string());
    }

    let file_metadata = file_metadata.unwrap();
    drop(db_service); // 락 해제

    // 파일 서비스를 통해 파일 내용 읽기
    let mut file_service = app_state
        .file_service
        .lock()
        .map_err(|e| format!("파일 서비스 잠금 실패: {}", e))?;

    // 파일 내용 읽기 (동기 버전)
    let decrypted_data = file_service
        .get_file_content(&file_id)
        .map_err(|e| format!("파일 읽기 실패: {}", e))?;

    // 압축 해제 (필요한 경우)
    if file_metadata.is_compressed {
        let compression_service = app_state
            .compression_service
            .lock()
            .map_err(|e| format!("압축 서비스 잠금 실패: {}", e))?;
        let decompressed_data = compression_service
            .decompress_data(&decrypted_data)
            .map_err(|e| format!("압축 해제 실패: {}", e))?;
        Ok(decompressed_data)
    } else {
        Ok(decrypted_data)
    }
}

/// 파일 내용을 업데이트합니다 (청크 업로드 완료 후 호출됨).
///
/// # 매개변수
/// * `file_id` - 파일 ID
/// * `content` - 새로운 파일 내용 (바이트 배열)
/// * `state` - 애플리케이션 상태
///
/// # 반환값
/// * `Result<(), String>` - 업데이트 결과
#[tauri::command]
pub async fn update_file_content(
    _file_id: String,
    _content: Vec<u8>,
    _state: State<'_, Mutex<AppState>>,
) -> Result<(), String> {
    // TODO: 파일 서비스 구현 후 활성화
    Err("파일 내용 업데이트 기능이 아직 구현되지 않았습니다.".to_string())
}

/// 폴더를 볼트에 추가합니다 (재귀적으로 내부 파일과 하위 폴더 포함).
///
/// # 매개변수
/// * `folderPath` - 추가할 폴더 경로
/// * `targetFolderId` - 대상 폴더 ID (None이면 루트)
/// * `state` - 애플리케이션 상태
///
/// # 반환값
/// * `Result<AddFolderResult, String>` - 추가 결과 (폴더 수, 파일 수)
#[tauri::command]
pub async fn add_folder_to_vault(
    folder_path: String,
    target_folder_id: Option<String>,
    state: State<'_, Mutex<AppState>>,
) -> Result<AddFolderResult, String> {
    use std::collections::HashMap;
    use std::fs;
    use std::path::Path;
    use walkdir::WalkDir;

    log::info!(
        "폴더 추가 요청: folder_path={}, target_folder_id={:?}",
        folder_path,
        target_folder_id
    );

    // 대상 폴더 ID 변환 및 검증
    let target_folder_uuid = if let Some(id_str) = &target_folder_id {
        log::info!("대상 폴더 ID 문자열 파싱 시도: {}", id_str);
        match uuid::Uuid::parse_str(id_str) {
            Ok(uuid) => {
                log::info!("대상 폴더 ID 파싱 성공: {}", uuid);
                Some(uuid)
            }
            Err(e) => {
                log::error!("대상 폴더 ID 파싱 실패: {} -> {}", id_str, e);
                return Err("잘못된 대상 폴더 ID 형식입니다.".to_string());
            }
        }
    } else {
        log::info!("루트 폴더에 폴더 추가 (target_folder_id가 None)");
        None
    };

    // 폴더 존재 확인
    let source_path = Path::new(&folder_path);
    if !source_path.exists() {
        log::error!("폴더가 존재하지 않습니다: {}", folder_path);
        return Err("폴더가 존재하지 않습니다.".to_string());
    }

    if !source_path.is_dir() {
        log::error!("지정된 경로가 폴더가 아닙니다: {}", folder_path);
        return Err("지정된 경로가 폴더가 아닙니다.".to_string());
    }

    let app_state = state.lock().map_err(|e| {
        log::error!("상태 잠금 실패: {}", e);
        format!("상태 잠금 실패: {}", e)
    })?;

    // 볼트 초기화 확인 및 수행
    let mut file_service = app_state
        .file_service
        .lock()
        .map_err(|e| format!("파일 서비스 잠금 실패: {}", e))?;

    if !file_service.is_initialized() {
        log::info!("볼트가 초기화되지 않음, 초기화 수행");
        // 볼트 디렉토리 초기화
        crate::initialize_vault_directory_simple()
            .map_err(|e| format!("볼트 초기화 실패: {}", e))?;

        // 파일 서비스에 기본 볼트 정보 설정
        let vault_path = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
        // 임시 마스터 키 - 실제로는 앱 초기화 시 설정됨
        let temp_master_key = [0u8; 32];
        file_service.set_vault_info(vault_path.to_str().unwrap_or("."), temp_master_key);
        log::info!("볼트 초기화 완료");
    }

    // 마스터 키 가져오기 (file_service drop 전에)
    let master_key = file_service.get_master_key().ok_or_else(|| {
        log::error!("마스터 키를 가져올 수 없습니다.");
        "마스터 키가 설정되지 않았습니다. 로그인이 필요합니다.".to_string()
    })?;
    log::info!("마스터 키 획득 완료 (Key0={:02X})", master_key[0]);

    drop(file_service);

    let mut database_service = app_state.database_service.lock().map_err(|e| {
        log::error!("데이터베이스 서비스 잠금 실패: {}", e);
        format!("데이터베이스 서비스 잠금 실패: {}", e)
    })?;

    // 데이터 디렉토리 미리 생성 (병렬 처리 시 중복 체크 방지)
    let vault_path = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    let data_dir = vault_path.join(".securevault").join("files");
    if !data_dir.exists() {
        fs::create_dir_all(&data_dir).map_err(|e| format!("데이터 디렉토리 생성 실패: {}", e))?;
    }

    // 폴더 구조를 순회하며 폴더와 파일 정보 수집
    let mut folder_count = 0;
    let mut file_count = 0;
    let mut folder_map: HashMap<String, uuid::Uuid> = HashMap::new();

    // 루트 폴더명 추출
    let root_folder_name = source_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("imported_folder")
        .to_string();

    log::info!("폴더 구조 분석 시작: {}", root_folder_name);

    // 먼저 모든 폴더를 생성 (깊이 우선 순회)
    for entry in WalkDir::new(source_path).into_iter() {
        let entry = entry.map_err(|e| format!("폴더 순회 실패: {}", e))?;
        let entry_path = entry.path();

        if entry_path.is_dir() {
            // 상대 경로 계산
            let relative_path = entry_path
                .strip_prefix(source_path)
                .map_err(|e| format!("상대 경로 계산 실패: {}", e))?;

            let folder_name = if relative_path.as_os_str().is_empty() {
                // 루트 폴더
                root_folder_name.clone()
            } else {
                entry_path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown_folder")
                    .to_string()
            };

            // 부모 폴더 ID 결정
            let parent_folder_id = if relative_path.as_os_str().is_empty() {
                // 루트 폴더는 대상 폴더의 하위에 생성
                target_folder_uuid
            } else {
                // 부모 폴더 경로 계산
                if let Some(parent_path) = relative_path.parent() {
                    if parent_path.as_os_str().is_empty() {
                        // 직접 하위 폴더는 루트 폴더의 하위
                        folder_map.get(&root_folder_name).copied()
                    } else {
                        // 더 깊은 하위 폴더
                        let parent_key = parent_path.to_string_lossy().to_string();
                        folder_map.get(&parent_key).copied()
                    }
                } else {
                    target_folder_uuid
                }
            };

            // 폴더 엔트리 생성
            let folder_path = if relative_path.as_os_str().is_empty() {
                format!("/{}", folder_name)
            } else {
                format!("/{}", relative_path.to_string_lossy().replace('\\', "/"))
            };

            let folder_entry = crate::models::folder::FolderEntry::new(
                folder_name.clone(),
                parent_folder_id,
                folder_path,
            );

            let folder_id = folder_entry.id;

            // 데이터베이스에 폴더 저장
            if let Err(e) = database_service.add_folder(&folder_entry) {
                log::error!("폴더 메타데이터 저장 실패: {}", e);
                return Err(format!("폴더 메타데이터 저장 실패: {}", e));
            }

            // 폴더 맵에 추가
            let folder_key = if relative_path.as_os_str().is_empty() {
                root_folder_name.clone()
            } else {
                relative_path.to_string_lossy().to_string()
            };
            folder_map.insert(folder_key, folder_id);

            folder_count += 1;
            log::info!("폴더 생성: {} (ID: {})", folder_name, folder_id);
        }
    }

    // 파일들을 크기별로 분류하여 100MB 이상만 병렬 처리
    let mut large_files = Vec::new(); // 100MB 이상 - 병렬 처리
    let mut small_files = Vec::new(); // 100MB 미만 - 순차 처리

    for entry in WalkDir::new(source_path).into_iter() {
        let entry = entry.map_err(|e| format!("폴더 순회 실패: {}", e))?;
        let entry_path = entry.path();

        if entry_path.is_file() {
            let metadata =
                fs::metadata(entry_path).map_err(|e| format!("파일 정보 읽기 실패: {}", e))?;
            let file_size = metadata.len();

            if file_size >= 100 * 1024 * 1024 {
                large_files.push((entry_path.to_path_buf(), file_size));
            } else {
                small_files.push((entry_path.to_path_buf(), file_size));
            }
        }
    }

    log::info!(
        "파일 분류 완료: 큰 파일 {}개 (병렬 처리), 작은 파일 {}개 (순차 처리)",
        large_files.len(),
        small_files.len()
    );

    // 1. 큰 파일들은 병렬 암호화/압축으로 처리 (최대 성능)
    for (file_path, file_size) in large_files {
        log::info!(
            "큰 파일 병렬 처리 시작: {} ({}MB)",
            file_path.display(),
            file_size / (1024 * 1024)
        );

        // 개별 파일 내부에서 병렬 압축 + 병렬 암호화 사용
        process_large_file_with_parallel(
            &file_path,
            file_size,
            &folder_map,
            &root_folder_name,
            source_path,
            &app_state,
            &database_service,
        )?;
        file_count += 1;
    }

    // 2. 작은 파일들은 병렬 처리 (rayon 사용)
    use rayon::prelude::*;
    let small_files_results: Vec<Result<crate::models::file::FileEntry, String>> = small_files
        .par_iter()
        .map(|(file_path, file_size)| {
            process_small_file_parallel_phase1(
                file_path,
                *file_size,
                &folder_map,
                &root_folder_name,
                source_path,
                &data_dir,
                &master_key,
            )
        })
        .collect();

    // DB에 결과 저장 (배치 처리로 성능 최적화)
    let mut file_entries_to_add = Vec::new();

    for result in small_files_results {
        match result {
            Ok(file_entry) => {
                file_entries_to_add.push(file_entry);
            }
            Err(e) => {
                log::error!("파일 처리 실패: {}", e);
                // 실패 처리는 로그만 남기고 계속 진행
            }
        }
    }

    if !file_entries_to_add.is_empty() {
        if let Err(e) = database_service.add_files_batch(&file_entries_to_add) {
            log::error!("파일 메타데이터 배치 추가 실패: {}", e);
            // 배치 실패 시 개별 파일 롤백은 복잡하므로 로그만 남김
        } else {
            file_count += file_entries_to_add.len() as u32;
        }
    }

    log::info!("모든 파일 처리 완료: {} 파일", file_count);

    log::info!(
        "폴더 추가 완료: 폴더 {}개, 파일 {}개",
        folder_count,
        file_count
    );

    Ok(AddFolderResult {
        folder_count: folder_count,
        file_count: file_count,
    })
}

/// 폴더를 볼트 외부로 내보냅니다 (재귀적으로 내부 파일과 하위 폴더 포함).
///
/// # 매개변수
/// * `folder_id` - 폴더 ID
/// * `export_path` - 내보낼 경로
/// * `state` - 애플리케이션 상태
///
/// # 반환값
/// * `Result<ExportFolderResult, String>` - 내보내기 결과 (폴더 수, 파일 수)
#[tauri::command]
pub async fn export_folder_from_vault(
    folder_id: String,
    export_path: String,
    state: State<'_, Mutex<AppState>>,
) -> Result<ExportFolderResult, String> {
    use std::fs;
    use std::path::Path;

    log::info!(
        "폴더 내보내기 요청: folder_id={}, export_path={}",
        folder_id,
        export_path
    );

    // 폴더 ID 파싱
    let folder_uuid = match uuid::Uuid::parse_str(&folder_id) {
        Ok(uuid) => {
            log::info!("폴더 ID 파싱 성공: {}", uuid);
            uuid
        }
        Err(e) => {
            log::error!("폴더 ID 파싱 실패: {} -> {}", folder_id, e);
            return Err("잘못된 폴더 ID 형식입니다.".to_string());
        }
    };

    let app_state = state.lock().map_err(|e| {
        log::error!("상태 잠금 실패: {}", e);
        format!("상태 잠금 실패: {}", e)
    })?;

    let database_service = app_state.database_service.lock().map_err(|e| {
        log::error!("데이터베이스 서비스 잠금 실패: {}", e);
        format!("데이터베이스 서비스 잠금 실패: {}", e)
    })?;

    // 폴더 메타데이터 조회
    let folder_entry = match database_service.get_folder(&folder_uuid) {
        Ok(Some(folder)) => folder,
        Ok(None) => {
            log::error!("폴더를 찾을 수 없습니다: {}", folder_uuid);
            return Err("폴더를 찾을 수 없습니다.".to_string());
        }
        Err(e) => {
            log::error!("폴더 조회 실패: {}", e);
            return Err(format!("폴더 조회 실패: {}", e));
        }
    };

    log::info!("폴더 메타데이터 조회 완료: {}", folder_entry.name);

    // 내보낼 경로 검증
    let export_path_obj = Path::new(&export_path);

    // 부모 디렉토리가 존재하는지 확인
    if let Some(parent_dir) = export_path_obj.parent() {
        if !parent_dir.exists() {
            log::error!("대상 디렉토리가 존재하지 않습니다: {:?}", parent_dir);
            return Err("대상 디렉토리가 존재하지 않습니다.".to_string());
        }
    }

    // 폴더가 이미 존재하는지 확인
    if export_path_obj.exists() {
        log::warn!("대상 폴더가 이미 존재합니다: {}", export_path);
        return Err("대상 폴더가 이미 존재합니다. 다른 이름을 선택해주세요.".to_string());
    }

    // 대상 폴더 생성
    fs::create_dir_all(&export_path_obj).map_err(|e| {
        log::error!("대상 폴더 생성 실패: {} -> {}", export_path, e);
        format!("대상 폴더 생성 실패: {}", e)
    })?;

    // 폴더 내용을 재귀적으로 내보내기
    let mut folder_count = 1; // 현재 폴더 포함
    let mut file_count = 0;

    // 현재 폴더의 파일들 내보내기
    let files = database_service
        .get_files_by_folder(Some(folder_uuid))
        .map_err(|e| format!("폴더 파일 목록 조회 실패: {}", e))?;

    for file in files {
        // TODO: 실제 구현에서는 암호화된 파일을 복호화해야 함
        let file_export_path = export_path_obj.join(&file.original_file_name);

        let dummy_content = format!(
            "SecureVault 내보내기 파일\n\n파일명: {}\n원본명: {}\n크기: {} 바이트\n생성일: {}\n수정일: {}\n\n이 파일은 SecureVault에서 내보낸 파일입니다.",
            file.file_name,
            file.original_file_name,
            file.file_size,
            file.created_date.format("%Y-%m-%d %H:%M:%S"),
            file.modified_date.format("%Y-%m-%d %H:%M:%S")
        );

        fs::write(&file_export_path, dummy_content.as_bytes()).map_err(|e| {
            log::error!("파일 내보내기 실패: {:?} -> {}", file_export_path, e);
            format!("파일 내보내기 실패: {}", e)
        })?;

        file_count += 1;
        log::info!("파일 내보내기 완료: {}", file.original_file_name);
    }

    // 하위 폴더들 재귀적으로 내보내기
    let subfolders = get_subfolders_recursive(&*database_service, Some(folder_uuid))
        .map_err(|e| format!("하위 폴더 조회 실패: {}", e))?;

    for subfolder in subfolders {
        let subfolder_export_path = export_path_obj.join(&subfolder.name);

        // 하위 폴더 재귀 내보내기
        let result =
            export_folder_recursive(&*database_service, &subfolder, &subfolder_export_path)
                .map_err(|e| format!("하위 폴더 내보내기 실패: {}", e))?;

        folder_count += result.folder_count;
        file_count += result.file_count;
    }

    log::info!(
        "폴더 내보내기 완료: {} -> {} (폴더 {}개, 파일 {}개)",
        folder_entry.name,
        export_path,
        folder_count,
        file_count
    );

    Ok(ExportFolderResult {
        folder_count: folder_count,
        file_count: file_count,
    })
}

/// 폴더 내보내기 결과 구조체
#[derive(serde::Serialize)]
pub struct ExportFolderResult {
    /// 내보낸 폴더 수
    pub folder_count: u32,
    /// 내보낸 파일 수
    pub file_count: u32,
}

/// 하위 폴더들을 재귀적으로 조회합니다.
fn get_subfolders_recursive(
    _database_service: &crate::services::database::DatabaseService,
    _parent_id: Option<uuid::Uuid>,
) -> Result<Vec<crate::models::folder::FolderEntry>, crate::models::error::VaultError> {
    // TODO: 실제 구현에서는 데이터베이스에서 하위 폴더 목록을 조회해야 함
    // 현재는 빈 벡터 반환
    Ok(Vec::new())
}

/// 폴더를 재귀적으로 내보냅니다.
fn export_folder_recursive(
    database_service: &crate::services::database::DatabaseService,
    folder: &crate::models::folder::FolderEntry,
    export_path: &std::path::Path,
) -> Result<ExportFolderResult, String> {
    use std::fs;

    // 폴더 생성
    fs::create_dir_all(export_path).map_err(|e| format!("폴더 생성 실패: {}", e))?;

    let mut folder_count = 1;
    let mut file_count = 0;

    // 폴더 내 파일들 내보내기
    let files = database_service
        .get_files_by_folder(Some(folder.id))
        .map_err(|e| format!("파일 목록 조회 실패: {}", e))?;

    for file in files {
        let file_export_path = export_path.join(&file.original_file_name);

        let dummy_content = format!(
            "SecureVault 내보내기 파일\n\n파일명: {}\n원본명: {}\n크기: {} 바이트\n생성일: {}\n수정일: {}\n\n이 파일은 SecureVault에서 내보낸 파일입니다.",
            file.file_name,
            file.original_file_name,
            file.file_size,
            file.created_date.format("%Y-%m-%d %H:%M:%S"),
            file.modified_date.format("%Y-%m-%d %H:%M:%S")
        );

        fs::write(&file_export_path, dummy_content.as_bytes())
            .map_err(|e| format!("파일 내보내기 실패: {}", e))?;

        file_count += 1;
    }

    // 하위 폴더들 재귀 처리
    let subfolders = get_subfolders_recursive(database_service, Some(folder.id))
        .map_err(|e| format!("하위 폴더 조회 실패: {}", e))?;

    for subfolder in subfolders {
        let subfolder_export_path = export_path.join(&subfolder.name);
        let result = export_folder_recursive(database_service, &subfolder, &subfolder_export_path)?;

        folder_count += result.folder_count;
        file_count += result.file_count;
    }

    Ok(ExportFolderResult {
        folder_count: folder_count,
        file_count: file_count,
    })
}
#[derive(serde::Serialize)]
pub struct AddFolderResult {
    /// 추가된 폴더 수
    pub folder_count: u32,
    /// 추가된 파일 수
    pub file_count: u32,
}

/// 청크 기반 파일 업로드를 시작합니다.
///
/// # 매개변수
/// * `file_name` - 파일명
/// * `file_size` - 전체 파일 크기
/// * `folder_id` - 폴더 ID (None이면 루트)
/// * `state` - 애플리케이션 상태
///
/// # 반환값
/// * `Result<String, String>` - 업로드 세션 ID
#[tauri::command]
pub async fn start_chunked_upload(
    file_name: String,
    file_size: u64,
    folder_id: Option<String>,
    _state: State<'_, Mutex<AppState>>,
) -> Result<String, String> {
    log::info!(
        "청크 업로드 시작: file_name={}, file_size={}, folder_id={:?}",
        file_name,
        file_size,
        folder_id
    );

    // 폴더 ID 변환 및 검증
    let folder_uuid = if let Some(id_str) = &folder_id {
        match uuid::Uuid::parse_str(id_str) {
            Ok(uuid) => Some(uuid),
            Err(e) => {
                log::error!("폴더 ID 파싱 실패: {} -> {}", id_str, e);
                return Err("잘못된 폴더 ID 형식입니다.".to_string());
            }
        }
    } else {
        None
    };

    // 파일명 유효성 검사
    if file_name.trim().is_empty() {
        return Err("파일명이 비어있습니다.".to_string());
    }

    // 파일 크기 제한 검사 (5GB)
    const MAX_FILE_SIZE: u64 = 5 * 1024 * 1024 * 1024;
    if file_size > MAX_FILE_SIZE {
        return Err(format!(
            "파일 크기가 너무 큽니다. 최대 {}GB까지 지원됩니다.",
            MAX_FILE_SIZE / (1024 * 1024 * 1024)
        ));
    }

    // 업로드 세션 생성
    let session_id = uuid::Uuid::new_v4().to_string();

    // 임시 디렉토리 경로 계산 (볼트 내부 .securevault/tmp 사용)
    let current_dir = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    let mut vault_dir = current_dir.join(".securevault");
    if !vault_dir.exists() {
        if let Some(parent) = current_dir.parent() {
            let parent_vault = parent.join(".securevault");
            if parent_vault.exists() {
                vault_dir = parent_vault;
            }
        }
    }

    // 볼트 내 임시 폴더 사용 (같은 드라이브여야 rename이 빠름)
    let temp_base_dir = vault_dir.join("tmp");
    let temp_dir = temp_base_dir.join(&session_id);

    // 임시 디렉토리 생성
    std::fs::create_dir_all(&temp_dir).map_err(|e| {
        log::error!("임시 디렉토리 생성 실패: {:?} -> {}", temp_dir, e);
        format!("임시 디렉토리 생성 실패: {}", e)
    })?;

    // 업로드 세션 정보 생성
    let session = UploadSession {
        _session_id: session_id.clone(),
        file_name: file_name.clone(),
        _file_size: file_size,
        folder_id: folder_uuid,
        temp_dir: temp_dir.clone(),
        _created_at: chrono::Utc::now(),
    };

    // 세션 정보를 전역 맵에 저장
    {
        let mut sessions = UPLOAD_SESSIONS.lock().map_err(|e| {
            log::error!("세션 맵 잠금 실패: {}", e);
            format!("세션 맵 잠금 실패: {}", e)
        })?;
        sessions.insert(session_id.clone(), session);
    }

    log::info!(
        "청크 업로드 세션 생성: session_id={}, file_name={}, temp_dir={:?}",
        session_id,
        file_name,
        temp_dir
    );
    Ok(session_id)
}

/// 파일 청크를 업로드합니다.
///
/// # 매개변수
/// * `session_id` - 업로드 세션 ID
/// * `chunk_index` - 청크 인덱스 (0부터 시작)
/// * `chunk_data` - 청크 데이터 (base64 인코딩)
/// * `is_last_chunk` - 마지막 청크 여부
/// * `state` - 애플리케이션 상태
///
/// # 반환값
/// * `Result<bool, String>` - 업로드 완료 여부
#[tauri::command]
pub async fn upload_file_chunk(
    session_id: String,
    chunk_index: u32,
    chunk_data: String,
    is_last_chunk: bool,
    state: State<'_, Mutex<AppState>>,
) -> Result<bool, String> {
    use base64::{engine::general_purpose, Engine as _};
    use std::io::Write;

    log::info!(
        "청크 업로드: session_id={}, chunk_index={}, is_last={}",
        session_id,
        chunk_index,
        is_last_chunk
    );

    // 세션 ID 검증
    let _session_uuid = uuid::Uuid::parse_str(&session_id).map_err(|e| {
        log::error!("세션 ID 파싱 실패: {} -> {}", session_id, e);
        format!("잘못된 세션 ID 형식: {}", e)
    })?;

    // 세션 정보 조회
    let session = {
        let sessions = UPLOAD_SESSIONS.lock().map_err(|e| {
            log::error!("세션 맵 잠금 실패: {}", e);
            format!("세션 맵 잠금 실패: {}", e)
        })?;

        sessions.get(&session_id).cloned().ok_or_else(|| {
            log::error!("업로드 세션을 찾을 수 없습니다: {}", session_id);
            "업로드 세션을 찾을 수 없습니다.".to_string()
        })?
    };

    let temp_dir = &session.temp_dir;

    // base64 디코딩
    let binary_data = general_purpose::STANDARD.decode(&chunk_data).map_err(|e| {
        log::error!("청크 데이터 디코딩 실패: {}", e);
        format!("청크 데이터 디코딩 실패: {}", e)
    })?;

    // 청크 파일 저장 (압축 적용)
    let mut processed_data = binary_data.clone();
    let mut is_chunk_compressed = false;

    // 청크 크기가 충분히 클 때만 압축 시도 (1KB 이상)
    if binary_data.len() > 1024 {
        // 압축 시도 - 더 빠른 압축 레벨 사용
        use flate2::write::GzEncoder;
        use flate2::Compression;
        use std::io::Write;

        // 빠른 압축 레벨 사용 (메모리와 속도 최적화)
        let mut encoder = GzEncoder::new(Vec::new(), Compression::fast());
        if let Ok(()) = encoder.write_all(&binary_data) {
            if let Ok(compressed_data) = encoder.finish() {
                // 압축률이 좋을 때만 압축된 데이터 사용 (5% 이상 절약으로 완화)
                if compressed_data.len() < binary_data.len() * 95 / 100 {
                    processed_data = compressed_data;
                    is_chunk_compressed = true;
                    log::info!(
                        "청크 압축 완료: {} -> {} bytes ({:.1}% 절약)",
                        binary_data.len(),
                        processed_data.len(),
                        (1.0 - processed_data.len() as f64 / binary_data.len() as f64) * 100.0
                    );
                } else {
                    log::debug!(
                        "청크 압축 효과 미미, 원본 사용: {} bytes",
                        binary_data.len()
                    );
                }
            } else {
                log::debug!("청크 압축 실패, 원본 사용: {} bytes", binary_data.len());
            }
        } else {
            log::debug!(
                "청크 압축 쓰기 실패, 원본 사용: {} bytes",
                binary_data.len()
            );
        }
    } else {
        log::debug!("청크 크기 작음, 압축 건너뜀: {} bytes", binary_data.len());
    }

    let chunk_file_path = temp_dir.join(format!(
        "chunk_{:06}_{}",
        chunk_index,
        if is_chunk_compressed { "gz" } else { "raw" }
    ));
    let mut chunk_file = std::fs::File::create(&chunk_file_path).map_err(|e| {
        log::error!("청크 파일 생성 실패: {:?} -> {}", chunk_file_path, e);
        format!("청크 파일 생성 실패: {}", e)
    })?;

    chunk_file.write_all(&processed_data).map_err(|e| {
        log::error!("청크 데이터 쓰기 실패: {}", e);
        format!("청크 데이터 쓰기 실패: {}", e)
    })?;

    log::info!(
        "청크 저장 완료: {:?} ({} bytes, 압축: {})",
        chunk_file_path,
        processed_data.len(),
        is_chunk_compressed
    );

    // 마지막 청크인 경우 파일 조립 및 볼트에 저장
    if is_last_chunk {
        log::info!(
            "마지막 청크 수신, 파일 조립 및 볼트 저장 시작: session_id={}",
            session_id
        );

        // 모든 청크 파일을 하나로 합치기 - 메모리 효율적인 스트리밍 방식
        let final_file_path = temp_dir.join("assembled_file");
        let mut final_file = std::fs::File::create(&final_file_path).map_err(|e| {
            log::error!("최종 파일 생성 실패: {:?} -> {}", final_file_path, e);
            format!("최종 파일 생성 실패: {}", e)
        })?;

        // 청크들을 순서대로 읽어서 합치기 (스트리밍 방식으로 메모리 절약)
        let mut current_chunk = 0;
        let mut total_size = 0u64;
        const BUFFER_SIZE: usize = 1024 * 1024; // 1MB 버퍼로 증가 (성능 향상)

        loop {
            // 압축된 청크와 원본 청크 모두 확인
            let chunk_path_gz = temp_dir.join(format!("chunk_{:06}_gz", current_chunk));
            let chunk_path_raw = temp_dir.join(format!("chunk_{:06}_raw", current_chunk));

            let (chunk_path, is_compressed) = if chunk_path_gz.exists() {
                (chunk_path_gz, true)
            } else if chunk_path_raw.exists() {
                (chunk_path_raw, false)
            } else {
                // 더 이상 청크가 없음
                break;
            };

            // 압축된 청크인 경우 압축 해제하면서 스트리밍
            if is_compressed {
                use flate2::read::GzDecoder;
                use std::io::{BufReader, Read};

                let chunk_file = std::fs::File::open(&chunk_path).map_err(|e| {
                    log::error!("청크 파일 열기 실패: {:?} -> {}", chunk_path, e);
                    format!("청크 파일 열기 실패: {}", e)
                })?;

                let mut decoder = GzDecoder::new(BufReader::new(chunk_file));
                let mut buffer = vec![0u8; BUFFER_SIZE];

                loop {
                    match decoder.read(&mut buffer) {
                        Ok(0) => break, // EOF
                        Ok(n) => {
                            final_file.write_all(&buffer[..n]).map_err(|e| {
                                log::error!("최종 파일 쓰기 실패: {}", e);
                                format!("최종 파일 쓰기 실패: {}", e)
                            })?;
                            total_size += n as u64;
                        }
                        Err(e) => {
                            log::error!("청크 압축 해제 실패: {:?} -> {}", chunk_path, e);
                            return Err(format!("청크 압축 해제 실패: {}", e));
                        }
                    }
                }

                log::debug!("압축 청크 처리 완료: {:?}", chunk_path);
            } else {
                // 원본 청크를 스트리밍으로 복사
                use std::io::BufReader;

                let chunk_file = std::fs::File::open(&chunk_path).map_err(|e| {
                    log::error!("청크 파일 열기 실패: {:?} -> {}", chunk_path, e);
                    format!("청크 파일 열기 실패: {}", e)
                })?;

                let mut reader = BufReader::new(chunk_file);
                let mut buffer = vec![0u8; BUFFER_SIZE];

                loop {
                    match reader.read(&mut buffer) {
                        Ok(0) => break, // EOF
                        Ok(n) => {
                            final_file.write_all(&buffer[..n]).map_err(|e| {
                                log::error!("최종 파일 쓰기 실패: {}", e);
                                format!("최종 파일 쓰기 실패: {}", e)
                            })?;
                            total_size += n as u64;
                        }
                        Err(e) => {
                            log::error!("청크 파일 읽기 실패: {:?} -> {}", chunk_path, e);
                            return Err(format!("청크 파일 읽기 실패: {}", e));
                        }
                    }
                }

                log::debug!("원본 청크 처리 완료: {:?}", chunk_path);
            }

            // 청크 파일 즉시 삭제하여 디스크 공간 절약
            let _ = std::fs::remove_file(&chunk_path);
            current_chunk += 1;

            // 주기적으로 로그 출력 (진행 상황 확인)
            if current_chunk % 10 == 0 {
                log::info!(
                    "청크 조립 진행: {}/{} 청크 처리 완료",
                    current_chunk,
                    current_chunk
                );
            }
        }

        // 파일 쓰기 완료
        drop(final_file);

        log::info!(
            "파일 조립 완료: {:?} ({} 청크, {} bytes)",
            final_file_path,
            current_chunk,
            total_size
        );

        // TODO: 세션 정보에서 파일명과 폴더 ID 가져오기
        let file_name = session.file_name.clone();
        let folder_id = session.folder_id;

        // 스트리밍 방식으로 파일 암호화 및 저장 구현
        let file_entry = {
            let app_state = state.lock().map_err(|e| {
                log::error!("상태 잠금 실패: {}", e);
                format!("상태 잠금 실패: {}", e)
            })?;

            // 파일 크기 확인 (메모리에 로드하지 않고)
            let original_size = std::fs::metadata(&final_file_path)
                .map_err(|e| {
                    log::error!("조립된 파일 정보 읽기 실패: {:?} -> {}", final_file_path, e);
                    format!("조립된 파일 정보 읽기 실패: {}", e)
                })?
                .len();

            // 파일 확장자 추출
            let file_extension = std::path::Path::new(&file_name)
                .extension()
                .and_then(|ext| ext.to_str())
                .unwrap_or("")
                .to_string();

            // 압축 서비스를 통한 파일 압축 처리 (스트리밍 방식)
            let (processed_file_path, compression_result) = {
                let compression_service = app_state.compression_service.lock().map_err(|e| {
                    log::error!("압축 서비스 잠금 실패: {}", e);
                    format!("압축 서비스 잠금 실패: {}", e)
                })?;

                // 파일 크기에 따른 최적화된 처리 방식 선택
                if original_size > 100 * 1024 * 1024 {
                    // 100MB 이상은 병렬 처리
                    log::info!(
                        "큰 파일 감지 ({}MB), 병렬 압축 + 병렬 암호화 사용",
                        original_size / (1024 * 1024)
                    );

                    // 압축된 파일을 위한 임시 파일 생성
                    let compressed_file_path = temp_dir.join("compressed_file");

                    // 병렬 압축 수행
                    match compression_service.compress_file_parallel_streaming(
                        &final_file_path,
                        &compressed_file_path,
                        &file_extension,
                    ) {
                        Ok(result) => {
                            log::info!(
                                "병렬 압축 완료: {} -> {} ({:.1}% 절약)",
                                result.original_size,
                                result.compressed_size,
                                result.space_saved_percent()
                            );
                            (compressed_file_path, Some(result))
                        }
                        Err(e) => {
                            log::warn!("병렬 압축 실패, 원본 사용: {}", e);
                            (final_file_path.clone(), None)
                        }
                    }
                } else {
                    // 작은 파일은 기존 방식 사용
                    let file_data = std::fs::read(&final_file_path).map_err(|e| {
                        log::error!("파일 읽기 실패: {:?} -> {}", final_file_path, e);
                        format!("파일 읽기 실패: {}", e)
                    })?;

                    let (processed_data, compression_result) = compression_service
                        .compress_file_data(&file_data, &file_extension)
                        .map_err(|e| {
                            log::error!("파일 압축 실패: {}", e);
                            format!("파일 압축 실패: {}", e)
                        })?;

                    if compression_result.is_some() {
                        // 압축된 데이터를 임시 파일에 저장
                        let compressed_file_path = temp_dir.join("compressed_file");
                        std::fs::write(&compressed_file_path, &processed_data).map_err(|e| {
                            log::error!(
                                "압축된 파일 저장 실패: {:?} -> {}",
                                compressed_file_path,
                                e
                            );
                            format!("압축된 파일 저장 실패: {}", e)
                        })?;
                        (compressed_file_path, compression_result)
                    } else {
                        (final_file_path.clone(), None)
                    }
                }
            };

            // 압축 정보 추출
            let (is_compressed, compressed_size, compression_ratio) =
                if let Some(result) = &compression_result {
                    log::info!(
                        "청크 파일 압축 완료: {} -> {} ({:.1}% 절약)",
                        result.original_size,
                        result.compressed_size,
                        result.space_saved_percent()
                    );
                    (true, result.compressed_size, result.compression_ratio)
                } else {
                    log::info!("청크 파일 압축 건너뜀: {}", file_name);
                    (false, original_size, 1.0)
                };

            // 스트리밍 암호화 및 저장 (Vault Path도 함께 반환)
            let (encrypted_file_name, encrypted_size, vault_path_opt) = {
                let mut file_service = app_state.file_service.lock().map_err(|e| {
                    log::error!("파일 서비스 잠금 실패: {}", e);
                    format!("파일 서비스 잠금 실패: {}", e)
                })?;

                // 파일 서비스 초기화 확인
                if !file_service.is_initialized() {
                    log::error!("파일 서비스가 초기화되지 않았습니다. (Master Key 없음)");
                    return Err("로그인이 필요합니다. (파일 서비스 미초기화)".to_string());
                }

                let current_vault_path = file_service.get_vault_path();

                let encrypted_file_name = format!("encrypted_{}", uuid::Uuid::new_v4());

                // 암호화된 파일을 저장할 경로 설정
                let vault_path = current_vault_path.clone().unwrap_or_else(|| {
                    std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."))
                });

                let data_dir = vault_path.join(".securevault").join("files");

                if !data_dir.exists() {
                    std::fs::create_dir_all(&data_dir).map_err(|e| {
                        log::error!("데이터 디렉토리 생성 실패: {:?} -> {}", data_dir, e);
                        format!("데이터 디렉토리 생성 실패: {}", e)
                    })?;
                }

                let encrypted_file_path = data_dir.join(&encrypted_file_name);

                // 파일 크기에 따른 최적화된 암호화 방식 선택
                let encrypted_size = if original_size > 100 * 1024 * 1024 {
                    // 100MB 이상은 병렬 암호화
                    log::info!(
                        "큰 파일 병렬 암호화 시작: {}MB",
                        original_size / (1024 * 1024)
                    );
                    file_service
                        .encrypt_file_parallel_streaming(&processed_file_path, &encrypted_file_path)
                        .map_err(|e| {
                            log::error!("병렬 암호화 실패: {}", e);
                            format!("병렬 암호화 실패: {}", e)
                        })?
                } else {
                    // 작은 파일은 기존 스트리밍 암호화 사용
                    file_service
                        .encrypt_file_streaming(&processed_file_path, &encrypted_file_path)
                        .map_err(|e| {
                            log::error!("스트리밍 암호화 실패: {}", e);
                            format!("스트리밍 암호화 실패: {}", e)
                        })?
                };

                log::info!(
                    "스트리밍 암호화 완료: {} -> {} bytes",
                    processed_file_path.display(),
                    encrypted_size
                );

                (encrypted_file_name, encrypted_size, current_vault_path)
            };

            // MIME 타입 추정
            let mime_type = match file_extension.to_lowercase().as_str() {
                // 텍스트 파일
                "txt" => "text/plain",
                "md" => "text/markdown",
                "json" => "application/json",
                "xml" => "application/xml",
                "csv" => "text/csv",
                "html" => "text/html",
                "css" => "text/css",
                "js" => "application/javascript",
                "py" => "text/x-python",
                "cs" => "text/x-csharp",
                "java" => "text/x-java-source",
                "cpp" => "text/x-c++src",
                "sql" => "application/sql",
                "yaml" | "yml" => "application/x-yaml",
                "ini" => "text/plain",
                "log" => "text/plain",

                // 이미지 파일
                "jpg" | "jpeg" => "image/jpeg",
                "png" => "image/png",
                "gif" => "image/gif",
                "bmp" => "image/bmp",
                "webp" => "image/webp",
                "svg" => "image/svg+xml",
                "ico" => "image/x-icon",

                // 오디오 파일
                "mp3" => "audio/mpeg",
                "wav" => "audio/wav",
                "ogg" => "audio/ogg",
                "m4a" => "audio/mp4",
                "flac" => "audio/flac",

                // 비디오 파일
                "mp4" => "video/mp4",
                "avi" => "video/x-msvideo",
                "mov" => "video/quicktime",
                "wmv" => "video/x-ms-wmv",
                "webm" => "video/webm",

                // 문서 파일
                "pdf" => "application/pdf",
                "doc" => "application/msword",
                "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
                "xls" => "application/vnd.ms-excel",
                "xlsx" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
                "ppt" => "application/vnd.ms-powerpoint",
                "pptx" => {
                    "application/vnd.openxmlformats-officedocument.presentationml.presentation"
                }

                // 압축 파일
                "zip" => "application/zip",
                "rar" => "application/vnd.rar",
                "7z" => "application/x-7z-compressed",
                "tar" => "application/x-tar",
                "gz" => "application/gzip",

                // 실행 파일
                "exe" => "application/x-msdownload",
                "msi" => "application/x-msi",
                "msix" => "application/x-msix",
                "appx" => "application/x-appx",

                // 기본값
                _ => "application/octet-stream",
            }
            .to_string();

            // 압축 정보와 함께 파일 엔트리 생성
            let file_entry = crate::models::file::FileEntry::new_with_compression(
                file_name.clone(),
                file_name.clone(),
                original_size,
                file_extension,
                mime_type,
                "".to_string(), // TODO: 체크섬 계산
                folder_id,
                encrypted_file_name.clone(),
                encrypted_size,
                is_compressed,
                compressed_size,
                compression_ratio,
            );

            // 데이터베이스에 파일 메타데이터 저장
            let mut database_service = app_state.database_service.lock().map_err(|e| {
                log::error!("데이터베이스 서비스 잠금 실패: {}", e);
                format!("데이터베이스 서비스 잠금 실패: {}", e)
            })?;

            // 데이터베이스 초기화 확인 및 복구
            if !database_service.is_initialized() {
                log::warn!(
                    "Global DatabaseService connection lost. Attempting re-initialization..."
                );
                let vault_path = vault_path_opt.unwrap_or_else(|| {
                    std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."))
                });

                if let Some(path_str) = vault_path.to_str() {
                    if let Err(e) = database_service.initialize(path_str) {
                        log::error!("Failed to re-initialize database: {}", e);
                    } else {
                        log::info!("Database re-initialized successfully.");
                    }
                }
            }

            if let Err(e) = database_service.add_file(&file_entry) {
                log::error!("파일 메타데이터 저장 실패: {}", e);
                // 암호화된 파일 삭제 (롤백)
                let vault_path =
                    std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
                let data_dir = vault_path.join(".securevault").join("data").join("files");
                let encrypted_file_path = data_dir.join(&encrypted_file_name);
                let _ = std::fs::remove_file(&encrypted_file_path);
                return Err(format!("파일 메타데이터 저장 실패: {}", e));
            }

            log::info!(
                "청크 기반 파일 저장 완료: {} (원본: {}MB, 압축: {}MB, 암호화: {}MB)",
                file_name,
                original_size / (1024 * 1024),
                compressed_size / (1024 * 1024),
                encrypted_size / (1024 * 1024)
            );

            file_entry
        };

        // TODO: 실제 파일 암호화 및 저장 구현
        // 현재는 임시 파일만 생성하고 정리

        // 임시 디렉토리 정리
        let _ = std::fs::remove_dir_all(&temp_dir);

        // 세션 정보 제거
        {
            let mut sessions = UPLOAD_SESSIONS.lock().map_err(|e| {
                log::error!("세션 맵 잠금 실패: {}", e);
                format!("세션 맵 잠금 실패: {}", e)
            })?;
            sessions.remove(&session_id);
        }

        log::info!(
            "청크 기반 파일 업로드 완료: {} (ID: {})",
            file_name,
            file_entry.id
        );
        return Ok(true); // 업로드 완료
    }

    Ok(false) // 아직 업로드 진행 중
}

/// 청크 업로드를 취소합니다.
///
/// # 매개변수
/// * `session_id` - 업로드 세션 ID
/// * `state` - 애플리케이션 상태
///
/// # 반환값
/// * `Result<(), String>` - 취소 결과
#[tauri::command]
pub async fn cancel_chunked_upload(
    session_id: String,
    _state: State<'_, Mutex<AppState>>,
) -> Result<(), String> {
    log::info!("청크 업로드 취소: session_id={}", session_id);

    // 세션 정보 조회 및 제거
    let session = {
        let mut sessions = UPLOAD_SESSIONS.lock().map_err(|e| {
            log::error!("세션 맵 잠금 실패: {}", e);
            format!("세션 맵 잠금 실패: {}", e)
        })?;

        sessions.remove(&session_id).ok_or_else(|| {
            log::warn!("취소할 업로드 세션을 찾을 수 없습니다: {}", session_id);
            "업로드 세션을 찾을 수 없습니다.".to_string()
        })?
    };

    // 임시 디렉토리와 모든 청크 파일 삭제
    if session.temp_dir.exists() {
        std::fs::remove_dir_all(&session.temp_dir).map_err(|e| {
            log::error!("임시 디렉토리 삭제 실패: {:?} -> {}", session.temp_dir, e);
            format!("임시 디렉토리 삭제 실패: {}", e)
        })?;

        log::info!("청크 업로드 취소 완료: {:?}", session.temp_dir);
    }

    Ok(())
}

/// 큰 파일을 병렬 처리로 압축 및 암호화합니다 (100MB 이상).
///
/// # 매개변수
/// * `file_path` - 파일 경로
/// * `file_size` - 파일 크기
/// * `folder_map` - 폴더 맵
/// * `root_folder_name` - 루트 폴더명
/// * `source_path` - 소스 경로
/// * `app_state` - 애플리케이션 상태
/// * `database_service` - 데이터베이스 서비스
///
/// # 반환값
/// * `Result<(), String>` - 처리 결과
fn process_large_file_with_parallel(
    file_path: &std::path::Path,
    file_size: u64,
    folder_map: &std::collections::HashMap<String, uuid::Uuid>,
    root_folder_name: &str,
    source_path: &std::path::Path,
    app_state: &crate::AppState,
    database_service: &crate::services::database::DatabaseService,
) -> Result<(), String> {
    use std::fs;

    log::info!(
        "큰 파일 병렬 처리 시작: {} ({}MB)",
        file_path.display(),
        file_size / (1024 * 1024)
    );

    // 상대 경로 계산
    let relative_path = file_path
        .strip_prefix(source_path)
        .map_err(|e| format!("상대 경로 계산 실패: {}", e))?;

    let file_name = file_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown_file")
        .to_string();

    // 부모 폴더 ID 결정
    let parent_folder_id = if let Some(parent_path) = relative_path.parent() {
        if parent_path.as_os_str().is_empty() {
            // 루트 폴더의 직접 하위 파일
            folder_map.get(root_folder_name).copied()
        } else {
            // 더 깊은 하위 폴더의 파일
            let parent_key = parent_path.to_string_lossy().to_string();
            folder_map.get(&parent_key).copied()
        }
    } else {
        folder_map.get(root_folder_name).copied()
    };

    // 파일 확장자 추출
    let file_extension = file_path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("")
        .to_string();

    // 임시 파일 경로들 (볼트 내부 .securevault/tmp 사용)
    let mut vault_dir = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    if !vault_dir.join(".securevault").exists() {
        if let Some(parent) = vault_dir.parent() {
            if parent.join(".securevault").exists() {
                vault_dir = parent.to_path_buf();
            }
        }
    }

    let temp_dir = vault_dir.join(".securevault").join("tmp").join("parallel");
    fs::create_dir_all(&temp_dir).map_err(|e| format!("임시 디렉토리 생성 실패: {}", e))?;

    let compressed_file_path = temp_dir.join(format!("compressed_{}", uuid::Uuid::new_v4()));
    let encrypted_file_path = temp_dir.join(format!("encrypted_{}", uuid::Uuid::new_v4()));

    // 1. 병렬 압축 수행
    let compression_result = {
        let compression_service = app_state
            .compression_service
            .lock()
            .map_err(|e| format!("압축 서비스 잠금 실패: {}", e))?;

        compression_service
            .compress_file_parallel_streaming(file_path, &compressed_file_path, &file_extension)
            .map_err(|e| format!("병렬 압축 실패: {}", e))?
    };

    log::info!(
        "병렬 압축 완료: {} -> {} ({:.1}% 절약)",
        compression_result.original_size,
        compression_result.compressed_size,
        compression_result.space_saved_percent()
    );

    // 2. 병렬 암호화 수행
    let encrypted_size = {
        let file_service = app_state
            .file_service
            .lock()
            .map_err(|e| format!("파일 서비스 잠금 실패: {}", e))?;

        file_service
            .encrypt_file_parallel_streaming(&compressed_file_path, &encrypted_file_path)
            .map_err(|e| format!("병렬 암호화 실패: {}", e))?
    };

    log::info!(
        "병렬 암호화 완료: {} -> {} bytes",
        compression_result.compressed_size,
        encrypted_size
    );

    // 3. 최종 저장 위치로 이동 (FileService와 동일한 경로 사용)
    let vault_path = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    let data_dir = vault_path.join(".securevault").join("files");

    if !data_dir.exists() {
        fs::create_dir_all(&data_dir).map_err(|e| format!("데이터 디렉토리 생성 실패: {}", e))?;
    }

    // 4. 파일 엔트리 먼저 생성 (ID 생성됨)
    // MIME 타입 추론 (파일 확장자 기반)
    let mime_type = match file_extension.to_lowercase().as_str() {
        "mp3" => "audio/mpeg",
        "wav" => "audio/wav",
        "flac" => "audio/flac",
        "aac" => "audio/aac",
        "ogg" => "audio/ogg",
        "m4a" => "audio/mp4",
        "mp4" => "video/mp4",
        "avi" => "video/x-msvideo",
        "mov" => "video/quicktime",
        "mkv" => "video/x-matroska",
        "webm" => "video/webm",
        "jpg" | "jpeg" => "image/jpeg",
        "png" => "image/png",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "pdf" => "application/pdf",
        "txt" => "text/plain",
        "json" => "application/json",
        "xml" => "application/xml",
        "zip" => "application/zip",
        "rar" => "application/x-rar-compressed",
        _ => "application/octet-stream",
    }
    .to_string();

    let mut file_entry = crate::models::file::FileEntry::new_with_compression(
        file_name.clone(),
        file_name.clone(),
        compression_result.original_size,
        file_extension,
        mime_type,
        "".to_string(), // TODO: 체크섬 계산
        parent_folder_id,
        String::new(), // 나중에 설정
        encrypted_size,
        true, // 압축됨
        compression_result.compressed_size,
        compression_result.compression_ratio,
    );

    // file_entry.id를 사용하여 암호화된 파일명 생성 (ID 일치 보장)
    let encrypted_file_name = format!("{}.enc", file_entry.id);
    file_entry.encrypted_file_name = encrypted_file_name.clone();
    let final_encrypted_path = data_dir.join(&encrypted_file_name);

    // 암호화된 파일 이동 (다른 드라이브 간 이동 지원을 위해 copy fallback 추가)
    if let Err(e) = fs::rename(&encrypted_file_path, &final_encrypted_path) {
        log::warn!("암호화된 파일 이동 실패 (rename), 복사 시도: {}", e);
        // rename 실패 시 (예: 다른 드라이브) 복사 후 삭제 시도
        fs::copy(&encrypted_file_path, &final_encrypted_path).map_err(|e| {
            format!(
                "암호화된 파일 복사 실패 (Cross-device move fallback): {}",
                e
            )
        })?;
        fs::remove_file(&encrypted_file_path).map_err(|e| format!("임시 파일 삭제 실패: {}", e))?;
    }

    // 데이터베이스에 저장
    if let Err(e) = database_service.add_file(&file_entry) {
        log::error!("파일 메타데이터 저장 실패: {}", e);
        // 암호화된 파일 삭제 (롤백)
        let _ = fs::remove_file(&final_encrypted_path);
        return Err(format!("파일 메타데이터 저장 실패: {}", e));
    }

    // 임시 파일들 정리
    let _ = fs::remove_file(&compressed_file_path);
    let _ = fs::remove_dir_all(&temp_dir);

    log::info!(
        "큰 파일 병렬 처리 완료: {} (원본: {}MB, 압축: {}MB, 암호화: {}MB)",
        file_name,
        compression_result.original_size / (1024 * 1024),
        compression_result.compressed_size / (1024 * 1024),
        encrypted_size / (1024 * 1024)
    );

    Ok(())
}

/// 작은 파일을 순차 처리로 압축 및 암호화합니다 (100MB 미만).
///
/// # 매개변수
/// * `file_path` - 파일 경로
/// * `file_size` - 파일 크기
/// * `folder_map` - 폴더 맵
/// * `root_folder_name` - 루트 폴더명
/// * `source_path` - 소스 경로
/// * `database_service` - 데이터베이스 서비스
///
/// 작은 파일을 병렬로 압축 및 암호화합니다 (DB 저장 제외).
fn process_small_file_parallel_phase1(
    file_path: &std::path::Path,
    file_size: u64,
    folder_map: &std::collections::HashMap<String, uuid::Uuid>,
    root_folder_name: &str,
    source_path: &std::path::Path,
    data_dir: &std::path::Path,
    master_key: &[u8; 32],
) -> Result<crate::models::file::FileEntry, String> {
    use std::fs;

    log::debug!(
        "작은 파일 순차 처리: {} ({}KB)",
        file_path.display(),
        file_size / 1024
    );

    // 상대 경로 계산
    let relative_path = file_path
        .strip_prefix(source_path)
        .map_err(|e| format!("상대 경로 계산 실패: {}", e))?;

    let file_name = file_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown_file")
        .to_string();

    // 부모 폴더 ID 결정
    let parent_folder_id = if let Some(parent_path) = relative_path.parent() {
        if parent_path.as_os_str().is_empty() {
            // 루트 폴더의 직접 하위 파일
            folder_map.get(root_folder_name).copied()
        } else {
            // 더 깊은 하위 폴더의 파일
            let parent_key = parent_path.to_string_lossy().to_string();
            folder_map.get(&parent_key).copied()
        }
    } else {
        folder_map.get(root_folder_name).copied()
    };

    // 파일 데이터 읽기
    let file_data = fs::read(file_path).map_err(|e| format!("파일 읽기 실패: {}", e))?;

    let original_size = file_data.len() as u64;

    // 파일 확장자 추출
    let file_extension = file_path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("")
        .to_string();

    // 순차 압축 처리 (메모리 기반)
    let compression_service = crate::services::compression::CompressionService::new_with_defaults();
    let (processed_data, compression_result) = compression_service
        .compress_file_data(&file_data, &file_extension)
        .map_err(|e| format!("파일 압축 실패: {}", e))?;

    // 원본 데이터 메모리 해제 (메모리 사용량 최적화)
    drop(file_data);

    // 압축 정보 추출
    let (is_compressed, compressed_size, compression_ratio) =
        if let Some(result) = &compression_result {
            log::debug!(
                "작은 파일 압축 완료: {} -> {} ({:.1}% 절약)",
                result.original_size,
                result.compressed_size,
                result.space_saved_percent()
            );
            (true, result.compressed_size, result.compression_ratio)
        } else {
            log::debug!("작은 파일 압축 건너뜀: {}", file_name);
            (false, original_size, 1.0)
        };

    // 순차 암호화 처리 (전달받은 마스터 키 사용)
    let crypto_service = crate::services::crypto::CryptoService::new();
    let encrypted_data = crypto_service
        .encrypt_data_csharp_compatible(&processed_data, master_key)
        .map_err(|e| format!("파일 암호화 실패: {}", e))?;

    // 처리된 데이터 메모리 해제 (메모리 사용량 최적화)
    drop(processed_data);

    let encrypted_size = encrypted_data.len() as u64;

    // MIME 타입 추론 (파일 확장자 기반)
    let mime_type = match file_extension.to_lowercase().as_str() {
        "mp3" => "audio/mpeg",
        "wav" => "audio/wav",
        "flac" => "audio/flac",
        "aac" => "audio/aac",
        "ogg" => "audio/ogg",
        "m4a" => "audio/mp4",
        "mp4" => "video/mp4",
        "avi" => "video/x-msvideo",
        "mov" => "video/quicktime",
        "mkv" => "video/x-matroska",
        "webm" => "video/webm",
        "jpg" | "jpeg" => "image/jpeg",
        "png" => "image/png",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "pdf" => "application/pdf",
        "txt" => "text/plain",
        "json" => "application/json",
        "xml" => "application/xml",
        "zip" => "application/zip",
        "rar" => "application/x-rar-compressed",
        _ => "application/octet-stream",
    }
    .to_string();

    // 파일 엔트리 먼저 생성 (ID 생성됨)
    let mut file_entry = crate::models::file::FileEntry::new_with_compression(
        file_name.clone(),
        file_name.clone(),
        original_size,
        file_extension,
        mime_type,
        "".to_string(), // TODO: 체크섬 계산
        parent_folder_id,
        String::new(), // 나중에 설정
        encrypted_size,
        is_compressed,
        compressed_size,
        compression_ratio,
    );

    // file_entry.id를 사용하여 암호화된 파일명 생성 (ID 일치 보장)
    let encrypted_file_name = format!("{}.enc", file_entry.id);
    file_entry.encrypted_file_name = encrypted_file_name.clone();

    // 암호화된 파일을 디스크에 저장 (미리 생성된 data_dir 사용)
    // 매번 경로 계산하고 exists() 체크하는 오버헤드 제거
    let encrypted_file_path = data_dir.join(&encrypted_file_name);
    fs::write(&encrypted_file_path, &encrypted_data)
        .map_err(|e| format!("암호화된 파일 저장 실패: {}", e))?;

    // 데이터베이스 저장 부분 제거하고 file_entry 반환
    log::debug!(
        "작은 파일 처리 완료 (DB 저장 대기): {} (원본: {}KB, 압축: {}KB, 암호화: {}KB)",
        file_name,
        original_size / 1024,
        compressed_size / 1024,
        encrypted_size / 1024
    );

    Ok(file_entry)
}

/// 파일을 외부로 내보냅니다 (복호화 + 압축해제 후 저장).
///
/// # 매개변수
/// * `file_id` - 내보낼 파일의 ID
/// * `export_path` - 내보낼 경로 (파일 전체 경로)
/// * `state` - 애플리케이션 상태
///
/// # 반환값
/// * `Result<(), String>` - 성공 시 빈 값, 실패 시 에러 메시지
#[tauri::command]
pub async fn export_file(
    file_id: String,
    export_path: String,
    state: State<'_, Mutex<AppState>>,
) -> Result<(), String> {
    log::info!(
        "파일 내보내기 요청 (Delegated): file_id={}, export_path={}",
        file_id,
        export_path
    );

    // 1. 파일 ID 파싱
    let file_uuid = uuid::Uuid::parse_str(&file_id).map_err(|e| {
        log::error!("파일 ID 파싱 실패: {} -> {}", file_id, e);
        format!("잘못된 파일 ID 형식: {}", e)
    })?;

    // 2. 파일 서비스 준비
    let file_service_clone = {
        let app_state = state.lock().map_err(|e| {
            log::error!("상태 잠금 실패: {}", e);
            format!("상태 잠금 실패: {}", e)
        })?;
        let file_service_guard = app_state.file_service.lock().map_err(|e| {
            log::error!("파일 서비스 잠금 실패: {}", e);
            format!("파일 서비스 잠금 실패: {}", e)
        })?;
        file_service_guard.clone()
    };
    let mut file_service = file_service_clone;

    // 3. FileService::export_file 호출
    file_service
        .export_file(&file_uuid, &export_path)
        .await
        .map_err(|e| {
            log::error!("파일 내보내기 실패: {}", e);
            format!("파일 내보내기 실패: {}", e)
        })?;

    log::info!("파일 내보내기 완료: {}", export_path);
    Ok(())
}

/// 유틸리티 함수: 중복된 파일명 처리
/// 폴더 내에 동일한 이름의 파일이 있으면 (1), (2) 등을 붙여 고유한 이름을 생성합니다.
fn get_unique_filename(
    state: &State<'_, Mutex<AppState>>,
    folder_id: Option<uuid::Uuid>,
    original_name: &str,
) -> Result<String, String> {
    use std::path::Path;

    // DB 서비스 잠금
    let app_state = state.lock().map_err(|e| format!("상태 잠금 실패: {}", e))?;
    let db_service = app_state
        .database_service
        .lock()
        .map_err(|e| format!("데이터베이스 서비스 잠금 실패: {}", e))?;

    // 해당 폴더의 모든 파일 가져오기
    let files = db_service
        .get_files_by_folder(folder_id)
        .map_err(|e| format!("파일 목록 조회 실패: {}", e))?;

    // 이름 집합 생성
    let mut names = std::collections::HashSet::new();
    for file in files {
        names.insert(file.file_name);
    }

    // 중복 검사
    if !names.contains(original_name) {
        return Ok(original_name.to_string());
    }

    // 중복 해결 (번호 붙이기)
    let path = Path::new(original_name);
    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(original_name);
    let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");

    let mut i = 1;
    loop {
        let new_name = if ext.is_empty() {
            format!("{}({})", stem, i)
        } else {
            format!("{}({}).{}", stem, i, ext)
        };

        if !names.contains(&new_name) {
            return Ok(new_name);
        }
        i += 1;
    }
}

/// 파일을 다른 폴더로 이동합니다.
///
/// # 매개변수
/// * `file_id` - 파일 ID
/// * `target_folder_id` - 대상 폴더 ID (None이면 루트)
/// * `state` - 애플리케이션 상태
///
/// # 반환값
/// * `Result<(), String>` - 이동 결과
#[tauri::command]
pub async fn move_file(
    file_id: String,
    target_folder_id: Option<String>,
    state: State<'_, Mutex<AppState>>,
) -> Result<(), String> {
    log::info!(
        "파일 이동 요청: file_id={}, target_folder_id={:?}",
        file_id,
        target_folder_id
    );

    // 파일 ID 파싱
    let file_uuid = match uuid::Uuid::parse_str(&file_id) {
        Ok(uuid) => uuid,
        Err(e) => return Err(format!("잘못된 파일 ID 형식: {}", e)),
    };

    // 대상 폴더 ID 파싱
    let target_folder_uuid = if let Some(id_str) = target_folder_id {
        match uuid::Uuid::parse_str(&id_str) {
            Ok(uuid) => Some(uuid),
            Err(e) => return Err(format!("잘못된 폴더 ID 형식: {}", e)),
        }
    } else {
        None
    };

    let app_state = state.lock().map_err(|e| format!("상태 잠금 실패: {}", e))?;
    let database_service = app_state
        .database_service
        .lock()
        .map_err(|e| format!("데이터베이스 서비스 잠금 실패: {}", e))?;

    // 파일 존재 확인
    let mut file_entry = match database_service.get_file(&file_uuid) {
        Ok(Some(file)) => file,
        Ok(None) => return Err("파일을 찾을 수 없습니다.".to_string()),
        Err(e) => return Err(format!("파일 조회 실패: {}", e)),
    };

    // 이동하려는 폴더가 현재 폴더와 같은지 확인
    if file_entry.folder_id == target_folder_uuid {
        return Ok(()); // 변경 없음
    }

    // 대상 폴더 존재 확인 (루트가 아닌 경우)
    if let Some(folder_id) = target_folder_uuid {
        match database_service.get_folder(&folder_id) {
            Ok(Some(_)) => {}
            Ok(None) => return Err("대상 폴더를 찾을 수 없습니다.".to_string()),
            Err(e) => return Err(format!("대상 폴더 조회 실패: {}", e)),
        }
    }

    // 대상 폴더에 같은 이름의 파일이 있는지 확인
    let existing_files = database_service
        .get_files_by_folder(target_folder_uuid)
        .map_err(|e| format!("대상 폴더 파일 목록 조회 실패: {}", e))?;

    if existing_files
        .iter()
        .any(|f| f.file_name.eq_ignore_ascii_case(&file_entry.file_name))
    {
        return Err("대상 폴더에 같은 이름의 파일이 이미 존재합니다.".to_string());
    }

    // 폴더 ID 업데이트
    file_entry.folder_id = target_folder_uuid;
    file_entry.modified_date = chrono::Utc::now();

    // DB 업데이트
    database_service
        .update_file(&file_entry)
        .map_err(|e| format!("파일 이동 실패: {}", e))?;

    log::info!(
        "파일 이동 완료: {} -> {:?}",
        file_entry.file_name,
        target_folder_uuid
    );
    Ok(())
}
