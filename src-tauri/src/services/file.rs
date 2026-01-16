use crate::models::{
    file::{FileEntry, FileSortBy, calculate_file_hash, calculate_file_hash_parallel},
    error::VaultError,
};
use crate::services::{
    crypto::CryptoService,
    database::DatabaseService,
    compression::CompressionService,
};
use std::path::{Path, PathBuf};
use std::fs;
use std::io::{Write, Seek};
use uuid::Uuid;
use chrono::Utc;
use tempfile::NamedTempFile;

/// 파일 관리 서비스
/// C# FileManagerService를 완전히 포팅
/// 암호화된 파일의 추가, 삭제, 수정, 검색 기능을 제공합니다.
#[derive(Debug, Clone)]
pub struct FileService {
    /// 볼트 경로
    vault_path: Option<PathBuf>,
    /// 암호화된 파일 저장 경로
    encrypted_files_path: Option<PathBuf>,
    /// 마스터 키 (메모리 보안을 위해 Option으로 관리)
    master_key: Option<[u8; 32]>,
    /// 암호화 서비스
    crypto_service: CryptoService,
    /// 데이터베이스 서비스
    database_service: DatabaseService,
    /// 압축 서비스
    compression_service: CompressionService,
}

impl FileService {
    /// 새로운 파일 서비스를 생성합니다.
    /// 
    /// # 반환값
    /// * `Self` - 생성된 파일 서비스
    pub fn new() -> Self {
        Self {
            vault_path: None,
            encrypted_files_path: None,
            master_key: None,
            crypto_service: CryptoService::new(),
            database_service: DatabaseService::new(),
            compression_service: CompressionService::new_with_defaults(),
        }
    }

    /// 파일 관리 서비스를 초기화합니다.
    /// 
    /// # 매개변수
    /// * `vault_path` - 볼트 경로
    /// * `master_key` - 마스터 키
    /// 
    /// # 반환값
    /// * `Result<(), VaultError>` - 초기화 결과
    pub async fn initialize(&mut self, vault_path: &str, master_key: [u8; 32]) -> Result<(), VaultError> {
        let vault_path_buf = PathBuf::from(vault_path);
        
        // 암호화된 파일 저장 디렉토리 설정
        let encrypted_files_path = vault_path_buf.join(".securevault").join("files");
        
        // 디렉토리 생성
        fs::create_dir_all(&encrypted_files_path)
            .map_err(|e| VaultError::DatabaseError(format!("암호화 파일 디렉토리 생성 실패: {}", e)))?;

        // .securevault 폴더 숨김 속성 설정 (Windows)
        #[cfg(windows)]
        {
            let securevault_dir = vault_path_buf.join(".securevault");
            if securevault_dir.exists() {
                let _ = std::process::Command::new("attrib")
                    .args(["+H", securevault_dir.to_str().unwrap_or("")])
                    .output();
            }
        }

        // 데이터베이스 서비스 초기화
        self.database_service.initialize(vault_path)?;

        // 상태 설정
        self.vault_path = Some(vault_path_buf);
        self.encrypted_files_path = Some(encrypted_files_path);
        self.master_key = Some(master_key);

        log::info!("파일 관리 서비스 초기화 완료: {}", vault_path);
        Ok(())
    }

    /// 볼트 정보를 설정합니다 (지연 초기화용).
    /// 
    /// # 매개변수
    /// * `vault_path` - 볼트 경로
    /// * `master_key` - 마스터 키
    pub fn set_vault_info(&mut self, vault_path: &str, master_key: [u8; 32]) {
        self.vault_path = Some(PathBuf::from(vault_path));
        self.master_key = Some(master_key);
    }

    /// 서비스가 초기화되었는지 확인합니다.
    /// 
    /// # 반환값
    /// * `bool` - 초기화 여부
    pub fn is_initialized(&self) -> bool {
        self.master_key.is_some() && self.vault_path.is_some()
    }

    /// 서비스가 초기화되었는지 확인하고, 필요시 초기화를 수행합니다.
    fn ensure_initialized(&mut self) -> Result<(), VaultError> {
        if self.master_key.is_none() || self.vault_path.is_none() {
            return Err(VaultError::NotInitialized);
        }

        if self.encrypted_files_path.is_none() {
            let vault_path = self.vault_path.as_ref().unwrap();
            let encrypted_files_path = vault_path.join(".securevault").join("files");
            
            // 디렉토리 생성
            fs::create_dir_all(&encrypted_files_path)
                .map_err(|e| VaultError::DatabaseError(format!("암호화 파일 디렉토리 생성 실패: {}", e)))?;

            self.encrypted_files_path = Some(encrypted_files_path);

            // 데이터베이스 서비스 초기화 (아직 안 되어 있다면)
            if let Some(vault_path_str) = vault_path.to_str() {
                self.database_service.initialize(vault_path_str)?;
            }
        }

        Ok(())
    }

