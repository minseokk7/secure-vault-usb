use crate::models::{
    file::FileEntry,
    folder::FolderEntry,
    error::VaultError,
};
use rusqlite::{Connection, Result as SqliteResult, params, Row};
use std::path::Path;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde_json;

/// 데이터베이스 서비스
/// SQLite 기반 메타데이터 데이터베이스를 관리합니다.
/// C# MetadataService와 VaultConfig의 기능을 완전히 포팅
#[derive(Debug)]
pub struct DatabaseService {
    /// SQLite 연결
    connection: Option<Connection>,
    /// 데이터베이스 파일 경로
    db_path: Option<String>,
}

impl Clone for DatabaseService {
    fn clone(&self) -> Self {
        // SQLite Connection은 Clone을 구현하지 않으므로
        // 새로운 연결을 생성하거나 None으로 초기화
        Self {
            connection: None, // 새로운 인스턴스는 연결을 다시 설정해야 함
            db_path: self.db_path.clone(),
        }
    }
}

impl DatabaseService {
    /// 새로운 데이터베이스 서비스를 생성합니다.
    pub fn new() -> Self {
        Self {
            connection: None,
            db_path: None,
        }
    }

    /// 데이터베이스를 초기화합니다.
    /// 
    /// # 매개변수
    /// * `vault_path` - 볼트 경로
    /// 
    /// # 반환값
    /// * `Result<(), VaultError>` - 초기화 결과
    pub fn initialize(&mut self, vault_path: &str) -> Result<(), VaultError> {
        let db_path = Path::new(vault_path)
            .join(".securevault")
            .join("metadata.db");

        // 부모 디렉토리 생성
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| VaultError::DatabaseError(format!("디렉토리 생성 실패: {}", e)))?;
        }

        // SQLite 연결 생성
        let conn = Connection::open(&db_path)
            .map_err(|e| VaultError::DatabaseError(format!("데이터베이스 연결 실패: {}", e)))?;

        // 스키마 생성
        self.create_schema(&conn)?;
        
        // 스키마 마이그레이션 실행
        self.migrate_schema(&conn)?;

        self.connection = Some(conn);
        self.db_path = Some(db_path.to_string_lossy().to_string());

        log::info!("데이터베이스 초기화 완료: {:?}", db_path);
        Ok(())
    }

    /// 데이터베이스 스키마를 생성합니다.
    /// C# 버전의 FileMetadata와 VaultConfig 구조를 기반으로 설계
    fn create_schema(&self, conn: &Connection) -> Result<(), VaultError> {
        // 파일 메타데이터 테이블 (C# FileMetadata 포팅 + 압축 필드 추가)
        conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS files (
                id TEXT PRIMARY KEY,
                file_name TEXT NOT NULL,
                original_file_name TEXT NOT NULL,
                file_size INTEGER NOT NULL,
                file_extension TEXT NOT NULL DEFAULT '',
                mime_type TEXT NOT NULL DEFAULT 'application/octet-stream',
                checksum TEXT NOT NULL DEFAULT '',
                created_date TEXT NOT NULL,
                modified_date TEXT NOT NULL,
                last_access_date TEXT NOT NULL,
                folder_id TEXT,
                encrypted_file_name TEXT NOT NULL,
                encrypted_size INTEGER NOT NULL,
                is_compressed INTEGER DEFAULT 0,
                compressed_size INTEGER DEFAULT 0,
                compression_ratio REAL DEFAULT 1.0,
                tags TEXT DEFAULT '[]',
                description TEXT DEFAULT '',
                version INTEGER DEFAULT 1,
                is_favorite INTEGER DEFAULT 0,
                is_deleted INTEGER DEFAULT 0,
                deleted_date TEXT,
                custom_properties TEXT DEFAULT '{}',
                access_count INTEGER DEFAULT 0,
                security_level INTEGER DEFAULT 0
            )
            "#,
            [],
        ).map_err(|e| VaultError::DatabaseError(format!("파일 테이블 생성 실패: {}", e)))?;

        // 폴더 테이블 (C# FolderEntry 포팅)
        conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS folders (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                parent_id TEXT,
                path TEXT NOT NULL,
                created_at TEXT NOT NULL,
                modified_at TEXT NOT NULL,
                status INTEGER DEFAULT 0,
                subfolder_count INTEGER DEFAULT 0,
                file_count INTEGER DEFAULT 0,
                total_size INTEGER DEFAULT 0,
                child_folder_ids TEXT DEFAULT '[]',
                file_ids TEXT DEFAULT '[]'
            )
            "#,
            [],
        ).map_err(|e| VaultError::DatabaseError(format!("폴더 테이블 생성 실패: {}", e)))?;

        // 볼트 설정 테이블 (C# VaultConfig 포팅)
        conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS vault_config (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                created_date TEXT NOT NULL,
                modified_date TEXT NOT NULL
            )
            "#,
            [],
        ).map_err(|e| VaultError::DatabaseError(format!("설정 테이블 생성 실패: {}", e)))?;

        // 인덱스 생성 (성능 최적화)
        conn.execute("CREATE INDEX IF NOT EXISTS idx_files_folder_id ON files(folder_id)", [])
            .map_err(|e| VaultError::DatabaseError(format!("파일 폴더 인덱스 생성 실패: {}", e)))?;

        conn.execute("CREATE INDEX IF NOT EXISTS idx_files_deleted ON files(is_deleted)", [])
            .map_err(|e| VaultError::DatabaseError(format!("파일 삭제 인덱스 생성 실패: {}", e)))?;

        conn.execute("CREATE INDEX IF NOT EXISTS idx_folders_parent_id ON folders(parent_id)", [])
            .map_err(|e| VaultError::DatabaseError(format!("폴더 부모 인덱스 생성 실패: {}", e)))?;

        conn.execute("CREATE INDEX IF NOT EXISTS idx_folders_path ON folders(path)", [])
            .map_err(|e| VaultError::DatabaseError(format!("폴더 경로 인덱스 생성 실패: {}", e)))?;

        log::info!("데이터베이스 스키마 생성 완료");
        Ok(())
    }

    /// 데이터베이스 스키마를 마이그레이션합니다.
    /// 기존 데이터베이스에 누락된 컬럼을 추가합니다.
    fn migrate_schema(&self, conn: &Connection) -> Result<(), VaultError> {
        // 현재 스키마 버전 확인
        let schema_version = self.get_schema_version(conn)?;
        
        log::info!("현재 스키마 버전: {}", schema_version);
        
        // 버전별 마이그레이션 실행
        if schema_version < 1 {
            self.migrate_to_version_1(conn)?;
        }
        
        // 최신 버전으로 업데이트
        self.set_schema_version(conn, 1)?;
        
        log::info!("데이터베이스 마이그레이션 완료");
        Ok(())
    }
    
    /// 스키마 버전을 조회합니다.
    fn get_schema_version(&self, conn: &Connection) -> Result<i32, VaultError> {
        // vault_config 테이블에서 schema_version 조회
        let version_result = conn.query_row(
            "SELECT value FROM vault_config WHERE key = 'schema_version'",
            [],
            |row| {
                let version_str: String = row.get(0)?;
                Ok(version_str.parse::<i32>().unwrap_or(0))
            }
        );
        
        match version_result {
            Ok(version) => Ok(version),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(0), // 새 데이터베이스
            Err(e) => {
                // vault_config 테이블이 없는 경우도 처리
                if e.to_string().contains("no such table") {
                    Ok(0)
                } else {
                    Err(VaultError::DatabaseError(format!("스키마 버전 조회 실패: {}", e)))
                }
            }
        }
    }
    
    /// 스키마 버전을 설정합니다.
    fn set_schema_version(&self, conn: &Connection, version: i32) -> Result<(), VaultError> {
        let now = Utc::now().to_rfc3339();
        
        conn.execute(
            r#"
            INSERT OR REPLACE INTO vault_config (key, value, created_date, modified_date)
            VALUES ('schema_version', ?1, ?2, ?3)
            "#,
            params![version.to_string(), now, now],
        ).map_err(|e| VaultError::DatabaseError(format!("스키마 버전 설정 실패: {}", e)))?;
        
        Ok(())
    }
    
    /// 버전 1로 마이그레이션: is_compressed 관련 컬럼 추가
    fn migrate_to_version_1(&self, conn: &Connection) -> Result<(), VaultError> {
        log::info!("스키마 버전 1로 마이그레이션 시작");
        
        // files 테이블에 압축 관련 컬럼이 있는지 확인
        let has_compressed_column = conn.prepare("SELECT is_compressed FROM files LIMIT 1").is_ok();
        
        if !has_compressed_column {
            log::info!("files 테이블에 압축 관련 컬럼 추가");
            
            // is_compressed 컬럼 추가
            conn.execute(
                "ALTER TABLE files ADD COLUMN is_compressed INTEGER DEFAULT 0",
                [],
            ).map_err(|e| VaultError::DatabaseError(format!("is_compressed 컬럼 추가 실패: {}", e)))?;
            
            // compressed_size 컬럼 추가
            conn.execute(
                "ALTER TABLE files ADD COLUMN compressed_size INTEGER DEFAULT 0",
                [],
            ).map_err(|e| VaultError::DatabaseError(format!("compressed_size 컬럼 추가 실패: {}", e)))?;
            
            // compression_ratio 컬럼 추가
            conn.execute(
                "ALTER TABLE files ADD COLUMN compression_ratio REAL DEFAULT 1.0",
                [],
            ).map_err(|e| VaultError::DatabaseError(format!("compression_ratio 컬럼 추가 실패: {}", e)))?;
            
            // 기존 파일들의 compressed_size를 encrypted_size와 동일하게 설정
            conn.execute(
                "UPDATE files SET compressed_size = encrypted_size WHERE compressed_size = 0",
                [],
            ).map_err(|e| VaultError::DatabaseError(format!("기존 파일 압축 크기 업데이트 실패: {}", e)))?;
            
            log::info!("압축 관련 컬럼 추가 완료");
        } else {
            log::info!("압축 관련 컬럼이 이미 존재함");
        }
        
        Ok(())
    }

    /// 파일 메타데이터를 추가합니다.
    /// 
    /// # 매개변수
    /// * `file_entry` - 파일 엔트리
    /// 
    /// # 반환값
    /// * `Result<(), VaultError>` - 추가 결과
    pub fn add_file(&self, file_entry: &FileEntry) -> Result<(), VaultError> {
        let conn = self.connection.as_ref()
            .ok_or_else(|| VaultError::DatabaseError("데이터베이스가 초기화되지 않았습니다.".to_string()))?;

        let tags_json = serde_json::to_string(&file_entry.tags)
            .map_err(|e| VaultError::DatabaseError(format!("태그 직렬화 실패: {}", e)))?;

        let custom_properties_json = serde_json::to_string(&file_entry.custom_properties)
            .map_err(|e| VaultError::DatabaseError(format!("사용자 속성 직렬화 실패: {}", e)))?;

        conn.execute(
            r#"
            INSERT INTO files (
                id, file_name, original_file_name, file_size, file_extension,
                mime_type, checksum, created_date, modified_date, last_access_date,
                folder_id, encrypted_file_name, encrypted_size, is_compressed,
                compressed_size, compression_ratio, tags, description,
                version, is_favorite, is_deleted, deleted_date, custom_properties,
                access_count, security_level
            ) VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10,
                ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23, ?24, ?25
            )
            "#,
            params![
                file_entry.id.to_string(),
                file_entry.file_name,
                file_entry.original_file_name,
                file_entry.file_size as i64,
                file_entry.file_extension,
                file_entry.mime_type,
                file_entry.checksum,
                file_entry.created_date.to_rfc3339(),
                file_entry.modified_date.to_rfc3339(),
                file_entry.last_access_date.to_rfc3339(),
                file_entry.folder_id.map(|id| id.to_string()),
                file_entry.encrypted_file_name,
                file_entry.encrypted_size as i64,
                if file_entry.is_compressed { 1 } else { 0 },
                file_entry.compressed_size as i64,
                file_entry.compression_ratio,
                tags_json,
                file_entry.description,
                file_entry.version as i32,
                if file_entry.is_favorite { 1 } else { 0 },
                if file_entry.is_deleted { 1 } else { 0 },
                file_entry.deleted_date.map(|d| d.to_rfc3339()),
                custom_properties_json,
                file_entry.access_count as i32,
                file_entry.security_level as i32
            ],
        ).map_err(|e| VaultError::DatabaseError(format!("파일 추가 실패: {}", e)))?;

        log::info!("파일 메타데이터 추가 완료: {}", file_entry.file_name);
        Ok(())
    }

    /// 파일 메타데이터를 조회합니다.
    /// 
    /// # 매개변수
    /// * `file_id` - 파일 ID
    /// 
    /// # 반환값
    /// * `Result<Option<FileEntry>, VaultError>` - 파일 엔트리
    pub fn get_file(&self, file_id: &Uuid) -> Result<Option<FileEntry>, VaultError> {
        let conn = self.connection.as_ref()
            .ok_or_else(|| VaultError::DatabaseError("데이터베이스가 초기화되지 않았습니다.".to_string()))?;

        let mut stmt = conn.prepare(
            "SELECT * FROM files WHERE id = ?1 AND is_deleted = 0"
        ).map_err(|e| VaultError::DatabaseError(format!("쿼리 준비 실패: {}", e)))?;

        let file_result = stmt.query_row(params![file_id.to_string()], |row| {
            self.row_to_file_entry(row)
        });

        match file_result {
            Ok(file_entry) => Ok(Some(file_entry)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(VaultError::DatabaseError(format!("파일 조회 실패: {}", e))),
        }
    }

    /// 파일 메타데이터를 조회합니다 (문자열 ID 버전).
    /// 
    /// # 매개변수
    /// * `file_id` - 파일 ID (문자열)
    /// 
    /// # 반환값
    /// * `Result<Option<FileEntry>, VaultError>` - 파일 엔트리
    pub fn get_file_metadata(&self, file_id: &str) -> Result<Option<FileEntry>, VaultError> {
        let uuid = Uuid::parse_str(file_id)
            .map_err(|_| VaultError::DatabaseError("잘못된 파일 ID 형식입니다.".to_string()))?;
        
        self.get_file(&uuid)
    }

    /// 폴더의 파일 목록을 조회합니다.
    /// 
    /// # 매개변수
    /// * `folder_id` - 폴더 ID (None이면 루트)
    /// 
    /// # 반환값
    /// * `Result<Vec<FileEntry>, VaultError>` - 파일 목록
    pub fn get_files_by_folder(&self, folder_id: Option<Uuid>) -> Result<Vec<FileEntry>, VaultError> {
        let conn = self.connection.as_ref()
            .ok_or_else(|| VaultError::DatabaseError("데이터베이스가 초기화되지 않았습니다.".to_string()))?;

        if let Some(folder_id) = folder_id {
            let mut stmt = conn.prepare("SELECT * FROM files WHERE folder_id = ?1 AND is_deleted = 0 ORDER BY file_name")
                .map_err(|e| VaultError::DatabaseError(format!("쿼리 준비 실패: {}", e)))?;
            
            let file_iter = stmt.query_map(params![folder_id.to_string()], |row| self.row_to_file_entry(row))
                .map_err(|e| VaultError::DatabaseError(format!("파일 목록 조회 실패: {}", e)))?;

            let mut files = Vec::new();
            for file_result in file_iter {
                match file_result {
                    Ok(file_entry) => files.push(file_entry),
                    Err(e) => log::warn!("파일 엔트리 변환 실패: {}", e),
                }
            }
            Ok(files)
        } else {
            let mut stmt = conn.prepare("SELECT * FROM files WHERE folder_id IS NULL AND is_deleted = 0 ORDER BY file_name")
                .map_err(|e| VaultError::DatabaseError(format!("쿼리 준비 실패: {}", e)))?;
            
            let file_iter = stmt.query_map([], |row| self.row_to_file_entry(row))
                .map_err(|e| VaultError::DatabaseError(format!("파일 목록 조회 실패: {}", e)))?;

            let mut files = Vec::new();
            for file_result in file_iter {
                match file_result {
                    Ok(file_entry) => files.push(file_entry),
                    Err(e) => log::warn!("파일 엔트리 변환 실패: {}", e),
                }
            }
            Ok(files)
        }
    }

    /// 파일 메타데이터를 삭제합니다.
    /// 
    /// # 매개변수
    /// * `file_id` - 파일 ID
    /// 
    /// # 반환값
    /// * `Result<(), VaultError>` - 삭제 결과
    pub fn remove_file(&self, file_id: &Uuid) -> Result<(), VaultError> {
        let conn = self.connection.as_ref()
            .ok_or_else(|| VaultError::DatabaseError("데이터베이스가 초기화되지 않았습니다.".to_string()))?;

        conn.execute(
            "DELETE FROM files WHERE id = ?1",
            params![file_id.to_string()],
        ).map_err(|e| VaultError::DatabaseError(format!("파일 삭제 실패: {}", e)))?;

        log::info!("파일 메타데이터 삭제 완료: {}", file_id);
        Ok(())
    }

    /// 폴더를 추가합니다.
    /// 
    /// # 매개변수
    /// * `folder_entry` - 폴더 엔트리
    /// 
    /// # 반환값
    /// * `Result<(), VaultError>` - 추가 결과
    pub fn add_folder(&self, folder_entry: &FolderEntry) -> Result<(), VaultError> {
        let conn = self.connection.as_ref()
            .ok_or_else(|| VaultError::DatabaseError("데이터베이스가 초기화되지 않았습니다.".to_string()))?;

        let child_folder_ids_json = serde_json::to_string(&folder_entry.child_folder_ids)
            .map_err(|e| VaultError::DatabaseError(format!("하위 폴더 ID 직렬화 실패: {}", e)))?;

        let file_ids_json = serde_json::to_string(&folder_entry.file_ids)
            .map_err(|e| VaultError::DatabaseError(format!("파일 ID 직렬화 실패: {}", e)))?;

        conn.execute(
            r#"
            INSERT INTO folders (
                id, name, parent_id, path, created_at, modified_at,
                status, subfolder_count, file_count, total_size,
                child_folder_ids, file_ids
            ) VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12
            )
            "#,
            params![
                folder_entry.id.to_string(),
                folder_entry.name,
                folder_entry.parent_id.map(|id| id.to_string()),
                folder_entry.path,
                folder_entry.created_at.to_rfc3339(),
                folder_entry.modified_at.to_rfc3339(),
                folder_entry.status as i32,
                folder_entry.subfolder_count as i32,
                folder_entry.file_count as i32,
                folder_entry.total_size as i64,
                child_folder_ids_json,
                file_ids_json
            ],
        ).map_err(|e| VaultError::DatabaseError(format!("폴더 추가 실패: {}", e)))?;

        log::info!("폴더 추가 완료: {}", folder_entry.name);
        Ok(())
    }

    /// 폴더를 조회합니다.
    /// 
    /// # 매개변수
    /// * `folder_id` - 폴더 ID
    /// 
    /// # 반환값
    /// * `Result<Option<FolderEntry>, VaultError>` - 폴더 엔트리
    pub fn get_folder(&self, folder_id: &Uuid) -> Result<Option<FolderEntry>, VaultError> {
        let conn = self.connection.as_ref()
            .ok_or_else(|| VaultError::DatabaseError("데이터베이스가 초기화되지 않았습니다.".to_string()))?;

        let mut stmt = conn.prepare(
            "SELECT * FROM folders WHERE id = ?1"
        ).map_err(|e| VaultError::DatabaseError(format!("쿼리 준비 실패: {}", e)))?;

        let folder_result = stmt.query_row(params![folder_id.to_string()], |row| {
            self.row_to_folder_entry(row)
        });

        match folder_result {
            Ok(folder_entry) => Ok(Some(folder_entry)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(VaultError::DatabaseError(format!("폴더 조회 실패: {}", e))),
        }
    }

    /// 모든 폴더를 조회합니다 (실시간 파일 개수 및 용량 포함).
    /// 
    /// # 반환값
    /// * `Result<Vec<FolderEntry>, VaultError>` - 폴더 목록
    pub fn get_all_folders(&self) -> Result<Vec<FolderEntry>, VaultError> {
        let conn = self.connection.as_ref()
            .ok_or_else(|| VaultError::DatabaseError("데이터베이스가 초기화되지 않았습니다.".to_string()))?;

        let mut stmt = conn.prepare("SELECT * FROM folders ORDER BY path")
            .map_err(|e| VaultError::DatabaseError(format!("쿼리 준비 실패: {}", e)))?;

        let folder_iter = stmt.query_map([], |row| self.row_to_folder_entry(row))
            .map_err(|e| VaultError::DatabaseError(format!("폴더 목록 조회 실패: {}", e)))?;

        let mut folders = Vec::new();
        for folder_result in folder_iter {
            match folder_result {
                Ok(mut folder_entry) => {
                    // 실시간 파일 개수 계산
                    let file_count = self.count_files_in_folder(Some(folder_entry.id))?;
                    folder_entry.file_count = file_count as u32;
                    
                    // 실시간 폴더 총 용량 계산
                    let total_size = self.calculate_folder_size(Some(folder_entry.id))?;
                    folder_entry.total_size = total_size as u64;
                    
                    // 하위 폴더 개수도 실시간 계산
                    let subfolder_count = self.count_subfolders(Some(folder_entry.id))?;
                    folder_entry.subfolder_count = subfolder_count as u32;
                    
                    folders.push(folder_entry);
                },
                Err(e) => log::warn!("폴더 엔트리 변환 실패: {}", e),
            }
        }

        Ok(folders)
    }

    /// 폴더를 삭제합니다.
    /// 
    /// # 매개변수
    /// * `folder_id` - 폴더 ID
    /// 
    /// # 반환값
    /// * `Result<(), VaultError>` - 삭제 결과
    pub fn remove_folder(&self, folder_id: &Uuid) -> Result<(), VaultError> {
        let conn = self.connection.as_ref()
            .ok_or_else(|| VaultError::DatabaseError("데이터베이스가 초기화되지 않았습니다.".to_string()))?;

        conn.execute(
            "DELETE FROM folders WHERE id = ?1",
            params![folder_id.to_string()],
        ).map_err(|e| VaultError::DatabaseError(format!("폴더 삭제 실패: {}", e)))?;

        log::info!("폴더 삭제 완료: {}", folder_id);
        Ok(())
    }

    /// 파일 메타데이터를 업데이트합니다.
    /// 
    /// # 매개변수
    /// * `file_entry` - 파일 엔트리
    /// 
    /// # 반환값
    /// * `Result<(), VaultError>` - 업데이트 결과
    pub fn update_file(&self, file_entry: &FileEntry) -> Result<(), VaultError> {
        let conn = self.connection.as_ref()
            .ok_or_else(|| VaultError::DatabaseError("데이터베이스가 초기화되지 않았습니다.".to_string()))?;

        let tags_json = serde_json::to_string(&file_entry.tags)
            .map_err(|e| VaultError::DatabaseError(format!("태그 직렬화 실패: {}", e)))?;

        let custom_properties_json = serde_json::to_string(&file_entry.custom_properties)
            .map_err(|e| VaultError::DatabaseError(format!("사용자 속성 직렬화 실패: {}", e)))?;

        conn.execute(
            r#"
            UPDATE files SET
                file_name = ?2, original_file_name = ?3, file_size = ?4, file_extension = ?5,
                mime_type = ?6, checksum = ?7, modified_date = ?8, last_access_date = ?9,
                folder_id = ?10, encrypted_file_name = ?11, encrypted_size = ?12, is_compressed = ?13,
                compressed_size = ?14, compression_ratio = ?15, tags = ?16,
                description = ?17, version = ?18, is_favorite = ?19, is_deleted = ?20,
                deleted_date = ?21, custom_properties = ?22, access_count = ?23, security_level = ?24
            WHERE id = ?1
            "#,
            params![
                file_entry.id.to_string(),
                file_entry.file_name,
                file_entry.original_file_name,
                file_entry.file_size as i64,
                file_entry.file_extension,
                file_entry.mime_type,
                file_entry.checksum,
                file_entry.modified_date.to_rfc3339(),
                file_entry.last_access_date.to_rfc3339(),
                file_entry.folder_id.map(|id| id.to_string()),
                file_entry.encrypted_file_name,
                file_entry.encrypted_size as i64,
                if file_entry.is_compressed { 1 } else { 0 },
                file_entry.compressed_size as i64,
                file_entry.compression_ratio,
                tags_json,
                file_entry.description,
                file_entry.version as i32,
                if file_entry.is_favorite { 1 } else { 0 },
                if file_entry.is_deleted { 1 } else { 0 },
                file_entry.deleted_date.map(|d| d.to_rfc3339()),
                custom_properties_json,
                file_entry.access_count as i32,
                file_entry.security_level as i32
            ],
        ).map_err(|e| VaultError::DatabaseError(format!("파일 업데이트 실패: {}", e)))?;

        log::info!("파일 메타데이터 업데이트 완료: {}", file_entry.file_name);
        Ok(())
    }

    /// 폴더를 업데이트합니다.
    /// 
    /// # 매개변수
    /// * `folder_entry` - 폴더 엔트리
    /// 
    /// # 반환값
    /// * `Result<(), VaultError>` - 업데이트 결과
    pub fn update_folder(&self, folder_entry: &FolderEntry) -> Result<(), VaultError> {
        let conn = self.connection.as_ref()
            .ok_or_else(|| VaultError::DatabaseError("데이터베이스가 초기화되지 않았습니다.".to_string()))?;

        let child_folder_ids_json = serde_json::to_string(&folder_entry.child_folder_ids)
            .map_err(|e| VaultError::DatabaseError(format!("하위 폴더 ID 직렬화 실패: {}", e)))?;

        let file_ids_json = serde_json::to_string(&folder_entry.file_ids)
            .map_err(|e| VaultError::DatabaseError(format!("파일 ID 직렬화 실패: {}", e)))?;

        conn.execute(
            r#"
            UPDATE folders SET
                name = ?2, parent_id = ?3, path = ?4, modified_at = ?5,
                status = ?6, subfolder_count = ?7, file_count = ?8, total_size = ?9,
                child_folder_ids = ?10, file_ids = ?11
            WHERE id = ?1
            "#,
            params![
                folder_entry.id.to_string(),
                folder_entry.name,
                folder_entry.parent_id.map(|id| id.to_string()),
                folder_entry.path,
                folder_entry.modified_at.to_rfc3339(),
                folder_entry.status as i32,
                folder_entry.subfolder_count as i32,
                folder_entry.file_count as i32,
                folder_entry.total_size as i64,
                child_folder_ids_json,
                file_ids_json
            ],
        ).map_err(|e| VaultError::DatabaseError(format!("폴더 업데이트 실패: {}", e)))?;

        log::info!("폴더 업데이트 완료: {}", folder_entry.name);
        Ok(())
    }

    /// 데이터베이스 행을 FileEntry로 변환합니다.
    fn row_to_file_entry(&self, row: &Row) -> SqliteResult<FileEntry> {
        let tags_json: String = row.get("tags")?;
        let tags = serde_json::from_str(&tags_json).unwrap_or_default();

        let custom_properties_json: String = row.get("custom_properties")?;
        let custom_properties = serde_json::from_str(&custom_properties_json).unwrap_or_default();

        let folder_id_str: Option<String> = row.get("folder_id")?;
        let folder_id = folder_id_str.and_then(|s| Uuid::parse_str(&s).ok());

        let deleted_date_str: Option<String> = row.get("deleted_date")?;
        let deleted_date = deleted_date_str.and_then(|s| DateTime::parse_from_rfc3339(&s).ok().map(|dt| dt.with_timezone(&Utc)));

        Ok(FileEntry {
            id: Uuid::parse_str(&row.get::<_, String>("id")?).unwrap(),
            file_name: row.get("file_name")?,
            original_file_name: row.get("original_file_name")?,
            file_size: row.get::<_, i64>("file_size")? as u64,
            file_extension: row.get("file_extension")?,
            mime_type: row.get("mime_type")?,
            checksum: row.get("checksum")?,
            created_date: DateTime::parse_from_rfc3339(&row.get::<_, String>("created_date")?).unwrap().with_timezone(&Utc),
            modified_date: DateTime::parse_from_rfc3339(&row.get::<_, String>("modified_date")?).unwrap().with_timezone(&Utc),
            last_access_date: DateTime::parse_from_rfc3339(&row.get::<_, String>("last_access_date")?).unwrap().with_timezone(&Utc),
            folder_id,
            encrypted_file_name: row.get("encrypted_file_name")?,
            encrypted_size: row.get::<_, i64>("encrypted_size")? as u64,
            is_compressed: row.get::<_, i32>("is_compressed")? != 0,
            compressed_size: row.get::<_, i64>("compressed_size")? as u64,
            compression_ratio: row.get("compression_ratio")?,
            tags,
            description: row.get("description")?,
            version: row.get::<_, i32>("version")? as u32,
            is_favorite: row.get::<_, i32>("is_favorite")? != 0,
            is_deleted: row.get::<_, i32>("is_deleted")? != 0,
            deleted_date,
            custom_properties,
            access_count: row.get::<_, i32>("access_count")? as u32,
            security_level: crate::models::file::FileSecurityLevel::from(row.get::<_, i32>("security_level")?),
        })
    }

    /// 데이터베이스 행을 FolderEntry로 변환합니다.
    fn row_to_folder_entry(&self, row: &Row) -> SqliteResult<FolderEntry> {
        let child_folder_ids_json: String = row.get("child_folder_ids")?;
        let child_folder_ids = serde_json::from_str(&child_folder_ids_json).unwrap_or_default();

        let file_ids_json: String = row.get("file_ids")?;
        let file_ids = serde_json::from_str(&file_ids_json).unwrap_or_default();

        let parent_id_str: Option<String> = row.get("parent_id")?;
        let parent_id = parent_id_str.and_then(|s| Uuid::parse_str(&s).ok());

        Ok(FolderEntry {
            id: Uuid::parse_str(&row.get::<_, String>("id")?).unwrap(),
            name: row.get("name")?,
            parent_id,
            path: row.get("path")?,
            created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>("created_at")?).unwrap().with_timezone(&Utc),
            modified_at: DateTime::parse_from_rfc3339(&row.get::<_, String>("modified_at")?).unwrap().with_timezone(&Utc),
            status: crate::models::folder::FolderStatus::from(row.get::<_, i32>("status")?),
            subfolder_count: row.get::<_, i32>("subfolder_count")? as u32,
            file_count: row.get::<_, i32>("file_count")? as u32,
            total_size: row.get::<_, i64>("total_size")? as u64,
            child_folder_ids,
            file_ids,
            children: None, // 런타임에 설정됨
        })
    }

    /// 폴더의 총 크기를 계산합니다 (하위 폴더 포함)
    /// 
    /// # 매개변수
    /// * `folder_id` - 폴더 ID (None이면 루트)
    /// 
    /// # 반환값
    /// * `Result<u64, VaultError>` - 총 크기 (바이트)
    pub fn calculate_folder_size(&self, folder_id: Option<Uuid>) -> Result<u64, VaultError> {
        let conn = self.connection.as_ref()
            .ok_or_else(|| VaultError::DatabaseError("데이터베이스가 초기화되지 않았습니다.".to_string()))?;

        // 재귀적으로 폴더와 하위 폴더의 파일 크기 합계 계산
        let mut total_size = 0u64;

        // 현재 폴더의 파일들 크기 합계
        let folder_id_str = folder_id.map(|id| id.to_string());
        let mut stmt = conn.prepare(
            "SELECT COALESCE(SUM(file_size), 0) as total_size FROM files WHERE folder_id = ?1 AND is_deleted = 0"
        ).map_err(|e| VaultError::DatabaseError(format!("쿼리 준비 실패: {}", e)))?;

        let size: i64 = stmt.query_row(params![folder_id_str], |row| {
            Ok(row.get("total_size")?)
        }).map_err(|e| VaultError::DatabaseError(format!("폴더 크기 계산 실패: {}", e)))?;

        total_size += size as u64;

        // 하위 폴더들의 크기도 재귀적으로 계산
        let mut stmt = conn.prepare(
            "SELECT id FROM folders WHERE parent_id = ?1"
        ).map_err(|e| VaultError::DatabaseError(format!("하위 폴더 쿼리 준비 실패: {}", e)))?;

        let subfolder_ids: Vec<Uuid> = stmt.query_map(params![folder_id_str], |row| {
            let id_str: String = row.get("id")?;
            Ok(Uuid::parse_str(&id_str).unwrap())
        }).map_err(|e| VaultError::DatabaseError(format!("하위 폴더 조회 실패: {}", e)))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| VaultError::DatabaseError(format!("하위 폴더 수집 실패: {}", e)))?;

        // 각 하위 폴더의 크기를 재귀적으로 계산
        for subfolder_id in subfolder_ids {
            let subfolder_size = self.calculate_folder_size(Some(subfolder_id))?;
            total_size += subfolder_size;
        }

        Ok(total_size)
    }

    /// 폴더 내 파일 개수를 계산합니다 (하위 폴더 포함)
    /// 
    /// # 매개변수
    /// * `folder_id` - 폴더 ID (None이면 루트)
    /// 
    /// # 반환값
    /// * `Result<u32, VaultError>` - 파일 개수
    pub fn count_files_in_folder(&self, folder_id: Option<Uuid>) -> Result<u32, VaultError> {
        let conn = self.connection.as_ref()
            .ok_or_else(|| VaultError::DatabaseError("데이터베이스가 초기화되지 않았습니다.".to_string()))?;

        let folder_id_str = folder_id.map(|id| id.to_string());
        
        // 현재 폴더의 파일 개수
        let mut stmt = conn.prepare(
            "SELECT COUNT(*) as file_count FROM files WHERE folder_id = ?1 AND is_deleted = 0"
        ).map_err(|e| VaultError::DatabaseError(format!("쿼리 준비 실패: {}", e)))?;

        let mut file_count: i32 = stmt.query_row(params![folder_id_str], |row| {
            Ok(row.get("file_count")?)
        }).map_err(|e| VaultError::DatabaseError(format!("파일 개수 계산 실패: {}", e)))?;

        // 하위 폴더들의 파일 개수도 재귀적으로 계산
        let mut stmt = conn.prepare(
            "SELECT id FROM folders WHERE parent_id = ?1"
        ).map_err(|e| VaultError::DatabaseError(format!("하위 폴더 쿼리 준비 실패: {}", e)))?;

        let subfolder_ids: Vec<Uuid> = stmt.query_map(params![folder_id_str], |row| {
            let id_str: String = row.get("id")?;
            Ok(Uuid::parse_str(&id_str).unwrap())
        }).map_err(|e| VaultError::DatabaseError(format!("하위 폴더 조회 실패: {}", e)))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| VaultError::DatabaseError(format!("하위 폴더 수집 실패: {}", e)))?;

        // 각 하위 폴더의 파일 개수를 재귀적으로 계산
        for subfolder_id in subfolder_ids {
            let subfolder_file_count = self.count_files_in_folder(Some(subfolder_id))?;
            file_count += subfolder_file_count as i32;
        }

        Ok(file_count as u32)
    }

    /// 하위 폴더 개수를 계산합니다
    /// 
    /// # 매개변수
    /// * `folder_id` - 폴더 ID (None이면 루트)
    /// 
    /// # 반환값
    /// * `Result<u32, VaultError>` - 하위 폴더 개수
    pub fn count_subfolders(&self, folder_id: Option<Uuid>) -> Result<u32, VaultError> {
        let conn = self.connection.as_ref()
            .ok_or_else(|| VaultError::DatabaseError("데이터베이스가 초기화되지 않았습니다.".to_string()))?;

        let folder_id_str = folder_id.map(|id| id.to_string());
        
        let mut stmt = conn.prepare(
            "SELECT COUNT(*) as subfolder_count FROM folders WHERE parent_id = ?1"
        ).map_err(|e| VaultError::DatabaseError(format!("쿼리 준비 실패: {}", e)))?;

        let subfolder_count: i32 = stmt.query_row(params![folder_id_str], |row| {
            Ok(row.get("subfolder_count")?)
        }).map_err(|e| VaultError::DatabaseError(format!("하위 폴더 개수 계산 실패: {}", e)))?;

        Ok(subfolder_count as u32)
    }
}

