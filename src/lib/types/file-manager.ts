/**
 * 파일 매니저 관련 타입 정의
 * C# FileMetadata 및 FolderEntry 모델 기반
 */

// 파일 메타데이터 (Rust FileEntry 포팅)
export interface FileMetadata {
  id: string;
  file_name: string;
  original_file_name: string;
  file_size: number;
  file_extension: string;
  mime_type: string;
  checksum: string;
  created_date: string;
  modified_date: string;
  last_access_date: string;
  folder_id?: string;
  encrypted_file_name: string;
  encrypted_size: number;
  // 압축 관련 정보
  is_compressed: boolean;
  compressed_size: number;
  compression_ratio: number;
  tags: string[];
  description: string;
  version: number;
  is_favorite: boolean;
  is_deleted: boolean;
  deleted_date?: string;
  custom_properties: Record<string, string>;
  access_count: number;
  security_level: 'Normal' | 'High' | 'Critical';
}

// 폴더 엔트리 (Rust FolderEntry 포팅)
export interface FolderEntry {
  id: string;
  name: string;
  path: string;
  parent_id?: string;
  created_at: string;
  modified_at: string;
  file_count: number;
  subfolder_count: number;
  total_size: number;
  status: 'Active' | 'Deleted' | 'Hidden';
  child_folder_ids: string[];
  file_ids: string[];
  description?: string;
  tags?: string[];
  custom_properties?: Record<string, string>;
  children?: FolderEntry[];
}

// 파일 뷰 모드
export type FileViewMode = 'list' | 'grid' | 'details';

// 정렬 옵션
export interface SortOption {
  field: 'name' | 'size' | 'type' | 'modified' | 'created';
  direction: 'asc' | 'desc';
}

// 파일 선택 상태
export interface FileSelection {
  selectedFiles: Set<string>;
  selectedFolders: Set<string>;
  lastSelected?: string;
  selectionType?: 'file' | 'folder';
}

// 파일 작업 타입
export type FileOperation = 
  | 'create'
  | 'upload'
  | 'download'
  | 'delete'
  | 'rename'
  | 'move'
  | 'copy'
  | 'open'
  | 'view';

// 파일 작업 결과
export interface FileOperationResult {
  success: boolean;
  message: string;
  data?: any;
  error?: string;
}

// 파일 업로드 상태
export interface FileUploadStatus {
  fileName: string;
  progress: number;
  status: 'pending' | 'uploading' | 'completed' | 'error';
  error?: string;
}

// 파일 크기 정보 (압축 고려)
export interface FileSizeInfo {
  originalSize: number;
  displaySize: number;
  compressedSize?: number;
  compressionRatio?: number;
  isCompressed: boolean;
  spaceSaved?: number;
  spaceSavedPercent?: number;
}

// 파일 크기를 사람이 읽기 쉬운 형태로 포맷하는 함수
export function formatFileSize(bytes: number): string {
  const units = ['B', 'KB', 'MB', 'GB', 'TB'];
  let size = bytes;
  let unitIndex = 0;

  while (size >= 1024 && unitIndex < units.length - 1) {
    size /= 1024;
    unitIndex++;
  }

  if (unitIndex === 0) {
    return `${bytes} ${units[unitIndex]}`;
  } else {
    return `${size.toFixed(1)} ${units[unitIndex]}`;
  }
}

// 파일 메타데이터에서 크기 정보를 추출하는 함수
export function getFileSizeInfo(file: FileMetadata): FileSizeInfo {
  const originalSize = file.file_size;
  const isCompressed = file.is_compressed;
  
  if (isCompressed) {
    const compressedSize = file.compressed_size;
    const spaceSaved = originalSize > compressedSize ? originalSize - compressedSize : 0;
    const spaceSavedPercent = originalSize > 0 ? (spaceSaved / originalSize) * 100 : 0;
    
    return {
      originalSize,
      displaySize: originalSize, // 사용자에게는 원본 크기 표시
      compressedSize,
      compressionRatio: file.compression_ratio,
      isCompressed: true,
      spaceSaved,
      spaceSavedPercent
    };
  } else {
    return {
      originalSize,
      displaySize: originalSize,
      isCompressed: false
    };
  }
}

// 압축 정보를 포함한 파일 크기 표시 문자열 생성
export function formatFileSizeWithCompression(file: FileMetadata, showCompressionInfo: boolean = false): string {
  const sizeInfo = getFileSizeInfo(file);
  const displaySizeStr = formatFileSize(sizeInfo.displaySize);
  
  if (!showCompressionInfo || !sizeInfo.isCompressed) {
    return displaySizeStr;
  }
  
  const compressedSizeStr = formatFileSize(sizeInfo.compressedSize!);
  const savedPercent = sizeInfo.spaceSavedPercent!.toFixed(1);
  
  return `${displaySizeStr} (압축: ${compressedSizeStr}, ${savedPercent}% 절약)`;
}

// 검색 필터
export interface SearchFilter {
  query: string;
  fileTypes?: string[];
  sizeRange?: {
    min?: number;
    max?: number;
  };
  dateRange?: {
    from?: string;
    to?: string;
  };
  folderId?: string;
}

// 컨텍스트 메뉴 아이템
export interface ContextMenuItem {
  id: string;
  label: string;
  icon?: string;
  disabled?: boolean;
  separator?: boolean;
  action?: () => void;
  children?: ContextMenuItem[];
}

// 파일 매니저 상태
export interface FileManagerState {
  currentFolder: FolderEntry | null;
  files: FileMetadata[];
  folders: FolderEntry[];
  folderTree: FolderEntry[];
  selection: FileSelection;
  viewMode: FileViewMode;
  sortOption: SortOption;
  searchFilter: SearchFilter;
  isLoading: boolean;
  error: string | null;
}

// 드래그 앤 드롭 데이터
export interface DragDropData {
  type: 'file' | 'folder';
  items: string[];
  sourceFolder: string;
}

// 파일 미리보기 정보
export interface FilePreview {
  fileId: string;
  previewType: 'text' | 'image' | 'video' | 'audio' | 'pdf' | 'none';
  previewUrl?: string;
  thumbnailUrl?: string;
  canPreview: boolean;
}