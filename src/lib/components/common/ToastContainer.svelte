<script lang="ts">
  import { toastStore, toastEnabled } from '../../stores/toast';
  import Toast from './Toast.svelte';

  // 토스트 컨테이너 속성
  export let maxToasts: number = 5;
  export let position: 'top-right' | 'top-left' | 'bottom-right' | 'bottom-left' | 'top-center' | 'bottom-center' = 'top-right';

  // 토스트 제거 핸들러
  function removeToast(id: string) {
    toastStore.remove(id);
  }

  // 최대 토스트 수 제한 및 활성화 상태 확인
  $: visibleToasts = $toastEnabled ? $toastStore.slice(0, maxToasts) : [];

  // 위치별 컨테이너 클래스
  $: containerClasses = [
    'toast-container',
    `toast-container-${position}`
  ].join(' ');
</script>

<div class={containerClasses}>
  {#each visibleToasts as toast (toast.id)}
    <Toast
      type={toast.type}
      title={toast.title}
      message={toast.message}
      duration={toast.duration}
      persistent={toast.persistent}
      closable={toast.closable}
      position={position}
      showIcon={toast.showIcon}
      showProgress={toast.showProgress}
      on:close={() => removeToast(toast.id)}
      on:click={() => toast.onClick?.(toast)}
    />
  {/each}
</div>

<style>
  .toast-container {
    @apply fixed z-50 pointer-events-none;
  }

  .toast-container-top-right {
    @apply top-4 right-4;
  }

  .toast-container-top-left {
    @apply top-4 left-4;
  }

  .toast-container-bottom-right {
    @apply bottom-4 right-4;
  }

  .toast-container-bottom-left {
    @apply bottom-4 left-4;
  }

  .toast-container-top-center {
    @apply top-4 left-1/2 transform -translate-x-1/2;
  }

  .toast-container-bottom-center {
    @apply bottom-4 left-1/2 transform -translate-x-1/2;
  }

  /* 토스트들이 포인터 이벤트를 받을 수 있도록 */
  .toast-container :global(.toast) {
    @apply pointer-events-auto mb-2;
  }

  /* 반응형 디자인 */
  @media (max-width: 640px) {
    .toast-container-top-center,
    .toast-container-bottom-center {
      @apply left-4 right-4 transform-none;
    }
  }
</style>