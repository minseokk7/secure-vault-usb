// 암호화 관련 Tauri Commands
// 프론트엔드에서 암호화 서비스를 호출할 수 있는 인터페이스를 제공합니다.

use crate::services::CryptoService;
use crate::AppState;
use tauri::State;
use uuid::Uuid;
use std::sync::Mutex;
use base64::{Engine as _, engine::general_purpose};

/// PIN으로부터 마스터 키를 유도합니다.
/// 
/// C# EncryptionService.DeriveKeyFromPin()과 동일한 기능을 제공합니다.
/// PBKDF2-HMAC-SHA256 알고리즘을 사용하여 100,000회 반복합니다.
/// 
/// # 매개변수
/// * `pin` - 사용자 PIN (4-8자리 숫자)
/// * `salt_hex` - 32바이트 솔트 (16진수 문자열)
/// * `state` - 애플리케이션 상태
/// 
/// # 반환값
/// * `Result<(), String>` - 키 유도 결과
#[tauri::command]
pub async fn derive_master_key_from_pin(
    pin: String,
    salt_hex: String,
    state: State<'_, Mutex<AppState>>
) -> Result<(), String> {
    log::info!("PIN으로부터 마스터 키 유도 요청");
    
    // 16진수 솔트를 바이트로 변환
    let salt = hex::decode(&salt_hex)
        .map_err(|_| "솔트 형식이 올바르지 않습니다.")?;
    
    // 앱 상태에서 암호화 서비스 가져오기
    let mut app_state = state.lock().map_err(|_| "상태 잠금 실패")?;
    
    // 마스터 키 유도
    app_state.crypto_service.derive_master_key(&pin, &salt)
        .map_err(|e| e.to_string())?;
    
    log::info!("마스터 키 유도 완료");
    Ok(())
}

/// 32바이트 랜덤 솔트를 생성합니다.
/// 
/// C# EncryptionService.GenerateSalt()와 동일한 기능을 제공합니다.
/// 
/// # 반환값
/// * `Result<String, String>` - 32바이트 솔트 (16진수 문자열)
#[tauri::command]
pub async fn generate_salt() -> Result<String, String> {
    log::debug!("솔트 생성 요청");
    
    let salt = CryptoService::generate_salt();
    let salt_hex = hex::encode(salt);
    
    log::debug!("솔트 생성 완료: {} bytes", salt.len());
    Ok(salt_hex)
}

/// 256비트 복구 키를 생성합니다.
/// 
/// C# EncryptionService.GenerateRecoveryKey()와 동일한 기능을 제공합니다.
/// 
/// # 반환값
/// * `Result<String, String>` - 256비트 복구 키 (16진수 문자열)
#[tauri::command]
pub async fn generate_crypto_recovery_key() -> Result<String, String> {
    log::info!("복구 키 생성 요청");
    
    let recovery_key = CryptoService::generate_recovery_key();
    let recovery_key_hex = hex::encode(recovery_key);
    
    log::info!("복구 키 생성 완료: {} bytes", recovery_key.len());
    Ok(recovery_key_hex)
}

/// 데이터를 C# 호환 형식으로 암호화합니다.
/// 
/// C# EncryptionService.EncryptData()와 동일한 형식을 사용합니다:
/// - AES-256-GCM 알고리즘
/// - 결과 형식: IV + 암호문 + 인증태그
/// 
/// # 매개변수
/// * `data_base64` - 암호화할 데이터 (Base64 인코딩)
/// * `key_hex` - 256비트 암호화 키 (16진수 문자열)
/// * `state` - 애플리케이션 상태
/// 
/// # 반환값
/// * `Result<String, String>` - 암호화된 데이터 (Base64 인코딩)
#[tauri::command]
pub async fn encrypt_data_csharp_compatible(
    data_base64: String,
    key_hex: String,
    state: State<'_, Mutex<AppState>>
) -> Result<String, String> {
    log::debug!("C# 호환 데이터 암호화 요청");
    
    // Base64 데이터 디코딩
    let data = general_purpose::STANDARD.decode(&data_base64)
        .map_err(|_| "데이터 형식이 올바르지 않습니다.")?;
    
    // 16진수 키를 바이트로 변환
    let key = hex::decode(&key_hex)
        .map_err(|_| "키 형식이 올바르지 않습니다.")?;
    
    // 앱 상태에서 암호화 서비스 가져오기
    let app_state = state.lock().map_err(|_| "상태 잠금 실패")?;
    
    // 데이터 암호화
    let encrypted_data = app_state.crypto_service.encrypt_data_csharp_compatible(&data, &key)
        .map_err(|e| e.to_string())?;
    
    // Base64로 인코딩하여 반환
    let encrypted_base64 = general_purpose::STANDARD.encode(encrypted_data);
    
    log::debug!("C# 호환 데이터 암호화 완료: {} bytes -> {} bytes", 
               data.len(), encrypted_base64.len());
    
    Ok(encrypted_base64)
}

