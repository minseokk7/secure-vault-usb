use tauri::State;
use crate::AppState;
use base64::{Engine as _, engine::general_purpose};
use std::sync::Mutex;

/// 텍스트 파일 내용 읽기
/// 
/// # 매개변수
/// * `file_id` - 파일 ID
/// * `app_state` - 애플리케이션 상태
/// 
/// # 반환값
/// * `Result<String, String>` - 텍스트 내용 또는 에러 메시지
#[tauri::command]
pub fn get_text_file_content(
    file_id: String,
    app_state: State<'_, Mutex<AppState>>,
) -> Result<String, String> {
    let app_state = app_state.lock().map_err(|e| format!("상태 잠금 실패: {}", e))?;
    let mut file_service = app_state.file_service.lock().map_err(|e| format!("파일 서비스 잠금 실패: {}", e))?;
    
    // 동기적으로 파일 내용 읽기 (임시 구현)
    match file_service.get_file_content(&file_id) {
        Ok(data) => {
            // UTF-8로 변환 시도
            match String::from_utf8(data) {
                Ok(text) => Ok(text),
                Err(_) => {
                    // UTF-8이 아닌 경우 인코딩 감지 시도
                    Err("텍스트 파일이 아니거나 지원하지 않는 인코딩입니다.".to_string())
                }
            }
        }
        Err(e) => Err(e.to_string())
    }
}

/// 바이너리 파일 내용 읽기 (Base64 인코딩)
/// 
/// # 매개변수
/// * `file_id` - 파일 ID
/// * `app_state` - 애플리케이션 상태
/// 
/// # 반환값
/// * `Result<String, String>` - Base64 인코딩된 데이터 또는 에러 메시지
#[tauri::command]
pub fn get_binary_file_content(
    file_id: String,
    app_state: State<'_, Mutex<AppState>>,
) -> Result<String, String> {
    let app_state = app_state.lock().map_err(|e| format!("상태 잠금 실패: {}", e))?;
    let mut file_service = app_state.file_service.lock().map_err(|e| format!("파일 서비스 잠금 실패: {}", e))?;
    
    let data = file_service.get_file_content(&file_id).map_err(|e| e.to_string())?;
    
    // Base64로 인코딩하여 프론트엔드에 전송
    Ok(general_purpose::STANDARD.encode(data))
}

/// 텍스트 파일 저장
/// 
/// # 매개변수
/// * `file_id` - 파일 ID
/// * `content` - 저장할 텍스트 내용
/// * `app_state` - 애플리케이션 상태
/// 
/// # 반환값
/// * `Result<(), String>` - 성공 또는 에러 메시지
#[tauri::command]
pub fn save_text_file(
    file_id: String,
    content: String,
    app_state: State<'_, Mutex<AppState>>,
) -> Result<(), String> {
    println!("save_text_file 명령어 호출됨: file_id={}, content_length={}", file_id, content.len());
    log::info!("텍스트 파일 저장 요청: file_id={}, content_length={}", file_id, content.len());
    
    let app_state = app_state.lock().map_err(|e| {
        let error_msg = format!("상태 잠금 실패: {}", e);
        println!("에러: {}", error_msg);
        error_msg
    })?;
    
    let mut file_service = app_state.file_service.lock().map_err(|e| {
        let error_msg = format!("파일 서비스 잠금 실패: {}", e);
        println!("에러: {}", error_msg);
        error_msg
    })?;
    
    let data = content.as_bytes().to_vec();
    
    // 실제 파일 저장 구현
    println!("파일 저장 처리 중...");
    
    match file_service.update_file_content(&file_id, data) {
        Ok(_) => {
            println!("파일 저장 완료");
            log::info!("파일 저장 완료: file_id={}", file_id);
            Ok(())
        }
        Err(e) => {
            let error_msg = format!("파일 저장 실패: {}", e);
            println!("에러: {}", error_msg);
            log::error!("파일 저장 실패: file_id={}, error={}", file_id, e);
            Err(error_msg)
        }
    }
}

