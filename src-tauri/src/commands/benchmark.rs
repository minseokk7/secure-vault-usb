/// 병렬 처리 성능 벤치마크 명령어
/// 
/// 다양한 파일 크기에서 병렬 처리 성능을 측정하고 결과를 반환합니다.

use crate::utils::parallel_benchmark::{
    benchmark_compression, benchmark_hash_calculation, benchmark_full_pipeline,
    print_system_info, analyze_parallel_effectiveness, BenchmarkResult
};
use crate::AppState;
use tauri::State;
use std::sync::Mutex;

/// 병렬 처리 벤치마크를 실행합니다.
/// 
/// # 매개변수
/// * `test_sizes_mb` - 테스트할 파일 크기들 (MB 단위)
/// * `state` - 애플리케이션 상태
/// 
/// # 반환값
/// * `Result<BenchmarkSummary, String>` - 벤치마크 결과 요약
#[tauri::command]
pub async fn run_parallel_benchmark(
    test_sizes_mb: Vec<u32>,
    _state: State<'_, Mutex<AppState>>
) -> Result<BenchmarkSummary, String> {
    log::info!("병렬 처리 벤치마크 시작: {:?}MB", test_sizes_mb);
    
    // 시스템 정보 출력
    print_system_info();
    
    // MB를 바이트로 변환
    let test_sizes: Vec<u64> = test_sizes_mb.iter()
        .map(|&size_mb| (size_mb as u64) * 1024 * 1024)
        .collect();
    
    // 전체 파이프라인 벤치마크 실행
    let results = benchmark_full_pipeline(&test_sizes);
    
    // 결과 분석
    let analysis = analyze_parallel_effectiveness(&results);
    log::info!("{}", analysis);
    
    // 결과 요약 생성 (results를 복제하여 사용)
    let results_dto: Vec<BenchmarkResultDto> = results.iter().map(|r| BenchmarkResultDto {
        file_size_mb: r.file_size / (1024 * 1024),
        sequential_time_ms: r.sequential_time_ms,
        parallel_time_ms: r.parallel_time_ms,
        speedup_factor: r.speedup_factor,
        thread_count: r.thread_count,
    }).collect();
    
    let summary = BenchmarkSummary {
        system_info: SystemInfo {
            logical_cpu_count: num_cpus::get(),
            physical_cpu_count: num_cpus::get_physical(),
            parallel_threshold_mb: 100,
        },
        results: results_dto,
        analysis_summary: analysis,
        recommendation: generate_recommendation(&results),
    };
    
    log::info!("병렬 처리 벤치마크 완료: {} 테스트 실행", results.len());
    Ok(summary)
}

/// 압축 성능만 벤치마크합니다.
/// 
/// # 매개변수
/// * `file_size_mb` - 테스트할 파일 크기 (MB)
/// * `file_extension` - 파일 확장자
/// * `state` - 애플리케이션 상태
/// 
/// # 반환값
/// * `Result<BenchmarkResultDto, String>` - 벤치마크 결과
#[tauri::command]
pub async fn benchmark_compression_only(
    file_size_mb: u32,
    file_extension: String,
    _state: State<'_, Mutex<AppState>>
) -> Result<BenchmarkResultDto, String> {
    log::info!("압축 벤치마크 시작: {}MB, 확장자: {}", file_size_mb, file_extension);
    
    // 테스트 데이터 생성
    let file_size = (file_size_mb as u64) * 1024 * 1024;
    let pattern = b"SecureVault compression benchmark data with repetitive patterns for testing. ";
    let mut test_data = Vec::with_capacity(file_size as usize);
    while test_data.len() < file_size as usize {
        let remaining = file_size as usize - test_data.len();
        let copy_size = std::cmp::min(pattern.len(), remaining);
        test_data.extend_from_slice(&pattern[..copy_size]);
    }
    
    // 압축 벤치마크 실행
    let result = benchmark_compression(&test_data, &file_extension)
        .map_err(|e| format!("압축 벤치마크 실패: {}", e))?;
    
    let dto = BenchmarkResultDto {
        file_size_mb: result.file_size / (1024 * 1024),
        sequential_time_ms: result.sequential_time_ms,
        parallel_time_ms: result.parallel_time_ms,
        speedup_factor: result.speedup_factor,
        thread_count: result.thread_count,
    };
    
    log::info!("압축 벤치마크 완료: {}", result.format_summary());
    Ok(dto)
}

