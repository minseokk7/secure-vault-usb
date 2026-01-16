<script lang="ts">
  import { createEventDispatcher, onMount } from 'svelte';
  import { fade, fly } from 'svelte/transition';

  // 토스트 컴포넌트 속성 정의
  export let type: 'success' | 'error' | 'warning' | 'info' = 'info';
  export let title: string = '';
  export let message: string = '';
  export let duration: number = 5000; // 5초 기본값
  export let persistent: boolean = false; // 자동 닫힘 방지
  export let closable: boolean = true;
  export let position: 'top-right' | 'top-left' | 'bottom-right' | 'bottom-left' | 'top-center' | 'bottom-center' = 'top-right';
  export let showIcon: boolean = true;
  export let showProgress: boolean = true;

  // 이벤트 디스패처
  const dispatch = createEventDispatcher<{
    close: void;
    click: void;
  }>();

  let toastElement: HTMLDivElement;
  let progressElement: HTMLDivElement;
  let timeoutId: number | null = null;
  let startTime: number;
  let remainingTime: number = duration;
  let isPaused: boolean = false;

  // 토스트 아이콘 맵
  const icons = {
    success: `<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"/>
      <polyline points="22,4 12,14.01 9,11.01"/>
    </svg>`,
    error: `<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <circle cx="12" cy="12" r="10"/>
      <line x1="15" y1="9" x2="9" y2="15"/>
      <line x1="9" y1="9" x2="15" y2="15"/>
    </svg>`,
    warning: `<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <path d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z"/>
      <line x1="12" y1="9" x2="12" y2="13"/>
      <line x1="12" y1="17" x2="12.01" y2="17"/>
    </svg>`,
    info: `<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <circle cx="12" cy="12" r="10"/>
      <line x1="12" y1="16" x2="12" y2="12"/>
      <line x1="12" y1="8" x2="12.01" y2="8"/>
    </svg>`
  };

  // 컴포넌트 마운트 시 자동 닫힘 타이머 시작
  onMount(() => {
    if (!persistent && duration > 0) {
      startAutoClose();
    }
  });

  // 자동 닫힘 타이머 시작
  function startAutoClose() {
    startTime = Date.now();
    timeoutId = window.setTimeout(() => {
      closeToast();
    }, remainingTime);

    // 프로그레스 바 애니메이션
    if (showProgress && progressElement) {
      progressElement.style.transition = `width ${remainingTime}ms linear`;
      progressElement.style.width = '0%';
    }
  }

  // 자동 닫힘 타이머 일시정지
  function pauseAutoClose() {
    if (timeoutId && !persistent) {
      clearTimeout(timeoutId);
      timeoutId = null;
      isPaused = true;
      
      // 남은 시간 계산
      const elapsed = Date.now() - startTime;
      remainingTime = Math.max(0, remainingTime - elapsed);

      // 프로그레스 바 일시정지
      if (showProgress && progressElement) {
        progressElement.style.transition = 'none';
        const currentWidth = (elapsed / duration) * 100;
        progressElement.style.width = `${Math.min(100, currentWidth)}%`;
      }
    }
  }

  // 자동 닫힘 타이머 재개
  function resumeAutoClose() {
    if (isPaused && remainingTime > 0 && !persistent) {
      isPaused = false;
      startAutoClose();
    }
  }

  // 토스트 닫기
  function closeToast() {
    if (timeoutId) {
      clearTimeout(timeoutId);
      timeoutId = null;
    }
    dispatch('close');
  }

  // 토스트 클릭 처리
  function handleClick() {
    dispatch('click');
  }

  // 마우스 이벤트 핸들러
  function handleMouseEnter() {
    pauseAutoClose();
  }

  function handleMouseLeave() {
    resumeAutoClose();
  }

  // 키보드 이벤트 핸들러
  function handleKeydown(event: KeyboardEvent) {
    if (event.key === 'Escape' && closable) {
      closeToast();
    }
  }

  // 동적 클래스 계산
  $: toastClasses = [
    'toast',
    `toast-${type}`,
    `toast-${position}`,
    closable && 'toast-closable'
  ].filter(Boolean).join(' ');

  // 위치별 애니메이션 방향 계산
  $: flyDirection = position.includes('right') ? { x: 300 } : 
                   position.includes('left') ? { x: -300 } :
                   position.includes('top') ? { y: -100 } : { y: 100 };
</script>

<div
  bind:this={toastElement}
  class={toastClasses}
  transition:fly={{ ...flyDirection, duration: 300 }}
  on:mouseenter={handleMouseEnter}
  on:mouseleave={handleMouseLeave}
  on:click={handleClick}
  on:keydown={handleKeydown}
  role="alert"
  aria-live="polite"
  tabindex={closable ? 0 : undefined}
