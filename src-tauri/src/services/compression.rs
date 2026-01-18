use crate::models::compression::{
    CompressionError, CompressionLevel, CompressionResult, CompressionSettings,
};
use flate2::{read::GzDecoder, write::GzEncoder, Compression};
use std::io::{Read, Write};
use std::time::Instant;

/// 압축 서비스
/// 파일 데이터의 압축 및 압축 해제를 담당합니다.
/// C# 버전에서는 설정만 있고 실제 구현이 없었으므로, Rust에서 완전히 새로 구현합니다.
#[derive(Debug, Clone)]
pub struct CompressionService {
    /// 압축 설정
    settings: CompressionSettings,
}

impl CompressionService {
    /// 새로운 압축 서비스를 생성합니다.
    ///
    /// # 매개변수
    /// * `settings` - 압축 설정
    ///
    /// # 반환값
    /// * `Self` - 압축 서비스 인스턴스
    pub fn new(settings: CompressionSettings) -> Self {
        Self { settings }
    }

    /// 기본 설정으로 압축 서비스를 생성합니다.
    ///
    /// # 반환값
    /// * `Self` - 기본 설정의 압축 서비스 인스턴스
    pub fn new_with_defaults() -> Self {
        Self {
            settings: CompressionSettings::default(),
        }
    }

    /// 압축 설정을 업데이트합니다.
    ///
    /// # 매개변수
    /// * `settings` - 새로운 압축 설정
    pub fn update_settings(&mut self, settings: CompressionSettings) {
        self.settings = settings;
    }

    /// 현재 압축 설정을 반환합니다.
    ///
    /// # 반환값
    /// * `&CompressionSettings` - 현재 압축 설정
    pub fn get_settings(&self) -> &CompressionSettings {
        &self.settings
    }

    /// 파일이 압축 대상인지 확인합니다.
    ///
    /// # 매개변수
    /// * `file_size` - 파일 크기 (바이트)
    /// * `file_extension` - 파일 확장자
    ///
    /// # 반환값
    /// * `bool` - 압축 대상 여부
    pub fn should_compress(&self, file_size: u64, file_extension: &str) -> bool {
        self.settings.should_compress(file_size, file_extension)
    }

    /// 데이터를 압축합니다.
    ///
    /// # 매개변수
    /// * `data` - 압축할 데이터
    /// * `level` - 압축 레벨 (None이면 설정의 기본값 사용)
    ///
    /// # 반환값
    /// * `Result<(Vec<u8>, CompressionResult), CompressionError>` - 압축된 데이터와 결과 정보
    pub fn compress_data(
        &self,
        data: &[u8],
        level: Option<CompressionLevel>,
    ) -> Result<(Vec<u8>, CompressionResult), CompressionError> {
        if data.is_empty() {
            return Err(CompressionError::CompressionFailed(
                "빈 데이터는 압축할 수 없습니다.".to_string(),
            ));
        }

        let compression_level = level.unwrap_or(self.settings.level);
        let start_time = Instant::now();
        let original_size = data.len() as u64;

        // Gzip 압축 레벨 변환
        let gzip_level = match compression_level {
            CompressionLevel::Fast => Compression::fast(),
            CompressionLevel::Normal => Compression::default(),
            CompressionLevel::Maximum => Compression::best(),
        };

        // 압축 수행
        let mut encoder = GzEncoder::new(Vec::new(), gzip_level);
        encoder
            .write_all(data)
            .map_err(|e| CompressionError::CompressionFailed(format!("압축 중 오류: {}", e)))?;

        let compressed_data = encoder.finish().map_err(|e| {
            CompressionError::CompressionFailed(format!("압축 완료 중 오류: {}", e))
        })?;

        let compression_time = start_time.elapsed().as_millis() as u64;
        let compressed_size = compressed_data.len() as u64;

        // 압축 결과 생성
        let result = CompressionResult::new(
            original_size,
            compressed_size,
            compression_time,
            compression_level,
        );

        // 압축 효과가 없는 경우 원본 데이터 반환
        if compressed_size >= original_size {
            log::debug!(
                "압축 효과가 없어 원본 데이터를 반환합니다. 원본: {}바이트, 압축: {}바이트",
                original_size,
                compressed_size
            );
            return Ok((data.to_vec(), result));
        }

        log::debug!(
            "압축 완료: {}바이트 -> {}바이트 ({:.1}% 절약)",
            original_size,
            compressed_size,
            result.space_saved_percent()
        );

        Ok((compressed_data, result))
    }

