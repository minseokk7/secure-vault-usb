<!--
  매우 간단한 이름 변경 다이얼로그
  입력 필드 문제를 완전히 해결하기 위한 최소한의 구현
-->
<script>
  import { createEventDispatcher } from 'svelte';

  // Props
  export let show = false;
  export let itemType = 'file'; // 'file' 또는 'folder'
  export let currentName = '';

  // 이벤트 디스패처
  const dispatch = createEventDispatcher();

  // 상태 변수
  let newName = '';

  // 이름 유효성 검사
  $: isValid = newName && newName.trim().length > 0 && newName.trim() !== currentName;

  // 이름 변경 처리
  function handleRename() {
    if (isValid) {
      dispatch('renamed', { newName: newName.trim() });
    }
  }

  // 다이얼로그 닫기
  function handleClose() {
    dispatch('close');
  }

  // 다이얼로그가 열릴 때마다 초기화 및 포커스
  $: if (show) {
    newName = currentName;
    setTimeout(() => {
      const input = document.getElementById('simpleRenameInput');
      if (input) {
        input.focus();
        input.select();
      }
    }, 100);
  }
</script>

{#if show}
  <!-- 매우 간단한 모달 오버레이 -->
  <div 
    style="
      position: fixed;
      top: 0;
      left: 0;
      right: 0;
      bottom: 0;
      background: rgba(0, 0, 0, 0.5);
      display: flex;
      align-items: center;
      justify-content: center;
      z-index: 1000;
    "
    on:click={handleClose}
  >
    <!-- 모달 콘텐츠 -->
    <div 
      style="
        background: white;
        border-radius: 8px;
        padding: 2rem;
        min-width: 400px;
        box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
      "
      on:click={(e) => e.stopPropagation()}
    >
      <h2 style="margin: 0 0 1.5rem 0; color: #333;">
        ✏️ {itemType === 'file' ? '파일' : '폴더'} 이름 변경
      </h2>
      
      <!-- 현재 이름 표시 -->
      <div style="margin-bottom: 1rem;">
        <label style="display: block; margin-bottom: 0.5rem; font-weight: 500;">
          현재 {itemType === 'file' ? '파일명' : '폴더명'}:
        </label>
        <div style="
          padding: 0.75rem;
          background: #f8f9fa;
          border: 1px solid #e9ecef;
          border-radius: 4px;
          color: #6c757d;
          font-family: monospace;
        ">
          {currentName}
        </div>
      </div>

      <!-- 새 이름 입력 -->
      <div style="margin-bottom: 1.5rem;">
        <label style="display: block; margin-bottom: 0.5rem; font-weight: 500;">
          새 {itemType === 'file' ? '파일명' : '폴더명'}:
        </label>
        <input
          id="simpleRenameInput"
          type="text"
          bind:value={newName}
          placeholder="새 이름을 입력하세요"
          style="
            width: 100%;
            padding: 0.75rem;
            border: 2px solid #ddd;
            border-radius: 4px;
            font-size: 1rem;
            font-family: monospace;
            box-sizing: border-box;
          "
        />
      </div>

      <!-- 버튼 그룹 -->
      <div style="display: flex; gap: 0.5rem; justify-content: flex-end;">
        <button
          type="button"
          on:click={handleClose}
          style="
            padding: 0.75rem 1.5rem;
            border: 1px solid #ddd;
            background: white;
            border-radius: 4px;
            cursor: pointer;
          "
        >
          취소
        </button>
        <button
          type="button"
          on:click={handleRename}
          disabled={!isValid}
          style="
            padding: 0.75rem 1.5rem;
            border: none;
            background: {isValid ? '#007bff' : '#ccc'};
            color: white;
            border-radius: 4px;
            cursor: {isValid ? 'pointer' : 'not-allowed'};
          "
        >
          이름 변경
        </button>
      </div>
    </div>
  </div>
{/if}