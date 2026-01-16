<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { fade, scale } from 'svelte/transition';

  // Props 인터페이스 정의
  interface Props {
    isOpen?: boolean;
    title?: string;
    size?: 'small' | 'medium' | 'large' | 'full';
    closable?: boolean;
    closeOnBackdrop?: boolean;
    closeOnEscape?: boolean;
    showHeader?: boolean;
    showFooter?: boolean;
    persistent?: boolean;
    zIndex?: number;
    onClose?: () => void;
    onOpen?: () => void;
    onConfirm?: () => void;
    onCancel?: () => void;
  }

  // Props 받기
  const { 
    isOpen = false,
    title = '',
    size = 'medium',
    closable = true,
    closeOnBackdrop = true,
    closeOnEscape = true,
    showHeader = true,
    showFooter = true,
    persistent = false,
    zIndex = 1000,
    onClose,
    onOpen,
    onConfirm,
    onCancel
  }: Props = $props();

  let modalElement: HTMLDivElement;
  let previousActiveElement: HTMLElement | null = null;

  // 모달 열기/닫기 처리
  $effect(() => {
    if (isOpen) {
      handleOpen();
    } else {
      handleClose();
    }
  });

  // 모달 열기 처리
  function handleOpen() {
    // 이전 포커스 요소 저장
    previousActiveElement = document.activeElement as HTMLElement;
    
    // 바디 스크롤 방지
    document.body.style.overflow = 'hidden';
    
    // 모달에 포커스
    setTimeout(() => {
      if (modalElement) {
        modalElement.focus();
      }
    }, 100);

    if (onOpen) {
      onOpen();
    }
  }

  // 모달 닫기 처리
  function handleClose() {
    // 바디 스크롤 복원
    document.body.style.overflow = '';
    
    // 이전 포커스 요소로 복원
    if (previousActiveElement) {
      previousActiveElement.focus();
      previousActiveElement = null;
    }

    if (onClose) {
      onClose();
    }
  }

  // 모달 닫기 함수
  function closeModal() {
    if (!persistent && onClose) {
      onClose();
    }
  }

  // 백드롭 클릭 처리
  function handleBackdropClick(event: MouseEvent) {
    if (closeOnBackdrop && event.target === event.currentTarget) {
      closeModal();
    }
  }

  // 키보드 이벤트 처리
  function handleKeydown(event: KeyboardEvent) {
    if (event.key === 'Escape' && closeOnEscape) {
      closeModal();
    }
    
    // 포커스 트랩 (Tab 키 처리)
    if (event.key === 'Tab') {
      trapFocus(event);
    }
  }

  // 포커스 트랩 구현
  function trapFocus(event: KeyboardEvent) {
    if (!modalElement) return;

    const focusableElements = modalElement.querySelectorAll(
      'button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])'
    );
    
    const firstElement = focusableElements[0] as HTMLElement;
    const lastElement = focusableElements[focusableElements.length - 1] as HTMLElement;

    if (event.shiftKey) {
      // Shift + Tab
      if (document.activeElement === firstElement) {
        event.preventDefault();
        lastElement.focus();
      }
    } else {
      // Tab
      if (document.activeElement === lastElement) {
        event.preventDefault();
        firstElement.focus();
      }
    }
  }

  // 확인 버튼 클릭
  function handleConfirm() {
    if (onConfirm) {
      onConfirm();
    }
  }

  // 취소 버튼 클릭
  function handleCancel() {
    if (onCancel) {
      onCancel();
    }
    closeModal();
  }

  // 동적 클래스 계산
  const modalClasses = $derived([
    'modal-content',
    `modal-${size}`,
  ].filter(Boolean).join(' '));

  // 키보드 이벤트 리스너 관리
  $effect(() => {
    if (isOpen) {
      document.addEventListener('keydown', handleKeydown);
      return () => {
        document.removeEventListener('keydown', handleKeydown);
      };
    }
  });

  // 컴포넌트 언마운트 시 정리
  onDestroy(() => {
    // 바디 스크롤 복원
    document.body.style.overflow = '';
    
    // 이벤트 리스너 제거
    document.removeEventListener('keydown', handleKeydown);
  });
</script>

