use crate::models::recovery::{RecoveryKeyInfo, RecoveryVerificationResult};
use crate::AppState;
use std::sync::Mutex;
use tauri::State;

/// 복구 키 생성 커맨드
/// C# RecoveryKeyDialog에서 호출되는 기능을 Tauri로 포팅
///
/// # 반환값
/// * `Ok(String)` - Base64로 인코딩된 256비트 복구 키
/// * `Err(String)` - 오류 메시지 (한국어)
#[tauri::command]
pub async fn generate_recovery_key(state: State<'_, Mutex<AppState>>) -> Result<String, String> {
    let app_state = state.lock().map_err(|_| "상태 잠금 실패")?;
    let recovery_service = &app_state.recovery_service;

    recovery_service
        .generate_recovery_key()
        .map_err(|e| format!("복구 키 생성 실패: {}", e))
}

/// 복구 키 해시 생성 커맨드
/// 복구 키를 저장하기 위한 SHA-256 해시 생성
///
/// # 매개변수
/// * `recovery_key` - Base64 형식의 복구 키
///
/// # 반환값
/// * `Ok(String)` - Base64로 인코딩된 SHA-256 해시값
/// * `Err(String)` - 오류 메시지 (한국어)
#[tauri::command]
pub async fn hash_recovery_key(
    recovery_key: String,
    state: State<'_, Mutex<AppState>>,
) -> Result<String, String> {
    let app_state = state.lock().map_err(|_| "상태 잠금 실패")?;
    let recovery_service = &app_state.recovery_service;

    recovery_service
        .hash_recovery_key(&recovery_key)
        .map_err(|e| format!("복구 키 해시 생성 실패: {}", e))
}

/// 복구 키 검증 커맨드
/// 입력된 복구 키가 저장된 해시와 일치하는지 확인
///
/// # 매개변수
/// * `input_recovery_key` - 사용자가 입력한 복구 키
/// * `stored_hash` - 저장된 복구 키 해시값
///
/// # 반환값
/// * `Ok(bool)` - 검증 결과 (true: 일치, false: 불일치)
/// * `Err(String)` - 오류 메시지 (한국어)
#[tauri::command]
pub async fn verify_recovery_key(
    input_recovery_key: String,
    stored_hash: String,
    state: State<'_, Mutex<AppState>>,
) -> Result<bool, String> {
    let app_state = state.lock().map_err(|_| "상태 잠금 실패")?;
    let recovery_service = &app_state.recovery_service;

    recovery_service
        .verify_recovery_key(&input_recovery_key, &stored_hash)
        .map_err(|e| format!("복구 키 검증 실패: {}", e))
}

/// 복구 키로 마스터 키 유도 커맨드
/// 복구 키를 사용하여 볼트 암호화에 사용할 마스터 키 생성
///
/// # 매개변수
/// * `recovery_key` - Base64 형식의 복구 키
/// * `salt` - Base64로 인코딩된 32바이트 솔트
/// * `iterations` - PBKDF2 반복 횟수 (선택사항, 기본값: 100,000)
///
/// # 반환값
/// * `Ok(Vec<u8>)` - 32바이트 마스터 키
/// * `Err(String)` - 오류 메시지 (한국어)
#[tauri::command]
pub async fn derive_key_from_recovery_key(
    recovery_key: String,
    salt: String,
    iterations: Option<u32>,
    state: State<'_, Mutex<AppState>>,
) -> Result<Vec<u8>, String> {
    let app_state = state.lock().map_err(|_| "상태 잠금 실패")?;
    let recovery_service = &app_state.recovery_service;

    // Base64 솔트 디코딩
    let salt_bytes = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &salt)
        .map_err(|_| "올바르지 않은 솔트 형식입니다.".to_string())?;

    if salt_bytes.len() != 32 {
        return Err("솔트는 32바이트여야 합니다.".to_string());
    }

    let salt_array: [u8; 32] = salt_bytes
        .try_into()
        .map_err(|_| "솔트 변환 오류".to_string())?;

    recovery_service
        .derive_key_from_recovery_key(&recovery_key, &salt_array, iterations)
        .map(|key| key.to_vec())
        .map_err(|e| format!("마스터 키 유도 실패: {}", e))
}

/// 복구 키 검증 및 마스터 키 유도 통합 커맨드
/// 복구 키 검증과 마스터 키 유도를 한 번에 수행
///
/// # 매개변수
/// * `input_recovery_key` - 사용자가 입력한 복구 키
/// * `stored_hash` - 저장된 복구 키 해시값
/// * `salt` - Base64로 인코딩된 32바이트 솔트
/// * `iterations` - PBKDF2 반복 횟수 (선택사항)
///
/// # 반환값
/// * `Ok(RecoveryVerificationResult)` - 검증 결과 및 마스터 키
/// * `Err(String)` - 오류 메시지 (한국어)
#[tauri::command]
pub async fn verify_and_derive_key(
    input_recovery_key: String,
    stored_hash: String,
    salt: String,
    iterations: Option<u32>,
    state: State<'_, Mutex<AppState>>,
) -> Result<RecoveryVerificationResult, String> {
    let app_state = state.lock().map_err(|_| "상태 잠금 실패")?;
    let recovery_service = &app_state.recovery_service;

    // Base64 솔트 디코딩
    let salt_bytes = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &salt)
        .map_err(|_| "올바르지 않은 솔트 형식입니다.".to_string())?;

    if salt_bytes.len() != 32 {
        return Err("솔트는 32바이트여야 합니다.".to_string());
    }

    let salt_array: [u8; 32] = salt_bytes
        .try_into()
        .map_err(|_| "솔트 변환 오류".to_string())?;

    let result = recovery_service.verify_and_derive_key(
        &input_recovery_key,
        &stored_hash,
        &salt_array,
        iterations,
    );

    Ok(result)
}

