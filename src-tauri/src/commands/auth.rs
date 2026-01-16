// 인증 관련 Tauri 커맨드
// 프론트엔드에서 호출할 수 있는 인증 관련 함수들을 정의합니다.

use crate::{AppState, models::PinComplexity};
use tauri::State;
use std::sync::Mutex;

/// PIN으로 인증합니다.
/// 
/// # 매개변수
/// * `pin` - 사용자가 입력한 PIN
/// * `state` - 애플리케이션 상태
/// 
/// # 반환값
/// * `Result<bool, String>` - 인증 성공 여부
#[tauri::command]
pub async fn authenticate_pin(
    pin: String,
    state: State<'_, Mutex<AppState>>
) -> Result<bool, String> {
    let mut app_state = state.lock().map_err(|_| "상태 잠금 실패")?;
    
    match app_state.auth_service.verify_pin(&pin) {
        Ok(result) => {
            use crate::models::PinValidationResult;
            match result {
                PinValidationResult::Valid => {
                    // 암호화 서비스 초기화
                    if let Some(pin_info) = app_state.auth_service.get_pin_info() {
                        // pin_info를 복사하여 borrow 문제 해결
                        let salt = pin_info.salt.clone();
                        if let Err(e) = app_state.crypto_service.derive_master_key(&pin, &salt) {
                            log::error!("마스터 키 유도 실패: {}", e);
                            return Err("인증 처리 중 오류가 발생했습니다.".to_string());
                        }
                        
                        // 파일 서비스 초기화
                        if let Some(master_key) = app_state.crypto_service.get_master_key() {
                            let vault_path = std::env::current_dir()
                                .unwrap_or_else(|_| std::path::PathBuf::from("."))
                                .to_string_lossy()
                                .to_string();
                            
                            let mut file_service = app_state.file_service.lock().map_err(|_| "파일 서비스 잠금 실패")?;
                            file_service.set_vault_info(&vault_path, master_key);
                            log::info!("파일 서비스 초기화 완료");
                        }
                    }
                    Ok(true)
                }
                PinValidationResult::Invalid => Ok(false),
                PinValidationResult::InvalidFormat => {
                    Err("PIN 형식이 올바르지 않습니다.".to_string())
                }
                PinValidationResult::AccountLocked(seconds) => {
                    Err(format!("보안을 위해 {}초 후 다시 시도해주세요.", seconds))
                }
                PinValidationResult::Expired => {
                    Err("PIN이 만료되었습니다. 새로운 PIN을 설정해주세요.".to_string())
                }
            }
        }
        Err(e) => {
            log::error!("PIN 인증 오류: {}", e);
            Err("인증 처리 중 오류가 발생했습니다.".to_string())
        }
    }
}

/// PIN을 설정합니다.
/// 
/// # 매개변수
/// * `pin` - 설정할 PIN
/// * `complexity` - PIN 복잡도 레벨 ("basic", "medium", "high")
/// * `state` - 애플리케이션 상태
/// 
/// # 반환값
/// * `Result<(), String>` - 설정 결과
#[tauri::command]
pub async fn set_pin_code(
    pin: String,
    complexity: String,
    state: State<'_, Mutex<AppState>>
) -> Result<(), String> {
    let complexity_level = match complexity.as_str() {
        "basic" => PinComplexity::Basic,
        "medium" => PinComplexity::Medium,
        "high" => PinComplexity::High,
        _ => return Err("올바르지 않은 복잡도 레벨입니다.".to_string()),
    };
    
    let mut app_state = state.lock().map_err(|_| "상태 잠금 실패")?;
    
    match app_state.auth_service.set_pin(&pin, complexity_level) {
        Ok(()) => {
            log::info!("PIN이 성공적으로 설정되었습니다.");
            Ok(())
        }
        Err(e) => {
            log::error!("PIN 설정 오류: {}", e);
            Err(e.user_friendly_message())
        }
    }
}

/// 복구 키로 인증합니다.
/// 
/// # 매개변수
/// * `recovery_key` - 복구 키 (Base64 문자열)
/// * `state` - 애플리케이션 상태
/// 
/// # 반환값
/// * `Result<bool, String>` - 인증 성공 여부
#[tauri::command]
pub async fn authenticate_recovery_key(
    recovery_key: String,
    state: State<'_, Mutex<AppState>>
) -> Result<bool, String> {
    let mut app_state = state.lock().map_err(|_| "상태 잠금 실패")?;
    
    match app_state.auth_service.verify_recovery_key(&recovery_key) {
        Ok(result) => {
            use crate::models::RecoveryKeyValidationResult;
            match result {
                RecoveryKeyValidationResult::Valid => {
                    // 암호화 서비스 초기화 (복구 키 기반)
                    if let Some(_recovery_info) = app_state.auth_service.get_recovery_key_info() {
                        // TODO: 복구 키로부터 마스터 키 유도 구현 필요
                        log::info!("복구 키 인증 성공, 암호화 서비스 초기화 필요");
                    }
                    Ok(true)
                }
                RecoveryKeyValidationResult::Invalid => Ok(false),
                RecoveryKeyValidationResult::InvalidFormat => {
                    Err("복구 키 형식이 올바르지 않습니다. Base64 형식의 32바이트 키를 입력해주세요.".to_string())
                }
                RecoveryKeyValidationResult::Deactivated => {
                    Err("복구 키가 비활성화되었습니다.".to_string())
                }
            }
        }
        Err(e) => {
            log::error!("복구 키 인증 오류: {}", e);
            Err("복구 키 인증 중 오류가 발생했습니다.".to_string())
        }
    }
}

