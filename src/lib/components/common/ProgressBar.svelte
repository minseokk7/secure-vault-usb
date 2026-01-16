<!--
업로드/다운로드 진행률 표시 컴포넌트
파일 처리 진행 상황을 시각적으로 표시합니다.
-->
<script lang="ts">
  export let progress: number = 0; // 0-100 사이의 진행률
  export let label: string = ''; // 진행률 라벨
  export let showPercentage: boolean = true; // 퍼센트 표시 여부
  export let showCancel: boolean = false; // 취소 버튼 표시 여부
  export let size: 'small' | 'medium' | 'large' = 'medium'; // 크기
  export let color: 'primary' | 'success' | 'warning' | 'error' = 'primary'; // 색상
  export let animated: boolean = true; // 애니메이션 여부
  
  // 이벤트
  import { createEventDispatcher } from 'svelte';
  const dispatch = createEventDispatcher();
  
  // 진행률을 0-100 범위로 제한
  $: clampedProgress = Math.max(0, Math.min(100, progress));
  
  // 색상 클래스 매핑
  $: colorClass = {
    primary: 'bg-blue-500',
    success: 'bg-green-500', 
    warning: 'bg-yellow-500',
    error: 'bg-red-500'
  }[color];
  
  // 크기 클래스 매핑
  $: sizeClass = {
    small: 'h-2',
    medium: 'h-3',
    large: 'h-4'
  }[size];
  
  function handleCancel() {
    dispatch('cancel');
  }
</script>

<div class="progress-container">
  <!-- 라벨과 퍼센트 -->
  {#if label || showPercentage}
    <div class="flex justify-between items-center mb-2">
      {#if label}
        <span class="text-sm font-medium text-gray-700 dark:text-gray-300">
          {label}
        </span>
      {/if}
      
      <div class="flex items-center gap-2">
        {#if showPercentage}
          <span class="text-sm font-medium text-gray-600 dark:text-gray-400">
            {clampedProgress.toFixed(1)}%
          </span>
        {/if}
        
        {#if showCancel}
          <button
            type="button"
            class="text-gray-400 hover:text-gray-600 dark:hover:text-gray-200 transition-colors"
            on:click={handleCancel}
            title="취소"
          >
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        {/if}
      </div>
    </div>
  {/if}
  
  <!-- 진행률 바 -->
  <div class="progress-bar-container {sizeClass} bg-gray-200 dark:bg-gray-700 rounded-full overflow-hidden">
    <div 
      class="progress-bar {colorClass} {sizeClass} rounded-full transition-all duration-300 ease-out {animated ? 'animate-pulse' : ''}"
      style="width: {clampedProgress}%"
    >
      <!-- 진행률 바 내부 광택 효과 -->
      <div class="h-full bg-gradient-to-r from-transparent via-white/20 to-transparent"></div>
    </div>
  </div>
  
  <!-- 상세 정보 슬롯 -->
  <slot name="details" />
</div>

<style>
  .progress-container {
    @apply w-full;
  }
  
  .progress-bar-container {
    @apply relative;
  }
  
  .progress-bar {
    @apply relative;
  }
  
  /* 애니메이션 효과 */
  @keyframes progress-shimmer {
    0% {
      background-position: -200px 0;
    }
    100% {
      background-position: calc(200px + 100%) 0;
    }
  }
  
  .progress-bar.animate-pulse {
    background-image: linear-gradient(
      90deg,
      currentColor 0%,
      rgba(255, 255, 255, 0.3) 50%,
      currentColor 100%
    );
    background-size: 200px 100%;
    animation: progress-shimmer 2s infinite;
  }
</style>