    /// 압축된 데이터를 해제합니다.
    ///
    /// # 매개변수
    /// * `compressed_data` - 압축된 데이터
    ///
    /// # 반환값
    /// * `Result<Vec<u8>, CompressionError>` - 압축 해제된 데이터
    pub fn decompress_data(&self, compressed_data: &[u8]) -> Result<Vec<u8>, CompressionError> {
        if compressed_data.is_empty() {
            return Err(CompressionError::DecompressionFailed(
                "빈 데이터는 압축 해제할 수 없습니다.".to_string(),
            ));
        }

        let start_time = Instant::now();

        // Gzip 압축 해제
        let mut decoder = GzDecoder::new(compressed_data);
        let mut decompressed_data = Vec::new();

        decoder.read_to_end(&mut decompressed_data).map_err(|e| {
            CompressionError::DecompressionFailed(format!("압축 해제 중 오류: {}", e))
        })?;

        let decompression_time = start_time.elapsed().as_millis();
        log::debug!(
            "압축 해제 완료: {}바이트 -> {}바이트 ({}ms)",
            compressed_data.len(),
            decompressed_data.len(),
            decompression_time
        );

        Ok(decompressed_data)
    }

    /// 파일을 병렬 스트리밍 방식으로 압축합니다 (최고 성능).
    ///
    /// # 매개변수
    /// * `input_path` - 입력 파일 경로
    /// * `output_path` - 출력 파일 경로
    /// * `file_extension` - 파일 확장자
    ///
    /// # 반환값
    /// * `Result<CompressionResult, CompressionError>` - 압축 결과
    pub fn compress_file_parallel_streaming<P: AsRef<std::path::Path>>(
        &self,
        input_path: P,
        output_path: P,
        file_extension: &str,
    ) -> Result<CompressionResult, CompressionError> {
        use std::fs::File;
        use std::io::{BufWriter, Write};
        use std::sync::{Arc, Mutex};
        use std::thread;

        let input_path = input_path.as_ref();
        let output_path = output_path.as_ref();

        // 파일 크기 확인
        let file_size = std::fs::metadata(input_path)
            .map_err(|e| {
                CompressionError::CompressionFailed(format!("입력 파일 정보 읽기 실패: {}", e))
            })?
            .len();

        // 작은 파일은 기존 방식 사용
        if file_size < 100 * 1024 * 1024 {
            // 100MB 미만
            return self.compress_file_streaming(input_path, output_path, file_extension);
        }

        // 압축 대상인지 확인
        if !self.should_compress(file_size, file_extension) {
            std::fs::copy(input_path, output_path).map_err(|e| {
                CompressionError::CompressionFailed(format!("파일 복사 실패: {}", e))
            })?;
            return Ok(CompressionResult::new(
                file_size,
                file_size,
                0,
                self.settings.level,
            ));
        }

        let start_time = Instant::now();

        log::info!("병렬 압축 시작: {}MB", file_size / (1024 * 1024));

        // 병렬 처리용 청크 크기 (32MB)
        const PARALLEL_CHUNK_SIZE: usize = 32 * 1024 * 1024;
        let num_chunks =
            ((file_size as usize + PARALLEL_CHUNK_SIZE - 1) / PARALLEL_CHUNK_SIZE).max(1);
        let num_threads = std::cmp::min(num_chunks, num_cpus::get()).max(1);

        log::info!(
            "병렬 압축: {} 청크, {} 스레드 사용",
            num_chunks,
            num_threads
        );

        // 입력 파일 읽기
        let input_data = std::fs::read(input_path)
            .map_err(|e| CompressionError::CompressionFailed(format!("파일 읽기 실패: {}", e)))?;

        // 병렬 압축 처리
        let compressed_chunks = Arc::new(Mutex::new(Vec::with_capacity(num_chunks)));
        let mut handles = Vec::new();

        // 초고속 압축 레벨 사용
        let gzip_level = Compression::fast();

        for chunk_idx in 0..num_chunks {
            let start = chunk_idx * PARALLEL_CHUNK_SIZE;
            let end = std::cmp::min(start + PARALLEL_CHUNK_SIZE, input_data.len());
            let chunk_data = input_data[start..end].to_vec();

            let compressed_chunks_clone = Arc::clone(&compressed_chunks);

            let handle = thread::spawn(move || {
                use flate2::write::GzEncoder;
                use std::io::Write;

                // 각 청크를 개별적으로 압축
                let mut encoder = GzEncoder::new(Vec::new(), gzip_level);
                encoder.write_all(&chunk_data)?;
                let compressed_chunk = encoder.finish()?;

                let mut chunks = compressed_chunks_clone.lock().unwrap();
                chunks.push((chunk_idx, compressed_chunk));

                Ok::<(), CompressionError>(())
            });

            handles.push(handle);
        }

        // 모든 스레드 완료 대기
        for handle in handles {
            handle
                .join()
                .map_err(|_| {
                    CompressionError::CompressionFailed("병렬 압축 스레드 실패".to_string())
                })?
                .map_err(|e| e)?;
        }

        // 결과 정렬 및 파일 쓰기
        let mut compressed_chunks = compressed_chunks.lock().unwrap();
        compressed_chunks.sort_by_key(|(idx, _)| *idx);

        let output_file = File::create(output_path).map_err(|e| {
            CompressionError::CompressionFailed(format!("출력 파일 생성 실패: {}", e))
        })?;
        let mut writer = BufWriter::new(output_file);

        // 병렬 압축 헤더 작성 (청크 수 정보)
        let chunk_count = compressed_chunks.len() as u32;
        writer
            .write_all(&chunk_count.to_le_bytes())
            .map_err(|e| CompressionError::CompressionFailed(format!("헤더 쓰기 실패: {}", e)))?;

        let mut total_compressed_size = 4u64; // 헤더 크기

        for (_, compressed_chunk) in compressed_chunks.iter() {
            // 청크 크기 저장 (4바이트)
            let chunk_size = compressed_chunk.len() as u32;
            writer.write_all(&chunk_size.to_le_bytes()).map_err(|e| {
                CompressionError::CompressionFailed(format!("청크 크기 쓰기 실패: {}", e))
            })?;

            // 압축된 청크 데이터 저장
            writer.write_all(compressed_chunk).map_err(|e| {
                CompressionError::CompressionFailed(format!("압축된 청크 쓰기 실패: {}", e))
            })?;

            total_compressed_size += 4 + compressed_chunk.len() as u64;
        }

        writer.flush().map_err(|e| {
            CompressionError::CompressionFailed(format!("파일 쓰기 완료 실패: {}", e))
        })?;

        let compression_time = start_time.elapsed().as_millis() as u64;
        let result = CompressionResult::new(
            file_size,
            total_compressed_size,
            compression_time,
            CompressionLevel::Fast,
        );

        log::info!(
            "병렬 압축 완료: {}MB -> {}MB ({:.1}% 절약, {}ms, {} 스레드)",
            file_size / (1024 * 1024),
            total_compressed_size / (1024 * 1024),
            result.space_saved_percent(),
            compression_time,
            num_threads
        );

        Ok(result)
    }

