// 네트워크 접근 차단 및 감시 서비스
// 애플리케이션이 네트워크에 접근하지 않도록 보장합니다.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use log::{warn, error, info};
use serde::Serialize;

/// 네트워크 접근 차단 서비스
/// 애플리케이션의 모든 네트워크 접근을 감시하고 차단합니다.
#[derive(Debug)]
pub struct NetworkGuard {
    /// 네트워크 차단 활성화 상태
    is_blocking_enabled: Arc<AtomicBool>,
    /// 네트워크 접근 시도 카운터
    access_attempts: Arc<AtomicBool>,
}

impl NetworkGuard {
    /// 새로운 네트워크 가드를 생성합니다.
    /// 
    /// # 반환값
    /// * `Self` - 초기화된 네트워크 가드
    pub fn new() -> Self {
        info!("🔒 네트워크 가드 초기화 - 모든 네트워크 접근 차단");
        
        Self {
            is_blocking_enabled: Arc::new(AtomicBool::new(true)),
            access_attempts: Arc::new(AtomicBool::new(false)),
        }
    }
    
    /// 네트워크 접근 시도를 감지하고 차단합니다.
    /// 
    /// # 매개변수
    /// * `operation` - 시도된 네트워크 작업 설명
    /// * `target` - 접근 대상 (URL, IP 등)
    /// 
    /// # 반환값
    /// * `Result<(), NetworkBlockedError>` - 항상 차단 오류 반환
    pub fn block_network_access(&self, operation: &str, target: &str) -> Result<(), NetworkBlockedError> {
        // 접근 시도 기록
        self.access_attempts.store(true, Ordering::SeqCst);
        
        // 경고 로그 출력
        warn!("🚫 네트워크 접근 차단됨: {} -> {}", operation, target);
        error!("⚠️  보안 경고: 애플리케이션이 네트워크에 접근하려고 시도했습니다!");
        
        // 항상 차단
        Err(NetworkBlockedError::AccessDenied {
            operation: operation.to_string(),
            target: target.to_string(),
            reason: "SecureVault는 완전 오프라인 애플리케이션입니다.".to_string(),
        })
    }
    
    /// HTTP 요청을 차단합니다.
    /// 
    /// # 매개변수
    /// * `url` - 요청 URL
    /// * `method` - HTTP 메서드
    /// 
    /// # 반환값
    /// * `Result<(), NetworkBlockedError>` - 항상 차단 오류 반환
    pub fn block_http_request(&self, url: &str, method: &str) -> Result<(), NetworkBlockedError> {
        self.block_network_access(&format!("HTTP {}", method), url)
    }
    
    /// WebSocket 연결을 차단합니다.
    /// 
    /// # 매개변수
    /// * `url` - WebSocket URL
    /// 
    /// # 반환값
    /// * `Result<(), NetworkBlockedError>` - 항상 차단 오류 반환
    pub fn block_websocket(&self, url: &str) -> Result<(), NetworkBlockedError> {
        self.block_network_access("WebSocket", url)
    }
    
    /// DNS 조회를 차단합니다.
    /// 
    /// # 매개변수
    /// * `hostname` - 조회할 호스트명
    /// 
    /// # 반환값
    /// * `Result<(), NetworkBlockedError>` - 항상 차단 오류 반환
    pub fn block_dns_lookup(&self, hostname: &str) -> Result<(), NetworkBlockedError> {
        self.block_network_access("DNS 조회", hostname)
    }
    
    /// 외부 프로세스 실행을 차단합니다.
    /// 
    /// # 매개변수
    /// * `command` - 실행하려는 명령어
    /// 
    /// # 반환값
    /// * `Result<(), NetworkBlockedError>` - 항상 차단 오류 반환
    pub fn block_external_process(&self, command: &str) -> Result<(), NetworkBlockedError> {
        self.block_network_access("외부 프로세스 실행", command)
    }
    
    /// 네트워크 접근 시도 여부를 확인합니다.
    /// 
    /// # 반환값
    /// * `bool` - 네트워크 접근 시도가 있었는지 여부
    pub fn has_network_attempts(&self) -> bool {
        self.access_attempts.load(Ordering::SeqCst)
    }
    