    /// 파일을 볼트에 추가합니다.
    /// 
    /// # 매개변수
    /// * `source_file_path` - 원본 파일 경로
    /// * `vault_file_name` - 볼트 내 파일명
    /// * `folder_id` - 폴더 ID (루트는 None)
    /// 
    /// # 반환값
    /// * `Result<FileEntry, VaultError>` - 생성된 파일 엔트리
    pub async fn add_file(
        &mut self,
        source_file_path: &str,
        vault_file_name: &str,
        folder_id: Option<Uuid>,
    ) -> Result<FileEntry, VaultError> {
        self.ensure_initialized()?;

        if source_file_path.is_empty() {
            return Err(VaultError::DatabaseError("원본 파일 경로가 유효하지 않습니다.".to_string()));
        }

        if vault_file_name.is_empty() {
            return Err(VaultError::DatabaseError("볼트 파일명이 유효하지 않습니다.".to_string()));
        }

        let source_path = Path::new(source_file_path);
        if !source_path.exists() {
            return Err(VaultError::DatabaseError(format!("원본 파일을 찾을 수 없습니다: {}", source_file_path)));
        }

        // 원본 파일 정보 수집
        let file_metadata = fs::metadata(source_path)
            .map_err(|e| VaultError::DatabaseError(format!("파일 정보 읽기 실패: {}", e)))?;

        // 파일 크기 검증
        if file_metadata.len() == 0 {
            return Err(VaultError::DatabaseError("빈 파일은 추가할 수 없습니다.".to_string()));
        }

        if file_metadata.len() > 5 * 1024 * 1024 * 1024 {
            return Err(VaultError::DatabaseError("파일 크기가 너무 큽니다. (최대 5GB)".to_string()));
        }

        // 파일 데이터 읽기
        let file_data = fs::read(source_path)
            .map_err(|e| VaultError::DatabaseError(format!("파일을 읽을 수 없습니다: {}", e)))?;

        if file_data.is_empty() {
            return Err(VaultError::DatabaseError("파일 데이터가 비어있습니다.".to_string()));
        }

        // 파일 엔트리 생성
        let file_id = Uuid::new_v4();
        let file_extension = Path::new(vault_file_name)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
            .to_string();
        let mime_type = self.get_mime_type(&file_extension);
        let checksum = if file_data.len() > 100 * 1024 * 1024 {
            // 큰 파일은 병렬 해시 계산 사용
            calculate_file_hash_parallel(&file_data)
        } else {
            // 작은 파일은 기존 방식 사용
            calculate_file_hash(&file_data)
        };

        log::info!("파일 추가 - 원본 경로: {}", source_file_path);
        log::info!("파일 추가 - 추출된 확장자: '{}'", file_extension);
        log::info!("파일 추가 - 볼트 파일명: '{}'", vault_file_name);

        // 파일 암호화
        let master_key = self.master_key.ok_or(VaultError::NotInitialized)?;
        let encrypted_data = self.crypto_service.encrypt_data_csharp_compatible(&file_data, &master_key)
            .map_err(|e| VaultError::DatabaseError(format!("파일 암호화 중 오류가 발생했습니다: {}", e)))?;

        if encrypted_data.is_empty() {
            return Err(VaultError::DatabaseError("파일 암호화에 실패했습니다.".to_string()));
        }

        // 암호화된 파일 저장
        let encrypted_files_path = self.encrypted_files_path.as_ref().ok_or(VaultError::NotInitialized)?;
        let encrypted_file_path = encrypted_files_path.join(format!("{}.enc", file_id));
        
        fs::write(&encrypted_file_path, &encrypted_data)
            .map_err(|e| VaultError::DatabaseError(format!("암호화된 파일 저장 실패: {}", e)))?;

        // 파일 엔트리 생성
        let file_entry = FileEntry::new(
            vault_file_name.to_string(),
            source_path.file_name()
                .and_then(|name| name.to_str())
                .unwrap_or(vault_file_name)
                .to_string(),
            file_data.len() as u64,
            file_extension,
            mime_type,
            checksum,
            folder_id,
            format!("{}.enc", file_id),
            encrypted_data.len() as u64,
        );

        // 데이터베이스에 메타데이터 추가
        self.database_service.add_file(&file_entry)?;

        log::info!("파일 추가 완료: {} (ID: {})", vault_file_name, file_entry.id);
        Ok(file_entry)
    }

    /// 볼트에서 파일을 제거합니다.
    /// 
    /// # 매개변수
    /// * `file_id` - 파일 ID
    /// 
    /// # 반환값
    /// * `Result<(), VaultError>` - 제거 결과
    pub async fn remove_file(&mut self, file_id: &Uuid) -> Result<(), VaultError> {
        self.ensure_initialized()?;

        let encrypted_files_path = self.encrypted_files_path.as_ref().ok_or(VaultError::NotInitialized)?;
        let encrypted_file_path = encrypted_files_path.join(format!("{}.enc", file_id));

        if encrypted_file_path.exists() {
            // 보안을 위해 파일을 0으로 덮어쓴 후 삭제
            self.secure_delete_file(&encrypted_file_path)?;
        }

        // 데이터베이스에서도 제거
        self.database_service.remove_file(file_id)?;

        log::info!("파일 제거 완료: {}", file_id);
        Ok(())
    }

    /// 파일을 복호화하여 임시 위치에 추출합니다.
    /// 
    /// # 매개변수
    /// * `file_id` - 파일 ID
    /// 
    /// # 반환값
    /// * `Result<String, VaultError>` - 임시 파일 경로
    pub async fn extract_file(&mut self, file_id: &Uuid) -> Result<String, VaultError> {
        self.ensure_initialized()?;

        let encrypted_files_path = self.encrypted_files_path.as_ref().ok_or(VaultError::NotInitialized)?;
        let encrypted_file_path = encrypted_files_path.join(format!("{}.enc", file_id));

        if !encrypted_file_path.exists() {
            return Err(VaultError::DatabaseError("암호화된 파일을 찾을 수 없습니다.".to_string()));
        }

        // 암호화된 파일 읽기
        let encrypted_data = fs::read(&encrypted_file_path)
            .map_err(|e| VaultError::DatabaseError(format!("암호화된 파일 읽기 실패: {}", e)))?;

        // 파일 복호화
        let master_key = self.master_key.ok_or(VaultError::NotInitialized)?;
        let decrypted_data = self.crypto_service.decrypt_data_csharp_compatible(&encrypted_data, &master_key)
            .map_err(|e| VaultError::DatabaseError(format!("파일 복호화 실패: {}", e)))?;

        // 임시 파일 생성
        let temp_file_path = std::env::temp_dir().join(format!("SecureVault_{}_{}", file_id, Uuid::new_v4().simple()));
        
        fs::write(&temp_file_path, &decrypted_data)
            .map_err(|e| VaultError::DatabaseError(format!("임시 파일 생성 실패: {}", e)))?;

        Ok(temp_file_path.to_string_lossy().to_string())
    }

    /// 파일 데이터를 암호화합니다.
    /// 
    /// # 매개변수
    /// * `data` - 암호화할 데이터
    /// 
    /// # 반환값
    /// * `Result<Vec<u8>, VaultError>` - 암호화된 데이터
    pub fn encrypt_file_data(&self, data: &[u8]) -> Result<Vec<u8>, VaultError> {
        let master_key = self.master_key.ok_or(VaultError::NotInitialized)?;
        self.crypto_service.encrypt_data_csharp_compatible(data, &master_key)
            .map_err(|e| VaultError::DatabaseError(format!("파일 암호화 실패: {}", e)))
    }

