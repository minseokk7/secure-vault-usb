// SecureVault 타입 정의

/**
 * 볼트 설정 정보
 */
export interface VaultConfig {
  /** 볼트 이름 */
  name: string;
  /** 생성 일시 */
  created_at: string;
  /** 마지막 수정 일시 */
  updated_at: string;
  /** 압축 설정 */
  compression: CompressionConfig;
  /** 파일 이력 설정 */
  file_history: FileHistoryConfig;
  /** 보안 설정 */
  security: SecurityConfig;
}

/**
 * 압축 설정
 */
export interface CompressionConfig {
  /** 압축 활성화 여부 */
  enabled: boolean;
  /** 압축 레벨 (0-9) */
  level: number;
  /** 압축 알고리즘 */
  algorithm: CompressionAlgorithm;
}

/**
 * 압축 알고리즘 타입
 */
export type CompressionAlgorithm = 'None' | 'Gzip' | 'Lz4' | 'Zstd';

/**
 * 파일 이력 설정
 */
export interface FileHistoryConfig {
  /** 파일 이력 추적 활성화 */
  enabled: boolean;
  /** 최대 보관 버전 수 */
  max_history_entries: number;
  /** 자동 백업 활성화 */
  auto_backup_enabled: boolean;
  /** 백업 간격 (일) */
  backup_interval_days: number;
}

/**
 * 보안 설정
 */
export interface SecurityConfig {
  /** PIN 최소 길이 */
  min_pin_length: number;
  /** PIN 최대 길이 */
  max_pin_length: number;
  /** 최대 로그인 시도 횟수 */
  max_login_attempts: number;
  /** 세션 타임아웃 (분) */
  session_timeout_minutes: number;
}

/**
 * 파일 엔트리
 */
export interface FileEntry {
  /** 고유 ID */
  id: string;
  /** 파일명 */
  name: string;
  /** 원본 파일 크기 */
  size: number;
  /** 압축된 크기 */
  compressed_size: number;
  /** 파일 타입 */
  file_type: string;
  /** MIME 타입 */
  mime_type?: string;
  /** 생성 일시 */
  created_at: string;
  /** 수정 일시 */
  modified_at: string;
  /** 폴더 ID */
  folder_id?: string;
  /** 암호화 메타데이터 */
  encryption: EncryptionMetadata;
  /** 압축 메타데이터 */
  compression?: CompressionMetadata;
}

/**
 * 폴더 엔트리
 */
export interface FolderEntry {
  /** 고유 ID */
  id: string;
  /** 폴더명 */
  name: string;
  /** 부모 폴더 ID */
  parent_id?: string;
  /** 생성 일시 */
  created_at: string;
  /** 수정 일시 */
  modified_at: string;
  /** 하위 폴더 목록 */
  children: string[];
}

/**
 * 암호화 메타데이터
 */
export interface EncryptionMetadata {
  /** 암호화 알고리즘 */
  algorithm: EncryptionAlgorithm;
  /** 초기화 벡터 (IV) */
  iv: number[];
  /** 인증 태그 */
  tag: number[];
  /** 솔트 */
  salt: number[];
}

/**
 * 압축 메타데이터
 */
export interface CompressionMetadata {
  /** 압축 알고리즘 */
  algorithm: CompressionAlgorithm;
  /** 압축 레벨 */
  level: number;
  /** 원본 크기 */
  original_size: number;
  /** 압축된 크기 */
  compressed_size: number;
}

/**
 * 암호화 알고리즘 타입
 */
export type EncryptionAlgorithm = 'AES256GCM' | 'ChaCha20Poly1305';

/**
 * 인증 상태
 */
export interface AuthState {
  /** 인증 여부 */
  isAuthenticated: boolean;
  /** 로딩 상태 */
  isLoading: boolean;
  /** 실패 횟수 */
  failedAttempts: number;
  /** 잠금 해제 시간 */
  lockoutUntil?: Date;
}

/**
 * 볼트 상태
 */
export interface VaultState {
  /** 볼트 설정 */
  config?: VaultConfig;
  /** 현재 경로 */
  currentPath: string[];
  /** 선택된 파일들 */
  selectedFiles: string[];
  /** 보기 모드 */
  viewMode: ViewMode;
  /** 정렬 기준 */
  sortBy: SortBy;
  /** 정렬 순서 */
  sortOrder: SortOrder;
}

/**
 * 파일 상태
 */
export interface FileState {
  /** 파일 목록 */
  files: FileEntry[];
  /** 폴더 목록 */
  folders: FolderEntry[];
  /** 로딩 상태 */
  isLoading: boolean;
  /** 오류 메시지 */
  error?: string;
}

/**
 * 보기 모드 타입
 */
export type ViewMode = 'list' | 'grid' | 'details';

/**
 * 정렬 기준 타입
 */
export type SortBy = 'name' | 'size' | 'date' | 'type';

/**
 * 정렬 순서 타입
 */
export type SortOrder = 'asc' | 'desc';

/**
 * 파일 작업 타입
 */
export type FileOperation = 'add' | 'delete' | 'rename' | 'move' | 'copy';

/**
 * 토스트 알림 타입
 */
export interface ToastMessage {
  /** 고유 ID */
  id: string;
  /** 메시지 내용 */
  message: string;
  /** 알림 타입 */
  type: ToastType;
  /** 표시 시간 (ms) */
  duration?: number;
}

/**
 * 토스트 타입
 */
export type ToastType = 'success' | 'error' | 'warning' | 'info';

/**
 * 테마 타입
 */
export type Theme = 'light' | 'dark' | 'auto';

/**
 * 언어 타입
 */
export type Language = 'ko' | 'en';

/**
 * 앱 설정
 */
export interface AppSettings {
  /** 테마 설정 */
  theme: Theme;
  /** 언어 설정 */
  language: Language;
  /** 자동 저장 활성화 */
  autoSave: boolean;
  /** 알림 활성화 */
  notifications: boolean;
}

/**
 * 검색 필터
 */
export interface SearchFilter {
  /** 검색어 */
  query: string;
  /** 파일 타입 필터 */
  fileTypes: string[];
  /** 날짜 범위 */
  dateRange?: {
    start: Date;
    end: Date;
  };
  /** 크기 범위 */
  sizeRange?: {
    min: number;
    max: number;
  };
}

/**
 * 진행 상황 정보
 */
export interface ProgressInfo {
  /** 현재 진행률 (0-100) */
  percentage: number;
  /** 현재 작업 */
  currentTask: string;
  /** 처리된 항목 수 */
  processed: number;
  /** 전체 항목 수 */
  total: number;
  /** 예상 남은 시간 (초) */
  estimatedTimeRemaining?: number;
}

/**
 * API 응답 타입
 */
export interface ApiResponse<T = any> {
  /** 성공 여부 */
  success: boolean;
  /** 응답 데이터 */
  data?: T;
  /** 오류 메시지 */
  error?: string;
  /** 오류 코드 */
  errorCode?: string;
}

/**
 * 파일 업로드 정보
 */
export interface FileUploadInfo {
  /** 파일 경로 */
  path: string;
  /** 파일명 */
  name: string;
  /** 파일 크기 */
  size: number;
  /** MIME 타입 */
  mimeType: string;
  /** 대상 폴더 ID */
  folderId?: string;
}