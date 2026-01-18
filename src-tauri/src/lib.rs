// SecureVault Tauri 애플리케이션 메인 라이브러리
// USB 포터블 보안 파일 매니저의 핵심 로직을 담당합니다.

use std::sync::Mutex;
use tauri::Manager;

// 모듈 선언
pub mod commands;
pub mod models;
pub mod services;
pub mod utils;

// 모델 및 서비스 재내보내기
pub use models::{
    auth_simple::*,
    encryption::*,
    error::VaultError,
    file::*,
    folder::{FolderEntry, FolderSortBy},
    vault::*,
};
pub use services::{
    auth::*, crypto::*, database::*, file::*, folder::*, network_guard::*, recovery::*,
};

// 타입 별칭 정의
/// SecureVault 전용 Result 타입
/// 모든 볼트 작업의 표준 반환 타입으로 사용됩니다.
pub type SecureVaultResult<T> = Result<T, VaultError>;

/// 애플리케이션 전역 상태
/// 모든 서비스들을 관리하고 스레드 안전성을 보장합니다.
#[derive(Debug)]
pub struct AppState {
    /// 인증 서비스 - PIN 및 복구 키 인증 담당
    pub auth_service: AuthService,
    /// 암호화 서비스 - 파일 암호화/복호화 담당
    pub crypto_service: CryptoService,
    /// 복구 키 서비스 - 복구 키 생성/검증/키 유도 담당
    pub recovery_service: services::recovery::RecoveryService,
    /// 폴더 서비스 - 계층적 폴더 구조 관리 담당
    pub folder_service: services::folder::FolderService,
    /// 파일 서비스 - 파일 CRUD 작업 및 암호화 관리 담당
    pub file_service: Mutex<services::file::FileService>,
    /// 데이터베이스 서비스 - SQLite 메타데이터 관리 담당
    pub database_service: Mutex<services::database::DatabaseService>,
    /// 네트워크 가드 - 모든 네트워크 접근 차단
    pub network_guard: NetworkGuard,
    /// 압축 서비스 - 파일 압축/해제 담당
    pub compression_service: Mutex<services::compression::CompressionService>,
    /// 뷰어 서비스 - 파일 뷰어 기능 담당
    pub viewer_service: Mutex<services::viewer::ViewerService>,
    /// 업로드 관리자 - 백그라운드 파일 업로드 관리
    pub upload_manager: services::upload_manager::UploadManager,
}

impl AppState {
    /// 새로운 애플리케이션 상태를 생성합니다.
    ///
    /// # 반환값
    /// * `Self` - 초기화된 애플리케이션 상태
    pub fn new() -> Self {
        // 네트워크 가드를 가장 먼저 초기화하여 모든 네트워크 접근 차단
        let network_guard = services::network_guard::initialize_network_guard();

        // 데이터베이스 서비스 초기화
        let mut database_service = services::database::DatabaseService::new();

        // 현재 디렉토리를 볼트 경로로 사용
        let vault_path = std::env::current_dir()
            .unwrap_or_else(|_| std::path::PathBuf::from("."))
            .to_string_lossy()
            .to_string();

        if let Err(e) = database_service.initialize(&vault_path) {
            log::error!("데이터베이스 초기화 실패: {}", e);
        } else {
            log::info!("데이터베이스 초기화 완료");
        }

        Self {
            auth_service: AuthService::new(),
            crypto_service: CryptoService::new(),
            recovery_service: services::recovery::RecoveryService::new(),
            folder_service: services::folder::FolderService::new(),
            file_service: Mutex::new(services::file::FileService::new()),
            database_service: Mutex::new(database_service),
            network_guard,
            compression_service: Mutex::new(
                services::compression::CompressionService::new_with_defaults(),
            ),
            viewer_service: Mutex::new(services::viewer::ViewerService::new(
                services::file::FileService::new(),
            )),
            upload_manager: services::upload_manager::UploadManager::new(),
        }
    }
}