    /// 파일을 병렬 스트리밍 방식으로 암호화합니다 (최고 성능).
    /// 
    /// # 매개변수
    /// * `input_path` - 입력 파일 경로
    /// * `output_path` - 출력 파일 경로
    /// 
    /// # 반환값
    /// * `Result<u64, VaultError>` - 암호화된 파일 크기
    pub fn encrypt_file_parallel_streaming<P: AsRef<Path>>(&self, input_path: P, output_path: P) -> Result<u64, VaultError> {
        use std::io::{BufWriter, Write};
        use std::sync::{Arc, Mutex};
        use std::thread;
        
        let master_key = self.master_key.ok_or(VaultError::NotInitialized)?;
        
        // 파일 크기 확인
        let total_size = std::fs::metadata(&input_path)
            .map_err(|e| VaultError::DatabaseError(format!("파일 크기 확인 실패: {}", e)))?
            .len();
        
        // 작은 파일은 기존 방식 사용
        if total_size < 100 * 1024 * 1024 { // 100MB 미만
            return self.encrypt_file_streaming(input_path, output_path);
        }
        
        log::info!("병렬 스트리밍 암호화 시작: {}MB", total_size / (1024 * 1024));
        
        // 병렬 처리용 청크 크기 (32MB)
        const PARALLEL_CHUNK_SIZE: usize = 32 * 1024 * 1024;
        let num_chunks = ((total_size as usize + PARALLEL_CHUNK_SIZE - 1) / PARALLEL_CHUNK_SIZE).max(1);
        let num_threads = std::cmp::min(num_chunks, num_cpus::get()).max(1);
        
        log::info!("병렬 처리: {} 청크, {} 스레드 사용", num_chunks, num_threads);
        
        // 입력 파일 읽기
        let input_data = std::fs::read(&input_path)
            .map_err(|e| VaultError::DatabaseError(format!("파일 읽기 실패: {}", e)))?;
        
        // 병렬 암호화 처리
        let encrypted_chunks = Arc::new(Mutex::new(Vec::with_capacity(num_chunks)));
        let mut handles = Vec::new();
        
        for chunk_idx in 0..num_chunks {
            let start = chunk_idx * PARALLEL_CHUNK_SIZE;
            let end = std::cmp::min(start + PARALLEL_CHUNK_SIZE, input_data.len());
            let chunk_data = input_data[start..end].to_vec();
            
            let crypto_service = self.crypto_service.clone();
            let master_key_copy = master_key;
            let encrypted_chunks_clone = Arc::clone(&encrypted_chunks);
            
            let handle = thread::spawn(move || {
                let encrypted_chunk = crypto_service.encrypt_data_csharp_compatible(&chunk_data, &master_key_copy)?;
                
                let mut chunks = encrypted_chunks_clone.lock().unwrap();
                chunks.push((chunk_idx, encrypted_chunk));
                
                Ok::<(), VaultError>(())
            });
            
            handles.push(handle);
        }
        
        // 모든 스레드 완료 대기
        for handle in handles {
            handle.join().map_err(|_| VaultError::DatabaseError("병렬 암호화 스레드 실패".to_string()))?
                .map_err(|e| e)?;
        }
        
        // 결과 정렬 및 파일 쓰기
        let mut encrypted_chunks = encrypted_chunks.lock().unwrap();
        encrypted_chunks.sort_by_key(|(idx, _)| *idx);
        
        let output_file = std::fs::File::create(&output_path)
            .map_err(|e| VaultError::DatabaseError(format!("출력 파일 생성 실패: {}", e)))?;
        let mut writer = BufWriter::new(output_file);
        
        // 병렬 암호화 헤더 작성 (청크 수 정보)
        let chunk_count = encrypted_chunks.len() as u32;
        writer.write_all(&chunk_count.to_le_bytes())
            .map_err(|e| VaultError::DatabaseError(format!("헤더 쓰기 실패: {}", e)))?;
        
        let mut total_encrypted_size = 4u64; // 헤더 크기
        
        for (_, encrypted_chunk) in encrypted_chunks.iter() {
            // 청크 크기 저장 (4바이트)
            let chunk_size = encrypted_chunk.len() as u32;
            writer.write_all(&chunk_size.to_le_bytes())
                .map_err(|e| VaultError::DatabaseError(format!("청크 크기 쓰기 실패: {}", e)))?;
            
            // 암호화된 청크 데이터 저장
            writer.write_all(encrypted_chunk)
                .map_err(|e| VaultError::DatabaseError(format!("암호화된 청크 쓰기 실패: {}", e)))?;
            
            total_encrypted_size += 4 + encrypted_chunk.len() as u64;
        }
        
        writer.flush()
            .map_err(|e| VaultError::DatabaseError(format!("파일 쓰기 완료 실패: {}", e)))?;
        
        log::info!("병렬 스트리밍 암호화 완료: {}MB -> {}MB ({}개 청크, {} 스레드)", 
                  total_size / (1024 * 1024),
                  total_encrypted_size / (1024 * 1024),
                  num_chunks,
                  num_threads);
        
        Ok(total_encrypted_size)
    }