    /// 파일을 스트리밍 방식으로 압축합니다 (메모리 효율적).
    ///
    /// # 매개변수
    /// * `input_path` - 입력 파일 경로
    /// * `output_path` - 출력 파일 경로
    /// * `file_extension` - 파일 확장자
    ///
    /// # 반환값
    /// * `Result<CompressionResult, CompressionError>` - 압축 결과
    pub fn compress_file_streaming<P: AsRef<std::path::Path>>(
        &self,
        input_path: P,
        output_path: P,
        file_extension: &str,
    ) -> Result<CompressionResult, CompressionError> {
        use std::fs::File;
        use std::io::{BufReader, BufWriter};

        let input_path = input_path.as_ref();
        let output_path = output_path.as_ref();

        // 파일 크기 확인
        let file_size = std::fs::metadata(input_path)
            .map_err(|e| {
                CompressionError::CompressionFailed(format!("입력 파일 정보 읽기 실패: {}", e))
            })?
            .len();

        // 압축 대상인지 확인
        if !self.should_compress(file_size, file_extension) {
            // 압축하지 않는 경우 파일 복사
            std::fs::copy(input_path, output_path).map_err(|e| {
                CompressionError::CompressionFailed(format!("파일 복사 실패: {}", e))
            })?;

            return Ok(CompressionResult::new(
                file_size,
                file_size,
                0,
                self.settings.level,
            ));
        }

        let start_time = Instant::now();

        // 파일 열기
        let input_file = File::open(input_path).map_err(|e| {
            CompressionError::CompressionFailed(format!("입력 파일 열기 실패: {}", e))
        })?;
        let output_file = File::create(output_path).map_err(|e| {
            CompressionError::CompressionFailed(format!("출력 파일 생성 실패: {}", e))
        })?;

        let mut reader = BufReader::new(input_file);
        let writer = BufWriter::new(output_file);

        // Gzip 압축 레벨 변환
        let gzip_level = match self.settings.level {
            CompressionLevel::Fast => Compression::fast(),
            CompressionLevel::Normal => Compression::default(),
            CompressionLevel::Maximum => Compression::best(),
        };

        // 스트리밍 압축 수행
        let mut encoder = GzEncoder::new(writer, gzip_level);

        // 1MB 버퍼로 스트리밍 압축 (성능 향상)
        const BUFFER_SIZE: usize = 1024 * 1024;
        let mut buffer = vec![0u8; BUFFER_SIZE];
        let mut total_read = 0u64;

        loop {
            let bytes_read = reader.read(&mut buffer).map_err(|e| {
                CompressionError::CompressionFailed(format!("파일 읽기 실패: {}", e))
            })?;

            if bytes_read == 0 {
                break; // EOF
            }

            encoder.write_all(&buffer[..bytes_read]).map_err(|e| {
                CompressionError::CompressionFailed(format!("압축 쓰기 실패: {}", e))
            })?;

            total_read += bytes_read as u64;

            // 주기적으로 진행 상황 로그 (100MB마다)
            if total_read % (100 * 1024 * 1024) == 0 {
                log::info!(
                    "스트리밍 압축 진행: {}MB 처리 완료",
                    total_read / (1024 * 1024)
                );
            }
        }

        // 압축 완료
        encoder
            .finish()
            .map_err(|e| CompressionError::CompressionFailed(format!("압축 완료 실패: {}", e)))?;

        let compression_time = start_time.elapsed().as_millis() as u64;

        // 압축된 파일 크기 확인
        let compressed_size = std::fs::metadata(output_path)
            .map_err(|e| {
                CompressionError::CompressionFailed(format!("출력 파일 정보 읽기 실패: {}", e))
            })?
            .len();

        let result = CompressionResult::new(
            file_size,
            compressed_size,
            compression_time,
            self.settings.level,
        );

        // 압축 효과가 없는 경우 원본 파일로 교체
        if compressed_size >= file_size {
            log::debug!(
                "스트리밍 압축 효과가 없어 원본 파일로 교체: {} -> {} bytes",
                file_size,
                compressed_size
            );
            std::fs::copy(input_path, output_path).map_err(|e| {
                CompressionError::CompressionFailed(format!("원본 파일 복사 실패: {}", e))
            })?;

            return Ok(CompressionResult::new(
                file_size,
                file_size,
                compression_time,
                self.settings.level,
            ));
        }

        log::info!(
            "스트리밍 압축 완료: {} -> {} bytes ({:.1}% 절약, {}ms)",
            file_size,
            compressed_size,
            result.space_saved_percent(),
            compression_time
        );

        Ok(result)
    }