/// 해시 계산 성능만 벤치마크합니다.
/// 
/// # 매개변수
/// * `file_size_mb` - 테스트할 파일 크기 (MB)
/// * `state` - 애플리케이션 상태
/// 
/// # 반환값
/// * `Result<BenchmarkResultDto, String>` - 벤치마크 결과
#[tauri::command]
pub async fn benchmark_hash_only(
    file_size_mb: u32,
    _state: State<'_, Mutex<AppState>>
) -> Result<BenchmarkResultDto, String> {
    log::info!("해시 계산 벤치마크 시작: {}MB", file_size_mb);
    
    // 테스트 데이터 생성
    let file_size = (file_size_mb as u64) * 1024 * 1024;
    let mut test_data = vec![0u8; file_size as usize];
    
    // 랜덤 데이터로 채우기 (해시 계산에 더 현실적)
    use rand::RngCore;
    let mut rng = rand::thread_rng();
    rng.fill_bytes(&mut test_data);
    
    // 해시 계산 벤치마크 실행
    let result = benchmark_hash_calculation(&test_data)
        .map_err(|e| format!("해시 벤치마크 실패: {}", e))?;
    
    let dto = BenchmarkResultDto {
        file_size_mb: result.file_size / (1024 * 1024),
        sequential_time_ms: result.sequential_time_ms,
        parallel_time_ms: result.parallel_time_ms,
        speedup_factor: result.speedup_factor,
        thread_count: result.thread_count,
    };
    
    log::info!("해시 계산 벤치마크 완료: {}", result.format_summary());
    Ok(dto)
}

/// 시스템 정보를 조회합니다.
/// 
/// # 매개변수
/// * `state` - 애플리케이션 상태
/// 
/// # 반환값
/// * `Result<SystemInfo, String>` - 시스템 정보
#[tauri::command]
pub async fn get_system_info(
    _state: State<'_, Mutex<AppState>>
) -> Result<SystemInfo, String> {
    let info = SystemInfo {
        logical_cpu_count: num_cpus::get(),
        physical_cpu_count: num_cpus::get_physical(),
        parallel_threshold_mb: 100,
    };
    
    log::info!("시스템 정보 조회: 논리 CPU {}개, 물리 CPU {}개", 
              info.logical_cpu_count, info.physical_cpu_count);
    
    Ok(info)
}

/// 벤치마크 결과 요약
#[derive(serde::Serialize)]
pub struct BenchmarkSummary {
    /// 시스템 정보
    pub system_info: SystemInfo,
    /// 벤치마크 결과들
    pub results: Vec<BenchmarkResultDto>,
    /// 분석 요약
    pub analysis_summary: String,
    /// 권장사항
    pub recommendation: String,
}

/// 시스템 정보
#[derive(serde::Serialize)]
pub struct SystemInfo {
    /// 논리 CPU 코어 수
    pub logical_cpu_count: usize,
    /// 물리 CPU 코어 수
    pub physical_cpu_count: usize,
    /// 병렬 처리 임계값 (MB)
    pub parallel_threshold_mb: u64,
}

/// 벤치마크 결과 DTO
#[derive(serde::Serialize)]
pub struct BenchmarkResultDto {
    /// 파일 크기 (MB)
    pub file_size_mb: u64,
    /// 순차 처리 시간 (밀리초)
    pub sequential_time_ms: u64,
    /// 병렬 처리 시간 (밀리초)
    pub parallel_time_ms: u64,
    /// 성능 향상 배수
    pub speedup_factor: f64,
    /// 사용된 스레드 수
    pub thread_count: usize,
}

