<!--
  폴더 추가 다이얼로그 컴포넌트
  외부 폴더를 볼트에 추가하기 위한 다이얼로그
-->
<script lang="ts">
  import Modal from '$lib/components/common/Modal.svelte';
  import Button from '$lib/components/common/Button.svelte';
  import { fileManagerState } from '$lib/stores/file-manager';
  import type { FolderEntry } from '$lib/types/file-manager';

  // Props 인터페이스 정의
  interface Props {
    isOpen: boolean;
    onConfirm?: (folderId?: string | null) => void;
    onCancel?: () => void;
  }

  // Props 받기
  const { isOpen, onConfirm, onCancel }: Props = $props();

  // 상태 변수
  let selectedFolderId = $state<string | null>(null);

  // 반응형 상태
  const folderTree = $derived($fileManagerState.folderTree);
  const currentFolder = $derived($fileManagerState.currentFolder);

  // 다이얼로그가 열릴 때마다 초기화
  $effect(() => {
    if (isOpen) {
      selectedFolderId = currentFolder?.id || null;
    }
  });

  // 확인 처리
  function handleConfirm() {
    if (onConfirm) {
      onConfirm(selectedFolderId);
    }
  }

  // 취소 처리
  function handleCancel() {
    if (onCancel) {
      onCancel();
    }
  }

  // 폴더 트리를 평면 목록으로 변환 (드롭다운용)
  function flattenFolderTree(folders: FolderEntry[], level: number = 0): Array<{id: string | null, name: string, path: string, level: number}> {
    const result: Array<{id: string | null, name: string, path: string, level: number}> = [];
    
    // 루트 폴더 추가 (레벨 0일 때만)
    if (level === 0) {
      result.push({
        id: null,
        name: '루트 폴더',
        path: '/',
        level: 0
      });
    }
    
    for (const folder of folders) {
      result.push({
        id: folder.id,
        name: folder.name,
        path: folder.path,
        level: level + 1
      });
      
      // 하위 폴더가 있으면 재귀적으로 추가
      if (folder.children && folder.children.length > 0) {
        result.push(...flattenFolderTree(folder.children, level + 1));
      }
    }
    
    return result;
  }

  // 폴더 목록 가져오기
  const availableFolders = $derived(flattenFolderTree(folderTree));
</script>

<Modal 
  {isOpen} 
  title="폴더 추가" 
  size="small"
  showFooter={false}
  onClose={handleCancel}
>
  <div class="add-folder-dialog">
    <!-- 폴더 아이콘 -->
    <div class="flex items-center justify-center mb-6">
      <div class="w-16 h-16 bg-green-100 rounded-full flex items-center justify-center">
        <svg class="w-8 h-8 text-green-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
                d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2H5a2 2 0 00-2 2z"/>
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
                d="M8 5a2 2 0 012-2h4a2 2 0 012 2v2H8V5z"/>
        </svg>
      </div>
    </div>

    <!-- 안내 메시지 -->
    <div class="text-center mb-6">
      <h3 class="text-lg font-medium text-gray-900 mb-2 text-korean">
        폴더를 추가할 위치를 선택하세요
      </h3>
      <p class="text-sm text-gray-600 text-korean">
        컴퓨터에서 폴더를 선택하여 볼트에 추가합니다.
      </p>
    </div>

    <!-- 대상 폴더 선택 -->
    <div class="folder-selection mb-6">
      <label class="block text-sm font-medium text-gray-700 mb-2 text-korean">
        대상 폴더
      </label>
      <select
        bind:value={selectedFolderId}
        class="w-full px-3 py-2 text-sm border border-gray-300 rounded-md focus:ring-2 focus:ring-green-500 focus:border-green-500"
      >
        {#each availableFolders as folder}
          <option value={folder.id}>
            {'  '.repeat(folder.level)}
            {folder.level > 0 ? '└ ' : ''}
            {folder.name}
            {folder.path !== '/' ? ` (${folder.path})` : ''}
          </option>
        {/each}
      </select>
    </div>

    <!-- 버튼 영역 -->
    <div class="flex justify-end gap-3 pt-4 border-t border-gray-200">
      <Button
        variant="outline"
        onclick={handleCancel}
      >
        취소
      </Button>
      
      <Button
        variant="primary"
        onclick={handleConfirm}
      >
        폴더 선택
      </Button>
    </div>
  </div>
</Modal>

<style>
  .add-folder-dialog {
    min-width: 400px;
  }

  /* 한국어 텍스트 최적화 */
  .text-korean {
    word-break: keep-all;
    line-height: 1.5;
  }

  /* 폴더 선택 드롭다운 포커스 스타일 */
  :global(.add-folder-dialog select:focus) {
    ring-color: #10b981;
    border-color: #10b981;
  }
</style>