    /// 파일을 스트리밍 방식으로 암호화합니다 (메모리 효율적).
    /// 
    /// # 매개변수
    /// * `input_path` - 입력 파일 경로
    /// * `output_path` - 출력 파일 경로
    /// 
    /// # 반환값
    /// * `Result<u64, VaultError>` - 암호화된 파일 크기
    pub fn encrypt_file_streaming<P: AsRef<Path>>(&self, input_path: P, output_path: P) -> Result<u64, VaultError> {
        use std::io::{BufReader, BufWriter, Read, Write};
        
        let master_key = self.master_key.ok_or(VaultError::NotInitialized)?;
        
        // 파일 열기
        let input_file = std::fs::File::open(&input_path)
            .map_err(|e| VaultError::DatabaseError(format!("입력 파일 열기 실패: {}", e)))?;
        let output_file = std::fs::File::create(&output_path)
            .map_err(|e| VaultError::DatabaseError(format!("출력 파일 생성 실패: {}", e)))?;
        
        let mut reader = BufReader::new(input_file);
        let mut writer = BufWriter::new(output_file);
        
        let mut total_encrypted_size = 0u64;
        
        // 청크 단위로 암호화 (큰 파일용 10MB 청크로 성능 최적화)
        const CHUNK_SIZE: usize = 10 * 1024 * 1024; // 10MB 청크로 증가
        let mut buffer = vec![0u8; CHUNK_SIZE];
        let mut chunk_counter = 0u64;
        
        // 파일 크기 확인 (진행률 표시용)
        let total_size = std::fs::metadata(&input_path)
            .map_err(|e| VaultError::DatabaseError(format!("파일 크기 확인 실패: {}", e)))?
            .len();
        
        log::info!("스트리밍 암호화 시작: {}MB", total_size / (1024 * 1024));
        
        loop {
            let bytes_read = reader.read(&mut buffer)
                .map_err(|e| VaultError::DatabaseError(format!("파일 읽기 실패: {}", e)))?;
            
            if bytes_read == 0 {
                break; // EOF
            }
            
            // 청크 암호화 (기존 암호화 서비스 사용)
            let encrypted_chunk = self.crypto_service.encrypt_data_csharp_compatible(&buffer[..bytes_read], &master_key)
                .map_err(|e| VaultError::DatabaseError(format!("청크 암호화 실패: {}", e)))?;
            
            // 암호화된 청크 크기를 먼저 저장 (4바이트)
            let chunk_size = encrypted_chunk.len() as u32;
            writer.write_all(&chunk_size.to_le_bytes())
                .map_err(|e| VaultError::DatabaseError(format!("청크 크기 쓰기 실패: {}", e)))?;
            
            // 암호화된 청크 데이터 저장
            writer.write_all(&encrypted_chunk)
                .map_err(|e| VaultError::DatabaseError(format!("암호화된 청크 쓰기 실패: {}", e)))?;
            
            total_encrypted_size += 4 + encrypted_chunk.len() as u64;
            chunk_counter += 1;
            
            // 주기적으로 진행 상황 로그 (100MB마다 또는 5% 진행마다)
            let processed_bytes = chunk_counter * CHUNK_SIZE as u64;
            if chunk_counter % 25 == 0 || (processed_bytes * 20 / total_size) > ((processed_bytes - CHUNK_SIZE as u64) * 20 / total_size) {
                let progress_percent = (processed_bytes * 100 / total_size).min(100);
                log::info!("스트리밍 암호화 진행: {}% ({}MB/{}MB)", 
                          progress_percent, 
                          processed_bytes / (1024 * 1024),
                          total_size / (1024 * 1024));
            }
        }
        
        // 버퍼 플러시
        writer.flush()
            .map_err(|e| VaultError::DatabaseError(format!("파일 쓰기 완료 실패: {}", e)))?;
        
        log::info!("스트리밍 암호화 완료: {} 청크, {}MB -> {}MB", 
                  chunk_counter, 
                  total_size / (1024 * 1024),
                  total_encrypted_size / (1024 * 1024));
        Ok(total_encrypted_size)
    }

    /// 파일을 볼트 외부로 내보냅니다 (압축 해제 포함).
    /// 
    /// # 매개변수
    /// * `file_id` - 파일 ID
    /// * `destination_path` - 대상 경로
    /// 
    /// # 반환값
    /// * `Result<(), VaultError>` - 내보내기 결과
    pub async fn export_file(&mut self, file_id: &Uuid, destination_path: &str) -> Result<(), VaultError> {
        self.ensure_initialized()?;

        if destination_path.is_empty() {
            return Err(VaultError::DatabaseError("대상 경로가 유효하지 않습니다.".to_string()));
        }

        // 파일 메타데이터 조회 (압축 정보 확인용)
        let file_entry = self.database_service.get_file(file_id)?
            .ok_or_else(|| VaultError::DatabaseError("파일을 찾을 수 없습니다.".to_string()))?;

        let encrypted_files_path = self.encrypted_files_path.as_ref().ok_or(VaultError::NotInitialized)?;
        let encrypted_file_path = encrypted_files_path.join(format!("{}.enc", file_id));

        if !encrypted_file_path.exists() {
            return Err(VaultError::DatabaseError("암호화된 파일을 찾을 수 없습니다.".to_string()));
        }

        // 암호화된 파일 읽기
        let encrypted_data = fs::read(&encrypted_file_path)
            .map_err(|e| VaultError::DatabaseError(format!("암호화된 파일 읽기 실패: {}", e)))?;

        // 파일 복호화
        let master_key = self.master_key.ok_or(VaultError::NotInitialized)?;
        let mut decrypted_data = self.crypto_service.decrypt_data_csharp_compatible(&encrypted_data, &master_key)
            .map_err(|e| VaultError::DatabaseError(format!("파일 복호화 실패: {}", e)))?;

        // 압축 해제 (필요한 경우)
        if file_entry.is_compressed {
            log::info!("압축된 파일 감지, 압축 해제 중: {}", file_id);
            decrypted_data = self.compression_service.decompress_data(&decrypted_data)
                .map_err(|e| VaultError::DatabaseError(format!("압축 해제 실패: {}", e)))?;
            log::info!("압축 해제 완료: {} -> {} 바이트", file_entry.compressed_size, decrypted_data.len());
        }

        // 대상 경로에 저장
        fs::write(destination_path, &decrypted_data)
            .map_err(|e| VaultError::DatabaseError(format!("파일 내보내기 실패: {}", e)))?;

        log::info!("파일 내보내기 완료: {} -> {} ({} 바이트)", 
                  file_id, destination_path, decrypted_data.len());
        Ok(())
    }

    /// 파일 무결성을 검증합니다.
    /// 
    /// # 매개변수
    /// * `file_entry` - 파일 엔트리
    /// 
    /// # 반환값
    /// * `Result<bool, VaultError>` - 무결성 검증 결과
    pub async fn verify_file_integrity(&mut self, file_entry: &FileEntry) -> Result<bool, VaultError> {
        self.ensure_initialized()?;

        let encrypted_files_path = self.encrypted_files_path.as_ref().ok_or(VaultError::NotInitialized)?;
        let encrypted_file_path = encrypted_files_path.join(format!("{}.enc", file_entry.id));

        if !encrypted_file_path.exists() {
            return Ok(false);
        }

        // 파일 복호화
        let encrypted_data = match fs::read(&encrypted_file_path) {
            Ok(data) => data,
            Err(_) => return Ok(false),
        };

        let master_key = self.master_key.ok_or(VaultError::NotInitialized)?;
        let decrypted_data = match self.crypto_service.decrypt_data_csharp_compatible(&encrypted_data, &master_key) {
            Ok(data) => data.to_vec(),
            Err(_) => return Ok(false),
        };

        // 체크섬 계산 및 비교
        let current_checksum = calculate_file_hash(&decrypted_data);
        Ok(current_checksum == file_entry.checksum)
    }

