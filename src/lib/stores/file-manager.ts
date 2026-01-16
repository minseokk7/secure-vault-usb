/**
 * 파일 매니저 상태 관리 스토어
 * C# MainForm 및 FileManagerService 기반 상태 관리
 */

import { writable, derived, get } from 'svelte/store';
import type { 
  FolderEntry, 
  FileManagerState, 
  FileViewMode, 
  SortOption, 
  SearchFilter
} from '$lib/types/file-manager';
import { addToast } from './toast';

// 초기 상태 정의
const initialState: FileManagerState = {
  currentFolder: null,
  files: [],
  folders: [],
  folderTree: [],
  selection: {
    selectedFiles: new Set(),
    selectedFolders: new Set(),
    lastSelected: undefined,
    selectionType: undefined
  },
  viewMode: 'list',
  sortOption: {
    field: 'name',
    direction: 'asc'
  },
  searchFilter: {
    query: '',
    fileTypes: [],
    sizeRange: {},
    dateRange: {},
    folderId: undefined
  },
  isLoading: false,
  error: null
};

// 메인 상태 스토어
export const fileManagerState = writable<FileManagerState>(initialState);

// 파생 스토어들 (computed values)
export const currentFolderPath = derived(
  fileManagerState,
  ($state) => $state.currentFolder?.path || '/'
);

export const selectedItemsCount = derived(
  fileManagerState,
  ($state) => $state.selection.selectedFiles.size + $state.selection.selectedFolders.size
);

export const hasSelection = derived(
  selectedItemsCount,
  ($count) => $count > 0
);

// 파일 선택 관련 파생 스토어들
export const hasFileSelection = derived(
  fileManagerState,
  ($state) => $state.selection.selectedFiles.size > 0
);

export const selectedFileCount = derived(
  fileManagerState,
  ($state) => $state.selection.selectedFiles.size
);

export const filteredFiles = derived(
  fileManagerState,
  ($state) => {
    let files = [...$state.files];
    
    // 검색 필터 적용
    if ($state.searchFilter.query) {
      const query = $state.searchFilter.query.toLowerCase();
      files = files.filter(file => 
        file.file_name.toLowerCase().includes(query) ||
        file.original_file_name.toLowerCase().includes(query)
      );
    }
    
    // 파일 타입 필터 적용
    if ($state.searchFilter.fileTypes && $state.searchFilter.fileTypes.length > 0) {
      files = files.filter(file => 
        $state.searchFilter.fileTypes!.some(type => 
          file.mime_type.startsWith(type)
        )
      );
    }
    
    // 크기 필터 적용
    if ($state.searchFilter.sizeRange?.min !== undefined) {
      files = files.filter(file => file.file_size >= $state.searchFilter.sizeRange!.min!);
    }
    if ($state.searchFilter.sizeRange?.max !== undefined) {
      files = files.filter(file => file.file_size <= $state.searchFilter.sizeRange!.max!);
    }
    
    // 정렬 적용
    files.sort((a, b) => {
      const { field, direction } = $state.sortOption;
      let comparison = 0;
      
      switch (field) {
        case 'name':
          comparison = a.file_name.localeCompare(b.file_name, 'ko');
          break;
        case 'size':
          comparison = a.file_size - b.file_size;
          break;
        case 'type':
          comparison = a.mime_type.localeCompare(b.mime_type);
          break;
        case 'modified':
          comparison = new Date(a.modified_date).getTime() - new Date(b.modified_date).getTime();
          break;
        case 'created':
          comparison = new Date(a.created_date).getTime() - new Date(b.created_date).getTime();
          break;
      }
      
      return direction === 'asc' ? comparison : -comparison;
    });
    
    return files;
  }
);

export const sortedFolders = derived(
  fileManagerState,
  ($state) => {
    const folders = [...$state.folders];
    const { field, direction } = $state.sortOption;
    
    folders.sort((a, b) => {
      let comparison = 0;
      
      switch (field) {
        case 'name':
          comparison = a.name.localeCompare(b.name, 'ko');
          break;
        case 'size':
          comparison = a.total_size - b.total_size;
          break;
        case 'modified':
          comparison = new Date(a.modified_at).getTime() - new Date(b.modified_at).getTime();
          break;
        case 'created':
          comparison = new Date(a.created_at).getTime() - new Date(b.created_at).getTime();
          break;
        default:
          comparison = a.name.localeCompare(b.name, 'ko');
      }
      
      return direction === 'asc' ? comparison : -comparison;
    });
    
    return folders;
  }
);

/**
 * 파일 매니저 서비스 클래스
 * C# MainForm의 파일 관리 로직을 포팅
 */