/// 파일의 MIME 타입 감지
/// 
/// # 매개변수
/// * `file_id` - 파일 ID
/// * `file_name` - 파일명
/// * `app_state` - 애플리케이션 상태
/// 
/// # 반환값
/// * `Result<String, String>` - MIME 타입 또는 에러 메시지
#[tauri::command]
pub fn detect_file_mime_type(
    file_id: String,
    file_name: String,
    app_state: State<'_, Mutex<AppState>>,
) -> Result<String, String> {
    // 파일 데이터의 일부만 읽어서 MIME 타입 감지 (성능 최적화)
    const SAMPLE_SIZE: usize = 1024; // 첫 1KB만 읽기
    
    let app_state = app_state.lock().map_err(|e| format!("상태 잠금 실패: {}", e))?;
    let mut file_service = app_state.file_service.lock().map_err(|e| format!("파일 서비스 잠금 실패: {}", e))?;
    let viewer_service = app_state.viewer_service.lock().map_err(|e| format!("뷰어 서비스 잠금 실패: {}", e))?;
    
    match file_service.get_file_content(&file_id) {
        Ok(data) => {
            let sample = if data.len() > SAMPLE_SIZE {
                &data[..SAMPLE_SIZE]
            } else {
                &data
            };
            
            let mime_type = viewer_service.detect_mime_type(&file_name, Some(sample));
            Ok(mime_type)
        }
        Err(_) => {
            // 파일을 읽을 수 없으면 파일명만으로 감지
            let mime_type = viewer_service.detect_mime_type(&file_name, None);
            Ok(mime_type)
        }
    }
}

/// 파일 뷰어 지원 여부 확인
/// 
/// # 매개변수
/// * `file_name` - 파일명
/// * `mime_type` - MIME 타입 (선택사항)
/// 
/// # 반환값
/// * `String` - 뷰어 타입 ("text", "image", "media", "unsupported")
#[tauri::command]
pub fn get_file_viewer_type(
    file_name: String,
    mime_type: Option<String>,
) -> String {
    let ext = get_file_extension(&file_name);
    
    // 텍스트 파일 확장자
    const TEXT_EXTENSIONS: &[&str] = &[
        ".txt", ".text", ".rtf", ".md", ".markdown", ".rst", ".asciidoc",
        ".json", ".xml", ".csv", ".yaml", ".yml", ".toml", ".ini", ".cfg", ".conf",
        ".html", ".htm", ".css", ".js", ".ts", ".jsx", ".tsx", ".vue", ".svelte",
        ".c", ".h", ".cpp", ".cxx", ".cc", ".hpp", ".cs", ".java", ".py", ".rs",
        ".go", ".php", ".rb", ".swift", ".kt", ".scala", ".dart", ".lua", ".r",
        ".sql", ".sh", ".bash", ".bat", ".ps1", ".log", ".out", ".err",
        ".gitignore", ".dockerignore", ".env", ".diff", ".patch"
    ];
    
    // 이미지 파일 확장자
    const IMAGE_EXTENSIONS: &[&str] = &[
        ".jpg", ".jpeg", ".png", ".gif", ".bmp", ".webp", ".svg", ".ico", ".tiff", ".tif"
    ];
    
    // 미디어 파일 확장자
    const MEDIA_EXTENSIONS: &[&str] = &[
        ".mp3", ".wav", ".ogg", ".aac", ".flac", ".m4a", ".wma", ".aiff", ".ape", ".opus",
        ".mp4", ".webm", ".avi", ".mov", ".mkv", ".flv", ".wmv", ".m4v", ".3gp"
    ];
    
    // 확장자 기반 판단
    if TEXT_EXTENSIONS.contains(&ext.as_str()) {
        return "text".to_string();
    }
    if IMAGE_EXTENSIONS.contains(&ext.as_str()) {
        return "image".to_string();
    }
    if MEDIA_EXTENSIONS.contains(&ext.as_str()) {
        return "media".to_string();
    }
    
    // MIME 타입 기반 판단
    if let Some(mime) = mime_type {
        if mime.starts_with("text/") {
            return "text".to_string();
        }
        if mime.starts_with("image/") {
            return "image".to_string();
        }
        if mime.starts_with("audio/") || mime.starts_with("video/") {
            return "media".to_string();
        }
        
        // 특별한 MIME 타입들
        match mime.as_str() {
            "application/json" | "application/xml" | "application/javascript" => {
                return "text".to_string();
            }
            _ => {}
        }
    }
    
    "unsupported".to_string()
}

