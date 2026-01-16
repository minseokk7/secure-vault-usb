<script lang="ts">
  import { onMount } from 'svelte';
  import { getCurrentWindow } from '@tauri-apps/api/window';

  // Props (Svelte 5 runes 모드)
  interface Props {
    title?: string;
    subtitle?: string;
    showIcon?: boolean;
    showTitle?: boolean;
    variant?: 'gradient' | 'solid';
    showMaximize?: boolean;
  }

  const {
    title = 'SecureVault',
    subtitle = '보안 파일 매니저',
    showIcon = true,
    showTitle = true,
    variant = 'gradient',
    showMaximize = true
  }: Props = $props();

  let appWindow: any;
  let isMaximized = $state(false);

  // 컴포넌트 마운트 시 윈도우 객체 가져오기
  onMount(async () => {
    appWindow = getCurrentWindow();
    
    if (showMaximize) {
      // 현재 최대화 상태 확인
      isMaximized = await appWindow.isMaximized();
      
      // 윈도우 상태 변경 이벤트 리스너
      const unlistenResize = await appWindow.onResized(() => {
        checkMaximized();
      });

      // 컴포넌트 언마운트 시 리스너 정리
      return () => {
        unlistenResize();
      };
    }
  });

  // 최대화 상태 확인
  async function checkMaximized() {
    if (appWindow && showMaximize) {
      isMaximized = await appWindow.isMaximized();
    }
  }

  // 윈도우 최소화
  async function minimizeWindow() {
    if (appWindow) {
      await appWindow.minimize();
    }
  }

  // 윈도우 최대화/복원
  async function toggleMaximize() {
    if (appWindow && showMaximize) {
      await appWindow.toggleMaximize();
      await checkMaximized();
    }
  }

  // 윈도우 닫기
  async function closeWindow() {
    if (appWindow) {
      await appWindow.close();
    }
  }

  // 윈도우 드래그 시작
  async function startDrag() {
    if (appWindow) {
      await appWindow.startDragging();
    }
  }

  // 더블클릭으로 최대화/복원 (최대화 버튼이 활성화된 경우에만)
  function handleDoubleClick() {
    if (showMaximize) {
      toggleMaximize();
    }
  }
</script>

<!-- 커스텀 타이틀바 -->
<div 
  class="titlebar {variant}"
  data-tauri-drag-region
  ondblclick={handleDoubleClick}
>
  <!-- 왼쪽: 로고 및 제목 -->
  <div class="titlebar-left" data-tauri-drag-region>
    {#if showIcon}
      <div class="titlebar-icon">
        <svg class="w-6 h-6 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
                d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z"/>
        </svg>
      </div>
    {/if}
    
    {#if showTitle}
      <div class="titlebar-title" data-tauri-drag-region>
        <h1 class="title-main text-korean">{title}</h1>
        <p class="title-sub text-korean">{subtitle}</p>
      </div>
    {/if}
  </div>

  <!-- 오른쪽: 윈도우 컨트롤 버튼들 -->
  <div class="titlebar-controls">
    <!-- 최소화 버튼 -->
    <button 
      class="control-button minimize"
      onclick={minimizeWindow}
      title="최소화"
      type="button"
    >
      <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M20 12H4"/>
      </svg>
    </button>

    {#if showMaximize}
      <!-- 최대화/복원 버튼 -->
      <button 
        class="control-button maximize"
        onclick={toggleMaximize}
        title={isMaximized ? "복원" : "최대화"}
        type="button"
      >
        {#if isMaximized}
          <!-- 복원 아이콘 -->
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
                  d="M8 7V3m8 4V3m-9 8h10M5 21h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z"/>
          </svg>
        {:else}
          <!-- 최대화 아이콘 -->
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
                  d="M4 8V4a2 2 0 012-2h12a2 2 0 012 2v4M4 16v4a2 2 0 002 2h12a2 2 0 002-2v-4"/>
          </svg>
        {/if}
      </button>
    {/if}

    <!-- 닫기 버튼 -->
    <button 
      class="control-button close"
      onclick={closeWindow}
      title="닫기"
      type="button"
    >
      <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
              d="M6 18L18 6M6 6l12 12"/>
      </svg>
    </button>
  </div>
</div>

<style>
  /* 타이틀바 기본 스타일 */
  .titlebar {
    @apply flex items-center justify-between h-16 px-4 select-none;
    @apply border-b border-white border-opacity-20;
    position: relative;
    z-index: 1000;
  }

  /* 그라데이션 변형 */
  .titlebar.gradient {
    background: linear-gradient(135deg, #3b82f6 0%, #1d4ed8 50%, #1e40af 100%);
  }

  /* 단색 변형 */
  .titlebar.solid {
    @apply bg-blue-600;
  }

  /* 왼쪽 영역 */
  .titlebar-left {
    @apply flex items-center gap-3 flex-1;
  }

  /* 아이콘 */
  .titlebar-icon {
    @apply w-8 h-8 bg-white bg-opacity-20 rounded-lg flex items-center justify-center;
    @apply backdrop-blur-sm;
  }

  /* 제목 영역 */
  .titlebar-title {
    @apply flex flex-col;
  }

  .title-main {
    @apply text-lg font-bold text-white leading-tight;
  }

  .title-sub {
    @apply text-sm text-blue-100 leading-tight;
  }

  /* 컨트롤 버튼 영역 */
  .titlebar-controls {
    @apply flex items-center;
  }

  /* 컨트롤 버튼 기본 스타일 */
  .control-button {
    @apply w-12 h-12 flex items-center justify-center text-white;
    @apply hover:bg-white hover:bg-opacity-10 transition-colors duration-200;
    @apply focus:outline-none focus:ring-2 focus:ring-white focus:ring-opacity-20;
    border: none;
    background: transparent;
  }

  /* 최소화 버튼 */
  .control-button.minimize:hover {
    @apply bg-yellow-500 bg-opacity-20;
  }

  /* 최대화 버튼 */
  .control-button.maximize:hover {
    @apply bg-green-500 bg-opacity-20;
  }

  /* 닫기 버튼 */
  .control-button.close:hover {
    @apply bg-red-500 bg-opacity-30 text-red-100;
  }

  .control-button.close:active {
    @apply bg-red-600 bg-opacity-40;
  }

  /* 반응형 디자인 */
  @media (max-width: 640px) {
    .titlebar {
      @apply h-14 px-3;
    }

    .titlebar-icon {
      @apply w-7 h-7;
    }

    .title-main {
      @apply text-base;
    }

    .title-sub {
      @apply text-xs;
    }

    .control-button {
      @apply w-10 h-10;
    }

    .control-button svg {
      @apply w-3.5 h-3.5;
    }
  }

  /* 한국어 텍스트 최적화 */
  .text-korean {
    word-break: keep-all;
    line-height: 1.4;
  }

  /* 드래그 영역 표시 (개발 시에만) */
  /* [data-tauri-drag-region]:hover {
    background: rgba(255, 255, 255, 0.05);
  } */

  /* 애니메이션 최적화 */
  .control-button {
    will-change: background-color;
  }

  .titlebar {
    will-change: background;
  }

  /* 포커스 접근성 */
  .control-button:focus-visible {
    @apply ring-2 ring-white ring-opacity-50;
  }

  /* 버튼 활성 상태 */
  .control-button:active {
    @apply scale-95;
    transition: transform 0.1s ease;
  }
</style>