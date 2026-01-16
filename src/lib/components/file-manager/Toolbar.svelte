<!--
  파일 매니저 툴바 컴포넌트
  C# MainForm의 메뉴 및 툴바 기능을 포팅
-->
<script lang="ts">
  import { fileManagerState, FileManagerService } from '$lib/stores/file-manager';
  import Button from '$lib/components/common/Button.svelte';
  import type { FileViewMode } from '$lib/types/file-manager';

  // Props 정의
  interface Props {
    onAddFile: () => void;
    onAddFolder: () => void;
    onCreateFile: () => void;
    onCreateFolder: () => void;
    onDeleteSelected: () => void;
    onExportSelected: () => void;
    onRenameSelected: () => void;
    hasSelection: boolean;
    selectionCount: number;
    hasFileSelection: boolean;
    selectedFileCount: number;
  }

  const {
    onAddFile,
    onAddFolder,
    onCreateFile,
    onCreateFolder,
    onDeleteSelected,
    onExportSelected,
    onRenameSelected,
    hasSelection,
    selectionCount,
    hasFileSelection,
    selectedFileCount
  }: Props = $props();

  // 반응형 상태
  const viewMode = $derived($fileManagerState.viewMode);
  const sortOption = $derived($fileManagerState.sortOption);
  const searchQuery = $derived($fileManagerState.searchFilter.query);

  // 검색 입력 상태
  let searchInput = $state('');

  // 뷰 모드 변경
  function handleViewModeChange(mode: FileViewMode) {
    FileManagerService.setViewMode(mode);
  }

  // 정렬 변경
  function handleSortChange(field: string) {
    const newDirection = sortOption.field === field && sortOption.direction === 'asc' ? 'desc' : 'asc';
    FileManagerService.setSortOption({
      field: field as any,
      direction: newDirection
    });
  }

  // 검색 실행
  function handleSearch() {
    FileManagerService.setSearchFilter({ query: searchInput.trim() });
  }

  // 검색 초기화
  function handleClearSearch() {
    searchInput = '';
    FileManagerService.setSearchFilter({ query: '' });
  }

  // Enter 키로 검색
  function handleSearchKeyDown(event: KeyboardEvent) {
    if (event.key === 'Enter') {
      handleSearch();
    }
  }

  // 모든 선택 해제
  function handleClearSelection() {
    FileManagerService.clearSelection();
  }
</script>

