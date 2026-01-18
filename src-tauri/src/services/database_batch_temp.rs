
/// 파일 메타데이터 여러 개를 일괄 추가합니다 (트랜잭션 사용, 성능 최적화).
///
/// # 매개변수
/// * `file_entries` - 추가할 파일 엔트리 리스트
///
/// # 반환값
/// * `Result<(), VaultError>` - 추가 결과
pub fn add_files_batch(&mut self, file_entries: &[FileEntry]) -> Result<(), VaultError> {
    let conn = self.connection.as_mut().ok_or_else(|| {
        VaultError::DatabaseError("데이터베이스가 초기화되지 않았습니다.".to_string())
    })?;

    // 트랜잭션 시작
    let tx = conn
        .transaction()
        .map_err(|e| VaultError::DatabaseError(format!("트랜잭션 시작 실패: {}", e)))?;

    {
        let mut stmt = tx
            .prepare(
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
            )
            .map_err(|e| VaultError::DatabaseError(format!("배치 쿼리 준비 실패: {}", e)))?;

        for file_entry in file_entries {
            let tags_json = serde_json::to_string(&file_entry.tags)
                .map_err(|e| VaultError::DatabaseError(format!("태그 직렬화 실패: {}", e)))?;

            let custom_properties_json = serde_json::to_string(&file_entry.custom_properties)
                .map_err(|e| {
                    VaultError::DatabaseError(format!("사용자 속성 직렬화 실패: {}", e))
                })?;

            stmt.execute(params![
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
            ])
            .map_err(|e| VaultError::DatabaseError(format!("파일 배포 추가 실패: {}", e)))?;
        }
    }

    // 트랜잭션 커밋
    tx.commit()
        .map_err(|e| VaultError::DatabaseError(format!("트랜잭션 커밋 실패: {}", e)))?;

    log::info!("파일 배치 추가 완료: {}개", file_entries.len());
    Ok(())
}
