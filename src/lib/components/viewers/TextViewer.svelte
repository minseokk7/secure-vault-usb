<!-- 텍스트 파일 뷰어 컴포넌트 -->
<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { getSyntaxLanguage } from '$lib/utils/file-type-detector';
  import type { FileMetadata } from '$lib/types/file-manager';
  import { addToast } from '$lib/stores/toast';
  
  // Props
  export let file: FileMetadata;
  export let onClose: () => void;
  export let onSave: (content: string) => Promise<void>;
  
  // 상태
  let content = '';
  let originalContent = '';
  let isLoading = true;
  let isModified = false;
  let isReadOnly = false;
  let error: string | null = null;
  
  // UI 요소
  let textareaElement: HTMLTextAreaElement;
  
  // 설정
  let showLineNumbers = true;
  let wordWrap = true;
  let fontSize = 14;
  let syntaxLanguage = 'text';
  
  onMount(async () => {
    const fileName = file.original_file_name || file.file_name || '';
    syntaxLanguage = getSyntaxLanguage(fileName);
    await loadFileContent();
    
    // 키보드 이벤트 리스너 추가
    document.addEventListener('keydown', handleKeydown);
  });
  
  onDestroy(() => {
    // 키보드 이벤트 리스너 제거
    document.removeEventListener('keydown', handleKeydown);
  });
  
  async function loadFileContent() {
    try {
      isLoading = true;
      error = null;
      
      content = await invoke<string>('get_text_file_content', {
        file_id: file.id
      });
      
      originalContent = content;
      isModified = false;
      
      // 성공 토스트는 파일 로드 시 표시하지 않음 (너무 빈번함)
      // addToast({
      //   type: 'success',
      //   message: '파일을 성공적으로 로드했습니다.'
      // });
    } catch (err) {
      error = `파일을 로드할 수 없습니다: ${err}`;
      console.error('파일 로드 오류:', err);
      addToast({
        type: 'error',
        message: `파일 로드에 실패했습니다: ${err}`
      });
    } finally {
      isLoading = false;
    }
  }
  
  function handleContentChange() {
    isModified = content !== originalContent;
  }
  
  async function handleSave() {
    if (!isModified || isReadOnly) {
      console.log('저장 건너뜀:', { isModified, isReadOnly });
      return;
    }
    
    try {
      console.log('텍스트 뷰어에서 저장 시작:', { file_id: file.id, contentLength: content.length });
      await onSave(content);
      originalContent = content;
      isModified = false;
      console.log('텍스트 뷰어 저장 완료');
      addToast({
        type: 'success',
        message: '파일이 성공적으로 저장되었습니다.'
      });
    } catch (err) {
      console.error('텍스트 뷰어 저장 오류:', err);
      error = `저장 중 오류가 발생했습니다: ${err}`;
      addToast({
        type: 'error',
        message: `파일 저장에 실패했습니다: ${err}`
      });
    }
  }
  
  function handleKeydown(event: KeyboardEvent) {
    // Ctrl+S: 저장
    if (event.ctrlKey && event.key === 's') {
      event.preventDefault();
      handleSave();
    }
    
    // Ctrl+F: 검색 (브라우저 기본 동작 사용)
    // Esc: 닫기
    if (event.key === 'Escape') {
      onClose();
    }
  }
  
  function toggleReadOnly() {
    isReadOnly = !isReadOnly;
  }
  
  function increaseFontSize() {
    fontSize = Math.min(fontSize + 2, 24);
  }
  
  function decreaseFontSize() {
    fontSize = Math.max(fontSize - 2, 10);
  }
  
  function toggleLineNumbers() {
    showLineNumbers = !showLineNumbers;
  }
  
  function toggleWordWrap() {
    wordWrap = !wordWrap;
  }
  
  function formatTime(seconds: number): string {
    if (!isFinite(seconds)) return '0:00';
    
    const mins = Math.floor(seconds / 60);
    const secs = Math.floor(seconds % 60);
    return `${mins}:${secs.toString().padStart(2, '0')}`;
  }
  
  // 파일 정보 계산
  $: lines = content.split('\n').length;
  $: characters = content.length;
  $: words = content.trim() ? content.trim().split(/\s+/).length : 0;
</script>