/// Tauri 애플리케이션 실행 함수
///
/// 애플리케이션을 초기화하고 모든 서비스와 커맨드를 등록합니다.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 로깅 초기화
    env_logger::init();

    // 볼트 디렉토리 초기화 (애플리케이션 시작 전)
    if let Err(e) = initialize_vault_directory_simple() {
        eprintln!("볼트 디렉토리 초기화 실패: {}", e);
    }

    tauri::Builder::<tauri::Wry>::default()
        // Tauri 플러그인 등록
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        // 중복 실행 방지 플러그인 등록
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            let _ = app
                .get_webview_window("main")
                .expect("no main window")
                .set_focus();
        }))
        // 커스텀 스트림 프로토콜 등록
        .register_uri_scheme_protocol("stream", |_app, request| {
            let response = (|| {
                // URI에서 경로 추출 (예: stream://C:/path/to/file)
                // "stream://" 스키마 부분을 제외
                let uri = request.uri().to_string();
                let path_str = uri
                    .strip_prefix("stream://")
                    .or_else(|| uri.strip_prefix("https://stream.localhost/"))
                    .or_else(|| uri.strip_prefix("http://stream.localhost/"))
                    .unwrap_or(&uri);

                // URL 디코딩
                let decoded_path = urlencoding::decode(path_str)
                    .map_err(|_| "URL 디코딩 실패")?
                    .to_string();

                // 윈도우 경로인 경우 앞의 슬래시 처리 등이 필요할 수 있음
                // 하지만 tauri 스키마는 보통 `stream://localhost/C:/...` 형식이 아닐 수 있음.
                // 보통 `stream://path` 로 옴.

                let path = std::path::PathBuf::from(&decoded_path);

                if !path.exists() {
                    return Err("파일이 존재하지 않습니다.");
                }

                let content = std::fs::read(&path).map_err(|_| "파일 읽기 실패")?;

                // MIME 타입 추론 (확장자 기반)
                let mime_type = if let Some(ext) = path.extension() {
                    match ext.to_string_lossy().to_lowercase().as_str() {
                        "mp3" => "audio/mpeg",
                        "wav" => "audio/wav",
                        "mp4" => "video/mp4",
                        "webm" => "video/webm",
                        "ogg" => "audio/ogg",
                        _ => "application/octet-stream",
                    }
                } else {
                    "application/octet-stream"
                };

                tauri::http::Response::builder()
                    .header("Content-Type", mime_type)
                    .header("Access-Control-Allow-Origin", "*")
                    .body(content)
                    .map_err(|_| "응답 생성 실패")
            })();

            match response {
                Ok(res) => res,
                Err(e) => tauri::http::Response::builder()
                    .status(404)
                    .body(e.as_bytes().to_vec())
                    .unwrap(),
            }
        })
        // 애플리케이션 상태 관리
        .manage(Mutex::new(AppState::new()))
        // 윈도우 설정 이벤트 핸들러
        .setup(|app| {
            // 메인 윈도우 가져오기 및 로그인 화면 크기로 설정
            if let Some(window) = app.get_webview_window("main") {
                // 로그인 화면 크기로 설정
                let _ = window.set_size(tauri::Size::Logical(tauri::LogicalSize {
                    width: 370.0,
                    height: 650.0,
                }));

                // 최소 크기 설정 (로그인 화면 기준)
                let _ = window.set_min_size(Some(tauri::Size::Logical(tauri::LogicalSize {
                    width: 370.0,
                    height: 650.0,
                })));

                // 리사이즈 비활성화 (로그인 화면에서는 고정 크기)
                let _ = window.set_resizable(false);

                // 윈도우를 화면 중앙에 배치
                let _ = window.center();

                // 윈도우 표시
                let _ = window.show();
                let _ = window.set_focus();

                log::info!("윈도우 크기가 로그인 화면용 370x650으로 설정되었습니다.");
            } else {
                log::error!("메인 윈도우를 찾을 수 없습니다.");
            }

            Ok(())
        })
        // Tauri 커맨드 등록
        .invoke_handler(tauri::generate_handler![
            // 기본 테스트 커맨드
            greet,
            // 인증 관련 커맨드
            commands::auth::authenticate_pin,
            commands::auth::set_pin_code,
            commands::auth::logout,
            commands::auth::check_auth_status,
            commands::auth::has_pin_set,
            commands::auth::has_recovery_key_set,
            commands::auth::get_session_remaining_time,
            commands::auth::change_pin,
            commands::auth::generate_new_recovery_key,
            commands::auth::authenticate_recovery_key,
            commands::auth::get_auto_logout_time,
            commands::auth::set_auto_logout_time,
            // 복구 키 관련 커맨드 (C# SecurityService 포팅)
            commands::recovery::generate_recovery_key,
            commands::recovery::hash_recovery_key,
            commands::recovery::verify_recovery_key,
            commands::recovery::derive_key_from_recovery_key,
            commands::recovery::verify_and_derive_key,
            commands::recovery::validate_recovery_key_format,
            commands::recovery::get_recovery_key_info,
            commands::recovery::mark_recovery_key_used,
            commands::recovery::clear_recovery_key,
            commands::recovery::authenticate_with_recovery_key,
            // 파일 관리 관련 커맨드 (C# FileManagerService 포팅)
            commands::files::get_files_in_folder,
            commands::files::add_file_to_vault,
            commands::files::add_folder_to_vault,
            commands::files::delete_file_from_vault,
            commands::files::rename_file_in_vault,
            commands::files::move_file,
            commands::files::extract_file_from_vault,
            commands::files::export_file_from_vault,
            commands::files::export_file,
            commands::files::export_folder_from_vault,
            commands::files::create_new_file_in_vault,
            commands::files::create_binary_file_in_vault,
            commands::files::get_file_content,
            commands::files::update_file_content,
            // 청크 기반 파일 업로드 커맨드
            commands::files::start_chunked_upload,
            commands::files::upload_file_chunk,
            commands::files::cancel_chunked_upload,
            // 폴더 관리 관련 커맨드 (C# FolderManager + MainForm 포팅)
            commands::folders::test_create_folder,
            commands::folders::create_folder,
            commands::folders::delete_folder,
            commands::folders::rename_folder,
            commands::folders::move_folder,
            commands::folders::get_folder_tree,
            commands::folders::get_subfolders,
            commands::folders::get_folder,
            commands::folders::get_folder_path,
            commands::folders::get_current_folder_id,
            commands::folders::get_all_folders,
            commands::folders::update_folder_stats,
            commands::folders::calculate_folder_stats,
            commands::folders::export_folder,
            // 데이터베이스 관련 커맨드 (C# MetadataService 포팅)
            commands::database::initialize_database,
            commands::database::add_file_metadata,
            commands::database::get_file_metadata,
            commands::database::get_files_by_folder,
            commands::database::remove_file_metadata,
            commands::database::add_folder_metadata,
            commands::database::get_folder_metadata,
            commands::database::get_all_folders_metadata,
            commands::database::update_folder_metadata,
            commands::database::remove_folder_metadata,
            // 암호화 관련 커맨드
            commands::crypto::derive_master_key_from_pin,
            commands::crypto::generate_salt,
            commands::crypto::generate_crypto_recovery_key,
            commands::crypto::encrypt_data_csharp_compatible,
            commands::crypto::decrypt_data_csharp_compatible,
            commands::crypto::encrypt_file,
            commands::crypto::decrypt_file,
            commands::crypto::has_master_key,
            commands::crypto::clear_sensitive_data,
            commands::crypto::get_encryption_algorithm,
            // 보안 관련 커맨드
            commands::security::get_security_status,
            commands::security::check_network_access,
            // 볼트 관련 커맨드
            commands::vault::initialize_vault,
            commands::vault::get_vault_config,
            commands::vault::update_vault_config,
            commands::vault::get_vault_stats,
            // 병렬 처리 벤치마크 커맨드
            commands::benchmark::run_parallel_benchmark,
            commands::benchmark::benchmark_compression_only,
            commands::benchmark::benchmark_hash_only,
            commands::benchmark::get_system_info,
            // 파일 뷰어 관련 커맨드
            commands::viewer::get_text_file_content,
            commands::viewer::get_binary_file_content,
            commands::viewer::save_text_file,
            commands::viewer::detect_file_mime_type,
            commands::viewer::get_file_viewer_type,
            commands::viewer::get_syntax_language,
            // 미디어 플레이어 관련 커맨드
            commands::media::get_media_metadata,
            commands::media::get_media_stream,
            commands::media::get_full_media_data,
            commands::media::is_media_file_supported,
            commands::media::prepare_media_stream,
            // 업로드 관리 커맨드
            commands::upload::start_file_upload,
            commands::upload::cancel_upload,
            commands::upload::get_upload_status,
            commands::upload::get_all_uploads,
        ])
        .run(tauri::generate_context!())
        .expect("SecureVault 애플리케이션 실행 중 오류가 발생했습니다.");
}