export class FileManagerService {
  
  /**
   * 현재 폴더 변경
   * @param folderId 폴더 ID (null이면 루트)
   */
  static async navigateToFolder(folderId: string | null = null): Promise<void> {
    fileManagerState.update(state => ({
      ...state,
      isLoading: true,
      error: null
    }));

    try {
      // Tauri 커맨드 호출하여 폴더 내용 가져오기
      const { invoke } = await import('@tauri-apps/api/core');
      
      // 파일 목록과 하위 폴더 목록을 각각 가져오기
      const files = await invoke('get_files_in_folder', { folderId }) as any[];
      const folders = await invoke('get_subfolders', { parentId: folderId }) as any[];
      const currentFolder = folderId ? await invoke('get_folder', { folderId }) : null;

      fileManagerState.update(state => ({
        ...state,
        currentFolder: currentFolder as any,
        files: files || [],
        folders: folders || [],
        isLoading: false,
        selection: {
          selectedFiles: new Set(),
          selectedFolders: new Set(),
          lastSelected: undefined,
          selectionType: undefined
        }
      }));

    } catch (error) {
      console.error('폴더 로드 실패:', error);
      const errorMessage = error instanceof Error ? error.message : '폴더를 불러올 수 없습니다.';
      
      fileManagerState.update(state => ({
        ...state,
        isLoading: false,
        error: errorMessage
      }));

      addToast({
        type: 'error',
        title: '폴더 로드 실패',
        message: errorMessage,
        duration: 5000
      });
    }
  }