    /// 압축된 파일을 스트리밍 방식으로 해제합니다 (메모리 효율적).
    ///
    /// # 매개변수
    /// * `input_path` - 압축된 파일 경로
    /// * `output_path` - 출력 파일 경로
    ///
    /// # 반환값
    /// * `Result<u64, CompressionError>` - 압축 해제된 파일 크기
    pub fn decompress_file_streaming<P: AsRef<std::path::Path>>(
        &self,
        input_path: P,
        output_path: P,
    ) -> Result<u64, CompressionError> {
        use std::fs::File;
        use std::io::{BufReader, BufWriter};

        let input_path = input_path.as_ref();
        let output_path = output_path.as_ref();

        let start_time = Instant::now();

        // 파일 열기
        let input_file = File::open(input_path).map_err(|e| {
            CompressionError::DecompressionFailed(format!("압축 파일 열기 실패: {}", e))
        })?;
        let output_file = File::create(output_path).map_err(|e| {
            CompressionError::DecompressionFailed(format!("출력 파일 생성 실패: {}", e))
        })?;

        let reader = BufReader::new(input_file);
        let mut writer = BufWriter::new(output_file);

        // 스트리밍 압축 해제 수행
        let mut decoder = GzDecoder::new(reader);

        // 1MB 버퍼로 스트리밍 압축 해제 (성능 향상)
        const BUFFER_SIZE: usize = 1024 * 1024;
        let mut buffer = vec![0u8; BUFFER_SIZE];
        let mut total_written = 0u64;

        loop {
            let bytes_read = decoder.read(&mut buffer).map_err(|e| {
                CompressionError::DecompressionFailed(format!("압축 해제 읽기 실패: {}", e))
            })?;

            if bytes_read == 0 {
                break; // EOF
            }

            writer.write_all(&buffer[..bytes_read]).map_err(|e| {
                CompressionError::DecompressionFailed(format!("파일 쓰기 실패: {}", e))
            })?;

            total_written += bytes_read as u64;

            // 주기적으로 진행 상황 로그 (100MB마다)
            if total_written % (100 * 1024 * 1024) == 0 {
                log::info!(
                    "스트리밍 압축 해제 진행: {}MB 처리 완료",
                    total_written / (1024 * 1024)
                );
            }
        }

        // 버퍼 플러시
        writer.flush().map_err(|e| {
            CompressionError::DecompressionFailed(format!("파일 쓰기 완료 실패: {}", e))
        })?;

        let decompression_time = start_time.elapsed().as_millis();
        log::info!(
            "스트리밍 압축 해제 완료: {}MB ({}ms)",
            total_written / (1024 * 1024),
            decompression_time
        );

        Ok(total_written)
    }