/// 구문 강조 언어 감지
/// 
/// # 매개변수
/// * `file_name` - 파일명
/// 
/// # 반환값
/// * `String` - 구문 강조 언어 ("javascript", "python", "rust", etc.)
#[tauri::command]
pub fn get_syntax_language(file_name: String) -> String {
    let ext = get_file_extension(&file_name);
    let lower_name = file_name.to_lowercase();
    
    // 특수 파일명 처리
    if lower_name.contains("dockerfile") {
        return "dockerfile".to_string();
    }
    if lower_name.contains("makefile") {
        return "makefile".to_string();
    }
    if lower_name.contains("gemfile") || lower_name.contains("rakefile") {
        return "ruby".to_string();
    }
    
    // 확장자 기반 언어 매핑
    match ext.as_str() {
        ".js" | ".mjs" => "javascript",
        ".ts" => "typescript",
        ".jsx" => "jsx",
        ".tsx" => "tsx",
        ".py" | ".pyw" | ".pyi" => "python",
        ".rs" => "rust",
        ".go" => "go",
        ".java" => "java",
        ".c" => "c",
        ".cpp" | ".cxx" | ".cc" => "cpp",
        ".h" | ".hpp" => "c",
        ".cs" => "csharp",
        ".php" | ".phtml" => "php",
        ".rb" | ".rake" | ".gemspec" => "ruby",
        ".swift" => "swift",
        ".kt" => "kotlin",
        ".scala" => "scala",
        ".dart" => "dart",
        ".lua" => "lua",
        ".r" | ".R" => "r",
        ".html" | ".htm" | ".xhtml" => "html",
        ".css" => "css",
        ".scss" => "scss",
        ".sass" => "sass",
        ".less" => "less",
        ".json" => "json",
        ".xml" | ".xsl" | ".xslt" => "xml",
        ".yaml" | ".yml" => "yaml",
        ".toml" => "toml",
        ".ini" | ".cfg" | ".conf" => "ini",
        ".md" | ".markdown" | ".mdown" | ".mkd" => "markdown",
        ".rst" => "rst",
        ".sql" | ".mysql" | ".pgsql" | ".sqlite" => "sql",
        ".sh" | ".bash" | ".zsh" | ".fish" => "bash",
        ".bat" | ".cmd" => "batch",
        ".ps1" | ".psm1" => "powershell",
        ".diff" | ".patch" => "diff",
        ".log" | ".logs" | ".out" | ".err" => "log",
        ".vue" => "vue",
        ".svelte" => "svelte",
        _ => "text"
    }.to_string()
}

/// 파일 확장자 추출
/// 
/// # 매개변수
/// * `file_name` - 파일명
/// 
/// # 반환값
/// * `String` - 소문자 확장자 (점 포함)
fn get_file_extension(file_name: &str) -> String {
    if let Some(pos) = file_name.rfind('.') {
        file_name[pos..].to_lowercase()
    } else {
        String::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_file_extension() {
        assert_eq!(get_file_extension("test.txt"), ".txt");
        assert_eq!(get_file_extension("test.TAR.GZ"), ".gz");
        assert_eq!(get_file_extension("README"), "");
        assert_eq!(get_file_extension("test.JS"), ".js");
    }
    
    #[test]
    fn test_viewer_type_detection() {
        assert_eq!(get_file_viewer_type("test.txt".to_string(), None), "text");
        assert_eq!(get_file_viewer_type("image.jpg".to_string(), None), "image");
        assert_eq!(get_file_viewer_type("music.mp3".to_string(), None), "media");
        assert_eq!(get_file_viewer_type("unknown.xyz".to_string(), None), "unsupported");
    }
    
    #[test]
    fn test_syntax_language_detection() {
        assert_eq!(get_syntax_language("script.js".to_string()), "javascript");
        assert_eq!(get_syntax_language("main.rs".to_string()), "rust");
        assert_eq!(get_syntax_language("Dockerfile".to_string()), "dockerfile");
        assert_eq!(get_syntax_language("config.yaml".to_string()), "yaml");
    }
}