/// C# 호환 형식의 암호화된 데이터를 복호화합니다.
/// 
/// C# EncryptionService.DecryptData()와 동일한 형식을 처리합니다:
/// - 입력 형식: IV + 암호문 + 인증태그
/// 
/// # 매개변수
/// * `encrypted_data_base64` - 암호화된 데이터 (Base64 인코딩)
/// * `key_hex` - 256비트 복호화 키 (16진수 문자열)
/// * `state` - 애플리케이션 상태
/// 
/// # 반환값
/// * `Result<String, String>` - 복호화된 데이터 (Base64 인코딩)
#[tauri::command]
pub async fn decrypt_data_csharp_compatible(
    encrypted_data_base64: String,
    key_hex: String,
    state: State<'_, Mutex<AppState>>
) -> Result<String, String> {
    log::debug!("C# 호환 데이터 복호화 요청");
    
    // Base64 암호화 데이터 디코딩
    let encrypted_data = general_purpose::STANDARD.decode(&encrypted_data_base64)
        .map_err(|_| "암호화된 데이터 형식이 올바르지 않습니다.")?;
    
    // 16진수 키를 바이트로 변환
    let key = hex::decode(&key_hex)
        .map_err(|_| "키 형식이 올바르지 않습니다.")?;
    
    // 앱 상태에서 암호화 서비스 가져오기
    let app_state = state.lock().map_err(|_| "상태 잠금 실패")?;
    
    // 데이터 복호화
    let decrypted_data = app_state.crypto_service.decrypt_data_csharp_compatible(&encrypted_data, &key)
        .map_err(|e| e.to_string())?;
    
    // Base64로 인코딩하여 반환
    let decrypted_base64 = general_purpose::STANDARD.encode(decrypted_data);
    
    log::debug!("C# 호환 데이터 복호화 완료: {} bytes -> {} bytes", 
               encrypted_data.len(), decrypted_base64.len());
    
    Ok(decrypted_base64)
}

/// 파일을 암호화합니다.
/// 
/// 파일별 고유 키를 사용하여 보안을 강화합니다.
/// 
/// # 매개변수
/// * `data_base64` - 파일 데이터 (Base64 인코딩)
/// * `file_id` - 파일 고유 ID
/// * `state` - 애플리케이션 상태
/// 
/// # 반환값
/// * `Result<String, String>` - 암호화된 파일 데이터 (Base64 인코딩)
#[tauri::command]
pub async fn encrypt_file(
    data_base64: String,
    file_id: String,
    state: State<'_, Mutex<AppState>>
) -> Result<String, String> {
    log::info!("파일 암호화 요청: {}", file_id);
    
    // Base64 데이터 디코딩
    let data = general_purpose::STANDARD.decode(&data_base64)
        .map_err(|_| "파일 데이터 형식이 올바르지 않습니다.")?;
    
    // 파일 ID를 UUID로 변환
    let uuid = Uuid::parse_str(&file_id)
        .map_err(|_| "파일 ID 형식이 올바르지 않습니다.")?;
    
    // 앱 상태에서 암호화 서비스 가져오기
    let app_state = state.lock().map_err(|_| "상태 잠금 실패")?;
    
    // 파일 암호화
    let encrypted_data = app_state.crypto_service.encrypt_file(&data, &uuid)
        .map_err(|e| e.to_string())?;
    
    // 암호화된 데이터를 JSON으로 직렬화
    let encrypted_json = serde_json::to_string(&encrypted_data)
        .map_err(|_| "암호화 결과 직렬화 실패")?;
    
    // Base64로 인코딩하여 반환
    let encrypted_base64 = general_purpose::STANDARD.encode(encrypted_json);
    
    log::info!("파일 암호화 완료: {} -> {} bytes ({}ms)", 
              data.len(), encrypted_data.ciphertext.len(), encrypted_data.encryption_time_ms);
    
    Ok(encrypted_base64)
}

