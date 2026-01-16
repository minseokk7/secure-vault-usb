<!--
이름 변경 다이얼로그 컴포넌트
파일 또는 폴더의 이름을 변경하기 위한 다이얼로그
-->
<script lang="ts">
  import { createEventDispatcher } from 'svelte';

  // Props
  export let show = false;
  export let itemType = 'file'; // 'file' 또는 'folder'
  export let currentName = '';

  // 이벤트 디스패처
  const dispatch = createEventDispatcher();

  // 상태 변수
  let newName = '';
  let isValid = false;

  // 이름 유효성 검사
  $: isValid = newName.trim().length > 0 && 
               newName.trim() !== currentName && 
               validateName(newName.trim());

  // 다이얼로그가 열릴 때마다 초기화
  $: if (show) {
    newName = currentName;
    // 다음 틱에서 입력 필드에 포커스 및 선택
    setTimeout(() => {
      const input = document.getElementById('rename-input');
      if (input) {
        input.focus();
        if (itemType === 'file') {
          // 파일인 경우 확장자를 제외한 부분만 선택
          const lastDotIndex = newName.lastIndexOf('.');
          if (lastDotIndex > 0) {
            input.setSelectionRange(0, lastDotIndex);
          } else {
            input.select();
          }
        } else {
          // 폴더인 경우 전체 선택
          input.select();
        }
      }
    }, 100);
  }

  // 이름 유효성 검사
  function validateName(name) {
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
    
    const nameWithoutExt = itemType === 'file' ? name.split('.')[0].toUpperCase() : name.toUpperCase();
    if (reservedNames.includes(nameWithoutExt)) {
      return false;
    }
    
    return true;
  }

  // 이름 변경 처리
  function handleRename() {
    if (isValid) {
      dispatch('renamed', { newName: newName.trim() });
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
      handleRename();
    } else if (event.key === 'Escape') {
      event.preventDefault();
      handleClose();
    }
  }

  // 에러 메시지 생성
  function getErrorMessage() {
    const trimmed = newName.trim();
    
    if (!trimmed) {
      return `${itemType === 'file' ? '파일명' : '폴더명'}이 비어있습니다.`;
    }
    
    if (trimmed === currentName) {
      return '이름이 변경되지 않았습니다.';
    }
    
    if (!validateName(trimmed)) {
      return `${itemType === 'file' ? '파일명' : '폴더명'}에 사용할 수 없는 문자가 포함되어 있습니다.`;
    }
    
    return '';
  }
</script>

{#if show}
  <div class="modal-overlay" onclick={handleClose}>
    <div class="modal-content" onclick={(e) => e.stopPropagation()}>
      <div class="modal-header">
        <h2>✏️ {itemType === 'file' ? '파일' : '폴더'} 이름 변경</h2>
        <button class="close-btn" onclick={handleClose}>
          ✕
        </button>
      </div>
      
      <div class="modal-body">
        <div class="rename-area">
          <div class="current-name">
            <label>현재 {itemType === 'file' ? '파일명' : '폴더명'}:</label>
            <div class="current-name-display">{currentName}</div>
          </div>
          
          <div class="input-group">
            <label for="rename-input">새 {itemType === 'file' ? '파일명' : '폴더명'}:</label>
            <input
              id="rename-input"
              type="text"
              bind:value={newName}
              placeholder="새 이름을 입력하세요"
              onkeydown={handleKeyDown}
              class="rename-input"
              autocomplete="off"
              spellcheck="false"
            />
          </div>
          
          {#if newName.trim() && !isValid}
            <div class="error-message">
              {getErrorMessage()}
            </div>
          {/if}
          
          <div class="info-message">
            <p>• {itemType === 'file' ? '파일명' : '폴더명'}에는 다음 문자를 사용할 수 없습니다: &lt; &gt; : " | ? * / \</p>
            <p>• 시스템 예약어(CON, PRN, AUX 등)는 사용할 수 없습니다</p>
          </div>
          
          <div class="button-group">
            <button class="cancel-btn" onclick={handleClose}>
              취소
            </button>
            <button 
              class="rename-btn" 
              onclick={handleRename}
              disabled={!isValid}
            >
              이름 변경
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
    min-width: 450px;
    max-width: 600px;
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

  .rename-area {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .current-name {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .current-name label {
    font-weight: 500;
    color: #495057;
    font-size: 0.9rem;
  }

  .current-name-display {
    padding: 0.8rem;
    background: #f8f9fa;
    border: 1px solid #e9ecef;
    border-radius: 6px;
    color: #6c757d;
    font-family: 'Consolas', 'Monaco', monospace;
    word-break: break-all;
    font-size: 0.9rem;
  }

  .input-group {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .input-group label {
    font-weight: 500;
    color: #495057;
    font-size: 0.9rem;
  }

  .rename-input {
    width: 100%;
    padding: 0.8rem;
    border: 2px solid #ced4da;
    border-radius: 6px;
    font-size: 1rem;
    font-family: 'Consolas', 'Monaco', monospace;
    transition: all 0.2s ease;
    background: white !important;
    color: #495057 !important;
    box-sizing: border-box;
    outline: none;
  }

  .rename-input:focus {
    border-color: #007bff !important;
    box-shadow: 0 0 0 3px rgba(0, 123, 255, 0.1) !important;
    background: white !important;
  }

  .rename-input:disabled {
    background: #f8f9fa;
    color: #6c757d;
    cursor: not-allowed;
  }

  .error-message {
    background: #f8d7da;
    color: #721c24;
    padding: 0.6rem;
    border-radius: 4px;
    font-size: 0.85rem;
    border: 1px solid #f5c6cb;
  }

  .info-message {
    background: #d1ecf1;
    color: #0c5460;
    padding: 0.8rem;
    border-radius: 4px;
    font-size: 0.8rem;
    border: 1px solid #bee5eb;
  }

  .info-message p {
    margin: 0;
    line-height: 1.4;
  }

  .info-message p + p {
    margin-top: 0.3rem;
  }

  .button-group {
    display: flex;
    gap: 0.8rem;
    justify-content: flex-end;
    margin-top: 1rem;
    padding-top: 1rem;
    border-top: 1px solid #e9ecef;
  }

  .cancel-btn, .rename-btn {
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

  .rename-btn {
    background: #007bff;
    color: white;
  }

  .rename-btn:hover:not(:disabled) {
    background: #0056b3;
    transform: translateY(-1px);
    box-shadow: 0 2px 8px rgba(0, 123, 255, 0.3);
  }

  .rename-btn:disabled {
    background: #6c757d;
    cursor: not-allowed;
    opacity: 0.6;
  }
</style>