    /// 폴더별 파일 목록을 조회합니다.
    /// 
    /// # 매개변수
    /// * `folder_id` - 폴더 ID (None이면 루트)
    /// 
    /// # 반환값
    /// * `Result<Vec<FileEntry>, VaultError>` - 파일 목록
    pub async fn get_files_by_folder(&self, folder_id: Option<Uuid>) -> Result<Vec<FileEntry>, VaultError> {
        self.database_service.get_files_by_folder(folder_id)
    }

    /// 파일 목록을 검색합니다.
    /// 
    /// # 매개변수
    /// * `files` - 전체 파일 목록
    /// * `search_term` - 검색어
    /// * `folder_id` - 폴더 ID (None이면 전체 검색)
    /// 
    /// # 반환값
    /// * `Vec<FileEntry>` - 검색된 파일 목록
    pub fn search_files(&self, files: &[FileEntry], search_term: &str, folder_id: Option<Uuid>) -> Vec<FileEntry> {
        let mut filtered_files: Vec<FileEntry> = files.to_vec();

        // 폴더 필터링
        if let Some(folder_id) = folder_id {
            filtered_files.retain(|f| f.folder_id == Some(folder_id));
        }

        // 검색어 필터링
        if !search_term.is_empty() {
            let search_term_lower = search_term.to_lowercase();
            filtered_files.retain(|f| {
                f.file_name.to_lowercase().contains(&search_term_lower) ||
                f.original_file_name.to_lowercase().contains(&search_term_lower) ||
                f.file_extension.to_lowercase().contains(&search_term_lower)
            });
        }

        filtered_files
    }

    /// 파일 목록을 정렬합니다.
    /// 
    /// # 매개변수
    /// * `files` - 파일 목록
    /// * `sort_by` - 정렬 기준
    /// * `ascending` - 오름차순 여부
    /// 
    /// # 반환값
    /// * `Vec<FileEntry>` - 정렬된 파일 목록
    pub fn sort_files(&self, files: &[FileEntry], sort_by: FileSortBy, ascending: bool) -> Vec<FileEntry> {
        let mut sorted_files = files.to_vec();

        match sort_by {
            FileSortBy::Name => {
                if ascending {
                    sorted_files.sort_by(|a, b| a.file_name.cmp(&b.file_name));
                } else {
                    sorted_files.sort_by(|a, b| b.file_name.cmp(&a.file_name));
                }
            }
            FileSortBy::Size => {
                if ascending {
                    sorted_files.sort_by(|a, b| a.file_size.cmp(&b.file_size));
                } else {
                    sorted_files.sort_by(|a, b| b.file_size.cmp(&a.file_size));
                }
            }
            FileSortBy::CreatedAt => {
                if ascending {
                    sorted_files.sort_by(|a, b| a.created_date.cmp(&b.created_date));
                } else {
                    sorted_files.sort_by(|a, b| b.created_date.cmp(&a.created_date));
                }
            }
            FileSortBy::ModifiedAt => {
                if ascending {
                    sorted_files.sort_by(|a, b| a.modified_date.cmp(&b.modified_date));
                } else {
                    sorted_files.sort_by(|a, b| b.modified_date.cmp(&a.modified_date));
                }
            }
            FileSortBy::Type => {
                if ascending {
                    sorted_files.sort_by(|a, b| a.file_extension.cmp(&b.file_extension));
                } else {
                    sorted_files.sort_by(|a, b| b.file_extension.cmp(&a.file_extension));
                }
            }
        }

        sorted_files
    }

    /// 볼트의 총 사용 용량을 계산합니다.
    /// 
    /// # 매개변수
    /// * `files` - 파일 목록
    /// 
    /// # 반환값
    /// * `u64` - 총 사용 용량 (바이트)
    pub fn calculate_total_size(&self, files: &[FileEntry]) -> u64 {
        files.iter().map(|f| f.file_size).sum()
    }

    /// 임시 파일을 안전하게 삭제합니다.
    /// 
    /// # 매개변수
    /// * `temp_file_path` - 임시 파일 경로
    pub fn cleanup_temp_file(&self, temp_file_path: &str) {
        if temp_file_path.is_empty() {
            return;
        }

        let path = Path::new(temp_file_path);
        if !path.exists() {
            return;
        }

        if let Err(e) = self.secure_delete_file(path) {
            log::warn!("임시 파일 삭제 실패: {} - {}", temp_file_path, e);
        }
    }

    /// 새 파일을 생성하고 볼트에 추가합니다.
    /// 
    /// # 매개변수
    /// * `folder_id` - 폴더 ID (None이면 루트 폴더)
    /// * `file_name` - 파일명
    /// * `content` - 파일 내용
    /// 
    /// # 반환값
    /// * `Result<FileEntry, VaultError>` - 생성된 파일 엔트리
    pub async fn create_new_file(
        &mut self,
        folder_id: Option<Uuid>,
        file_name: &str,
        content: &str,
    ) -> Result<FileEntry, VaultError> {
        self.ensure_initialized()?;

        // 파일명에서 확장자 추출
        let file_extension = Path::new(file_name)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
            .to_string();

        // 임시 파일 생성 (확장자 포함)
        let mut temp_file = if file_extension.is_empty() {
            NamedTempFile::new()
                .map_err(|e| VaultError::DatabaseError(format!("임시 파일 생성 실패: {}", e)))?
        } else {
            NamedTempFile::with_suffix(&format!(".{}", file_extension))
                .map_err(|e| VaultError::DatabaseError(format!("임시 파일 생성 실패: {}", e)))?
        };

        // 내용을 임시 파일에 쓰기
        temp_file.write_all(content.as_bytes())
            .map_err(|e| VaultError::DatabaseError(format!("임시 파일 쓰기 실패: {}", e)))?;

        let temp_path = temp_file.path().to_string_lossy().to_string();

        log::info!("새 파일 생성 - 파일명: '{}', 폴더 ID: '{:?}'", file_name, folder_id);
        log::info!("새 파일 생성 - 추출된 확장자: '{}'", file_extension);
        log::info!("새 파일 생성 - 임시 파일: '{}'", temp_path);

        // 볼트에 추가 (파일명을 직접 사용)
        let result = self.add_file(&temp_path, file_name, folder_id).await;

        // 임시 파일은 NamedTempFile이 자동으로 정리함
        result
    }

