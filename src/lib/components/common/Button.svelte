<script lang="ts">
  // 버튼 컴포넌트 속성 정의
  export let variant: 'primary' | 'secondary' | 'danger' | 'success' | 'outline' | 'ghost' = 'primary';
  export let size: 'small' | 'medium' | 'large' = 'medium';
  export let disabled: boolean = false;
  export let loading: boolean = false;
  export let fullWidth: boolean = false;
  export let type: 'button' | 'submit' | 'reset' = 'button';
  export let ariaLabel: string = '';
  export let onclick: (() => void) | undefined = undefined;
  export let title: string = '';

  // 클릭 이벤트 핸들러
  function handleClick(event: MouseEvent) {
    console.log('Button 클릭됨:', { disabled, loading, onclick: !!onclick }); // 디버깅용 로그
    
    if (disabled || loading) {
      event.preventDefault();
      return;
    }
    
    // onclick prop이 있으면 실행
    if (onclick) {
      console.log('onclick 함수 실행 중...'); // 디버깅용 로그
      onclick();
    }
  }

  // 동적 클래스 계산
  $: buttonClasses = [
    'btn',
    `btn-${variant}`,
    `btn-${size}`,
    disabled && 'btn-disabled',
    loading && 'btn-loading',
    fullWidth && 'btn-full-width'
  ].filter(Boolean).join(' ');
</script>

<button
  {type}
  class="{buttonClasses} no-drag"
  {disabled}
  {title}
  aria-label={ariaLabel}
  onclick={handleClick}
  draggable="false"
  ondragstart={(e) => { e.preventDefault(); return false; }}
  onselectstart={(e) => { e.preventDefault(); return false; }}
  oncontextmenu={(e) => { e.preventDefault(); return false; }}
>
  {#if loading}
    <span class="btn-spinner" aria-hidden="true">
      <svg class="animate-spin" width="16" height="16" viewBox="0 0 24 24" fill="none">
        <circle cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" stroke-opacity="0.3"/>
        <path d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z" fill="currentColor"/>
      </svg>
    </span>
  {/if}
  
  <span class="btn-content" class:sr-only={loading}>
    <slot />
  </span>
</button>

<style>
  /* 기본 버튼 스타일 */
  .btn {
    @apply inline-flex items-center justify-center gap-2 px-4 py-2 text-sm font-medium rounded-lg border transition-all duration-200 focus:outline-none focus:ring-2 focus:ring-offset-2 disabled:opacity-50 disabled:cursor-not-allowed;
    white-space: nowrap; /* 텍스트 줄바꿈 방지 */
  }

  /* 크기 변형 */
  .btn-small {
    @apply px-3 py-1.5 text-xs;
  }

  .btn-medium {
    @apply px-4 py-2 text-sm;
  }

  .btn-large {
    @apply px-6 py-3 text-base;
  }

  /* 색상 변형 */
  .btn-primary {
    @apply bg-blue-600 text-white border-blue-600 hover:bg-blue-700 focus:ring-blue-500;
  }

  .btn-secondary {
    @apply bg-gray-600 text-white border-gray-600 hover:bg-gray-700 focus:ring-gray-500;
  }

  .btn-danger {
    @apply bg-red-600 text-white border-red-600 hover:bg-red-700 focus:ring-red-500;
  }

  .btn-success {
    @apply bg-green-600 text-white border-green-600 hover:bg-green-700 focus:ring-green-500;
  }

  .btn-outline {
    @apply bg-transparent text-gray-700 border-gray-300 hover:bg-gray-50 focus:ring-gray-500;
  }

  .btn-ghost {
    @apply bg-transparent text-gray-600 border-transparent hover:bg-gray-100 focus:ring-gray-500;
  }

  /* 상태 변형 */
  .btn-disabled {
    @apply opacity-50 cursor-not-allowed;
  }

  .btn-loading {
    @apply cursor-wait;
  }

  .btn-full-width {
    @apply w-full;
  }

  /* 스피너 애니메이션 */
  .btn-spinner {
    @apply flex items-center justify-center;
  }

  .animate-spin {
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    from {
      transform: rotate(0deg);
    }
    to {
      transform: rotate(360deg);
    }
  }

  /* 접근성 */
  .sr-only {
    position: absolute;
    width: 1px;
    height: 1px;
    padding: 0;
    margin: -1px;
    overflow: hidden;
    clip: rect(0, 0, 0, 0);
    white-space: nowrap;
    border: 0;
  }

  /* 버튼 내용 스타일 */
  .btn-content {
    white-space: nowrap; /* 버튼 텍스트 줄바꿈 방지 */
  }

  /* 다크 모드 지원 */
  @media (prefers-color-scheme: dark) {
    .btn-outline {
      @apply text-gray-300 border-gray-600 hover:bg-gray-800;
    }
  }
</style>