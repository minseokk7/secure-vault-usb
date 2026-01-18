// 업로드 관리자 서비스
// 대용량 파일의 백그라운드 업로드, 진행률 추적, 작업 취소를 담당합니다.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

/// 취소 토큰
/// 작업 취소 신호를 스레드 간에 안전하게 전달합니다.
#[derive(Debug, Clone)]
pub struct CancellationToken {
    cancelled: Arc<AtomicBool>,
}

impl CancellationToken {
    /// 새로운 취소 토큰을 생성합니다.
    pub fn new() -> Self {
        Self {
            cancelled: Arc::new(AtomicBool::new(false)),
        }
    }

    /// 취소 신호를 보냅니다.
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
    }

    /// 취소 여부를 확인합니다.
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }
}

impl Default for CancellationToken {
    fn default() -> Self {
        Self::new()
    }
}

/// 업로드 작업 상태
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UploadStatus {
    /// 대기 중
    Pending,
    /// 처리 중
    Running,
    /// 완료됨
    Completed,
    /// 실패함
    Failed,
    /// 취소됨
    Cancelled,
}

/// 업로드 작업 정보
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadJob {
    /// 작업 ID
    pub id: Uuid,
    /// 원본 파일 경로
    pub file_path: String,
    /// 볼트 내 파일명
    pub file_name: String,
    /// 대상 폴더 ID
    pub folder_id: Option<Uuid>,
    /// 작업 상태
    pub status: UploadStatus,
    /// 진행률 (0.0 ~ 1.0)
    pub progress: f64,
    /// 처리된 바이트
    pub bytes_processed: u64,
    /// 전체 바이트
    pub total_bytes: u64,
    /// 생성 시간
    pub created_at: DateTime<Utc>,
    /// 완료 시간
    pub completed_at: Option<DateTime<Utc>>,
    /// 오류 메시지
    pub error: Option<String>,
    /// 결과 파일 ID (완료 시)
    pub result_file_id: Option<Uuid>,
}

impl UploadJob {
    /// 새로운 업로드 작업을 생성합니다.
    pub fn new(
        file_path: String,
        file_name: String,
        folder_id: Option<Uuid>,
        total_bytes: u64,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            file_path,
            file_name,
            folder_id,
            status: UploadStatus::Pending,
            progress: 0.0,
            bytes_processed: 0,
            total_bytes,
            created_at: Utc::now(),
            completed_at: None,
            error: None,
            result_file_id: None,
        }
    }
}

/// 실행 중인 작업의 진행률을 추적하는 구조체
#[derive(Debug)]
pub struct ProgressTracker {
    pub bytes_processed: Arc<AtomicU64>,
    pub total_bytes: u64,
    pub cancellation_token: CancellationToken,
}

impl ProgressTracker {
    pub fn new(total_bytes: u64) -> Self {
        Self {
            bytes_processed: Arc::new(AtomicU64::new(0)),
            total_bytes,
            cancellation_token: CancellationToken::new(),
        }
    }

    /// 처리된 바이트를 추가합니다.
    pub fn add_progress(&self, bytes: u64) {
        self.bytes_processed.fetch_add(bytes, Ordering::SeqCst);
    }

    /// 현재 진행률을 반환합니다 (0.0 ~ 1.0).
    pub fn get_progress(&self) -> f64 {
        let processed = self.bytes_processed.load(Ordering::SeqCst);
        if self.total_bytes == 0 {
            return 1.0;
        }
        (processed as f64) / (self.total_bytes as f64)
    }

    /// 처리된 바이트 수를 반환합니다.
    pub fn get_bytes_processed(&self) -> u64 {
        self.bytes_processed.load(Ordering::SeqCst)
    }
}

/// 업로드 관리자
/// 백그라운드에서 파일 업로드를 관리하고 상태를 추적합니다.
#[derive(Debug)]
pub struct UploadManager {
    /// 모든 작업 목록
    jobs: Arc<Mutex<HashMap<Uuid, UploadJob>>>,
    /// 작업 큐 (대기 중인 작업 ID)
    job_queue: Arc<Mutex<VecDeque<Uuid>>>,
    /// 실행 중인 작업의 진행률 추적기
    progress_trackers: Arc<Mutex<HashMap<Uuid, Arc<ProgressTracker>>>>,
    /// 동시 처리 가능한 최대 작업 수
    max_concurrent_jobs: usize,
    /// 현재 실행 중인 작업 수
    running_jobs: Arc<AtomicU64>,
}

impl UploadManager {
    /// 새로운 업로드 관리자를 생성합니다.
    pub fn new() -> Self {
        Self {
            jobs: Arc::new(Mutex::new(HashMap::new())),
            job_queue: Arc::new(Mutex::new(VecDeque::new())),
            progress_trackers: Arc::new(Mutex::new(HashMap::new())),
            max_concurrent_jobs: 2, // 동시에 2개 파일까지 처리
            running_jobs: Arc::new(AtomicU64::new(0)),
        }
    }

    /// 새로운 업로드 작업을 추가합니다.
    ///
    /// # 반환값
    /// * `Uuid` - 생성된 작업 ID
    pub fn add_job(
        &self,
        file_path: String,
        file_name: String,
        folder_id: Option<Uuid>,
        total_bytes: u64,
    ) -> Uuid {
        let job = UploadJob::new(file_path, file_name, folder_id, total_bytes);
        let job_id = job.id;
        let job_file_name = job.file_name.clone(); // 로그용으로 미리 복제

        // 작업 목록에 추가
        {
            let mut jobs = self.jobs.lock().unwrap();
            jobs.insert(job_id, job);
        }

        // 큐에 추가
        {
            let mut queue = self.job_queue.lock().unwrap();
            queue.push_back(job_id);
        }

        log::info!("업로드 작업 추가됨: {} ({})", job_id, job_file_name);
        job_id
    }