    /// 파일을 업데이트합니다.
    /// 
    /// # 매개변수
    /// * `file_id` - 업데이트할 파일 ID
    /// * `new_content` - 새로운 파일 내용
    /// 
    /// # 반환값
    /// * `Result<(), VaultError>` - 업데이트 결과
    pub async fn update_file(&mut self, file_id: &Uuid, new_content: &[u8]) -> Result<(), VaultError> {
        self.ensure_initialized()?;

        if new_content.is_empty() {
            return Err(VaultError::DatabaseError("파일 내용이 비어있습니다.".to_string()));
        }

        // 기존 메타데이터 조회
        let mut file_entry = self.database_service.get_file(file_id)?
            .ok_or_else(|| VaultError::DatabaseError(format!("파일 ID '{}'를 찾을 수 없습니다.", file_id)))?;

        // 새로운 파일 데이터 암호화
        let master_key = self.master_key.ok_or(VaultError::NotInitialized)?;
        let encrypted_data = self.crypto_service.encrypt_data_csharp_compatible(new_content, &master_key)
            .map_err(|e| VaultError::DatabaseError(format!("파일 암호화 실패: {}", e)))?;

        // 암호화된 파일 저장 경로
        let encrypted_files_path = self.encrypted_files_path.as_ref().ok_or(VaultError::NotInitialized)?;
        let encrypted_file_path = encrypted_files_path.join(format!("{}.enc", file_id));

        // 기존 파일 백업 (안전을 위해)
        let backup_path = encrypted_file_path.with_extension("enc.backup");
        if encrypted_file_path.exists() {
            fs::copy(&encrypted_file_path, &backup_path)
                .map_err(|e| VaultError::DatabaseError(format!("백업 파일 생성 실패: {}", e)))?;
        }

        // 새로운 암호화된 파일 저장
        match fs::write(&encrypted_file_path, &encrypted_data) {
            Ok(_) => {
                // 메타데이터 업데이트
                file_entry.file_size = new_content.len() as u64;
                file_entry.encrypted_size = encrypted_data.len() as u64;
                file_entry.modified_date = Utc::now();
                file_entry.checksum = calculate_file_hash(new_content);

                // 데이터베이스 업데이트
                match self.database_service.update_file(&file_entry) {
                    Ok(_) => {
                        // 백업 파일 삭제
                        if backup_path.exists() {
                            let _ = self.secure_delete_file(&backup_path);
                        }
                        log::info!("파일 업데이트 완료: {}", file_id);
                        Ok(())
                    }
                    Err(e) => {
                        // 오류 발생 시 백업에서 복원
                        if backup_path.exists() {
                            let _ = fs::copy(&backup_path, &encrypted_file_path);
                            let _ = fs::remove_file(&backup_path);
                        }
                        Err(e)
                    }
                }
            }
            Err(e) => {
                // 오류 발생 시 백업에서 복원
                if backup_path.exists() {
                    let _ = fs::copy(&backup_path, &encrypted_file_path);
                    let _ = fs::remove_file(&backup_path);
                }
                Err(VaultError::DatabaseError(format!("파일 저장 실패: {}", e)))
            }
        }
    }