  /**
   * 파일 업로드
   * @param filePaths 업로드할 파일 경로들
   * @param folderId 대상 폴더 ID
   */
  static async uploadFiles(filePaths: string[], folderId?: string): Promise<void> {
    fileManagerState.update(state => ({
      ...state,
      isLoading: true,
      error: null
    }));

    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const { readFile } = await import('@tauri-apps/plugin-fs');
      
      for (const filePath of filePaths) {
        // 파일 이름 추출
        const fileName = filePath.split(/[/\\]/).pop() || 'unknown';
        
        // 파일 데이터 읽기
        const fileData = await readFile(filePath);
        const uint8Array = new Uint8Array(fileData);
        
        // Base64로 인코딩
        const base64Content = btoa(String.fromCharCode(...uint8Array));
        
        // 바이너리 파일 생성 명령어 사용
        await invoke('create_binary_file_in_vault', { 
          folder_id: folderId,
          file_name: fileName,
          content: base64Content
        });
      }

      // 업로드 완료 후 현재 폴더 새로고침
      await this.refresh();

      addToast({
        type: 'success',
        title: '파일 업로드 완료',
        message: `${filePaths.length}개 파일이 성공적으로 업로드되었습니다.`,
        duration: 3000
      });

    } catch (error) {
      console.error('파일 업로드 실패:', error);
      const errorMessage = error instanceof Error ? error.message : '파일 업로드에 실패했습니다.';
      
      fileManagerState.update(state => ({
        ...state,
        isLoading: false,
        error: errorMessage
      }));

      addToast({
        type: 'error',
        title: '파일 업로드 실패',
        message: errorMessage,
        duration: 5000
      });
    }
  }

  /**
   * 폴더 추가 (내부 파일과 하위 폴더 포함)
   * @param folderPath 추가할 폴더 경로
   * @param targetFolderId 대상 폴더 ID
   */
  static async addFolder(folderPath: string, targetFolderId?: string): Promise<void> {
    fileManagerState.update(state => ({
      ...state,
      isLoading: true,
      error: null
    }));

    try {
      const { invoke } = await import('@tauri-apps/api/core');
      
      // 폴더 추가 커맨드 호출 (재귀적으로 내부 파일과 하위 폴더 포함)
      const result = await invoke('add_folder_to_vault', { 
        folderPath: folderPath,
        targetFolderId: targetFolderId 
      }) as { folderCount: number; fileCount: number };

      // 폴더 추가 완료 후 현재 폴더 새로고침
      await this.refresh();

      addToast({
        type: 'success',
        title: '폴더 추가 완료',
        message: `폴더 ${result.folderCount}개, 파일 ${result.fileCount}개가 성공적으로 추가되었습니다.`,
        duration: 4000
      });

    } catch (error) {
      console.error('폴더 추가 실패:', error);
      const errorMessage = error instanceof Error ? error.message : '폴더 추가에 실패했습니다.';
      
      fileManagerState.update(state => ({
        ...state,
        isLoading: false,
        error: errorMessage
      }));

      addToast({
        type: 'error',
        title: '폴더 추가 실패',
        message: errorMessage,
        duration: 5000
      });
    }
  }

  /**
   * 폴더 생성
   * @param folderName 폴더 이름
   * @param parentId 부모 폴더 ID
   */
  static async createFolder(folderName: string, parentId?: string): Promise<void> {
    fileManagerState.update(state => ({
      ...state,
      isLoading: true,
      error: null
    }));

    try {
      const { invoke } = await import('@tauri-apps/api/core');
      
      console.log('폴더 생성 요청:', { name: folderName, parent_id: parentId }); // 디버깅용 로그
      
      await invoke('create_folder', { 
        name: folderName, 
        parent_id: parentId 
      });

      // 폴더 생성 후 현재 폴더 새로고침
      await this.refresh();

      addToast({
        type: 'success',
        title: '폴더 생성 완료',
        message: `'${folderName}' 폴더가 생성되었습니다.`,
        duration: 3000
      });

    } catch (error) {
      console.error('폴더 생성 실패:', error);
      const errorMessage = error instanceof Error ? error.message : '폴더 생성에 실패했습니다.';
      
      fileManagerState.update(state => ({
        ...state,
        isLoading: false,
        error: errorMessage
      }));

      addToast({
        type: 'error',
        title: '폴더 생성 실패',
        message: errorMessage,
        duration: 5000
      });
    }
  }

  /**
   * 파일 다운로드
   * @param fileId 파일 ID
   * @param savePath 저장 경로
   */
  static async downloadFile(fileId: string, savePath: string): Promise<void> {
    fileManagerState.update(state => ({
      ...state,
      isLoading: true,
      error: null
    }));

    try {
      const { invoke } = await import('@tauri-apps/api/core');
      
      await invoke('download_file', { file_id: fileId, save_path: savePath });

      fileManagerState.update(state => ({
        ...state,
        isLoading: false
      }));

      addToast({
        type: 'success',
        title: '파일 다운로드 완료',
        message: '파일이 성공적으로 다운로드되었습니다.',
        duration: 3000
      });

    } catch (error) {
      console.error('파일 다운로드 실패:', error);
      const errorMessage = error instanceof Error ? error.message : '파일 다운로드에 실패했습니다.';
      
      fileManagerState.update(state => ({
        ...state,
        isLoading: false,
        error: errorMessage
      }));

      addToast({
        type: 'error',
        title: '파일 다운로드 실패',
        message: errorMessage,
        duration: 5000
      });
    }
  }

  /**
   * 항목 삭제 (파일 또는 폴더)
   * @param itemIds 삭제할 항목 ID들
   * @param itemType 항목 타입
   */
  static async deleteItems(itemIds: string[], itemType: 'file' | 'folder'): Promise<void> {
    fileManagerState.update(state => ({
      ...state,
      isLoading: true,
      error: null
    }));

    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const currentState = get(fileManagerState);
      const currentFolderId = currentState.currentFolder?.id;
      
      // 현재 폴더가 삭제되는지 확인
      const isDeletingCurrentFolder = itemType === 'folder' && 
        currentFolderId && 
        itemIds.includes(currentFolderId);
      
      for (const itemId of itemIds) {
        if (itemType === 'file') {
          await invoke('delete_file_from_vault', { file_id: itemId });
        } else {
          await invoke('delete_folder', { folder_id: itemId, recursive: true });
        }
      }

      // 현재 폴더가 삭제된 경우 부모 폴더로 이동
      if (isDeletingCurrentFolder) {
        const parentFolderId = currentState.currentFolder?.parent_id || null;
        await this.navigateToFolder(parentFolderId);
      } else {
        // 현재 폴더 유지하면서 새로고침
        await this.refresh();
      }

      addToast({
        type: 'success',
        title: '삭제 완료',
        message: `${itemIds.length}개 항목이 삭제되었습니다.`,
        duration: 3000
      });

    } catch (error) {
      console.error('항목 삭제 실패:', error);
      const errorMessage = error instanceof Error ? error.message : '항목 삭제에 실패했습니다.';
      
      fileManagerState.update(state => ({
        ...state,
        isLoading: false,
        error: errorMessage
      }));

      addToast({
        type: 'error',
        title: '삭제 실패',
        message: errorMessage,
        duration: 5000
      });
    }
  }

  /**
   * 항목 이름 변경
   * @param itemId 항목 ID
   * @param newName 새 이름
   * @param itemType 항목 타입
   */
  static async renameItem(itemId: string, newName: string, itemType: 'file' | 'folder'): Promise<void> {
    fileManagerState.update(state => ({
      ...state,
      isLoading: true,
      error: null
    }));

    try {
      const { invoke } = await import('@tauri-apps/api/core');
      
      if (itemType === 'file') {
        await invoke('rename_file_in_vault', { file_id: itemId, new_name: newName });
      } else {
        await invoke('rename_folder', { folder_id: itemId, new_name: newName });
      }

      // 이름 변경 완료 후 현재 폴더 새로고침
      await this.refresh();

      addToast({
        type: 'success',
        title: '이름 변경 완료',
        message: `'${newName}'으로 이름이 변경되었습니다.`,
        duration: 3000
      });

    } catch (error) {
      console.error('이름 변경 실패:', error);
      const errorMessage = error instanceof Error ? error.message : '이름 변경에 실패했습니다.';
      
      fileManagerState.update(state => ({
        ...state,
        isLoading: false,
        error: errorMessage
      }));

      addToast({
        type: 'error',
        title: '이름 변경 실패',
        message: errorMessage,
        duration: 5000
      });
    }
  }

  /**
   * 파일 선택 상태 업데이트
   * @param itemId 파일 또는 폴더 ID
   * @param itemType 아이템 타입
   * @param multiSelect 다중 선택 여부
   */
  static selectItem(itemId: string, itemType: 'file' | 'folder', multiSelect: boolean = false): void {
    fileManagerState.update(state => {
      const newSelection = { ...state.selection };

      if (!multiSelect) {
        // 단일 선택: 기존 선택 해제
        newSelection.selectedFiles.clear();
        newSelection.selectedFolders.clear();
      }

      // 새 아이템 선택/해제
      if (itemType === 'file') {
        if (newSelection.selectedFiles.has(itemId)) {
          newSelection.selectedFiles.delete(itemId);
        } else {
          newSelection.selectedFiles.add(itemId);
        }
      } else {
        if (newSelection.selectedFolders.has(itemId)) {
          newSelection.selectedFolders.delete(itemId);
        } else {
          newSelection.selectedFolders.add(itemId);
        }
      }

      newSelection.lastSelected = itemId;
      newSelection.selectionType = itemType;

      return {
        ...state,
        selection: newSelection
      };
    });
  }

  /**
   * 모든 선택 해제
   */
  static clearSelection(): void {
    fileManagerState.update(state => ({
      ...state,
      selection: {
        selectedFiles: new Set(),
        selectedFolders: new Set(),
        lastSelected: undefined,
        selectionType: undefined
      }
    }));
  }

  /**
   * 뷰 모드 변경
   * @param mode 새로운 뷰 모드
   */
  static setViewMode(mode: FileViewMode): void {
    fileManagerState.update(state => ({
      ...state,
      viewMode: mode
    }));
  }

  /**
   * 정렬 옵션 변경
   * @param sortOption 새로운 정렬 옵션
   */
  static setSortOption(sortOption: SortOption): void {
    fileManagerState.update(state => ({
      ...state,
      sortOption
    }));
  }

  /**
   * 검색 필터 업데이트
   * @param filter 검색 필터
   */
  static setSearchFilter(filter: Partial<SearchFilter>): void {
    fileManagerState.update(state => ({
      ...state,
      searchFilter: {
        ...state.searchFilter,
        ...filter
      }
    }));
  }

  /**
   * 폴더 트리 로드
   */
  static async loadFolderTree(): Promise<void> {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      
      const folderTree = await invoke('get_folder_tree') as FolderEntry[];

      fileManagerState.update(state => ({
        ...state,
        folderTree: folderTree || []
      }));

    } catch (error) {
      console.error('폴더 트리 로드 실패:', error);
      
      // 실패 시 빈 배열로 초기화
      fileManagerState.update(state => ({
        ...state,
        folderTree: []
      }));
      
      addToast({
        type: 'warning',
        title: '폴더 트리 로드 실패',
        message: '폴더 구조를 불러올 수 없어 기본 구조로 초기화했습니다.',
        duration: 5000
      });
    }
  }

  /**
   * 새로고침 - 현재 폴더 유지
   */
  static async refresh(): Promise<void> {
    const currentState = get(fileManagerState);
    const currentFolderId = currentState.currentFolder?.id || null;
    
    // 현재 폴더를 유지하면서 새로고침
    await this.navigateToFolder(currentFolderId);
    await this.loadFolderTree();
  }

  /**
   * 에러 상태 초기화
   */
  static clearError(): void {
    fileManagerState.update(state => ({
      ...state,
      error: null
    }));
  }
}

// 초기화 함수
export async function initializeFileManager(): Promise<void> {
  await FileManagerService.loadFolderTree();
  await FileManagerService.navigateToFolder(null); // 루트 폴더로 이동
}