<!--
  상태 표시줄 컴포넌트
  C# MainForm의 StatusStrip 기능을 포팅
-->
<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { fileManagerState, filteredFiles, sortedFolders, selectedItemsCount } from '$lib/stores/file-manager';
  import { authState } from '$lib/stores/auth';

  // 반응형 상태 변수들
  let files = [];
  let folders = [];
  let selectionCount = 0;
  let sessionTime = 0;
  let currentFolder = null;
  let isLoading = false;

  // 현재 시간 상태
  let currentTime = new Date();
  let timeInterval: number;

  // 스토어 구독
  let unsubscribeFiles: () => void;
  let unsubscribeFolders: () => void;
  let unsubscribeSelection: () => void;
  let unsubscribeAuth: () => void;
  let unsubscribeFileManager: () => void;

  // 컴포넌트 마운트 시 초기화
  onMount(() => {
    // 스토어 구독 설정
    unsubscribeFiles = filteredFiles.subscribe(value => {
      files = value;
    });
    
    unsubscribeFolders = sortedFolders.subscribe(value => {
      folders = value;
    });
    
    unsubscribeSelection = selectedItemsCount.subscribe(value => {
      selectionCount = value;
    });
    
    unsubscribeAuth = authState.subscribe(value => {
      sessionTime = value.sessionRemainingTime;
    });
    
    unsubscribeFileManager = fileManagerState.subscribe(value => {
      currentFolder = value.currentFolder;
      isLoading = value.isLoading;
    });

    // 시간 업데이트 시작
    timeInterval = setInterval(() => {
      currentTime = new Date();
    }, 1000);
  });

  // 컴포넌트 언마운트 시 정리
  onDestroy(() => {
    if (timeInterval) {
      clearInterval(timeInterval);
    }
    
    // 스토어 구독 해제
    if (unsubscribeFiles) unsubscribeFiles();
    if (unsubscribeFolders) unsubscribeFolders();
    if (unsubscribeSelection) unsubscribeSelection();
    if (unsubscribeAuth) unsubscribeAuth();
    if (unsubscribeFileManager) unsubscribeFileManager();
  });

  // 총 파일 크기 계산
  $: totalSize = (() => {
    const fileSize = files.reduce((sum, file) => sum + (file.fileSize || 0), 0);
    const folderSize = folders.reduce((sum, folder) => sum + (folder.totalSize || 0), 0);
    return fileSize + folderSize;
  })();

  // 파일 크기 포맷팅
  function formatFileSize(bytes: number): string {
    if (bytes === 0) return '0 B';
    
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    
    return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
  }

  // 시간 포맷팅
  function formatTime(date: Date): string {
    return date.toLocaleTimeString('ko-KR', {
      hour: '2-digit',
      minute: '2-digit',
      second: '2-digit'
    });
  }

  // 세션 시간 포맷팅
  function formatSessionTime(seconds: number): string {
    const minutes = Math.floor(seconds / 60);
    const remainingSeconds = seconds % 60;
    return `${minutes}:${remainingSeconds.toString().padStart(2, '0')}`;
  }

  // 상태 메시지 생성
  $: statusMessage = (() => {
    if (isLoading) {
      return '로딩 중...';
    }
    
    if (selectionCount > 0) {
      return `${selectionCount}개 항목 선택됨`;
    }
    
    const totalItems = files.length + folders.length;
    if (totalItems === 0) {
      return '항목 없음';
    }
    
    const folderText = folders.length > 0 ? `폴더 ${folders.length}개` : '';
    const fileText = files.length > 0 ? `파일 ${files.length}개` : '';
    
    if (folderText && fileText) {
      return `${folderText}, ${fileText}`;
    }
    
    return folderText || fileText;
  })();

  // 세션 시간 경고 색상
  $: sessionTimeColor = (() => {
    if (sessionTime <= 300) return 'text-red-600'; // 5분 이하
    if (sessionTime <= 600) return 'text-orange-600'; // 10분 이하
    return 'text-gray-600';
  })();
</script>