/// 파일을 복호화합니다.
/// 
/// # 매개변수
/// * `encrypted_data_base64` - 암호화된 파일 데이터 (Base64 인코딩)
/// * `file_id` - 파일 고유 ID
/// * `state` - 애플리케이션 상태
/// 
/// # 반환값
/// * `Result<String, String>` - 복호화된 파일 데이터 (Base64 인코딩)
#[tauri::command]
pub async fn decrypt_file(
    encrypted_data_base64: String,
    file_id: String,
    state: State<'_, Mutex<AppState>>
) -> Result<String, String> {
    log::info!("파일 복호화 요청: {}", file_id);
    
    // Base64 데이터 디코딩
    let encrypted_json = general_purpose::STANDARD.decode(&encrypted_data_base64)
        .map_err(|_| "암호화된 데이터 형식이 올바르지 않습니다.")?;
    
    // JSON을 EncryptedData로 역직렬화
    let encrypted_data: crate::models::EncryptedData = serde_json::from_slice(&encrypted_json)
        .map_err(|_| "암호화된 데이터 파싱 실패")?;
    
    // 파일 ID를 UUID로 변환
    let uuid = Uuid::parse_str(&file_id)
        .map_err(|_| "파일 ID 형식이 올바르지 않습니다.")?;
    
    // 앱 상태에서 암호화 서비스 가져오기
    let app_state = state.lock().map_err(|_| "상태 잠금 실패")?;
    
    // 파일 복호화
    let decrypted_data = app_state.crypto_service.decrypt_file(&encrypted_data, &uuid)
        .map_err(|e| e.to_string())?;
    
    // Base64로 인코딩하여 반환
    let decrypted_base64 = general_purpose::STANDARD.encode(&decrypted_data);
    
    log::info!("파일 복호화 완료: {} -> {} bytes", 
              encrypted_data.ciphertext.len(), decrypted_data.len());
    
    Ok(decrypted_base64)
}

/// 마스터 키 설정 여부를 확인합니다.
/// 
/// # 매개변수
/// * `state` - 애플리케이션 상태
/// 
/// # 반환값
/// * `Result<bool, String>` - 마스터 키 설정 여부
#[tauri::command]
pub async fn has_master_key(
    state: State<'_, Mutex<AppState>>
) -> Result<bool, String> {
    let app_state = state.lock().map_err(|_| "상태 잠금 실패")?;
    Ok(app_state.crypto_service.has_master_key())
}

/// 메모리에서 민감한 데이터를 안전하게 제거합니다.
/// 
/// 로그아웃 시나 애플리케이션 종료 시 호출합니다.
/// 
/// # 매개변수
/// * `state` - 애플리케이션 상태
/// 
/// # 반환값
/// * `Result<(), String>` - 정리 결과
#[tauri::command]
pub async fn clear_sensitive_data(
    state: State<'_, Mutex<AppState>>
) -> Result<(), String> {
    log::info!("민감한 데이터 정리 요청");
    
    let mut app_state = state.lock().map_err(|_| "상태 잠금 실패")?;
    app_state.crypto_service.clear_sensitive_data();
    
    log::info!("민감한 데이터 정리 완료");
    Ok(())
}

/// 현재 사용 중인 암호화 알고리즘을 반환합니다.
/// 
/// # 매개변수
/// * `state` - 애플리케이션 상태
/// 
/// # 반환값
/// * `Result<String, String>` - 암호화 알고리즘 이름
#[tauri::command]
pub async fn get_encryption_algorithm(
    state: State<'_, Mutex<AppState>>
) -> Result<String, String> {
    let app_state = state.lock().map_err(|_| "상태 잠금 실패")?;
    let algorithm = app_state.crypto_service.get_default_algorithm();
    
    let algorithm_name = match algorithm {
        crate::models::EncryptionAlgorithm::AES256GCM => "AES-256-GCM",
        crate::models::EncryptionAlgorithm::ChaCha20Poly1305 => "ChaCha20-Poly1305",
    };
    
    Ok(algorithm_name.to_string())
}