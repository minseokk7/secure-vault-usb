<!--
  매우 간단한 폴더 생성 다이얼로그
  입력 필드 문제를 완전히 해결하기 위한 최소한의 구현
-->
<script lang="ts">
  import { createEventDispatcher } from "svelte";

  // Props
  export let show = false;

  // 이벤트 디스패처
  const dispatch = createEventDispatcher();

  // 상태 변수
  let folderName = "";

  // 폴더명 유효성 검사
  $: isValid = folderName && folderName.trim().length > 0;

  // 폴더 생성 처리
  function handleCreate() {
    if (isValid) {
      dispatch("folderCreated", { name: folderName.trim() });
      // 폼 초기화
      folderName = "";
    }
  }

  // 다이얼로그 닫기
  function handleClose() {
    dispatch("close");
    // 폼 초기화
    folderName = "";
  }

  // 다이얼로그가 열릴 때 포커스 설정
  $: if (show) {
    folderName = "새 폴더";
    setTimeout(() => {
      const input = document.getElementById(
        "simpleFolderName",
      ) as HTMLInputElement;
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
        text-align: center;
      "
      on:click={(e) => e.stopPropagation()}
    >
      <div style="font-size: 3rem; margin-bottom: 1rem;">📂</div>
      <h2 style="margin: 0 0 1.5rem 0; color: #333;">새 폴더 생성</h2>

      <!-- 폴더명 입력 -->
      <div style="margin-bottom: 1.5rem; text-align: left;">
        <label style="display: block; margin-bottom: 0.5rem; font-weight: 500;">
          폴더 이름:
        </label>
        <input
          id="simpleFolderName"
          type="text"
          bind:value={folderName}
          placeholder="새 폴더"
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

      <!-- 버튼 그룹 -->
      <div style="display: flex; gap: 0.5rem; justify-content: center;">
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
          on:click={handleCreate}
          disabled={!isValid}
          style="
            padding: 0.75rem 1.5rem;
            border: none;
            background: {isValid ? '#28a745' : '#ccc'};
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