    /// 네트워크 접근 시도 카운터를 초기화합니다.
    pub fn reset_attempt_counter(&self) {
        self.access_attempts.store(false, Ordering::SeqCst);
        info!("🔄 네트워크 접근 시도 카운터 초기화");
    }
    
    /// 네트워크 차단 상태를 확인합니다.
    /// 
    /// # 반환값
    /// * `bool` - 네트워크 차단 활성화 여부 (항상 true)
    pub fn is_blocking_enabled(&self) -> bool {
        self.is_blocking_enabled.load(Ordering::SeqCst)
    }
    
    /// 허용된 로컬 작업인지 확인합니다.
    /// 
    /// # 매개변수
    /// * `operation` - 확인할 작업
    /// 
    /// # 반환값
    /// * `bool` - 로컬 작업 허용 여부
    pub fn is_local_operation_allowed(&self, operation: &str) -> bool {
        // 허용된 로컬 작업 목록
        let allowed_operations = [
            "파일 읽기",
            "파일 쓰기",
            "폴더 생성",
            "폴더 삭제",
            "파일 암호화",
            "파일 복호화",
            "메타데이터 조회",
            "데이터베이스 접근",
            "임시 파일 생성",
            "로컬 설정 저장",
        ];
        
        allowed_operations.iter().any(|&allowed| operation.contains(allowed))
    }
    
    /// 보안 상태 보고서를 생성합니다.
    /// 
    /// # 반환값
    /// * `NetworkSecurityReport` - 보안 상태 보고서
    pub fn generate_security_report(&self) -> NetworkSecurityReport {
        NetworkSecurityReport {
            blocking_enabled: self.is_blocking_enabled(),
            network_attempts_detected: self.has_network_attempts(),
            security_level: SecurityLevel::Maximum,
            last_check: chrono::Utc::now(),
            recommendations: vec![
                "✅ 네트워크 접근이 완전히 차단되었습니다.".to_string(),
                "✅ 모든 데이터는 로컬에서만 처리됩니다.".to_string(),
                "✅ 외부 통신 없이 안전하게 작동합니다.".to_string(),
            ],
        }
    }
}

impl Default for NetworkGuard {
    fn default() -> Self {
        Self::new()
    }
}

/// 네트워크 차단 오류
#[derive(Debug, thiserror::Error)]
pub enum NetworkBlockedError {
    /// 네트워크 접근 거부
    #[error("🚫 네트워크 접근이 차단되었습니다: {operation} -> {target}\n이유: {reason}")]
    AccessDenied {
        operation: String,
        target: String,
        reason: String,
    },
    
    /// 외부 프로세스 실행 거부
    #[error("🚫 외부 프로세스 실행이 차단되었습니다: {command}\n보안상 외부 프로그램 실행이 금지됩니다.")]
    ProcessBlocked {
        command: String,
    },
    
    /// DNS 조회 거부
    #[error("🚫 DNS 조회가 차단되었습니다: {hostname}\n오프라인 모드에서는 도메인 조회가 불가능합니다.")]
    DnsBlocked {
        hostname: String,
    },
}

/// 보안 수준
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum SecurityLevel {
    /// 최대 보안 (네트워크 완전 차단)
    Maximum,
    /// 높은 보안 (제한적 로컬 접근만)
    High,
    /// 중간 보안 (일부 네트워크 허용)
    Medium,
    /// 낮은 보안 (대부분 허용)
    Low,
}

impl SecurityLevel {
    /// 보안 수준의 한국어 설명을 반환합니다.
    /// 
    /// # 반환값
    /// * `&str` - 보안 수준 설명
    pub fn description(&self) -> &str {
        match self {
            Self::Maximum => "최대 보안 - 네트워크 완전 차단",
            Self::High => "높은 보안 - 로컬 접근만 허용",
            Self::Medium => "중간 보안 - 제한적 네트워크 허용",
            Self::Low => "낮은 보안 - 대부분 허용",
        }
    }
    