<div class="text-viewer">
  <!-- 툴바 -->
  <div class="toolbar">
    <div class="toolbar-left">
      <h3 class="file-name">{file.name}</h3>
      {#if isModified}
        <span class="modified-indicator" title="수정됨">●</span>
      {/if}
      <span class="language-badge">{syntaxLanguage}</span>
    </div>
    
    <div class="toolbar-right">
      <button
        class="btn btn-sm"
        on:click={toggleReadOnly}
        title={isReadOnly ? '편집 모드' : '읽기 전용 모드'}
      >
        {isReadOnly ? '📝' : '🔒'}
      </button>
      
      <button 
        class="btn btn-sm" 
        on:click={toggleLineNumbers} 
        title="줄 번호 표시/숨김"
        class:active={showLineNumbers}
      >
        #
      </button>
      
      <button 
        class="btn btn-sm" 
        on:click={toggleWordWrap} 
        title="자동 줄바꿈"
        class:active={wordWrap}
      >
        ↩️
      </button>
      
      <div class="font-size-controls">
        <button class="btn btn-sm" on:click={decreaseFontSize} title="글꼴 크기 줄이기">
          A-
        </button>
        <span class="font-size-display">{fontSize}px</span>
        <button class="btn btn-sm" on:click={increaseFontSize} title="글꼴 크기 늘리기">
          A+
        </button>
      </div>
      
      <button
        class="btn btn-sm btn-primary"
        on:click={handleSave}
        disabled={!isModified || isReadOnly}
        title="저장 (Ctrl+S)"
      >
        💾 저장
      </button>
      
      <button class="btn btn-sm" on:click={onClose} title="닫기 (Esc)">
        ✕
      </button>
    </div>
  </div>
  
  <!-- 에러 메시지 -->
  {#if error}
    <div class="error-message">
      <span class="error-icon">⚠️</span>
      {error}
    </div>
  {/if}
  
  <!-- 로딩 상태 -->
  {#if isLoading}
    <div class="loading">
      <div class="loading-spinner"></div>
      <p>파일을 로드하는 중...</p>
    </div>
  {:else}
    <!-- 텍스트 에디터 -->
    <div class="editor-container">
      <textarea
        bind:this={textareaElement}
        bind:value={content}
        on:input={handleContentChange}
        readonly={isReadOnly}
        class="text-editor"
        class:readonly={isReadOnly}
        class:word-wrap={wordWrap}
        style="font-size: {fontSize}px;"
        placeholder="파일 내용이 여기에 표시됩니다..."
        spellcheck="false"
      ></textarea>
    </div>
  {/if}
  
  <!-- 상태바 -->
  <div class="status-bar">
    <div class="status-left">
      <span>언어: {syntaxLanguage}</span>
      <span>줄: {lines.toLocaleString()}</span>
      <span>단어: {words.toLocaleString()}</span>
      <span>문자: {characters.toLocaleString()}</span>
    </div>
    
    <div class="status-right">
      {#if isModified}
        <span class="modified">수정됨</span>
      {/if}
      {#if isReadOnly}
        <span class="readonly">읽기 전용</span>
      {/if}
    </div>
  </div>
</div>

<style>
  .text-viewer {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: white;
    border-radius: 8px;
    overflow: hidden;
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
  }
  
  .toolbar {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 12px 16px;
    background: #f8f9fa;
    border-bottom: 1px solid #e9ecef;
    flex-shrink: 0;
  }
  
  .toolbar-left {
    display: flex;
    align-items: center;
    gap: 12px;
  }
  
  .file-name {
    margin: 0;
    font-size: 16px;
    font-weight: 600;
    color: #333;
  }
  
  .modified-indicator {
    color: #ff6b35;
    font-size: 18px;
    font-weight: bold;
  }
  
  .language-badge {
    background: #e9ecef;
    color: #495057;
    padding: 2px 8px;
    border-radius: 12px;
    font-size: 12px;
    font-weight: 500;
  }
  
  .toolbar-right {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  
  .font-size-controls {
    display: flex;
    align-items: center;
    gap: 4px;
  }
  
  .font-size-display {
    font-size: 12px;
    color: #666;
    min-width: 35px;
    text-align: center;
  }
  
  .error-message {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 12px 16px;
    background: #fee;
    color: #c33;
    border-bottom: 1px solid #fcc;
    flex-shrink: 0;
  }
  
  .error-icon {
    font-size: 16px;
  }
  
  .loading {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    flex: 1;
    gap: 16px;
    color: #666;
  }
  
  .loading-spinner {
    width: 32px;
    height: 32px;
    border: 3px solid #f3f3f3;
    border-top: 3px solid #007bff;
    border-radius: 50%;
    animation: spin 1s linear infinite;
  }
  
  @keyframes spin {
    0% { transform: rotate(0deg); }
    100% { transform: rotate(360deg); }
  }
  
  .editor-container {
    flex: 1;
    position: relative;
    overflow: hidden;
  }
  
  .text-editor {
    width: 100%;
    height: 100%;
    border: none;
    outline: none;
    padding: 16px;
    font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
    line-height: 1.5;
    resize: none;
    background: white;
    color: #333;
    tab-size: 2;
  }
  
  .text-editor.readonly {
    background: #f8f9fa;
    color: #666;
  }
  
  .text-editor.word-wrap {
    white-space: pre-wrap;
    word-wrap: break-word;
  }
  
  .text-editor:not(.word-wrap) {
    white-space: pre;
    overflow-x: auto;
  }
  
  .status-bar {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 8px 16px;
    background: #f8f9fa;
    border-top: 1px solid #e9ecef;
    font-size: 12px;
    color: #666;
    flex-shrink: 0;
  }
  
  .status-left {
    display: flex;
    gap: 16px;
  }
  
  .status-right {
    display: flex;
    gap: 12px;
  }
  
  .modified {
    color: #ff6b35;
    font-weight: 600;
  }
  
  .readonly {
    color: #6c757d;
  }
  
  .btn {
    padding: 6px 12px;
    border: 1px solid #ddd;
    background: white;
    border-radius: 4px;
    cursor: pointer;
    font-size: 12px;
    transition: all 0.2s;
    display: flex;
    align-items: center;
    gap: 4px;
  }
  
  .btn:hover {
    background: #f8f9fa;
    border-color: #adb5bd;
  }
  
  .btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
  
  .btn.active {
    background: #007bff;
    color: white;
    border-color: #007bff;
  }
  
  .btn-primary {
    background: #007bff;
    color: white;
    border-color: #007bff;
  }
  
  .btn-primary:hover:not(:disabled) {
    background: #0056b3;
    border-color: #0056b3;
  }
  
  .btn-sm {
    padding: 4px 8px;
    font-size: 11px;
  }
</style>