    /// 파일 데이터를 조건부로 압축합니다.
    /// 설정에 따라 압축 여부를 결정하고, 필요한 경우에만 압축을 수행합니다.
    ///
    /// # 매개변수
    /// * `data` - 원본 파일 데이터
    /// * `file_extension` - 파일 확장자
    ///
    /// # 반환값
    /// * `Result<(Vec<u8>, Option<CompressionResult>), CompressionError>` - 처리된 데이터와 압축 결과 (압축하지 않은 경우 None)
    pub fn compress_file_data(
        &self,
        data: &[u8],
        file_extension: &str,
    ) -> Result<(Vec<u8>, Option<CompressionResult>), CompressionError> {
        let file_size = data.len() as u64;

        // 압축 대상인지 확인
        if !self.should_compress(file_size, file_extension) {
            log::debug!(
                "압축 대상이 아님: 크기={}바이트, 확장자={}",
                file_size,
                file_extension
            );
            return Ok((data.to_vec(), None));
        }

        // 압축 수행
        match self.compress_data(data, None) {
            Ok((compressed_data, result)) => {
                // 압축 효과가 있는 경우에만 압축된 데이터 사용
                if result.space_saved() > 0 {
                    Ok((compressed_data, Some(result)))
                } else {
                    log::debug!("압축 효과가 없어 원본 데이터 사용");
                    Ok((data.to_vec(), None))
                }
            }
            Err(e) => {
                log::warn!("압축 실패, 원본 데이터 사용: {}", e);
                Ok((data.to_vec(), None))
            }
        }
    }