/// 볼트 디렉토리를 초기화합니다 (간단한 버전).
///
/// 애플리케이션 시작 시 필요한 디렉토리 구조를 생성합니다.
///
/// # 반환값
/// * `Result<(), Box<dyn std::error::Error>>` - 초기화 결과
pub fn initialize_vault_directory_simple() -> Result<(), Box<dyn std::error::Error>> {
    use std::fs;

    // 현재 실행 디렉토리 기준으로 볼트 디렉토리 생성
    let base_dir = std::env::current_dir()?;

    // 필요한 디렉토리들 생성
    let directories = [
        base_dir.join(".securevault"),
        base_dir.join(".securevault/config"),
        base_dir.join(".securevault/metadata"),
        base_dir.join(".securevault/data"),
        base_dir.join(".securevault/data/files"),
        base_dir.join(".securevault/data/temp"),
        base_dir.join(".securevault/logs"),
    ];

    for dir in &directories {
        if !dir.exists() {
            fs::create_dir_all(dir)?;
            log::info!("디렉토리 생성: {:?}", dir);
        }
    }

    // .securevault 폴더 숨김 속성 설정 (Windows)
    #[cfg(windows)]
    {
        let securevault_dir = base_dir.join(".securevault");
        if securevault_dir.exists() {
            // +H: 숨김, +S: 시스템 (더 강력한 숨김)
            let _ = std::process::Command::new("attrib")
                .args(["+H", "+S", securevault_dir.to_str().unwrap_or("")])
                .output();
            log::info!(".securevault 폴더 숨김 (+H +S) 속성 설정됨");
        }
    }

    log::info!("볼트 디렉토리 초기화 완료: {:?}", base_dir);
    Ok(())
}

/// 인사 메시지를 생성하는 기본 테스트 커맨드
///
/// 프론트엔드와 백엔드 간의 통신을 테스트하기 위한 간단한 커맨드입니다.
///
/// # 매개변수
/// * `name` - 인사할 대상의 이름
///
/// # 반환값
/// * `String` - 한국어 인사 메시지
#[tauri::command]
fn greet(name: &str) -> String {
    if name.trim().is_empty() {
        "안녕하세요! 이름을 입력해주세요.".to_string()
    } else {
        format!(
            "안녕하세요, {}님! SecureVault에 오신 것을 환영합니다. 🔒",
            name.trim()
        )
    }
}
