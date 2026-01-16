// 볼트 관련 상태 관리
import { writable, derived } from 'svelte/store';
import type { VaultState, VaultConfig, ViewMode, SortBy, SortOrder } from '../types';

/**
 * 볼트 상태 스토어
 * 볼트 설정, 현재 경로, 선택된 파일, 보기 모드 등을 관리합니다.
 */
export const vaultStore = writable<VaultState>({
  config: undefined,
  currentPath: [],
  selectedFiles: [],
  viewMode: 'list',
  sortBy: 'name',
  sortOrder: 'asc',
});

/**
 * 현재 폴더 ID 파생 스토어
 * 현재 경로의 마지막 요소를 현재 폴더 ID로 반환합니다.
 */
export const currentFolderId = derived(
  vaultStore,
  $vault => $vault.currentPath.length > 0 
    ? $vault.currentPath[$vault.currentPath.length - 1] 
    : undefined
);

/**
 * 선택된 파일 개수 파생 스토어
 */
export const selectedFileCount = derived(
  vaultStore,
  $vault => $vault.selectedFiles.length
);

/**
 * 볼트 설정 업데이트
 * @param config - 새로운 볼트 설정
 */
export function updateVaultConfig(config: VaultConfig) {
  vaultStore.update(state => ({
    ...state,
    config,
  }));
}

/**
 * 현재 경로 변경
 * @param path - 새로운 경로
 */
export function navigateToPath(path: string[]) {
  vaultStore.update(state => ({
    ...state,
    currentPath: [...path],
    selectedFiles: [], // 경로 변경 시 선택 해제
  }));
}

/**
 * 폴더로 이동
 * @param folderId - 이동할 폴더 ID
 */
export function navigateToFolder(folderId: string) {
  vaultStore.update(state => ({
    ...state,
    currentPath: [...state.currentPath, folderId],
    selectedFiles: [],
  }));
}

/**
 * 상위 폴더로 이동
 */
export function navigateUp() {
  vaultStore.update(state => ({
    ...state,
    currentPath: state.currentPath.slice(0, -1),
    selectedFiles: [],
  }));
}

/**
 * 루트 폴더로 이동
 */
export function navigateToRoot() {
  vaultStore.update(state => ({
    ...state,
    currentPath: [],
    selectedFiles: [],
  }));
}

/**
 * 파일 선택/해제
 * @param fileId - 파일 ID
 * @param selected - 선택 여부
 */
export function toggleFileSelection(fileId: string, selected?: boolean) {
  vaultStore.update(state => {
    const isCurrentlySelected = state.selectedFiles.includes(fileId);
    const shouldSelect = selected !== undefined ? selected : !isCurrentlySelected;
    
    if (shouldSelect && !isCurrentlySelected) {
      return {
        ...state,
        selectedFiles: [...state.selectedFiles, fileId],
      };
    } else if (!shouldSelect && isCurrentlySelected) {
      return {
        ...state,
        selectedFiles: state.selectedFiles.filter(id => id !== fileId),
      };
    }
    
    return state;
  });
}

/**
 * 모든 파일 선택/해제
 * @param fileIds - 파일 ID 목록
 * @param selected - 선택 여부
 */
export function toggleAllFiles(fileIds: string[], selected: boolean) {
  vaultStore.update(state => ({
    ...state,
    selectedFiles: selected ? [...fileIds] : [],
  }));
}

/**
 * 보기 모드 변경
 * @param mode - 새로운 보기 모드
 */
export function setViewMode(mode: ViewMode) {
  vaultStore.update(state => ({
    ...state,
    viewMode: mode,
  }));
}

/**
 * 정렬 설정 변경
 * @param sortBy - 정렬 기준
 * @param sortOrder - 정렬 순서
 */
export function setSortSettings(sortBy: SortBy, sortOrder: SortOrder) {
  vaultStore.update(state => ({
    ...state,
    sortBy,
    sortOrder,
  }));
}

/**
 * 정렬 기준 변경 (순서는 자동 토글)
 * @param sortBy - 새로운 정렬 기준
 */
export function toggleSort(sortBy: SortBy) {
  vaultStore.update(state => {
    const newSortOrder = state.sortBy === sortBy && state.sortOrder === 'asc' 
      ? 'desc' 
      : 'asc';
    
    return {
      ...state,
      sortBy,
      sortOrder: newSortOrder,
    };
  });
}