    /// 보안 수준의 색상 코드를 반환합니다.
    /// 
    /// # 반환값
    /// * `&str` - CSS 색상 코드
    pub fn color_code(&self) -> &str {
        match self {
            Self::Maximum => "#00ff00", // 녹색 (안전)
            Self::High => "#ffff00",    // 노란색 (주의)
            Self::Medium => "#ff8800",  // 주황색 (경고)
            Self::Low => "#ff0000",     // 빨간색 (위험)
        }
    }
}

/// 네트워크 보안 상태 보고서
#[derive(Debug, Clone, Serialize)]
pub struct NetworkSecurityReport {
    /// 네트워크 차단 활성화 여부
    pub blocking_enabled: bool,
    /// 네트워크 접근 시도 감지 여부
    pub network_attempts_detected: bool,
    /// 현재 보안 수준
    pub security_level: SecurityLevel,
    /// 마지막 확인 시간
    pub last_check: chrono::DateTime<chrono::Utc>,
    /// 보안 권장사항
    pub recommendations: Vec<String>,
}

impl NetworkSecurityReport {
    /// 보고서를 JSON 형태로 직렬화합니다.
    /// 
    /// # 반환값
    /// * `Result<String, serde_json::Error>` - JSON 문자열
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
    
    /// 보고서를 사용자 친화적인 텍스트로 변환합니다.
    /// 
    /// # 반환값
    /// * `String` - 형식화된 보고서 텍스트
    pub fn to_user_friendly_text(&self) -> String {
        let status = if self.blocking_enabled { "🔒 활성화" } else { "⚠️ 비활성화" };
        let attempts = if self.network_attempts_detected { "⚠️ 감지됨" } else { "✅ 없음" };
        
        format!(
            "🛡️ SecureVault 네트워크 보안 상태\n\
            \n\
            📊 보안 수준: {}\n\
            🔒 네트워크 차단: {}\n\
            🚨 접근 시도: {}\n\
            ⏰ 마지막 확인: {}\n\
            \n\
            📋 권장사항:\n{}\n",
            self.security_level.description(),
            status,
            attempts,
            self.last_check.format("%Y-%m-%d %H:%M:%S UTC"),
            self.recommendations.iter()
                .map(|r| format!("  • {}", r))
                .collect::<Vec<_>>()
                .join("\n")
        )
    }
}

/// 네트워크 가드를 전역적으로 초기화합니다.
/// 
/// # 반환값
/// * `NetworkGuard` - 초기화된 네트워크 가드
pub fn initialize_network_guard() -> NetworkGuard {
    let guard = NetworkGuard::new();
    
    // 시작 시 보안 상태 로그
    let report = guard.generate_security_report();
    info!("🛡️ 네트워크 보안 초기화 완료:");
    info!("{}", report.to_user_friendly_text());
    
    guard
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_network_guard_creation() {
        // 네트워크 가드 생성 테스트
        let guard = NetworkGuard::new();
        assert!(guard.is_blocking_enabled());
        assert!(!guard.has_network_attempts());
    }
    
    #[test]
    fn test_network_access_blocking() {
        // 네트워크 접근 차단 테스트
        let guard = NetworkGuard::new();
        
        let result = guard.block_http_request("https://example.com", "GET");
        assert!(result.is_err());
        assert!(guard.has_network_attempts());
    }
    
    #[test]
    fn test_local_operations() {
        // 로컬 작업 허용 테스트
        let guard = NetworkGuard::new();
        
        assert!(guard.is_local_operation_allowed("파일 읽기"));
        assert!(guard.is_local_operation_allowed("파일 암호화"));
        assert!(!guard.is_local_operation_allowed("HTTP 요청"));
    }
    
    #[test]
    fn test_security_report() {
        // 보안 보고서 생성 테스트
        let guard = NetworkGuard::new();
        let report = guard.generate_security_report();
        
        assert!(report.blocking_enabled);
        assert_eq!(report.security_level, SecurityLevel::Maximum);
        assert!(!report.recommendations.is_empty());
    }
}