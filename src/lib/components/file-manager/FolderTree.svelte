<!--
  폴더 트리 컴포넌트
  C# MainForm의 TreeView 기능을 포팅
-->
<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { fileManagerState, FileManagerService } from '$lib/stores/file-manager';
  import type { FolderEntry } from '$lib/types/file-manager';

  // 반응형 상태
  const folderTree = $derived($fileManagerState.folderTree);
  const currentFolder = $derived($fileManagerState.currentFolder);

  // 확장된 폴더 상태 관리
  let expandedFolders = $state(new Set<string>(['root']));

  onMount(() => {
    // 전역 드래그 방지 이벤트 리스너 추가
    const preventDrag = (e: DragEvent) => {
      e.preventDefault();
      e.stopPropagation();
      return false;
    };
    
    const preventDragStart = (e: DragEvent) => {
      e.preventDefault();
      e.stopPropagation();
      return false;
    };
    
    const preventSelectStart = (e: Event) => {
      const target = e.target as HTMLElement;
      // HTMLElement인지 확인
      if (target && typeof target.closest === 'function') {
        // 폴더 트리 영역에서는 텍스트 선택 완전 방지
        if (target.closest('.folder-tree')) {
          e.preventDefault();
          return false;
        }
      }
    };
    
    // 드래그 관련 이벤트 완전 차단
    document.addEventListener('dragstart', preventDragStart, true);
    document.addEventListener('drag', preventDrag, true);
    document.addEventListener('dragenter', preventDrag, true);
    document.addEventListener('dragover', preventDrag, true);
    document.addEventListener('dragleave', preventDrag, true);
    document.addEventListener('drop', preventDrag, true);
    document.addEventListener('dragend', preventDrag, true);
    document.addEventListener('selectstart', preventSelectStart, true);
  });

  onDestroy(() => {
    // 드래그 방지 이벤트 리스너 제거
    const preventDrag = (e: DragEvent) => {
      e.preventDefault();
      e.stopPropagation();
      return false;
    };
    
    const preventDragStart = (e: DragEvent) => {
      e.preventDefault();
      e.stopPropagation();
      return false;
    };
    
    const preventSelectStart = (e: Event) => {
      const target = e.target as HTMLElement;
      if (target.closest('.folder-tree')) {
        e.preventDefault();
        return false;
      }
    };
    
    document.removeEventListener('dragstart', preventDragStart, true);
    document.removeEventListener('drag', preventDrag, true);
    document.removeEventListener('dragenter', preventDrag, true);
    document.removeEventListener('dragover', preventDrag, true);
    document.removeEventListener('dragleave', preventDrag, true);
    document.removeEventListener('drop', preventDrag, true);
    document.removeEventListener('dragend', preventDrag, true);
    document.removeEventListener('selectstart', preventSelectStart, true);
  });

  // 폴더 확장/축소 토글
  function toggleFolder(folderId: string) {
    if (expandedFolders.has(folderId)) {
      expandedFolders.delete(folderId);
    } else {
      expandedFolders.add(folderId);
    }
    expandedFolders = new Set(expandedFolders); // 반응성 트리거
  }

  // 폴더 선택
  async function selectFolder(folderId: string | null) {
    await FileManagerService.navigateToFolder(folderId);
  }

  // 폴더 선택 (클릭 시 선택 상태 변경)
  function handleFolderClick(event: MouseEvent, folderId: string) {
    // Ctrl/Cmd 키를 누르고 클릭하면 선택 상태만 변경 (네비게이션 하지 않음)
    if (event.ctrlKey || event.metaKey) {
      event.preventDefault();
      const multiSelect = event.shiftKey;
      FileManagerService.selectItem(folderId, 'folder', multiSelect);
    } else {
      // 일반 클릭은 네비게이션
      selectFolder(folderId);
    }
  }

  // 폴더 단순 선택 (네비게이션 없이 선택만)
  function handleFolderSelect(event: MouseEvent, folderId: string) {
    event.preventDefault();
    event.stopPropagation();
    const multiSelect = event.ctrlKey || event.metaKey || event.shiftKey;
    FileManagerService.selectItem(folderId, 'folder', multiSelect);
  }

  // 폴더가 현재 선택된 폴더인지 확인
  function isCurrentFolder(folderId: string | null): boolean {
    if (folderId === null) {
      return currentFolder === null;
    }
    return currentFolder?.id === folderId;
  }

  // 폴더가 선택 상태인지 확인
  function isFolderSelected(folderId: string): boolean {
    const selection = $fileManagerState.selection;
    return selection.selectedFolders.has(folderId);
  }

  // 폴더 아이콘 가져오기
  function getFolderIcon(isExpanded: boolean) {
    return isExpanded ? '📂' : '📁';
  }

  // 폴더 컨텍스트 메뉴 처리 (우클릭) - 현재 비활성화
  function handleFolderContextMenu(event: MouseEvent, folderId: string) {
    event.preventDefault();
    // 컨텍스트 메뉴 기능 비활성화 - 툴바 삭제 버튼 사용
    console.log('폴더 우클릭:', folderId, '- 컨텍스트 메뉴 비활성화됨');
  }

  // 재귀적으로 폴더 렌더링
  function renderFolder(folder: FolderEntry, level: number) {
    const isExpanded = expandedFolders.has(folder.id);
    const isCurrent = isCurrentFolder(folder.id);
    const isSelected = isFolderSelected(folder.id);
    
    return {
      folder,
      level,
      isExpanded,
      isCurrent,
      isSelected,
      hasChildren: folder.children && folder.children.length > 0
    };
  }