<!-- 상태 표시줄 -->
<div class="status-bar bg-gray-50 border-t border-gray-200 px-4 py-2">
  <div class="flex items-center justify-between text-sm">
    <!-- 왼쪽: 상태 정보 -->
    <div class="flex items-center space-x-6">
      <!-- 현재 상태 -->
      <div class="flex items-center">
        {#if isLoading}
          <div class="animate-spin w-3 h-3 border border-blue-600 border-t-transparent rounded-full mr-2"></div>
        {:else}
          <div class="w-2 h-2 bg-green-500 rounded-full mr-2"></div>
        {/if}
        <span class="text-gray-700 text-korean">{statusMessage}</span>
      </div>

      <!-- 총 크기 -->
      {#if totalSize > 0}
        <div class="flex items-center text-gray-600">
          <svg class="w-3 h-3 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
                  d="M4 7v10c0 2.21 3.582 4 8 4s8-1.79 8-4V7M4 7c0 2.21 3.582 4 8 4s8-1.79 8-4M4 7c0-2.21 3.582-4 8-4s8 1.79 8 4"/>
          </svg>
          <span class="text-korean">총 크기: {formatFileSize(totalSize)}</span>
        </div>
      {/if}

      <!-- 현재 폴더 경로 -->
      {#if currentFolder}
        <div class="flex items-center text-gray-600">
          <svg class="w-3 h-3 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
                  d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2H5a2 2 0 00-2 2z"/>
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
                  d="M8 5a2 2 0 012-2h4a2 2 0 012 2v2H8V5z"/>
          </svg>
          <span class="font-mono text-xs">{currentFolder.path}</span>
        </div>
      {/if}
    </div>

    <!-- 오른쪽: 시간 및 세션 정보 -->
    <div class="flex items-center space-x-4">
      <!-- 세션 남은 시간 -->
      {#if sessionTime > 0}
        <div class="flex items-center {sessionTimeColor}">
          <svg class="w-3 h-3 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
                  d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z"/>
          </svg>
          <span class="text-korean">세션: {formatSessionTime(sessionTime)}</span>
        </div>
      {/if}

      <!-- 현재 시간 -->
      <div class="flex items-center text-gray-600">
        <svg class="w-3 h-3 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
                d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z"/>
        </svg>
        <span class="font-mono">{formatTime(currentTime)}</span>
      </div>

      <!-- 암호화 상태 -->
      <div class="flex items-center text-green-600">
        <svg class="w-3 h-3 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
                d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z"/>
        </svg>
        <span class="text-korean">암호화 활성</span>
      </div>
    </div>
  </div>

  <!-- 진행률 표시 (로딩 중일 때) -->
  {#if isLoading}
    <div class="mt-1">
      <div class="w-full bg-gray-200 rounded-full h-1">
        <div class="bg-blue-600 h-1 rounded-full animate-pulse" style="width: 100%"></div>
      </div>
    </div>
  {/if}
</div>

<style>
  .status-bar {
    user-select: none;
    font-size: 0.8125rem;
    min-height: 2.5rem;
  }

  /* 한국어 텍스트 최적화 */
  .text-korean {
    word-break: keep-all;
    line-height: 1.4;
  }

  /* 애니메이션 */
  .animate-pulse {
    animation: pulse 2s cubic-bezier(0.4, 0, 0.6, 1) infinite;
  }

  @keyframes pulse {
    0%, 100% {
      opacity: 1;
    }
    50% {
      opacity: 0.5;
    }
  }

  /* 반응형 디자인 */
  @media (max-width: 1024px) {
    .status-bar .flex {
      flex-wrap: wrap;
      gap: 0.5rem;
    }
  }

  @media (max-width: 768px) {
    .status-bar {
      padding: 0.5rem;
    }
    
    .status-bar .flex {
      flex-direction: column;
      align-items: flex-start;
      space-x: 0;
      gap: 0.25rem;
    }
    
    .status-bar .flex > div {
      flex-wrap: wrap;
      gap: 0.5rem;
    }
  }

  @media (max-width: 640px) {
    .status-bar {
      font-size: 0.75rem;
    }
    
    /* 작은 화면에서는 일부 정보 숨김 */
    .status-bar .flex > div:first-child > div:nth-child(n+3) {
      display: none;
    }
  }
</style>