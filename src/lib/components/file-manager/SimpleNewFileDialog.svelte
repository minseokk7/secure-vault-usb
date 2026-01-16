<!--
  매우 간단한 파일 생성 다이얼로그
  입력 필드 문제를 완전히 해결하기 위한 최소한의 구현
-->
<script>
  import { createEventDispatcher } from 'svelte';

  // Props
  export let show = false;

  // 이벤트 디스패처
  const dispatch = createEventDispatcher();

  // 상태 변수 - 매우 간단하게
  let fileName = '새파일.txt';
  let fileContent = '';

  // 파일명 유효성 검사
  $: isValid = fileName && fileName.trim().length > 0;

  // 생성 버튼 클릭
  function handleCreate() {
    if (isValid) {
      dispatch('fileCreated', { 
        fileName: fileName.trim(), 
        content: fileContent 
      });
      // 폼 초기화
      fileName = '새파일.txt';
      fileContent = '';
    }
  }

  // 취소 버튼 클릭
  function handleCancel() {
    dispatch('close');
    // 폼 초기화
    fileName = '새파일.txt';
    fileContent = '';
  }

  // 다이얼로그가 열릴 때 포커스 설정
  $: if (show) {
    setTimeout(() => {
      const input = document.getElementById('simpleFileName');
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
    on:click={handleCancel}
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
      <h2 style="margin: 0 0 1.5rem 0; color: #333;">📄 새 파일 생성</h2>
      
      <!-- 파일명 입력 - 가장 기본적인 형태 -->
      <div style="margin-bottom: 1rem;">
        <label style="display: block; margin-bottom: 0.5rem; font-weight: 500;">
          파일명:
        </label>
        <input
          id="simpleFileName"
          type="text"
          bind:value={fileName}
          placeholder="파일명을 입력하세요"
          style="
            width: 100%;
            padding: 0.75rem;
            border: 2px solid #ddd;
            border-radius: 4px;
            font-size: 1rem;
            box-sizing: border-box;
          "
        />
      </div>

      <!-- 파일 내용 입력 -->
      <div style="margin-bottom: 1.5rem;">
        <label style="display: block; margin-bottom: 0.5rem; font-weight: 500;">
          초기 내용 (선택사항):
        </label>
        <textarea
          bind:value={fileContent}
          placeholder="파일의 초기 내용을 입력하세요"
          rows="6"
          style="
            width: 100%;
            padding: 0.75rem;
            border: 2px solid #ddd;
            border-radius: 4px;
            font-size: 0.9rem;
            font-family: monospace;
            resize: vertical;
            box-sizing: border-box;
          "
        ></textarea>
      </div>

      <!-- 버튼 그룹 -->
      <div style="display: flex; gap: 0.5rem; justify-content: flex-end;">
        <button
          type="button"
          on:click={handleCancel}
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
          on:click={handleCreate}
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
          생성
        </button>
      </div>
    </div>
  </div>
{/if}