    /// 압축 통계를 계산합니다.
    ///
    /// # 매개변수
    /// * `original_sizes` - 원본 파일 크기들
    /// * `compressed_sizes` - 압축된 파일 크기들
    ///
    /// # 반환값
    /// * `CompressionStats` - 압축 통계
    pub fn calculate_compression_stats(
        &self,
        original_sizes: &[u64],
        compressed_sizes: &[u64],
    ) -> CompressionStats {
        if original_sizes.len() != compressed_sizes.len() {
            return CompressionStats::default();
        }

        let total_original: u64 = original_sizes.iter().sum();
        let total_compressed: u64 = compressed_sizes.iter().sum();
        let total_saved = if total_original > total_compressed {
            total_original - total_compressed
        } else {
            0
        };

        let compression_ratio = if total_original > 0 {
            total_compressed as f64 / total_original as f64
        } else {
            1.0
        };

        let space_saved_percent = if total_original > 0 {
            (total_saved as f64 / total_original as f64) * 100.0
        } else {
            0.0
        };

        CompressionStats {
            total_files: original_sizes.len(),
            total_original_size: total_original,
            total_compressed_size: total_compressed,
            total_space_saved: total_saved,
            average_compression_ratio: compression_ratio,
            space_saved_percent,
        }
    }

    /// 압축 설정을 검증합니다.
    ///
    /// # 매개변수
    /// * `settings` - 검증할 압축 설정
    ///
    /// # 반환값
    /// * `Result<(), CompressionError>` - 검증 결과
    pub fn validate_settings(settings: &CompressionSettings) -> Result<(), CompressionError> {
        // 임계값 검증
        if settings.threshold_bytes > 5 * 1024 * 1024 * 1024 {
            return Err(CompressionError::CompressionFailed(
                "압축 임계값이 너무 큽니다. (최대 5GB)".to_string(),
            ));
        }

        // 제외 확장자 검증
        for ext in &settings.excluded_extensions {
            if ext.is_empty() {
                return Err(CompressionError::CompressionFailed(
                    "빈 확장자는 허용되지 않습니다.".to_string(),
                ));
            }
        }

        Ok(())
    }
}

/// 압축 통계 구조체
#[derive(Debug, Clone, serde::Serialize)]
pub struct CompressionStats {
    /// 총 파일 수
    pub total_files: usize,
    /// 총 원본 크기 (바이트)
    pub total_original_size: u64,
    /// 총 압축된 크기 (바이트)
    pub total_compressed_size: u64,
    /// 총 절약된 공간 (바이트)
    pub total_space_saved: u64,
    /// 평균 압축률
    pub average_compression_ratio: f64,
    /// 절약된 공간 백분율
    pub space_saved_percent: f64,
}

impl Default for CompressionStats {
    fn default() -> Self {
        Self {
            total_files: 0,
            total_original_size: 0,
            total_compressed_size: 0,
            total_space_saved: 0,
            average_compression_ratio: 1.0,
            space_saved_percent: 0.0,
        }
    }
}

impl CompressionStats {
    /// 압축 통계를 포맷된 문자열로 반환합니다.
    ///
    /// # 반환값
    /// * `String` - 포맷된 통계 문자열
    pub fn format_summary(&self) -> String {
        format!(
            "파일 {}개, 원본 {}, 압축 {}, 절약 {} ({:.1}%)",
            self.total_files,
            format_bytes(self.total_original_size),
            format_bytes(self.total_compressed_size),
            format_bytes(self.total_space_saved),
            self.space_saved_percent
        )
    }
}