{#if isOpen}
  <!-- 모달 백드롭 -->
  <div
    class="modal-backdrop"
    style="z-index: {zIndex}"
    onclick={handleBackdropClick}
    onkeydown={handleKeydown}
    transition:fade={{ duration: 200 }}
    role="dialog"
    aria-modal="true"
    aria-labelledby={title ? 'modal-title' : undefined}
  >
    <!-- 모달 컨테이너 -->
    <div
      bind:this={modalElement}
      class={modalClasses}
      transition:scale={{ duration: 200, start: 0.95 }}
      tabindex="-1"
    >
      <!-- 모달 헤더 -->
      {#if showHeader}
        <div class="modal-header">
          {#if title}
            <h2 id="modal-title" class="modal-title">
              {title}
            </h2>
          {:else}
            <div class="modal-title-slot">
              <slot name="title" />
            </div>
          {/if}

          {#if closable}
            <button
              type="button"
              class="modal-close-button"
              onclick={closeModal}
              aria-label="모달 닫기"
            >
              <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <line x1="18" y1="6" x2="6" y2="18"/>
                <line x1="6" y1="6" x2="18" y2="18"/>
              </svg>
            </button>
          {/if}
        </div>
      {/if}

      <!-- 모달 본문 -->
      <div class="modal-body">
        <slot />
      </div>

      <!-- 모달 푸터 -->
      {#if showFooter}
        <div class="modal-footer">
          <slot name="footer">
            <!-- 기본 푸터 버튼들 -->
            <div class="modal-footer-buttons">
              <button
                type="button"
                class="btn btn-secondary"
                onclick={handleCancel}
              >
                취소
              </button>
              <button
                type="button"
                class="btn btn-primary"
                onclick={handleConfirm}
              >
                확인
              </button>
            </div>
          </slot>
        </div>
      {/if}
    </div>
  </div>
{/if}

<style>
  /* 모달 백드롭 */
  .modal-backdrop {
    @apply fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center p-4;
    @apply backdrop-blur-sm;
  }

  /* 모달 컨테이너 */
  .modal-content {
    @apply bg-white dark:bg-gray-800 rounded-xl shadow-2xl max-h-full overflow-hidden;
    @apply flex flex-col border border-gray-200 dark:border-gray-700;
  }

  /* 모달 크기 변형 */
  .modal-small {
    @apply w-full max-w-md;
  }

  .modal-medium {
    @apply w-full max-w-lg;
  }

  .modal-large {
    @apply w-full max-w-4xl;
  }

  .modal-full {
    @apply w-full h-full max-w-none max-h-none rounded-none;
  }

  /* 모달 헤더 */
  .modal-header {
    @apply flex items-center justify-between p-6 border-b border-gray-200 dark:border-gray-700;
    @apply bg-gray-50 dark:bg-gray-800;
  }

  .modal-title {
    @apply text-xl font-semibold text-gray-900 dark:text-gray-100 text-korean;
  }

  .modal-title-slot {
    @apply flex-1;
  }

  .modal-close-button {
    @apply p-2 text-gray-400 hover:text-gray-600 dark:hover:text-gray-300;
    @apply rounded-lg hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors;
    @apply focus:outline-none focus:ring-2 focus:ring-blue-500;
  }

  /* 모달 본문 */
  .modal-body {
    @apply flex-1 p-6 overflow-y-auto;
    @apply text-gray-700 dark:text-gray-300 text-korean;
    overflow-x: hidden;
  }

  /* 모달 푸터 */
  .modal-footer {
    @apply p-6 border-t border-gray-200 dark:border-gray-700;
    @apply bg-gray-50 dark:bg-gray-800;
  }

  .modal-footer-buttons {
    @apply flex justify-end gap-3;
  }

  /* 반응형 디자인 */
  @media (max-width: 640px) {
    .modal-backdrop {
      @apply p-2;
    }

    .modal-content {
      @apply max-h-screen;
    }

    .modal-header,
    .modal-body,
    .modal-footer {
      @apply p-4;
    }

    .modal-footer-buttons {
      @apply flex-col-reverse gap-2;
    }

    .modal-footer-buttons button {
      @apply w-full;
    }
  }

  /* 애니메이션 최적화 */
  .modal-backdrop {
    will-change: opacity;
  }

  .modal-content {
    will-change: transform, opacity;
  }

  /* 접근성 개선 */
  .modal-content:focus {
    @apply outline-none;
  }

  /* 스크롤바 스타일 */
  .modal-body::-webkit-scrollbar {
    @apply w-2;
  }

  .modal-body::-webkit-scrollbar-track {
    @apply bg-gray-100 dark:bg-gray-700;
  }

  .modal-body::-webkit-scrollbar-thumb {
    @apply bg-gray-300 dark:bg-gray-600 rounded-full;
  }

  .modal-body::-webkit-scrollbar-thumb:hover {
    @apply bg-gray-400 dark:bg-gray-500;
  }

  /* 한국어 텍스트 최적화 */
  .text-korean {
    word-break: keep-all;
    line-height: 1.7;
  }
</style>