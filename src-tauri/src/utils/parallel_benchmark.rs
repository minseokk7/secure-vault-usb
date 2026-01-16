/// 병렬 처리 성능 벤치마크 유틸리티
/// 
/// 다양한 파일 크기에서 병렬 처리와 순차 처리의 성능을 비교합니다.

use std::time::Instant;
use crate::services::{
    compression::CompressionService,
};
use crate::models::file::{calculate_file_hash, calculate_file_hash_parallel};

/// 병렬 처리 벤치마크 결과
#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    /// 파일 크기 (바이트)
    pub file_size: u64,
    /// 순차 처리 시간 (밀리초)
    pub sequential_time_ms: u64,
    /// 병렬 처리 시간 (밀리초)
    pub parallel_time_ms: u64,
    /// 성능 향상 배수
    pub speedup_factor: f64,
    /// 사용된 스레드 수
    pub thread_count: usize,
}

impl BenchmarkResult {
    /// 새로운 벤치마크 결과를 생성합니다.
    pub fn new(
        file_size: u64,
        sequential_time_ms: u64,
        parallel_time_ms: u64,
        thread_count: usize,
    ) -> Self {
        let speedup_factor = if parallel_time_ms > 0 {
            sequential_time_ms as f64 / parallel_time_ms as f64
        } else {
            0.0
        };

        Self {
            file_size,
            sequential_time_ms,
            parallel_time_ms,
            speedup_factor,
            thread_count,
        }
    }

    /// 결과를 포맷된 문자열로 반환합니다.
    pub fn format_summary(&self) -> String {
        format!(
            "파일 크기: {}MB | 순차: {}ms | 병렬: {}ms | 속도 향상: {:.2}x | 스레드: {}개",
            self.file_size / (1024 * 1024),
            self.sequential_time_ms,
            self.parallel_time_ms,
            self.speedup_factor,
            self.thread_count
        )
    }
}

/// 압축 성능 벤치마크를 실행합니다.
/// 
/// # 매개변수
/// * `test_data` - 테스트할 데이터
/// * `file_extension` - 파일 확장자
/// 
/// # 반환값
/// * `Result<BenchmarkResult, String>` - 벤치마크 결과
pub fn benchmark_compression(test_data: &[u8], file_extension: &str) -> Result<BenchmarkResult, String> {
    let compression_service = CompressionService::new_with_defaults();
    let file_size = test_data.len() as u64;
    let thread_count = num_cpus::get();

    log::info!("압축 벤치마크 시작: {}MB", file_size / (1024 * 1024));

    // 순차 압축 벤치마크
    let sequential_start = Instant::now();
    let _sequential_result = compression_service.compress_data(test_data, None)
        .map_err(|e| format!("순차 압축 실패: {}", e))?;
    let sequential_time = sequential_start.elapsed().as_millis() as u64;

    // 병렬 압축 벤치마크 (임시 파일 사용)
    let temp_dir = std::env::temp_dir();
    let input_path = temp_dir.join("benchmark_input");
    let output_path = temp_dir.join("benchmark_output");

    // 테스트 데이터를 임시 파일에 저장
    std::fs::write(&input_path, test_data)
        .map_err(|e| format!("임시 파일 생성 실패: {}", e))?;

    let parallel_start = Instant::now();
    let _parallel_result = compression_service.compress_file_parallel_streaming(&input_path, &output_path, file_extension)
        .map_err(|e| format!("병렬 압축 실패: {}", e))?;
    let parallel_time = parallel_start.elapsed().as_millis() as u64;

    // 임시 파일 정리
    let _ = std::fs::remove_file(&input_path);
    let _ = std::fs::remove_file(&output_path);

    let result = BenchmarkResult::new(file_size, sequential_time, parallel_time, thread_count);
    log::info!("압축 벤치마크 완료: {}", result.format_summary());

    Ok(result)
}

/// 해시 계산 성능 벤치마크를 실행합니다.
/// 
/// # 매개변수
/// * `test_data` - 테스트할 데이터
/// 
/// # 반환값
/// * `Result<BenchmarkResult, String>` - 벤치마크 결과
pub fn benchmark_hash_calculation(test_data: &[u8]) -> Result<BenchmarkResult, String> {
    let file_size = test_data.len() as u64;
    let thread_count = num_cpus::get();

    log::info!("해시 계산 벤치마크 시작: {}MB", file_size / (1024 * 1024));

    // 순차 해시 계산 벤치마크
    let sequential_start = Instant::now();
    let _sequential_hash = calculate_file_hash(test_data);
    let sequential_time = sequential_start.elapsed().as_millis() as u64;

    // 병렬 해시 계산 벤치마크
    let parallel_start = Instant::now();
    let _parallel_hash = calculate_file_hash_parallel(test_data);
    let parallel_time = parallel_start.elapsed().as_millis() as u64;

    let result = BenchmarkResult::new(file_size, sequential_time, parallel_time, thread_count);
    log::info!("해시 계산 벤치마크 완료: {}", result.format_summary());

    Ok(result)
}