/// 복구 키 형식 검증 커맨드
/// 사용자가 입력한 복구 키의 형식이 올바른지 확인
///
/// # 매개변수
/// * `recovery_key` - 검증할 복구 키
///
/// # 반환값
/// * `Ok(())` - 형식이 올바름
/// * `Err(String)` - 형식 오류 메시지 (한국어)
#[tauri::command]
pub async fn validate_recovery_key_format(
    recovery_key: String,
    state: State<'_, Mutex<AppState>>,
) -> Result<(), String> {
    let app_state = state.lock().map_err(|_| "상태 잠금 실패")?;
    let recovery_service = &app_state.recovery_service;

    recovery_service
        .validate_recovery_key_format(&recovery_key)
        .map_err(|e| format!("{}", e))
}

/// 현재 복구 키 정보 조회 커맨드
/// 현재 생성된 복구 키의 정보를 반환 (키 값 제외)
///
/// # 반환값
/// * `Ok(Option<RecoveryKeyInfo>)` - 복구 키 정보 (키 값은 보안상 제외)
/// * `Err(String)` - 오류 메시지
#[tauri::command]
pub async fn get_recovery_key_info(
    state: State<'_, Mutex<AppState>>,
) -> Result<Option<RecoveryKeyInfo>, String> {
    let app_state = state.lock().map_err(|_| "상태 잠금 실패")?;
    let recovery_service = &app_state.recovery_service;

    let mut info = recovery_service.get_current_recovery_key_info();

    // 보안을 위해 실제 키 값은 제거
    if let Some(ref mut recovery_info) = info {
        recovery_info.key = "***HIDDEN***".to_string();
    }

    Ok(info)
}

/// 복구 키 사용 표시 커맨드
/// 복구 키를 사용했음을 표시 (한 번만 사용 가능)
///
/// # 반환값
/// * `Ok(())` - 성공
/// * `Err(String)` - 오류 메시지 (한국어)
#[tauri::command]
pub async fn mark_recovery_key_used(state: State<'_, Mutex<AppState>>) -> Result<(), String> {
    let app_state = state.lock().map_err(|_| "상태 잠금 실패")?;
    let recovery_service = &app_state.recovery_service;

    recovery_service
        .mark_recovery_key_used()
        .map_err(|e| format!("복구 키 사용 표시 실패: {}", e))
}

/// 복구 키 정보 초기화 커맨드
/// 보안을 위해 메모리에서 복구 키 정보 삭제
///
/// # 반환값
/// * `Ok(())` - 성공
#[tauri::command]
pub async fn clear_recovery_key(state: State<'_, Mutex<AppState>>) -> Result<(), String> {
    let app_state = state.lock().map_err(|_| "상태 잠금 실패")?;
    let recovery_service = &app_state.recovery_service;
    recovery_service.clear_recovery_key();
    Ok(())
}

/// 복구 키 기반 인증 커맨드
/// 복구 키를 사용하여 볼트에 인증하고 새로운 PIN 설정 가능
///
/// # 매개변수
/// * `recovery_key` - 사용자가 입력한 복구 키
/// * `stored_hash` - 저장된 복구 키 해시
/// * `salt` - 볼트 솔트
///
/// # 반환값
/// * `Ok(Vec<u8>)` - 인증 성공 시 마스터 키
/// * `Err(String)` - 인증 실패 메시지 (한국어)
#[tauri::command]
pub async fn authenticate_with_recovery_key(
    recovery_key: String,
    stored_hash: String,
    salt: String,
    state: State<'_, Mutex<AppState>>,
) -> Result<Vec<u8>, String> {
    let app_state = state.lock().map_err(|_| "상태 잠금 실패")?;
    let recovery_service = &app_state.recovery_service;

    // 복구 키 형식 검증
    recovery_service
        .validate_recovery_key_format(&recovery_key)
        .map_err(|e| format!("복구 키 형식 오류: {}", e))?;

    // Base64 솔트 디코딩
    let salt_bytes = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &salt)
        .map_err(|_| "올바르지 않은 솔트 형식입니다.".to_string())?;

    if salt_bytes.len() != 32 {
        return Err("솔트는 32바이트여야 합니다.".to_string());
    }

    let salt_array: [u8; 32] = salt_bytes
        .try_into()
        .map_err(|_| "솔트 변환 오류".to_string())?;

    // 복구 키 검증 및 마스터 키 유도
    let result = recovery_service.verify_and_derive_key(
        &recovery_key,
        &stored_hash,
        &salt_array,
        Some(100_000), // C# 버전과 동일한 반복 횟수
    );

    if result.is_valid {
        if let Some(master_key) = result.master_key {
            // 복구 키 사용 표시
            recovery_service
                .mark_recovery_key_used()
                .map_err(|e| format!("복구 키 사용 표시 실패: {}", e))?;

            Ok(master_key)
        } else {
            Err("마스터 키 유도에 실패했습니다.".to_string())
        }
    } else {
        Err(result
            .error_message
            .unwrap_or_else(|| "복구 키 인증에 실패했습니다.".to_string()))
    }
}