/// 복구 키를 생성합니다.
/// 
/// # 매개변수
/// 로그아웃합니다.
/// 
/// # 매개변수
/// * `state` - 애플리케이션 상태
/// 
/// # 반환값
/// * `Result<(), String>` - 로그아웃 결과
#[tauri::command]
pub async fn logout(
    state: State<'_, Mutex<AppState>>
) -> Result<(), String> {
    let mut app_state = state.lock().map_err(|_| "상태 잠금 실패")?;
    
    // 인증 서비스 로그아웃
    if let Err(e) = app_state.auth_service.logout() {
        log::error!("로그아웃 오류: {}", e);
        return Err("로그아웃 중 오류가 발생했습니다.".to_string());
    }
    
    // 암호화 서비스 민감한 데이터 정리
    app_state.crypto_service.clear_sensitive_data();
    
    log::info!("로그아웃이 완료되었습니다.");
    Ok(())
}

/// 현재 인증 상태를 확인합니다.
/// 
/// # 매개변수
/// * `state` - 애플리케이션 상태
/// 
/// # 반환값
/// * `Result<bool, String>` - 인증 상태
#[tauri::command]
pub async fn check_auth_status(
    state: State<'_, Mutex<AppState>>
) -> Result<bool, String> {
    let mut app_state = state.lock().map_err(|_| "상태 잠금 실패")?;
    Ok(app_state.auth_service.is_session_valid())
}

/// PIN 설정 여부를 확인합니다.
/// 
/// # 매개변수
/// * `state` - 애플리케이션 상태
/// 
/// # 반환값
/// * `Result<bool, String>` - PIN 설정 여부
#[tauri::command]
pub async fn has_pin_set(
    state: State<'_, Mutex<AppState>>
) -> Result<bool, String> {
    let app_state = state.lock().map_err(|_| "상태 잠금 실패")?;
    Ok(app_state.auth_service.has_pin())
}

/// 복구 키 설정 여부를 확인합니다.
/// 
/// # 매개변수
/// * `state` - 애플리케이션 상태
/// 
/// # 반환값
/// * `Result<bool, String>` - 복구 키 설정 여부
#[tauri::command]
pub async fn has_recovery_key_set(
    state: State<'_, Mutex<AppState>>
) -> Result<bool, String> {
    let app_state = state.lock().map_err(|_| "상태 잠금 실패")?;
    Ok(app_state.auth_service.has_recovery_key())
}

/// 세션 남은 시간을 조회합니다.
/// 
/// # 매개변수
/// * `state` - 애플리케이션 상태
/// 
/// # 반환값
/// * `Result<Option<u64>, String>` - 남은 시간 (초)
#[tauri::command]
pub async fn get_session_remaining_time(
    state: State<'_, Mutex<AppState>>
) -> Result<Option<u64>, String> {
    let app_state = state.lock().map_err(|_| "상태 잠금 실패")?;
    Ok(app_state.auth_service.get_session_remaining_time())
}

/// PIN을 변경합니다.
/// 
/// # 매개변수
/// * `old_pin` - 기존 PIN
/// * `new_pin` - 새 PIN
/// * `complexity` - 새 PIN 복잡도
/// * `state` - 애플리케이션 상태
/// 
/// # 반환값
/// * `Result<(), String>` - 변경 결과
#[tauri::command]
pub async fn change_pin(
    old_pin: String,
    new_pin: String,
    complexity: String,
    state: State<'_, Mutex<AppState>>
) -> Result<(), String> {
    let complexity_level = match complexity.as_str() {
        "basic" => PinComplexity::Basic,
        "medium" => PinComplexity::Medium,
        "high" => PinComplexity::High,
        _ => return Err("올바르지 않은 복잡도 레벨입니다.".to_string()),
    };
    
    let mut app_state = state.lock().map_err(|_| "상태 잠금 실패")?;
    
    match app_state.auth_service.change_pin(&old_pin, &new_pin, complexity_level) {
        Ok(()) => {
            log::info!("PIN이 성공적으로 변경되었습니다.");
            Ok(())
        }
        Err(e) => {
            log::error!("PIN 변경 오류: {}", e);
            Err(e.user_friendly_message())
        }
    }
}