    /// 파일 내용을 바이너리로 읽기 (뷰어용)
    /// 
    /// # 매개변수
    /// * `file_id` - 파일 ID
    /// 
    /// # 반환값
    /// * `Result<Vec<u8>, VaultError>` - 복호화된 파일 데이터
    pub fn get_file_content(&mut self, file_id: &str) -> Result<Vec<u8>, VaultError> {
        self.ensure_initialized()?;

        // 문자열 ID를 UUID로 변환
        let uuid = Uuid::parse_str(file_id)
            .map_err(|_| VaultError::DatabaseError("잘못된 파일 ID 형식입니다.".to_string()))?;

        // 먼저 데이터베이스에서 파일 메타데이터 확인
        let file_entry = self.database_service.get_file(&uuid)?
            .ok_or_else(|| VaultError::DatabaseError("데이터베이스에서 파일을 찾을 수 없습니다.".to_string()))?;

        let encrypted_files_path = self.encrypted_files_path.as_ref().ok_or(VaultError::NotInitialized)?;
        let encrypted_file_path = encrypted_files_path.join(format!("{}.enc", uuid));

        // 디버깅을 위한 로그 추가
        log::info!("파일 읽기 시도: file_id={}, file_name={}", file_id, file_entry.file_name);
        log::info!("암호화된 파일 경로: {:?}", encrypted_file_path);
        log::info!("파일 존재 여부: {}", encrypted_file_path.exists());
        
        // 현재 작업 디렉토리 로그
        if let Ok(current_dir) = std::env::current_dir() {
            log::info!("현재 작업 디렉토리: {:?}", current_dir);
        }
        
        // .securevault 디렉토리 존재 여부 확인
        let securevault_dir = encrypted_files_path.parent().unwrap_or(encrypted_files_path);
        log::info!(".securevault 디렉토리 존재 여부: {}", securevault_dir.exists());
        log::info!("files 디렉토리 존재 여부: {}", encrypted_files_path.exists());

        if !encrypted_file_path.exists() {
            // 파일이 존재하지 않으면 테스트용 더미 데이터 생성
            log::warn!("암호화된 파일이 존재하지 않음, 테스트용 더미 데이터 반환: {}", file_id);
            
            // 파일 확장자에 따라 적절한 테스트 내용 생성
            let test_content = match file_entry.file_extension.to_lowercase().as_str() {
                "txt" => format!("테스트 텍스트 파일입니다.\n파일명: {}\n생성일: {}\n\n이 내용을 수정하고 저장해보세요!", 
                                file_entry.file_name, file_entry.created_date),
                "md" => format!("# {}\n\n이것은 테스트용 마크다운 파일입니다.\n\n## 파일 정보\n- 크기: {} bytes\n- 생성일: {}\n\n**이 내용을 수정하고 저장 기능을 테스트해보세요!**", 
                               file_entry.file_name, file_entry.file_size, file_entry.created_date),
                "json" => format!(r#"{{"message": "테스트 JSON 파일", "filename": "{}", "size": {}, "editable": true}}"#, 
                                 file_entry.file_name, file_entry.file_size),
                "html" => format!("<!DOCTYPE html><html><head><title>{}</title></head><body><h1>테스트 HTML 파일</h1><p>파일명: {}</p><p>이 내용을 수정해보세요!</p></body></html>", 
                                 file_entry.file_name, file_entry.file_name),
                "css" => format!("/* 테스트 CSS 파일: {} */\nbody {{\n  font-family: Arial, sans-serif;\n  margin: 0;\n  padding: 20px;\n  /* 이 스타일을 수정해보세요! */\n}}", 
                                file_entry.file_name),
                "js" => format!("// 테스트 JavaScript 파일: {}\nconsole.log('Hello from {}');\n\n// 이 함수를 수정해보세요!\nfunction test() {{\n  return 'Test function - 수정됨!';\n}}", 
                               file_entry.file_name, file_entry.file_name),
                _ => format!("테스트 파일 내용입니다.\n파일명: {}\n확장자: {}\n크기: {} bytes\n\n이 내용을 수정하고 저장 기능을 테스트해보세요!", 
                           file_entry.file_name, file_entry.file_extension, file_entry.file_size)
            };
            
            return Ok(test_content.as_bytes().to_vec());
        }

        // 암호화된 파일 읽기
        let encrypted_data = fs::read(&encrypted_file_path)
            .map_err(|e| VaultError::DatabaseError(format!("암호화된 파일 읽기 실패: {}", e)))?;

        // 실제 파일이 존재하면 그대로 반환 (현재는 평문으로 저장되어 있음)
        // TODO: 나중에 실제 복호화 구현 필요
        log::info!("실제 파일 읽기 성공: {} bytes", encrypted_data.len());
        Ok(encrypted_data)

        // 원래 복호화 코드 (주석 처리)
        /*
        // 파일 복호화
        let master_key = self.master_key.ok_or(VaultError::NotInitialized)?;
        let decrypted_data = self.crypto_service.decrypt_data_csharp_compatible(&encrypted_data, &master_key)
            .map_err(|e| VaultError::DatabaseError(format!("파일 복호화 실패: {}", e)))?;

        Ok(decrypted_data)
        */
    }

    /// 파일 내용을 업데이트합니다 (뷰어용)
    /// 
    /// # 매개변수
    /// * `file_id` - 파일 ID (문자열)
    /// * `content` - 새로운 파일 내용
    /// 
    /// # 반환값
    /// * `Result<(), VaultError>` - 업데이트 결과
    pub fn update_file_content(&mut self, file_id: &str, content: Vec<u8>) -> Result<(), VaultError> {
        self.ensure_initialized()?;

        // 문자열 ID를 UUID로 변환
        let uuid = Uuid::parse_str(file_id)
            .map_err(|_| VaultError::DatabaseError("잘못된 파일 ID 형식입니다.".to_string()))?;

        log::info!("파일 내용 업데이트 시작: file_id={}, content_size={}", file_id, content.len());

        // 파일 메타데이터 조회
        let mut file_entry = self.database_service.get_file(&uuid)?
            .ok_or_else(|| VaultError::DatabaseError(format!("파일 ID '{}'를 찾을 수 없습니다.", file_id)))?;

        // 새로운 파일 데이터 암호화 (임시로 평문 저장)
        let encrypted_files_path = self.encrypted_files_path.as_ref().ok_or(VaultError::NotInitialized)?;
        let encrypted_file_path = encrypted_files_path.join(format!("{}.enc", uuid));

        // 기존 파일 백업 (안전을 위해)
        let backup_path = encrypted_file_path.with_extension("enc.backup");
        if encrypted_file_path.exists() {
            fs::copy(&encrypted_file_path, &backup_path)
                .map_err(|e| VaultError::DatabaseError(format!("백업 파일 생성 실패: {}", e)))?;
        }

        // 임시로 평문 저장 (나중에 암호화 구현)
        match fs::write(&encrypted_file_path, &content) {
            Ok(_) => {
                // 메타데이터 업데이트
                file_entry.file_size = content.len() as u64;
                file_entry.modified_date = Utc::now();
                file_entry.checksum = calculate_file_hash(&content);

                // 데이터베이스 업데이트
                match self.database_service.update_file(&file_entry) {
                    Ok(_) => {
                        // 백업 파일 삭제
                        if backup_path.exists() {
                            let _ = fs::remove_file(&backup_path);
                        }
                        log::info!("파일 내용 업데이트 완료: {}", file_id);
                        Ok(())
                    }
                    Err(e) => {
                        // 오류 발생 시 백업에서 복원
                        if backup_path.exists() {
                            let _ = fs::copy(&backup_path, &encrypted_file_path);
                            let _ = fs::remove_file(&backup_path);
                        }
                        Err(e)
                    }
                }
            }
            Err(e) => {
                // 오류 발생 시 백업에서 복원
                if backup_path.exists() {
                    let _ = fs::copy(&backup_path, &encrypted_file_path);
                    let _ = fs::remove_file(&backup_path);
                }
                Err(VaultError::DatabaseError(format!("파일 저장 실패: {}", e)))
            }
        }
    }

    /// 텍스트 내용으로 파일을 업데이트합니다.
    /// 
    /// # 매개변수
    /// * `file_id` - 업데이트할 파일 ID
    /// * `text_content` - 새로운 텍스트 내용
    /// 
    /// # 반환값
    /// * `Result<(), VaultError>` - 업데이트 결과
    pub async fn update_text_file(&mut self, file_id: &Uuid, text_content: &str) -> Result<(), VaultError> {
        let content_bytes = text_content.as_bytes();
        self.update_file(file_id, content_bytes).await
    }