impl Default for DatabaseService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_database_initialization() {
        // 임시 디렉토리 생성
        let temp_dir = TempDir::new().unwrap();
        let vault_path = temp_dir.path().to_str().unwrap();

        // 데이터베이스 서비스 생성 및 초기화
        let mut db_service = DatabaseService::new();
        let result = db_service.initialize(vault_path);

        assert!(result.is_ok());
        assert!(db_service.connection.is_some());
    }

    #[test]
    fn test_folder_operations() {
        // 임시 디렉토리 생성
        let temp_dir = TempDir::new().unwrap();
        let vault_path = temp_dir.path().to_str().unwrap();

        // 데이터베이스 서비스 초기화
        let mut db_service = DatabaseService::new();
        db_service.initialize(vault_path).unwrap();

        // 폴더 생성
        let folder_entry = FolderEntry::new("테스트폴더".to_string(), None, "/테스트폴더".to_string());
        let folder_id = folder_entry.id;

        // 폴더 추가
        db_service.add_folder(&folder_entry).unwrap();

        // 폴더 조회
        let retrieved_folder = db_service.get_folder(&folder_id).unwrap();
        assert!(retrieved_folder.is_some());
        assert_eq!(retrieved_folder.unwrap().name, "테스트폴더");

        // 모든 폴더 조회
        let all_folders = db_service.get_all_folders().unwrap();
        assert_eq!(all_folders.len(), 1);

        // 폴더 삭제
        db_service.remove_folder(&folder_id).unwrap();

        // 삭제 확인
        let deleted_folder = db_service.get_folder(&folder_id).unwrap();
        assert!(deleted_folder.is_none());
    }
}