/// 벤치마크 결과를 바탕으로 권장사항을 생성합니다.
/// 
/// # 매개변수
/// * `results` - 벤치마크 결과들
/// 
/// # 반환값
/// * `String` - 권장사항 텍스트
fn generate_recommendation(results: &[BenchmarkResult]) -> String {
    if results.is_empty() {
        return "벤치마크 결과가 없습니다.".to_string();
    }

    let avg_speedup: f64 = results.iter().map(|r| r.speedup_factor).sum::<f64>() / results.len() as f64;
    let cpu_count = num_cpus::get();
    
    let mut recommendation = String::new();
    
    if avg_speedup > 2.0 {
        recommendation.push_str("✅ 병렬 처리가 매우 효과적입니다!\n");
        recommendation.push_str(&format!("평균 {}배 성능 향상으로 큰 파일 처리 시 상당한 시간 절약이 가능합니다.\n", avg_speedup as u32));
    } else if avg_speedup > 1.5 {
        recommendation.push_str("✅ 병렬 처리가 효과적입니다.\n");
        recommendation.push_str(&format!("평균 {:.1}배 성능 향상으로 큰 파일 처리 시 시간 절약이 가능합니다.\n", avg_speedup));
    } else if avg_speedup > 1.0 {
        recommendation.push_str("⚠️ 병렬 처리 효과가 제한적입니다.\n");
        recommendation.push_str("CPU 코어 수가 적거나 I/O 병목이 있을 수 있습니다.\n");
    } else {
        recommendation.push_str("❌ 병렬 처리가 효과적이지 않습니다.\n");
        recommendation.push_str("순차 처리를 사용하는 것이 더 효율적일 수 있습니다.\n");
    }
    
    recommendation.push_str(&format!("\n현재 시스템: {}개 논리 CPU 코어\n", cpu_count));
    recommendation.push_str("권장 설정: 100MB 이상 파일에 병렬 처리 적용\n");
    
    // 최적 파일 크기 범위 추천
    let effective_results: Vec<_> = results.iter().filter(|r| r.speedup_factor > 1.5).collect();
    if !effective_results.is_empty() {
        let min_effective_size = effective_results.iter().map(|r| r.file_size).min().unwrap();
        recommendation.push_str(&format!("최적 효과: {}MB 이상 파일", min_effective_size / (1024 * 1024)));
    }
    
    recommendation
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_benchmark_result_dto_creation() {
        let dto = BenchmarkResultDto {
            file_size_mb: 100,
            sequential_time_ms: 1000,
            parallel_time_ms: 400,
            speedup_factor: 2.5,
            thread_count: 8,
        };
        
        assert_eq!(dto.file_size_mb, 100);
        assert_eq!(dto.sequential_time_ms, 1000);
        assert_eq!(dto.parallel_time_ms, 400);
        assert!((dto.speedup_factor - 2.5).abs() < 0.01);
        assert_eq!(dto.thread_count, 8);
    }

    #[test]
    fn test_system_info_creation() {
        let info = SystemInfo {
            logical_cpu_count: 8,
            physical_cpu_count: 4,
            parallel_threshold_mb: 100,
        };
        
        assert_eq!(info.logical_cpu_count, 8);
        assert_eq!(info.physical_cpu_count, 4);
        assert_eq!(info.parallel_threshold_mb, 100);
    }

    #[test]
    fn test_generate_recommendation() {
        let results = vec![
            BenchmarkResult::new(100 * 1024 * 1024, 1000, 400, 8),
            BenchmarkResult::new(200 * 1024 * 1024, 2000, 800, 8),
        ];
        
        let recommendation = generate_recommendation(&results);
        
        assert!(recommendation.contains("병렬 처리가"));
        assert!(recommendation.contains("성능 향상"));
        assert!(recommendation.contains("CPU 코어"));
    }
}