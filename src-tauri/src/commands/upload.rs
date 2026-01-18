// 업로드 관련 Tauri 명령어
// 백그라운드 파일 업로드, 진행률 조회, 작업 취소 기능을 제공합니다.

use crate::services::upload_manager::UploadJob;
use crate::AppState;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, State};
use uuid::Uuid;

/// 파일 업로드를 시작합니다 (백그라운드 처리).
///
/// # 반환값
/// * `String` - 생성된 작업 ID
#[tauri::command]
pub async fn start_file_upload(
    file_path: String,
    file_name: Option<String>,
    folder_id: Option<String>,
    app_handle: AppHandle,
    state: State<'_, Mutex<AppState>>,
) -> Result<String, String> {
    use std::path::Path;

    log::info!("백그라운드 업로드 시작 요청: {}", file_path);

    // 파일 존재 확인
    let source_path = Path::new(&file_path);
    if !source_path.exists() {
        return Err("파일이 존재하지 않습니다.".to_string());
    }

    // 파일 크기 확인
    let file_size = std::fs::metadata(&source_path)
        .map_err(|e| format!("파일 정보 읽기 실패: {}", e))?
        .len();

    // 파일명 결정
    let actual_file_name = file_name.unwrap_or_else(|| {
        source_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown_file")
            .to_string()
    });

    // 폴더 ID 파싱
    let folder_uuid = if let Some(id_str) = &folder_id {
        match Uuid::parse_str(id_str) {
            Ok(uuid) => Some(uuid),
            Err(_) => return Err("잘못된 폴더 ID 형식입니다.".to_string()),
        }
    } else {
        None
    };

    // UploadManager에서 작업 ID 가져오기 및 필요한 서비스 복제
    let (job_id, upload_manager, file_service) = {
        let app_state = state.lock().map_err(|e| format!("상태 잠금 실패: {}", e))?;
        let job_id = app_state.upload_manager.add_job(
            file_path.clone(),
            actual_file_name.clone(),
            folder_uuid,
            file_size,
        );
        let upload_manager = app_state.upload_manager.clone();
        let file_service = app_state.file_service.lock().unwrap().clone();
        (job_id, upload_manager, file_service)
    };

    // 백그라운드 스레드에서 업로드 처리
    std::thread::spawn(move || {
        // 작업 시작 표시
        let tracker = upload_manager.mark_job_started(&job_id, file_size);

        let Some(tracker) = tracker else {
            log::error!("업로드 추적기 생성 실패: {}", job_id);
            return;
        };

        // FileService를 mutable로 재바인딩
        let mut file_service = file_service;

        // 취소 토큰 참조
        let cancel_token_ref = &tracker.cancellation_token;

        // 진행률 이벤트 발송 핸들
        let app_handle_for_progress = app_handle.clone();
        let job_id_for_events = job_id;

        // add_file_with_progress를 호출 (청크별 진행률 업데이트)
        let result = file_service.add_file_with_progress(
            &file_path,
            &actual_file_name,
            folder_uuid,
            Some(cancel_token_ref),
            |bytes_processed, total_bytes| {
                // ProgressTracker 업데이트
                tracker
                    .bytes_processed
                    .store(bytes_processed, std::sync::atomic::Ordering::SeqCst);

                // Tauri 이벤트 발송
                let progress = if total_bytes > 0 {
                    (bytes_processed as f64) / (total_bytes as f64)
                } else {
                    1.0
                };

                let _ = app_handle_for_progress.emit(
                    "upload://progress",
                    serde_json::json!({
                        "job_id": job_id_for_events.to_string(),
                        "progress": progress,
                        "bytes_processed": bytes_processed,
                        "total_bytes": total_bytes,
                    }),
                );
            },
        );

        // 진행률 100% 설정
        tracker
            .bytes_processed
            .store(file_size, std::sync::atomic::Ordering::SeqCst);

        match result {
            Ok(file_entry) => {
                upload_manager.mark_job_completed(&job_id, file_entry.id);

                // 완료 이벤트 발송
                let _ = app_handle.emit(
                    "upload://complete",
                    serde_json::json!({
                        "job_id": job_id.to_string(),
                        "file_id": file_entry.id.to_string(),
                    }),
                );
            }
            Err(error) => {
                // 취소된 경우 별도 처리
                if tracker.cancellation_token.is_cancelled() {
                    upload_manager.cancel_job(&job_id);

                    let _ = app_handle.emit(
                        "upload://cancelled",
                        serde_json::json!({
                            "job_id": job_id.to_string(),
                        }),
                    );
                } else {
                    let error_msg = format!("{}", error);
                    upload_manager.mark_job_failed(&job_id, error_msg.clone());

                    let _ = app_handle.emit(
                        "upload://error",
                        serde_json::json!({
                            "job_id": job_id.to_string(),
                            "error": error_msg,
                        }),
                    );
                }
            }
        }
    });

    log::info!("업로드 작업 생성됨: {}", job_id);
    Ok(job_id.to_string())
}

/// 업로드 작업을 취소합니다.
#[tauri::command]
pub async fn cancel_upload(
    job_id: String,
    state: State<'_, Mutex<AppState>>,
) -> Result<bool, String> {
    log::info!("업로드 취소 요청: {}", job_id);

    let job_uuid =
        Uuid::parse_str(&job_id).map_err(|_| "잘못된 작업 ID 형식입니다.".to_string())?;

    let app_state = state.lock().map_err(|e| format!("상태 잠금 실패: {}", e))?;
    let cancelled = app_state.upload_manager.cancel_job(&job_uuid);

    if cancelled {
        log::info!("업로드 취소 완료: {}", job_id);
    } else {
        log::warn!("취소할 수 없는 작업: {}", job_id);
    }

    Ok(cancelled)
}

/// 업로드 작업 상태를 조회합니다.
#[tauri::command]
pub async fn get_upload_status(
    job_id: String,
    state: State<'_, Mutex<AppState>>,
) -> Result<UploadJob, String> {
    let job_uuid =
        Uuid::parse_str(&job_id).map_err(|_| "잘못된 작업 ID 형식입니다.".to_string())?;

    let app_state = state.lock().map_err(|e| format!("상태 잠금 실패: {}", e))?;

    // 진행률 업데이트
    app_state.upload_manager.update_job_progress(&job_uuid);

    app_state
        .upload_manager
        .get_job(&job_uuid)
        .ok_or_else(|| "작업을 찾을 수 없습니다.".to_string())
}

/// 모든 업로드 작업 목록을 조회합니다.
#[tauri::command]
pub async fn get_all_uploads(state: State<'_, Mutex<AppState>>) -> Result<Vec<UploadJob>, String> {
    let app_state = state.lock().map_err(|e| format!("상태 잠금 실패: {}", e))?;
    Ok(app_state.upload_manager.get_all_jobs())
}
