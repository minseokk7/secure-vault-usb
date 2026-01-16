<!--
폴더 생성 다이얼로그 컴포넌트
새 폴더 생성을 위한 모달 다이얼로그
-->
<script lang="ts">
  import { createEventDispatcher } from 'svelte';

  // Props
  export let show = false;

  // 이벤트 디스패처
  const dispatch = createEventDispatcher();

  // 상태 변수
  let folderName = '';
  let isValid = false;

  // 폴더명 유효성 검사
  $: isValid = folderName.trim().length > 0 && validateFolderName(folderName.trim());

  // 다이얼로그가 열릴 때마다 초기화
  $: if (show) {
    folderName = '';
    // 다음 틱에서 입력 필드에 포커스
    setTimeout(() => {
      const input = document.getElementById('folder-name-input');
      if (input) {
        input.focus();
      }
    }, 100);
  }

  // 폴더명 유효성 검사
  function validateFolderName(name) {
    if (!name) return false;
    
    // Windows에서 허용되지 않는 문자들
    const invalidChars = ['<', '>', ':', '"', '|', '?', '*', '/', '\\'];
    for (const char of invalidChars) {
      if (name.includes(char)) {
        return false;
      }
    }
    
    // Windows 예약어 검사
    const reservedNames = [
      'CON', 'PRN', 'AUX', 'NUL',
      'COM1', 'COM2', 'COM3', 'COM4', 'COM5', 'COM6', 'COM7', 'COM8', 'COM9',
      'LPT1', 'LPT2', 'LPT3', 'LPT4', 'LPT5', 'LPT6', 'LPT7', 'LPT8', 'LPT9'
    ];
    
    if (reservedNames.includes(name.toUpperCase())) {
      return false;
    }
    
    return true;
  }

  // 폴더 생성 처리
  function handleCreate() {
    if (isValid) {
      dispatch('folderCreated', { name: folderName.trim() });
      // show = false; // 부모 컴포넌트에서 처리하도록 제거
    }
  }

  // 다이얼로그 닫기
  function handleClose() {
    dispatch('close'); // 닫기 이벤트 발생
    // show = false; // 부모 컴포넌트에서 처리하도록 제거
  }

  // 키보드 이벤트 처리
  function handleKeyDown(event) {
    if (event.key === 'Enter' && isValid) {
      event.preventDefault();
      handleCreate();
    } else if (event.key === 'Escape') {
      event.preventDefault();
      handleClose();
    }
  }
</script>

{#if show}
  <div class="modal-overlay" onclick={handleClose} role="dialog" aria-modal="true">
    <div class="modal-content" onclick={(e) => e.stopPropagation()} role="document">
      <div class="modal-header">
        <h2>📂 새 폴더 생성</h2>
        <button class="close-btn" onclick={handleClose}>
          ✕
        </button>
      </div>
      
      <div class="modal-body">
        <div class="folder-create-area">
          <div class="folder-icon">📂</div>
          <h3>새 폴더 이름을 입력하세요</h3>
          
          <div class="input-group">
            <label for="folder-name-input">폴더 이름:</label>
            <input
              id="folder-name-input"
              type="text"
              bind:value={folderName}
              placeholder="새 폴더"
              onkeydown={handleKeyDown}
              class="folder-input"
              autocomplete="off"
              spellcheck="false"
            />
          </div>
          
          {#if folderName.trim() && !validateFolderName(folderName.trim())}
            <div class="error-message">
              폴더명에 다음 문자는 사용할 수 없습니다: &lt; &gt; : " | ? * / \
            </div>
          {/if}
          
          <div class="button-group">
            <button class="cancel-btn" onclick={handleClose}>
              취소
            </button>
            <button 
              class="create-btn" 
              onclick={handleCreate}
              disabled={!isValid}
            >
              생성
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
{/if}

<style>
  .modal-overlay {
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
  }

  .modal-content {
    background: white;
    border-radius: 8px;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
    min-width: 400px;
    max-width: 500px;
    max-height: 80vh;
    overflow: hidden;
  }

  .modal-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 1rem 1.5rem;
    border-bottom: 1px solid #e9ecef;
    background: #f8f9fa;
  }

  .modal-header h2 {
    margin: 0;
    font-size: 1.2rem;
    font-weight: 600;
    color: #495057;
  }

  .close-btn {
    background: none;
    border: none;
    font-size: 1.2rem;
    cursor: pointer;
    padding: 0.5rem;
    border-radius: 4px;
    color: #6c757d;
    transition: all 0.2s ease;
  }

  .close-btn:hover {
    background: #e9ecef;
    color: #495057;
  }

  .modal-body {
    padding: 1.5rem;
  }

  .folder-create-area {
    text-align: center;
    padding: 1rem;
  }

  .folder-icon {
    font-size: 3rem;
    margin-bottom: 1rem;
  }

  .folder-create-area h3 {
    margin: 0 0 1.5rem 0;
    font-size: 1.1rem;
    font-weight: 600;
    color: #495057;
  }

  .input-group {
    text-align: left;
    margin-bottom: 1rem;
  }

  .input-group label {
    display: block;
    margin-bottom: 0.5rem;
    font-weight: 500;
    color: #495057;
    font-size: 0.9rem;
  }

  .folder-input {
    width: 100%;
    padding: 0.8rem;
    border: 2px solid #ced4da;
    border-radius: 6px;
    font-size: 1rem;
    transition: all 0.2s ease;
    background: white !important;
    color: #495057 !important;
    box-sizing: border-box;
    outline: none;
  }

  .folder-input:focus {
    border-color: #007bff !important;
    box-shadow: 0 0 0 3px rgba(0, 123, 255, 0.1) !important;
    background: white !important;
  }

  .folder-input:disabled {
    background: #f8f9fa;
    color: #6c757d;
    cursor: not-allowed;
  }

  .error-message {
    background: #f8d7da;
    color: #721c24;
    padding: 0.5rem;
    border-radius: 4px;
    font-size: 0.85rem;
    margin-bottom: 1rem;
    text-align: left;
  }

  .button-group {
    display: flex;
    gap: 0.8rem;
    justify-content: center;
    margin-top: 1.5rem;
  }

  .cancel-btn, .create-btn {
    padding: 0.8rem 1.5rem;
    border: none;
    border-radius: 6px;
    font-size: 0.9rem;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .cancel-btn {
    background: #6c757d;
    color: white;
  }

  .cancel-btn:hover {
    background: #545b62;
    transform: translateY(-1px);
  }

  .create-btn {
    background: #28a745;
    color: white;
  }

  .create-btn:hover:not(:disabled) {
    background: #218838;
    transform: translateY(-1px);
    box-shadow: 0 2px 8px rgba(40, 167, 69, 0.3);
  }

  .create-btn:disabled {
    background: #6c757d;
    cursor: not-allowed;
    opacity: 0.6;
  }
</style>