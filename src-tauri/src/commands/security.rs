// 보안 관련 Tauri 커맨드
// 네트워크 접근 차단 및 보안 상태 확인 기능을 제공합니다.

use crate::AppState;
use tauri::State;
use std::sync::Mutex;

/// 보안 상태를 조회합니다.
/// 
/// 네트워크 접근 차단 상태, 인증 상태 등 전반적인 보안 상태를 확인합니다.
/// 
/// # 매개변수
/// * `state` - 애플리케이션 상태
/// 
/// # 반환값
/// * `Result<String, String>` - JSON 형태의 보안 상태 정보
#[tauri::command]
pub async fn get_security_status(
    state: State<'_, Mutex<AppState>>
) -> Result<String, String> {
    // 네트워크 가드 상태 확인
    let network_report = {
        let app_state = state.lock().map_err(|_| "상태 잠금 실패")?;
        app_state.network_guard.generate_security_report()
    };
    
    // 인증 서비스 상태 확인 (각각 개별적으로 가져와서 borrow 문제 해결)
    let auth_status_str = {
        let app_state = state.lock().map_err(|_| "상태 잠금 실패")?;
        format!("{:?}", app_state.auth_service.get_auth_state())
    };
    
    let has_pin = {
        let app_state = state.lock().map_err(|_| "상태 잠금 실패")?;
        app_state.auth_service.has_pin()
    };
    
    let has_recovery_key = {
        let app_state = state.lock().map_err(|_| "상태 잠금 실패")?;
        app_state.auth_service.has_recovery_key()
    };
    
    let session_valid = {
        let mut app_state = state.lock().map_err(|_| "상태 잠금 실패")?;
        app_state.auth_service.is_session_valid()
    };
    
    // 보안 상태 정보 구성
    let security_status = serde_json::json!({
        "network_security": {
            "access_blocked": network_report.blocking_enabled,
            "blocked_attempts": network_report.network_attempts_detected,
            "last_block_time": network_report.last_check,
            "security_level": network_report.security_level
        },
        "authentication": {
            "state": auth_status_str,
            "has_pin": has_pin,
            "has_recovery_key": has_recovery_key,
            "session_valid": session_valid
        },
        "overall_status": "secure"
    });
    
    match serde_json::to_string_pretty(&security_status) {
        Ok(json_string) => {
            log::info!("보안 상태 조회 완료");
            Ok(json_string)
        }
        Err(e) => {
            log::error!("보안 상태 직렬화 오류: {}", e);
            Err("보안 상태 조회 중 오류가 발생했습니다.".to_string())
        }
    }
}

/// 네트워크 접근 시도를 확인합니다.
/// 
/// 애플리케이션의 네트워크 접근 시도 여부를 실시간으로 확인합니다.
/// 
/// # 매개변수
/// * `state` - 애플리케이션 상태
/// 
/// # 반환값
/// * `Result<bool, String>` - 네트워크 접근 차단 여부
#[tauri::command]
pub async fn check_network_access(
    state: State<'_, Mutex<AppState>>
) -> Result<bool, String> {
    let app_state = state.lock().map_err(|_| "상태 잠금 실패")?;
    let network_report = app_state.network_guard.generate_security_report();
    
    log::info!("네트워크 접근 상태 확인: 차단됨={}, 시도횟수={}", 
        network_report.blocking_enabled, 
        network_report.network_attempts_detected
    );
    
    Ok(network_report.blocking_enabled)
}