<!-- 툴바 -->
<div class="toolbar bg-white border-b border-gray-200 px-4 py-3">
  <div class="flex items-center justify-between">
    <!-- 왼쪽: 주요 액션 버튼들 -->
    <div class="flex items-center space-x-2">
      <!-- 파일 추가 -->
      <Button
        variant="primary"
        size="small"
        onclick={onAddFile}
        title="기존 파일 추가"
      >
        <svg class="w-4 h-4 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
                d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"/>
        </svg>
        <span class="text-korean">파일 추가</span>
      </Button>

      <!-- 폴더 추가 -->
      <Button
        variant="primary"
        size="small"
        onclick={onAddFolder}
        title="기존 폴더 추가 (내부 파일 포함)"
      >
        <svg class="w-4 h-4 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
                d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2H5a2 2 0 00-2 2z"/>
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
                d="M8 5a2 2 0 012-2h4a2 2 0 012 2v2H8V5z"/>
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
                d="M12 11v4m-2-2h4"/>
        </svg>
        <span class="text-korean">폴더 추가</span>
      </Button>

      <!-- 구분선 -->
      <div class="h-6 w-px bg-gray-300"></div>

      <!-- 파일 생성 -->
      <Button
        variant="secondary"
        size="small"
        onclick={onCreateFile}
        title="파일 생성"
      >
        <svg class="w-4 h-4 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
                d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"/>
        </svg>
        <span class="text-korean">파일 생성</span>
      </Button>

      <!-- 폴더 생성 -->
      <Button
        variant="secondary"
        size="small"
        onclick={onCreateFolder}
        title="새 폴더 생성"
      >
        <svg class="w-4 h-4 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
                d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2H5a2 2 0 00-2 2z"/>
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
                d="M8 5a2 2 0 012-2h4a2 2 0 012 2v2H8V5z"/>
        </svg>
        <span class="text-korean">폴더 생성</span>
      </Button>

      <!-- 구분선 -->
      <div class="h-6 w-px bg-gray-300"></div>

      <!-- 이름 변경 -->
      <Button
        variant="secondary"
        size="small"
        onclick={onRenameSelected}
        disabled={!hasSelection || selectionCount !== 1}
        title={hasSelection && selectionCount === 1 ? '선택된 항목 이름 변경 (F2)' : '이름을 변경할 항목을 하나만 선택하세요'}
      >
        <svg class="w-4 h-4 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
                d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z"/>
        </svg>
        <span class="text-korean">이름 변경</span>
      </Button>

      <!-- 삭제 -->
      <Button
        variant="danger"
        size="small"
        onclick={onDeleteSelected}
        disabled={!hasSelection}
        title={hasSelection ? `선택된 ${selectionCount}개 항목 삭제` : '삭제할 항목을 선택하세요'}
      >
        <svg class="w-4 h-4 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
                d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"/>
        </svg>
        <span class="text-korean">삭제</span>
      </Button>

      <!-- 내보내기 -->
      <Button
        variant="secondary"
        size="small"
        onclick={onExportSelected}
        disabled={!hasSelection}
        title={hasSelection ? `선택된 ${selectionCount}개 파일 내보내기` : '내보낼 파일을 선택하세요'}
      >
        <svg class="w-4 h-4 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
                d="M12 10v6m0 0l-3-3m3 3l3-3m2 8H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"/>
        </svg>
        <span class="text-korean">내보내기</span>
      </Button>
    </div>

    <!-- 오른쪽: 검색 및 뷰 옵션 -->
    <div class="flex items-center space-x-4">
      <!-- 검색 -->
      <div class="flex items-center space-x-2">
        <div class="relative">
          <input
            type="text"
            bind:value={searchInput}
            onkeydown={handleSearchKeyDown}
            placeholder="파일 검색..."
            class="w-64 px-3 py-1.5 text-sm text-gray-900 bg-white border border-gray-300 rounded-md focus:ring-2 focus:ring-blue-500 focus:border-blue-500 pr-8 placeholder-gray-500"
          />
          {#if searchInput}
            <button
              onclick={handleClearSearch}
              class="absolute right-2 top-1/2 transform -translate-y-1/2 text-gray-400 hover:text-gray-600"
              title="검색 초기화"
            >
              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
                      d="M6 18L18 6M6 6l12 12"/>
              </svg>
            </button>
          {/if}
        </div>
        <Button
          variant="outline"
          size="small"
          onclick={handleSearch}
          title="검색 실행"
        >
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
                  d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"/>
          </svg>
        </Button>
      </div>

      <!-- 구분선 -->
      <div class="h-6 w-px bg-gray-300"></div>

      <!-- 정렬 옵션 -->
      <div class="flex items-center space-x-1">
        <span class="text-sm text-gray-600 text-korean">정렬:</span>
        <select
          value={sortOption.field}
          onchange={(e) => handleSortChange(e.target.value)}
          class="text-sm text-gray-900 bg-white border border-gray-300 rounded px-2 py-1 focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
        >
          <option value="name">이름</option>
          <option value="size">크기</option>
          <option value="type">형식</option>
          <option value="modified">수정일</option>
          <option value="created">생성일</option>
        </select>
        <button
          onclick={() => handleSortChange(sortOption.field)}
          class="p-1 text-gray-500 hover:text-gray-700"
          title={sortOption.direction === 'asc' ? '오름차순' : '내림차순'}
        >
          {#if sortOption.direction === 'asc'}
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
                    d="M3 4h13M3 8h9m-9 4h6m4 0l4-4m0 0l4 4m-4-4v12"/>
            </svg>
          {:else}
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
                    d="M3 4h13M3 8h9m-9 4h9m5-4v12m0 0l-4-4m4 4l4-4"/>
            </svg>
          {/if}
        </button>
      </div>

      <!-- 뷰 모드 -->
      <div class="flex items-center space-x-1 bg-gray-100 rounded-md p-1">
        <button
          onclick={() => handleViewModeChange('list')}
          class={`p-1.5 rounded ${viewMode === 'list' ? 'bg-white shadow-sm text-blue-600' : 'text-gray-500 hover:text-gray-700'}`}
          title="목록 보기"
        >
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
                  d="M4 6h16M4 10h16M4 14h16M4 18h16"/>
          </svg>
        </button>
        <button
          onclick={() => handleViewModeChange('grid')}
          class={`p-1.5 rounded ${viewMode === 'grid' ? 'bg-white shadow-sm text-blue-600' : 'text-gray-500 hover:text-gray-700'}`}
          title="격자 보기"
        >
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
                  d="M4 6a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2H6a2 2 0 01-2-2V6zM14 6a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2h-2a2 2 0 01-2-2V6zM4 16a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2H6a2 2 0 01-2-2v-2zM14 16a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2h-2a2 2 0 01-2-2v-2z"/>
          </svg>
        </button>
        <button
          onclick={() => handleViewModeChange('details')}
          class={`p-1.5 rounded ${viewMode === 'details' ? 'bg-white shadow-sm text-blue-600' : 'text-gray-500 hover:text-gray-700'}`}
          title="자세히 보기"
        >
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
                  d="M4 6h16M4 10h16M4 14h16M4 18h16"/>
          </svg>
        </button>
      </div>
    </div>
  </div>

  <!-- 선택 상태 표시 -->
  {#if hasSelection}
    <div class="mt-2 flex items-center justify-between bg-blue-50 border border-blue-200 rounded-md px-3 py-2">
      <div class="flex items-center text-sm text-blue-700">
        <svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
                d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"/>
        </svg>
        <span class="text-korean">{selectionCount}개 항목이 선택됨</span>
      </div>
      <Button
        variant="ghost"
        size="small"
        onclick={handleClearSelection}
      >
        <span class="text-korean">선택 해제</span>
      </Button>
    </div>
  {/if}
</div>

<style>
  .toolbar {
    user-select: none;
  }

  /* 검색 입력 필드 스타일 */
  input[type="text"]:focus {
    outline: none;
  }

  /* 한국어 텍스트 최적화 */
  .text-korean {
    word-break: keep-all;
    line-height: 1.4;
    white-space: nowrap; /* 텍스트 줄바꿈 방지 */
  }

  /* 반응형 디자인 */
  @media (max-width: 1024px) {
    .toolbar .flex {
      flex-wrap: wrap;
      gap: 0.5rem;
    }
    
    input[type="text"] {
      width: 200px;
    }
  }

  @media (max-width: 768px) {
    .toolbar {
      padding: 0.75rem;
    }
    
    .toolbar .flex {
      flex-direction: column;
      align-items: stretch;
    }
    
    input[type="text"] {
      width: 100%;
    }
  }
</style>