    /// 작업을 취소합니다.
    ///
    /// # 반환값
    /// * `bool` - 취소 성공 여부
    pub fn cancel_job(&self, job_id: &Uuid) -> bool {
        // 진행률 추적기가 있으면 취소 신호 전송
        if let Some(tracker) = self.get_progress_tracker(job_id) {
            tracker.cancellation_token.cancel();
        }

        // 작업 상태 업데이트
        let mut jobs = self.jobs.lock().unwrap();
        if let Some(job) = jobs.get_mut(job_id) {
            if job.status == UploadStatus::Running || job.status == UploadStatus::Pending {
                job.status = UploadStatus::Cancelled;
                job.completed_at = Some(Utc::now());
                log::info!("업로드 작업 취소됨: {}", job_id);
                return true;
            }
        }
        false
    }

    /// 작업 상태를 조회합니다.
    pub fn get_job(&self, job_id: &Uuid) -> Option<UploadJob> {
        let jobs = self.jobs.lock().unwrap();
        jobs.get(job_id).cloned()
    }

    /// 모든 작업 목록을 조회합니다.
    pub fn get_all_jobs(&self) -> Vec<UploadJob> {
        let jobs = self.jobs.lock().unwrap();
        jobs.values().cloned().collect()
    }

    /// 다음 대기 중인 작업을 가져옵니다.
    pub fn get_next_pending_job(&self) -> Option<Uuid> {
        let current_running = self.running_jobs.load(Ordering::SeqCst);
        if current_running >= self.max_concurrent_jobs as u64 {
            return None;
        }

        let mut queue = self.job_queue.lock().unwrap();
        queue.pop_front()
    }

    /// 작업 시작을 표시합니다.
    pub fn mark_job_started(
        &self,
        job_id: &Uuid,
        total_bytes: u64,
    ) -> Option<Arc<ProgressTracker>> {
        let mut jobs = self.jobs.lock().unwrap();
        if let Some(job) = jobs.get_mut(job_id) {
            job.status = UploadStatus::Running;
            job.total_bytes = total_bytes;

            // 진행률 추적기 생성
            let tracker = Arc::new(ProgressTracker::new(total_bytes));
            {
                let mut trackers = self.progress_trackers.lock().unwrap();
                trackers.insert(*job_id, Arc::clone(&tracker));
            }

            self.running_jobs.fetch_add(1, Ordering::SeqCst);
            log::info!("업로드 작업 시작: {} ({}바이트)", job_id, total_bytes);
            return Some(tracker);
        }
        None
    }

    /// 작업 완료를 표시합니다.
    pub fn mark_job_completed(&self, job_id: &Uuid, result_file_id: Uuid) {
        let mut jobs = self.jobs.lock().unwrap();
        if let Some(job) = jobs.get_mut(job_id) {
            job.status = UploadStatus::Completed;
            job.progress = 1.0;
            job.bytes_processed = job.total_bytes;
            job.completed_at = Some(Utc::now());
            job.result_file_id = Some(result_file_id);

            self.running_jobs.fetch_sub(1, Ordering::SeqCst);
            log::info!(
                "업로드 작업 완료: {} -> 파일 ID: {}",
                job_id,
                result_file_id
            );
        }

        // 진행률 추적기 제거
        let mut trackers = self.progress_trackers.lock().unwrap();
        trackers.remove(job_id);
    }

    /// 작업 실패를 표시합니다.
    pub fn mark_job_failed(&self, job_id: &Uuid, error: String) {
        let mut jobs = self.jobs.lock().unwrap();
        if let Some(job) = jobs.get_mut(job_id) {
            job.status = UploadStatus::Failed;
            job.completed_at = Some(Utc::now());
            job.error = Some(error.clone());

            self.running_jobs.fetch_sub(1, Ordering::SeqCst);
            log::error!("업로드 작업 실패: {} - {}", job_id, error);
        }

        // 진행률 추적기 제거
        let mut trackers = self.progress_trackers.lock().unwrap();
        trackers.remove(job_id);
    }

    /// 진행률 추적기를 가져옵니다.
    pub fn get_progress_tracker(&self, job_id: &Uuid) -> Option<Arc<ProgressTracker>> {
        let trackers = self.progress_trackers.lock().unwrap();
        trackers.get(job_id).cloned()
    }

    /// 작업 진행률을 업데이트합니다.
    pub fn update_job_progress(&self, job_id: &Uuid) {
        if let Some(tracker) = self.get_progress_tracker(job_id) {
            let mut jobs = self.jobs.lock().unwrap();
            if let Some(job) = jobs.get_mut(job_id) {
                job.bytes_processed = tracker.get_bytes_processed();
                job.progress = tracker.get_progress();
            }
        }
    }

    /// 완료된 오래된 작업을 정리합니다.
    pub fn cleanup_old_jobs(&self, max_age_hours: i64) {
        let cutoff = Utc::now() - chrono::Duration::hours(max_age_hours);
        let mut jobs = self.jobs.lock().unwrap();
        jobs.retain(|_, job| {
            if let Some(completed_at) = job.completed_at {
                completed_at > cutoff
            } else {
                true // 미완료 작업은 유지
            }
        });
    }
}

impl Default for UploadManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for UploadManager {
    fn clone(&self) -> Self {
        Self {
            jobs: Arc::clone(&self.jobs),
            job_queue: Arc::clone(&self.job_queue),
            progress_trackers: Arc::clone(&self.progress_trackers),
            max_concurrent_jobs: self.max_concurrent_jobs,
            running_jobs: Arc::clone(&self.running_jobs),
        }
    }
}