</script>

<!-- 폴더 트리 -->
<div class="folder-tree">
  <!-- 루트 폴더 -->
  <div class="folder-item no-drag {isCurrentFolder(null) ? 'current' : ''}" style="padding-left: 12px" draggable="false">
    <div class="folder-content no-drag">
      <div class="expand-spacer"></div>
      <button
        class="folder-button no-drag"
        onclick={() => selectFolder(null)}
        title="루트 폴더"
        draggable="false"
      >
        <span class="folder-icon no-drag" draggable="false">🏠</span>
        <span class="folder-name text-korean no-drag">루트 폴더</span>
      </button>
    </div>
  </div>

  {#if folderTree.length === 0}
    <!-- 빈 상태 -->
    <div class="p-4 text-center text-gray-500">
      <p class="text-xs text-korean">폴더가 없습니다</p>
    </div>
  {:else}
    <!-- 폴더 트리 렌더링 -->
    <div class="folder-list">
      {#each folderTree as folder}
        {@const rendered = renderFolder(folder, 1)}
        
        <!-- 폴더 아이템 -->
        <div 
          class="folder-item no-drag {rendered.isCurrent ? 'current' : ''} {rendered.isSelected ? 'selected' : ''}"
          style="padding-left: {rendered.level * 20 + 12}px"
          draggable="false"
        >
          <div class="folder-content no-drag">
            <!-- 선택 체크박스 (맨 앞) -->
            <button
              class="folder-select-button no-drag"
              onclick={(e) => handleFolderSelect(e, folder.id)}
              title="폴더 선택"
              draggable="false"
            >
              <span class="select-indicator no-drag {rendered.isSelected ? 'selected' : ''}">
                {#if rendered.isSelected}✓{:else}○{/if}
              </span>
            </button>

            <!-- 확장/축소 버튼 -->
            {#if rendered.hasChildren}
              <button
                class="expand-button no-drag"
                onclick={() => toggleFolder(folder.id)}
                title={rendered.isExpanded ? '축소' : '확장'}
                draggable="false"
              >
                <svg 
                  class="w-3 h-3 transition-transform no-drag {rendered.isExpanded ? 'rotate-90' : ''}"
                  fill="none" 
                  stroke="currentColor" 
                  viewBox="0 0 24 24"
                  draggable="false"
                >
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7"/>
                </svg>
              </button>
            {:else}
              <div class="expand-spacer no-drag"></div>
            {/if}

            <!-- 폴더 네비게이션 버튼 -->
            <button
              class="folder-button no-drag"
              onclick={(e) => handleFolderClick(e, folder.id)}
              oncontextmenu={(e) => handleFolderContextMenu(e, folder.id)}
              title="{folder.path} (Ctrl+클릭: 선택)"
              draggable="false"
            >
              <span class="folder-icon no-drag" draggable="false">
                {getFolderIcon(rendered.isExpanded)}
              </span>
              <span class="folder-name text-korean no-drag">{folder.name}</span>
              {#if folder.file_count > 0}
                <span class="file-count no-drag">({folder.file_count})</span>
              {/if}
            </button>
          </div>
        </div>

        <!-- 하위 폴더들 (재귀) -->
        {#if rendered.isExpanded && folder.children}
          {#each folder.children as childFolder}
            {@const childRendered = renderFolder(childFolder, rendered.level + 1)}
            
            <div 
              class="folder-item no-drag {childRendered.isCurrent ? 'current' : ''} {childRendered.isSelected ? 'selected' : ''}"
              style="padding-left: {childRendered.level * 20 + 12}px"
              draggable="false"
            >
              <div class="folder-content no-drag">
                <!-- 선택 체크박스 (맨 앞) -->
                <button
                  class="folder-select-button no-drag"
                  onclick={(e) => handleFolderSelect(e, childFolder.id)}
                  title="폴더 선택"
                  draggable="false"
                >
                  <span class="select-indicator no-drag {childRendered.isSelected ? 'selected' : ''}">
                    {#if childRendered.isSelected}✓{:else}○{/if}
                  </span>
                </button>

                <!-- 확장/축소 버튼 -->
                {#if childRendered.hasChildren}
                  <button
                    class="expand-button"
                    onclick={() => toggleFolder(childFolder.id)}
                    title={childRendered.isExpanded ? '축소' : '확장'}
                  >
                    <svg 
                      class="w-3 h-3 transition-transform {childRendered.isExpanded ? 'rotate-90' : ''}"
                      fill="none" 
                      stroke="currentColor" 
                      viewBox="0 0 24 24"
                    >
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7"/>
                    </svg>
                  </button>
                {:else}
                  <div class="expand-spacer"></div>
                {/if}

                <!-- 폴더 네비게이션 버튼 -->
                <button
                  class="folder-button"
                  onclick={(e) => handleFolderClick(e, childFolder.id)}
                  oncontextmenu={(e) => handleFolderContextMenu(e, childFolder.id)}
                  title="{childFolder.path} (Ctrl+클릭: 선택)"
                >
                  <span class="folder-icon">
                    {getFolderIcon(childRendered.isExpanded)}
                  </span>
                  <span class="folder-name text-korean">{childFolder.name}</span>
                  {#if childFolder.file_count > 0}
                    <span class="file-count">({childFolder.file_count})</span>
                  {/if}
                </button>
              </div>
            </div>
          {/each}
        {/if}
      {/each}
    </div>
  {/if}
</div>

<style>
  .folder-tree {
    height: 100%;
    overflow-y: auto;
  }

  .folder-list {
    padding: 0.5rem 0;
  }

  .folder-item {
    position: relative;
  }

  .folder-content {
    display: flex;
    align-items: center;
    padding: 0.25rem 0;
    border-radius: 0.375rem;
    margin: 0 0.5rem;
    transition: background-color 0.15s ease;
  }

  .folder-item:hover .folder-content {
    background-color: #f3f4f6;
  }

  .folder-item.current .folder-content {
    background-color: #dbeafe;
    border: 1px solid #93c5fd;
  }

  .expand-button {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 1rem;
    height: 1rem;
    margin-right: 0.25rem;
    color: #6b7280;
    background: none;
    border: none;
    cursor: pointer;
    border-radius: 0.125rem;
    transition: color 0.15s ease;
  }

  .expand-button:hover {
    color: #374151;
    background-color: #f3f4f6;
  }

  .expand-spacer {
    width: 1rem;
    height: 1rem;
    margin-right: 0.25rem;
  }

  .folder-button {
    display: flex;
    align-items: center;
    flex: 1;
    padding: 0.25rem 0.5rem;
    background: none;
    border: none;
    cursor: pointer;
    border-radius: 0.25rem;
    text-align: left;
    transition: background-color 0.15s ease;
  }

  .folder-button:hover {
    background-color: #f9fafb;
  }

  /* 폴더 버튼 컨테이너 */
  .folder-button-container {
    display: flex;
    align-items: center;
    flex: 1;
    gap: 0.25rem;
  }

  /* 폴더 선택 버튼 */
  .folder-select-button {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 1.25rem;
    height: 1.25rem;
    background: none;
    border: none;
    cursor: pointer;
    border-radius: 0.25rem;
    transition: background-color 0.15s ease;
  }

  .folder-select-button:hover {
    background-color: #e5e7eb;
  }

  /* 선택 표시 */
  .select-indicator {
    font-size: 0.75rem;
    color: #6b7280;
    transition: color 0.15s ease;
  }

  .select-indicator.selected {
    color: #2563eb;
    font-weight: bold;
  }

  .folder-icon {
    margin-right: 0.5rem;
    font-size: 0.875rem;
  }

  .folder-name {
    flex: 1;
    font-size: 0.875rem;
    color: #374151;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .file-count {
    font-size: 0.75rem;
    color: #6b7280;
    margin-left: 0.25rem;
  }

  /* 한국어 텍스트 최적화 */
  .text-korean {
    word-break: keep-all;
    line-height: 1.4;
  }

  /* 스크롤바 스타일링 */
  .folder-tree::-webkit-scrollbar {
    width: 4px;
  }

  .folder-tree::-webkit-scrollbar-track {
    background: transparent;
  }

  .folder-tree::-webkit-scrollbar-thumb {
    background: #d1d5db;
    border-radius: 2px;
  }

  .folder-tree::-webkit-scrollbar-thumb:hover {
    background: #9ca3af;
  }

  /* 포커스 스타일 */
  .expand-button:focus,
  .folder-button:focus {
    outline: 2px solid #3b82f6;
    outline-offset: 1px;
  }

  /* 애니메이션 */
  .expand-button svg {
    transition: transform 0.2s ease;
  }

  /* 반응형 디자인 */
  @media (max-width: 768px) {
    .folder-content {
      margin: 0 0.25rem;
    }
    
    .folder-name {
      font-size: 0.8125rem;
    }
    
    .file-count {
      font-size: 0.6875rem;
    }
  }
</style>