>
  <!-- 프로그레스 바 -->
  {#if showProgress && !persistent && duration > 0}
    <div class="toast-progress">
      <div bind:this={progressElement} class="toast-progress-bar"></div>
    </div>
  {/if}

  <!-- 토스트 내용 -->
  <div class="toast-content">
    <!-- 아이콘 -->
    {#if showIcon}
      <div class="toast-icon" aria-hidden="true">
        {@html icons[type]}
      </div>
    {/if}

    <!-- 텍스트 내용 -->
    <div class="toast-text">
      {#if title}
        <div class="toast-title">{title}</div>
      {/if}
      {#if message}
        <div class="toast-message">{message}</div>
      {:else}
        <div class="toast-message">
          <slot />
        </div>
      {/if}
    </div>

    <!-- 닫기 버튼 -->
    {#if closable}
      <button
        type="button"
        class="toast-close"
        on:click|stopPropagation={closeToast}
        aria-label="알림 닫기"
      >
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <line x1="18" y1="6" x2="6" y2="18"/>
          <line x1="6" y1="6" x2="18" y2="18"/>
        </svg>
      </button>
    {/if}
  </div>
</div>

<style>
  /* 기본 토스트 스타일 */
  .toast {
    @apply fixed z-50 max-w-sm w-full rounded-lg shadow-lg border;
    @apply cursor-pointer transition-all duration-200 hover:shadow-xl;
    @apply focus:outline-none focus:ring-2 focus:ring-offset-2;
  }

  /* 위치별 스타일 - 파일 매니저용 추가 여백 */
  .toast-top-right {
    @apply top-28 right-4;
  }

  .toast-top-left {
    @apply top-28 left-4;
  }

  .toast-bottom-right {
    @apply bottom-4 right-4;
  }

  .toast-bottom-left {
    @apply bottom-4 left-4;
  }

  .toast-top-center {
    @apply top-28 left-1/2 transform -translate-x-1/2;
  }

  .toast-bottom-center {
    @apply bottom-4 left-1/2 transform -translate-x-1/2;
  }

  /* 타입별 색상 */
  .toast-success {
    @apply bg-green-50 dark:bg-green-900 border-green-200 dark:border-green-700 focus:ring-green-500;
  }

  .toast-error {
    @apply bg-red-50 dark:bg-red-900 border-red-200 dark:border-red-700 focus:ring-red-500;
  }

  .toast-warning {
    @apply bg-yellow-50 dark:bg-yellow-900 border-yellow-200 dark:border-yellow-700 focus:ring-yellow-500;
  }

  .toast-info {
    @apply bg-blue-50 dark:bg-blue-900 border-blue-200 dark:border-blue-700 focus:ring-blue-500;
  }

  /* 프로그레스 바 */
  .toast-progress {
    @apply absolute top-0 left-0 right-0 h-1 bg-gray-200 dark:bg-gray-700 rounded-t-lg overflow-hidden;
  }

  .toast-progress-bar {
    @apply h-full w-full transition-all;
  }

  .toast-success .toast-progress-bar {
    @apply bg-green-500;
  }

  .toast-error .toast-progress-bar {
    @apply bg-red-500;
  }

  .toast-warning .toast-progress-bar {
    @apply bg-yellow-500;
  }

  .toast-info .toast-progress-bar {
    @apply bg-blue-500;
  }

  /* 토스트 내용 */
  .toast-content {
    @apply flex items-start gap-3 p-4;
  }

  /* 아이콘 */
  .toast-icon {
    @apply flex-shrink-0 mt-0.5;
  }

  .toast-success .toast-icon {
    @apply text-green-500;
  }

  .toast-error .toast-icon {
    @apply text-red-500;
  }

  .toast-warning .toast-icon {
    @apply text-yellow-500;
  }

  .toast-info .toast-icon {
    @apply text-blue-500;
  }

  /* 텍스트 내용 */
  .toast-text {
    @apply flex-1 min-w-0;
  }

  .toast-title {
    @apply font-semibold text-sm text-korean;
  }

  .toast-message {
    @apply text-sm text-korean;
    @apply mt-1;
  }

  /* 타입별 텍스트 색상 */
  .toast-success .toast-title {
    @apply text-green-800 dark:text-green-100;
  }

  .toast-success .toast-message {
    @apply text-green-700 dark:text-green-200;
  }

  .toast-error .toast-title {
    @apply text-red-800 dark:text-red-100;
  }

  .toast-error .toast-message {
    @apply text-red-700 dark:text-red-200;
  }

  .toast-warning .toast-title {
    @apply text-yellow-800 dark:text-yellow-100;
  }

  .toast-warning .toast-message {
    @apply text-yellow-700 dark:text-yellow-200;
  }

  .toast-info .toast-title {
    @apply text-blue-800 dark:text-blue-100;
  }

  .toast-info .toast-message {
    @apply text-blue-700 dark:text-blue-200;
  }

  .toast-title + .toast-message {
    @apply mt-1;
  }

  /* 닫기 버튼 */
  .toast-close {
    @apply flex-shrink-0 text-gray-400 hover:text-gray-600 dark:hover:text-gray-300;
    @apply p-1 rounded transition-colors focus:outline-none focus:ring-2 focus:ring-gray-500;
  }

  /* 반응형 디자인 */
  @media (max-width: 640px) {
    .toast {
      @apply max-w-none mx-4;
    }

    .toast-top-center,
    .toast-bottom-center {
      @apply left-4 right-4 transform-none;
    }
  }

  /* 애니메이션 최적화 */
  .toast {
    will-change: transform, opacity;
  }

  /* 호버 효과 */
  .toast-closable:hover {
    @apply scale-105;
  }

  /* 한국어 텍스트 최적화 */
  .text-korean {
    word-break: keep-all;
    line-height: 1.6;
  }

  /* 다크 모드 추가 스타일 */
  @media (prefers-color-scheme: dark) {
    .toast {
      @apply bg-gray-800 border-gray-700;
    }
  }
</style>