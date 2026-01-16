<!--
  파일 생성 다이얼로그 컴포넌트
  간단하고 확실하게 작동하는 버전 - 입력 필드 문제 해결
-->
<script lang="ts">
  import { createEventDispatcher } from 'svelte';

  // Props
  export let isOpen = false;

  // 이벤트 디스패처
  const dispatch = createEventDispatcher();

  // 상태 관리 - 기본값으로 초기화
  let fileName = '새파일.txt';
  let fileContent = '';
  let selectedFolderId = null;
  let userModifiedContent = false;

  // 간단한 폴더 목록
  const availableFolders = [
    { id: null, name: '루트 폴더', path: '/', level: 0 }
  ];

  // 파일명 유효성 검사
  $: isValid = fileName.trim().length > 0 && !/[<>:"/\\|?*]/.test(fileName);

  // 파일 확장자에 따른 초기 내용 생성
  function getInitialContent(fileName) {
    const extension = fileName.split('.').pop()?.toLowerCase() || '';
    
    const templates = {
      'txt': '',
      'md': '# 제목\n\n내용을 입력하세요...',
      'json': '{\n  "key": "value"\n}',
      'html': '<!DOCTYPE html>\n<html>\n<head>\n  <title>제목</title>\n</head>\n<body>\n  <h1>안녕하세요!</h1>\n</body>\n</html>',
      'js': '// JavaScript 코드\nconsole.log(\'안녕하세요, 세계!\');',
      'py': '#!/usr/bin/env python3\nprint(\'안녕하세요, 세계!\')',
      'css': '/* CSS 스타일 */\nbody {\n  font-family: \'Malgun Gothic\', Arial, sans-serif;\n  margin: 0;\n  padding: 20px;\n}'
    };

    return templates[extension] || '';
  }

  // 생성 버튼 클릭
  function handleCreate() {
    if (isValid && fileName.trim()) {
      try {
        dispatch('fileCreated', { 
          fileName: fileName.trim(), 
          content: fileContent,
          folderId: selectedFolderId
        });
        resetForm();
      } catch (error) {
        console.error('파일 생성 실패:', error);
      }
    }
  }

  // 취소 버튼 클릭
  function handleCancel() {
    resetForm();
    dispatch('close');
  }

  // 키보드 이벤트 처리
  function handleKeyDown(event) {
    if (event.key === 'Enter' && isValid && fileName.trim()) {
      event.preventDefault();
      handleCreate();
    } else if (event.key === 'Escape') {
      event.preventDefault();
      handleCancel();
    }
  }

  // 폼 초기화
  function resetForm() {
    fileName = '새파일.txt';
    fileContent = '';
    selectedFolderId = null;
    userModifiedContent = false;
  }

  // 파일명 변경 시 자동으로 초기 내용 업데이트
  $: if (!userModifiedContent && fileName) {
    const newContent = getInitialContent(fileName);
    if (newContent !== fileContent) {
      fileContent = newContent;
    }
  }

  // 다이얼로그가 열릴 때 초기화 및 포커스
  $: if (isOpen) {
    resetForm();
    // 포커스 설정
    setTimeout(() => {
      const input = document.getElementById('fileName');
      if (input) {
        input.focus();
        // 확장자를 제외한 부분 선택
        const lastDotIndex = fileName.lastIndexOf('.');
        if (lastDotIndex > 0) {
          input.setSelectionRange(0, lastDotIndex);
        } else {
          input.select();
        }
      }
    }, 150);
  }

  // 사용자가 내용을 직접 수정했을 때 플래그 설정
  function handleContentChange() {
    userModifiedContent = true;
  }
</script>

{#if isOpen}
  <div class="modal-overlay" onclick={handleCancel}>
    <div class="modal-content" onclick={(e) => e.stopPropagation()}>
      <div class="modal-header">
        <h2>📄 새 파일 생성</h2>
        <button class="close-btn" onclick={handleCancel}>
          ✕
        </button>
      </div>
      
      <div class="modal-body">
        <div class="new-file-dialog">
          <!-- 대상 폴더 선택 -->
          <div class="form-group">
            <label for="targetFolder" class="form-label">
              생성할 폴더
            </label>
            <select
              id="targetFolder"
              bind:value={selectedFolderId}
              class="form-select"
            >
              {#each availableFolders as folder}
                <option value={folder.id}>
                  {'  '.repeat(folder.level)}
                  {folder.level > 0 ? '└ ' : ''}
                  {folder.name}
                  {folder.path !== '/' ? ` (${folder.path})` : ''}
                </option>
              {/each}
            </select>
          </div>

          <!-- 파일명 입력 - 매우 간단한 스타일 -->
          <div class="form-group">
            <label for="fileName" class="form-label">
              파일명 (확장자 포함)
            </label>
            <input
              id="fileName"
              type="text"
              bind:value={fileName}
              placeholder="파일명을 입력하세요"
              onkeydown={handleKeyDown}
              autocomplete="off"
              spellcheck="false"
              style="
                width: 100%;
                padding: 0.75rem;
                border: 2px solid #d1d5db;
                border-radius: 0.5rem;
                font-size: 1rem;
                background: white !important;
                color: #374151 !important;
                outline: none;
                box-sizing: border-box;
                font-family: inherit;
              "
              onfocus={(e) => {
                e.target.style.borderColor = '#3b82f6';
                e.target.style.boxShadow = '0 0 0 3px rgba(59, 130, 246, 0.1)';
              }}
              onblur={(e) => {
                e.target.style.borderColor = '#d1d5db';
                e.target.style.boxShadow = 'none';
              }}
            />
            {#if !isValid}
              <p class="error-text">올바른 파일명을 입력하세요</p>
            {/if}
          </div>

          <!-- 초기 내용 입력 -->
          <div class="form-group">
            <label for="fileContent" class="form-label">
              초기 내용 (선택사항)
            </label>
            <textarea
              id="fileContent"
              bind:value={fileContent}
              oninput={handleContentChange}
              placeholder="파일의 초기 내용을 입력하세요"
              class="content-textarea"
              rows="8"
              style="
                width: 100%;
                padding: 0.75rem;
                border: 2px solid #d1d5db;
                border-radius: 0.5rem;
                font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
                font-size: 0.875rem;
                line-height: 1.4;
                resize: vertical;
                min-height: 120px;
                background: white !important;
                color: #374151 !important;
                outline: none;
                box-sizing: border-box;
              "
              onfocus={(e) => {
                e.target.style.borderColor = '#3b82f6';
                e.target.style.boxShadow = '0 0 0 3px rgba(59, 130, 246, 0.1)';
              }}
              onblur={(e) => {
                e.target.style.borderColor = '#d1d5db';
                e.target.style.boxShadow = 'none';
              }}
            ></textarea>
          </div>

          <!-- 버튼 영역 -->
          <div class="button-group">
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
              onclick={handleCreate}
              disabled={!isValid || !fileName.trim()}
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
    min-width: 480px;
    max-width: 520px;
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

  .new-file-dialog {
    overflow: hidden;
    box-sizing: border-box;
  }

  .form-group {
    margin-bottom: 1rem;
  }

  .form-label {
    display: block;
    margin-bottom: 0.375rem;
    font-weight: 500;
    font-size: 0.875rem;
    color: #374151;
  }

  .form-select {
    width: 100%;
    padding: 0.5rem 0.75rem;
    font-size: 0.875rem;
    border: 1px solid #d1d5db;
    border-radius: 0.375rem;
    background-color: white;
    color: #495057;
    transition: all 0.2s;
    box-sizing: border-box;
  }

  .form-select:focus {
    outline: none;
    border-color: #3b82f6;
    box-shadow: 0 0 0 2px rgba(59, 130, 246, 0.1);
    background-color: white;
  }

  .error-text {
    margin-top: 0.25rem;
    font-size: 0.75rem;
    color: #ef4444;
  }

  .button-group {
    display: flex;
    justify-content: flex-end;
    gap: 0.5rem;
    margin-top: 1.25rem;
    padding-top: 0.75rem;
    border-top: 1px solid #e5e7eb;
  }

  .btn {
    padding: 0.5rem 1rem;
    border-radius: 0.375rem;
    font-size: 0.875rem;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s;
    border: 1px solid;
    min-width: 70px;
  }

  .btn-secondary {
    background-color: white;
    color: #374151;
    border-color: #d1d5db;
  }

  .btn-secondary:hover {
    background-color: #f9fafb;
    border-color: #9ca3af;
  }

  .btn-primary {
    background-color: #3b82f6;
    color: white;
    border-color: #3b82f6;
  }

  .btn-primary:hover:not(:disabled) {
    background-color: #2563eb;
    border-color: #2563eb;
  }

  .btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  /* 반응형 디자인 */
  @media (max-width: 640px) {
    .modal-content {
      min-width: auto;
      margin: 1rem;
    }

    .modal-body {
      padding: 1rem;
    }

    .button-group {
      flex-direction: column-reverse;
    }

    .btn {
      width: 100%;
    }
  }
</style>