/// 전체 파일 처리 파이프라인 벤치마크를 실행합니다.
/// 
/// # 매개변수
/// * `test_sizes` - 테스트할 파일 크기들 (바이트)
/// 
/// # 반환값
/// * `Vec<BenchmarkResult>` - 각 크기별 벤치마크 결과
pub fn benchmark_full_pipeline(test_sizes: &[u64]) -> Vec<BenchmarkResult> {
    let mut results = Vec::new();

    for &size in test_sizes {
        log::info!("파이프라인 벤치마크 시작: {}MB", size / (1024 * 1024));

        // 테스트 데이터 생성 (반복 패턴으로 압축 가능한 데이터)
        let pattern = b"SecureVault benchmark data with some repetitive content for compression testing. ";
        let mut test_data = Vec::with_capacity(size as usize);
        while test_data.len() < size as usize {
            let remaining = size as usize - test_data.len();
            let copy_size = std::cmp::min(pattern.len(), remaining);
            test_data.extend_from_slice(&pattern[..copy_size]);
        }

        // 압축 벤치마크
        if let Ok(compression_result) = benchmark_compression(&test_data, "txt") {
            results.push(compression_result);
        }

        // 해시 계산 벤치마크
        if let Ok(hash_result) = benchmark_hash_calculation(&test_data) {
            results.push(hash_result);
        }
    }

    results
}

/// 시스템 정보를 출력합니다.
pub fn print_system_info() {
    let cpu_count = num_cpus::get();
    let physical_cpu_count = num_cpus::get_physical();
    
    log::info!("=== 시스템 정보 ===");
    log::info!("논리 CPU 코어: {} 개", cpu_count);
    log::info!("물리 CPU 코어: {} 개", physical_cpu_count);
    log::info!("병렬 처리 최적화: {}MB 이상 파일에 적용", 100);
    log::info!("==================");
}

/// 병렬 처리 효과를 분석합니다.
/// 
/// # 매개변수
/// * `results` - 벤치마크 결과들
/// 
/// # 반환값
/// * `String` - 분석 결과 요약
pub fn analyze_parallel_effectiveness(results: &[BenchmarkResult]) -> String {
    if results.is_empty() {
        return "분석할 결과가 없습니다.".to_string();
    }

    let mut analysis = String::new();
    analysis.push_str("=== 병렬 처리 효과 분석 ===\n");

    let avg_speedup: f64 = results.iter().map(|r| r.speedup_factor).sum::<f64>() / results.len() as f64;
    let max_speedup = results.iter().map(|r| r.speedup_factor).fold(0.0, f64::max);
    let min_speedup = results.iter().map(|r| r.speedup_factor).fold(f64::INFINITY, f64::min);

    analysis.push_str(&format!("평균 속도 향상: {:.2}x\n", avg_speedup));
    analysis.push_str(&format!("최대 속도 향상: {:.2}x\n", max_speedup));
    analysis.push_str(&format!("최소 속도 향상: {:.2}x\n", min_speedup));

    // 효과적인 파일 크기 범위 분석
    let effective_results: Vec<_> = results.iter().filter(|r| r.speedup_factor > 1.5).collect();
    if !effective_results.is_empty() {
        let min_effective_size = effective_results.iter().map(|r| r.file_size).min().unwrap();
        let max_effective_size = effective_results.iter().map(|r| r.file_size).max().unwrap();
        
        analysis.push_str(&format!("효과적인 파일 크기 범위: {}MB - {}MB\n", 
                                 min_effective_size / (1024 * 1024),
                                 max_effective_size / (1024 * 1024)));
    }

    analysis.push_str("========================\n");
    analysis
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_benchmark_result_creation() {
        let result = BenchmarkResult::new(100 * 1024 * 1024, 1000, 400, 8);
        
        assert_eq!(result.file_size, 100 * 1024 * 1024);
        assert_eq!(result.sequential_time_ms, 1000);
        assert_eq!(result.parallel_time_ms, 400);
        assert_eq!(result.thread_count, 8);
        assert!((result.speedup_factor - 2.5).abs() < 0.01);
    }

    #[test]
    fn test_format_summary() {
        let result = BenchmarkResult::new(100 * 1024 * 1024, 1000, 400, 8);
        let summary = result.format_summary();
        
        assert!(summary.contains("100MB"));
        assert!(summary.contains("1000ms"));
        assert!(summary.contains("400ms"));
        assert!(summary.contains("2.50x"));
        assert!(summary.contains("8개"));
    }

    #[test]
    fn test_hash_benchmark() {
        // 작은 테스트 데이터로 해시 벤치마크 테스트
        let test_data = vec![0u8; 1024]; // 1KB
        let result = benchmark_hash_calculation(&test_data);
        
        assert!(result.is_ok());
        let benchmark = result.unwrap();
        assert_eq!(benchmark.file_size, 1024);
        assert!(benchmark.sequential_time_ms >= 0);
        assert!(benchmark.parallel_time_ms >= 0);
    }
}