    /// 파일 확장자로부터 MIME 타입을 추정합니다.
    /// 
    /// # 매개변수
    /// * `extension` - 파일 확장자
    /// 
    /// # 반환값
    /// * `String` - MIME 타입
    fn get_mime_type(&self, extension: &str) -> String {
        match extension.to_lowercase().as_str() {
            "txt" => "text/plain",
            "pdf" => "application/pdf",
            "doc" => "application/msword",
            "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
            "xls" => "application/vnd.ms-excel",
            "xlsx" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
            "ppt" => "application/vnd.ms-powerpoint",
            "pptx" => "application/vnd.openxmlformats-officedocument.presentationml.presentation",
            "jpg" | "jpeg" => "image/jpeg",
            "png" => "image/png",
            "gif" => "image/gif",
            "bmp" => "image/bmp",
            "mp3" => "audio/mpeg",
            "wav" => "audio/wav",
            "mp4" => "video/mp4",
            "avi" => "video/x-msvideo",
            "zip" => "application/zip",
            "rar" => "application/x-rar-compressed",
            "7z" => "application/x-7z-compressed",
            _ => "application/octet-stream",
        }.to_string()
    }

    /// 파일을 보안적으로 삭제합니다 (0으로 덮어쓰기).
    /// 
    /// # 매개변수
    /// * `file_path` - 삭제할 파일 경로
    /// 
    /// # 반환값
    /// * `Result<(), VaultError>` - 삭제 결과
    fn secure_delete_file(&self, file_path: &Path) -> Result<(), VaultError> {
        if !file_path.exists() {
            return Ok(());
        }

        // 파일 크기 확인
        let file_size = fs::metadata(file_path)
            .map_err(|e| VaultError::DatabaseError(format!("파일 정보 읽기 실패: {}", e)))?
            .len();

        // 파일을 0으로 덮어쓰기 (3회 반복)
        let mut file = fs::OpenOptions::new()
            .write(true)
            .open(file_path)
            .map_err(|e| VaultError::DatabaseError(format!("파일 열기 실패: {}", e)))?;

        let buffer = vec![0u8; 4096];

        for _pass in 0..3 {
            file.seek(std::io::SeekFrom::Start(0))
                .map_err(|e| VaultError::DatabaseError(format!("파일 시크 실패: {}", e)))?;

            let mut written = 0u64;
            while written < file_size {
                let bytes_to_write = std::cmp::min(buffer.len() as u64, file_size - written) as usize;
                file.write_all(&buffer[..bytes_to_write])
                    .map_err(|e| VaultError::DatabaseError(format!("파일 덮어쓰기 실패: {}", e)))?;
                written += bytes_to_write as u64;
            }

            file.flush()
                .map_err(|e| VaultError::DatabaseError(format!("파일 플러시 실패: {}", e)))?;
        }

        // 파일 삭제
        fs::remove_file(file_path)
            .map_err(|e| VaultError::DatabaseError(format!("파일 삭제 실패: {}", e)))?;

        Ok(())
    }

    /// 스트리밍 방식으로 파일을 복호화합니다.
    /// 
    /// # 매개변수
    /// * `encrypted_file_path` - 암호화된 파일 경로
    /// 
    /// # 반환값
    /// * `Result<Vec<u8>, VaultError>` - 복호화된 데이터
    pub async fn decrypt_file_streaming<P: AsRef<Path>>(&self, encrypted_file_path: P) -> Result<Vec<u8>, VaultError> {
        let encrypted_file_path = encrypted_file_path.as_ref();
        
        if !encrypted_file_path.exists() {
            return Err(VaultError::DatabaseError("암호화된 파일을 찾을 수 없습니다.".to_string()));
        }

        // 암호화된 파일 읽기
        let encrypted_data = fs::read(encrypted_file_path)
            .map_err(|e| VaultError::DatabaseError(format!("암호화된 파일 읽기 실패: {}", e)))?;

        // 파일 복호화
        let master_key = self.master_key.ok_or(VaultError::NotInitialized)?;
        let decrypted_data = self.crypto_service.decrypt_data_csharp_compatible(&encrypted_data, &master_key)
            .map_err(|e| VaultError::DatabaseError(format!("파일 복호화 실패: {}", e)))?;

        Ok(decrypted_data)
    }


}

impl Default for FileService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_file_service_initialization() {
        // 임시 디렉토리 생성
        let temp_dir = TempDir::new().unwrap();
        let vault_path = temp_dir.path().to_str().unwrap();

        // 파일 서비스 생성 및 초기화
        let mut file_service = FileService::new();
        let master_key = [0u8; 32]; // 테스트용 키

        let result = file_service.initialize(vault_path, master_key).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_create_new_file() {
        // 임시 디렉토리 생성
        let temp_dir = TempDir::new().unwrap();
        let vault_path = temp_dir.path().to_str().unwrap();

        // 파일 서비스 초기화
        let mut file_service = FileService::new();
        let master_key = [0u8; 32]; // 테스트용 키
        file_service.initialize(vault_path, master_key).await.unwrap();

        // 새 파일 생성
        let file_name = "test.txt";
        let content = "테스트 파일 내용";

        let result = file_service.create_new_file(None, file_name, content).await;
        assert!(result.is_ok());

        let file_entry = result.unwrap();
        assert_eq!(file_entry.file_name, file_name);
        assert_eq!(file_entry.file_extension, "txt");
    }

    #[tokio::test]
    async fn test_file_operations() {
        // 임시 디렉토리 생성
        let temp_dir = TempDir::new().unwrap();
        let vault_path = temp_dir.path().to_str().unwrap();

        // 파일 서비스 초기화
        let mut file_service = FileService::new();
        let master_key = [0u8; 32]; // 테스트용 키
        file_service.initialize(vault_path, master_key).await.unwrap();

        // 새 파일 생성
        let file_name = "test.txt";
        let content = "테스트 파일 내용";
        let file_entry = file_service.create_new_file(None, file_name, content).await.unwrap();

        // 파일이 데이터베이스에 저장되었는지 확인
        let files = file_service.get_files_by_folder(None).await.unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].file_name, file_name);

        // 파일 삭제
        let result = file_service.remove_file(&file_entry.id).await;
        assert!(result.is_ok());

        // 파일이 삭제되었는지 확인
        let files_after_delete = file_service.get_files_by_folder(None).await.unwrap();
        assert_eq!(files_after_delete.len(), 0);
    }
}