/// 바이트 크기를 사람이 읽기 쉬운 형태로 포맷합니다.
///
/// # 매개변수
/// * `bytes` - 바이트 크기
///
/// # 반환값
/// * `String` - 포맷된 크기 문자열
fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", bytes, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size, UNITS[unit_index])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compression_service_creation() {
        // 기본 설정으로 서비스 생성 테스트
        let service = CompressionService::new_with_defaults();
        assert!(service.get_settings().enabled);
        assert_eq!(service.get_settings().level, CompressionLevel::Normal);
    }

    #[test]
    fn test_should_compress() {
        let service = CompressionService::new_with_defaults();

        // 크기가 임계값보다 작은 경우
        assert!(!service.should_compress(500, "txt"));

        // 제외된 확장자인 경우
        assert!(!service.should_compress(2000, "jpg"));
        assert!(!service.should_compress(2000, "zip"));

        // 압축 대상인 경우
        assert!(service.should_compress(2000, "txt"));
        assert!(service.should_compress(2000, "doc"));
    }

    #[test]
    fn test_compress_decompress_data() {
        let service = CompressionService::new_with_defaults();
        let original_data = b"Hello, World! This is a test string for compression. ".repeat(100);

        // 압축 테스트
        let (compressed_data, result) = service.compress_data(&original_data, None).unwrap();
        assert!(compressed_data.len() < original_data.len());
        assert!(result.space_saved() > 0);
        assert!(result.space_saved_percent() > 0.0);

        // 압축 해제 테스트
        let decompressed_data = service.decompress_data(&compressed_data).unwrap();
        assert_eq!(original_data, decompressed_data);
    }

    #[test]
    fn test_compress_file_data() {
        let service = CompressionService::new_with_defaults();

        // 압축 대상 파일 테스트
        let text_data = b"This is a large text file content. ".repeat(100);
        let (processed_data, result) = service.compress_file_data(&text_data, "txt").unwrap();
        assert!(result.is_some());
        assert!(processed_data.len() < text_data.len());

        // 압축 제외 파일 테스트
        let image_data = b"fake image data";
        let (processed_data, result) = service.compress_file_data(image_data, "jpg").unwrap();
        assert!(result.is_none());
        assert_eq!(processed_data, image_data);
    }

    #[test]
    fn test_compression_levels() {
        let service = CompressionService::new_with_defaults();
        let data = b"Test data for compression level testing. ".repeat(50);

        // 각 압축 레벨 테스트
        let (fast_data, _fast_result) = service
            .compress_data(&data, Some(CompressionLevel::Fast))
            .unwrap();
        let (normal_data, _normal_result) = service
            .compress_data(&data, Some(CompressionLevel::Normal))
            .unwrap();
        let (max_data, _max_result) = service
            .compress_data(&data, Some(CompressionLevel::Maximum))
            .unwrap();

        // 압축 레벨이 높을수록 더 작아야 함 (일반적으로)
        assert!(max_data.len() <= normal_data.len());
        assert!(normal_data.len() <= fast_data.len());

        // 모든 압축 해제가 원본과 동일해야 함
        assert_eq!(service.decompress_data(&fast_data).unwrap(), data);
        assert_eq!(service.decompress_data(&normal_data).unwrap(), data);
        assert_eq!(service.decompress_data(&max_data).unwrap(), data);
    }

    #[test]
    fn test_compression_stats() {
        let service = CompressionService::new_with_defaults();
        let original_sizes = vec![1000, 2000, 3000];
        let compressed_sizes = vec![600, 1200, 1800];

        let stats = service.calculate_compression_stats(&original_sizes, &compressed_sizes);

        assert_eq!(stats.total_files, 3);
        assert_eq!(stats.total_original_size, 6000);
        assert_eq!(stats.total_compressed_size, 3600);
        assert_eq!(stats.total_space_saved, 2400);
        assert_eq!(stats.average_compression_ratio, 0.6);
        assert_eq!(stats.space_saved_percent, 40.0);
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(512), "512 B");
        assert_eq!(format_bytes(1024), "1.0 KB");
        assert_eq!(format_bytes(1536), "1.5 KB");
        assert_eq!(format_bytes(1048576), "1.0 MB");
        assert_eq!(format_bytes(1073741824), "1.0 GB");
    }

    #[test]
    fn test_validate_settings() {
        // 유효한 설정
        let valid_settings = CompressionSettings::default();
        assert!(CompressionService::validate_settings(&valid_settings).is_ok());

        // 임계값이 너무 큰 설정
        let mut invalid_settings = CompressionSettings::default();
        invalid_settings.threshold_bytes = 200 * 1024 * 1024; // 200MB
        assert!(CompressionService::validate_settings(&invalid_settings).is_err());

        // 빈 확장자가 포함된 설정
        let mut invalid_settings = CompressionSettings::default();
        invalid_settings.excluded_extensions.push("".to_string());
        assert!(CompressionService::validate_settings(&invalid_settings).is_err());
    }

    #[test]
    fn test_empty_data_handling() {
        let service = CompressionService::new_with_defaults();

        // 빈 데이터 압축 시도
        let result = service.compress_data(&[], None);
        assert!(result.is_err());

        // 빈 데이터 압축 해제 시도
        let result = service.decompress_data(&[]);
        assert!(result.is_err());
    }
}
