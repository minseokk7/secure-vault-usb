<!-- 텍스트 파일 뷰어 컴포넌트 -->
<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { getSyntaxLanguage } from "$lib/utils/file-type-detector";
  import type { FileMetadata } from "$lib/types/file-manager";
  import { addToast } from "$lib/stores/toast";
  import { marked } from "marked";
  import DOMPurify from "dompurify";
  import TurndownService from "turndown";

  // Props
  export let file: FileMetadata;
  export let onClose: () => void;
  export let onSave: (content: string) => Promise<void>;

  // 상태
  let content = "";
  let originalContent = "";
  let isLoading = true;
  let isModified = false;
  let isReadOnly = false;
  let error: string | null = null;
  let previewHtml = "";

  // UI 요소
  let textareaElement: HTMLDivElement;

  // 설정
  let showLineNumbers = true;
  let wordWrap = true;
  let fontSize = 14;
  let syntaxLanguage = "text";

  // 파일명 계산
  $: fileName = file.original_file_name || file.file_name || "";

  onMount(async () => {
    syntaxLanguage = getSyntaxLanguage(fileName);
    await loadFileContent();

    // 키보드 이벤트 리스너 추가
    document.addEventListener("keydown", handleKeydown);
  });

  onDestroy(() => {
    // 키보드 이벤트 리스너 제거
    document.removeEventListener("keydown", handleKeydown);
  });

  async function loadFileContent() {
    try {
      isLoading = true;
      error = null;

      const rawContent = await invoke<string>("get_text_file_content", {
        fileId: file.id,
      });
      content = rawContent; // 마크다운 원본 유지

      // 마크다운 -> HTML 변환 및 Sanitization
      const rawHtml = await marked(rawContent);
      const sanitizedHtml = DOMPurify.sanitize(rawHtml);

      // 에디터에 HTML 주입
      if (textareaElement) {
        textareaElement.innerHTML = sanitizedHtml;
      } else {
        // 마운트 전이면 나중에 주입하기 위해 임시 저장
        previewHtml = sanitizedHtml;
      }

      originalContent = content;
      isModified = false;
    } catch (err) {
      error = `파일을 로드할 수 없습니다: ${err}`;
      console.error("파일 로드 오류:", err);
      addToast({
        type: "error",
        message: `파일 로드에 실패했습니다: ${err}`,
      });
    } finally {
      isLoading = false;
      // 로딩 완료 후 요소가 있으면 주입
      setTimeout(() => {
        if (textareaElement && previewHtml) {
          textareaElement.innerHTML = previewHtml;
          previewHtml = "";
        }
      }, 0);
    }
  }

  function handleContentChange() {
    isModified = true;
  }

  async function handleSave() {
    if (!isModified || isReadOnly) {
      console.log("저장 건너뜀:", { isModified, isReadOnly });
      return;
    }

    // HTML -> Markdown 변환
    const turndownService = new TurndownService({
      headingStyle: "atx",
      codeBlockStyle: "fenced",
    });

    // Turndown 설정: 간단한 줄바꿈 처리 등을 커스텀 가능

    if (textareaElement) {
      const htmlContent = textareaElement.innerHTML;
      const markdownContent = turndownService.turndown(htmlContent);
      content = markdownContent;
    }

    try {
      console.log("텍스트 뷰어에서 저장 시작:", {
        file_id: file.id,
        contentLength: content.length,
      });
      await onSave(content);
      originalContent = content;
      isModified = false;
      console.log("텍스트 뷰어 저장 완료");
      addToast({
        type: "success",
        message: "파일이 성공적으로 저장되었습니다.",
      });
    } catch (err) {
      console.error("텍스트 뷰어 저장 오류:", err);
      error = `저장 중 오류가 발생했습니다: ${err}`;
      addToast({
        type: "error",
        message: `파일 저장에 실패했습니다: ${err}`,
      });
    }
  }

  // 커서 위치 상태
  let cursorLine = 1;
  let cursorColumn = 1;

  // 서식 삽입 함수
  // 서식 삽입 함수 (execCommand 사용)
  function insertFormatting(type: string, arg?: string) {
    if (isReadOnly) return;

    // 포커스 확보
    if (textareaElement) textareaElement.focus();

    switch (type) {
      case "bold":
        document.execCommand("bold", false);
        break;
      case "italic":
        document.execCommand("italic", false);
        break;
      case "strikethrough":
        document.execCommand("strikeThrough", false);
        break;
      case "heading":
        const level = arg || "1";
        document.execCommand("formatBlock", false, `<H${level}>`);
        break;
      case "list":
        document.execCommand("insertUnorderedList", false);
        break;
      case "ordered-list":
        document.execCommand("insertOrderedList", false);
        break;
      case "link":
        const url = prompt("링크 URL을 입력하세요:", "http://");
        if (url) {
          document.execCommand("createLink", false, url);
        }
        break;
      case "quote":
        document.execCommand("formatBlock", false, "<blockquote>");
        break;
      case "code":
        // 코드 블럭은 execCommand로 깔끔하게 처리하기 어려움, 간단히 pre 태그로 감싸기 시도
        // 또는 선택 영역을 <code> 태그로 감싸기
        const selection = window.getSelection();
        if (selection && selection.rangeCount > 0) {
          const range = selection.getRangeAt(0);
          const codeNode = document.createElement("code");
          codeNode.textContent = selection.toString();
          range.deleteContents();
          range.insertNode(codeNode);
        }
        break;
      case "checkbox":
        // 체크박스는 특수 문자 삽입으로 대체
        document.execCommand("insertText", false, "☑ ");
        break;
      default:
        return;
    }

    handleContentChange();
    updateCursorPosition();
  }

  function handleKeydown(event: KeyboardEvent) {
    // Ctrl+S: 저장
    if (event.ctrlKey && event.key === "s") {
      event.preventDefault();
      handleSave();
      return;
    }

    // 서식 단축키
    if (event.ctrlKey && !isReadOnly) {
      switch (event.key.toLowerCase()) {
        case "b":
          event.preventDefault();
          insertFormatting("bold");
          return;
        case "i":
          event.preventDefault();
          insertFormatting("italic");
          return;
        case "u": // 취소선으로 대체 (HTML underline은 마크다운에 없음)
          event.preventDefault();
          insertFormatting("strikethrough");
          return;
        case "k":
          event.preventDefault();
          insertFormatting("link");
          return;
      }
    }

    // Ctrl+F: 검색 (브라우저 기본 동작 사용)
    // Esc: 닫기
    if (event.key === "Escape") {
      onClose();
    }

    // 커서 위치 업데이트 (일부 키 입력 시)
    setTimeout(updateCursorPosition, 0);
  }

  function updateCursorPosition() {
    if (!textareaElement) return;

    const selection = window.getSelection();
    if (!selection || selection.rangeCount === 0) return;

    // 간단한 근사치 계산 (정확한 행/열 계산은 contenteditable에서 복잡함)
    // 여기서는 전체 텍스트 내에서의 위치를 기반으로 계산 시도

    // 현재는 단순화하여 전체 글자 수 업데이트만 수행하고
    // 행/열은 정확하지 않을 수 있음을 감안하거나, Selection API로 더 정교하게 구현 필요
    // 일단 전체 텍스트 기준으로 라인 수 계산

    const text = textareaElement.innerText;
    const lines = text.split("\n");
    cursorLine = lines.length; // 현재 총 라인 수로 대체 표시
    // cursorColumn은 현재 계산 어려움
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
    if (!isFinite(seconds)) return "0:00";

    const mins = Math.floor(seconds / 60);
    const secs = Math.floor(seconds % 60);
    return `${mins}:${secs.toString().padStart(2, "0")}`;
  }

  // 파일 정보 계산
  $: lines = content.split("\n").length;
  $: characters = content.length;
  $: words = content.trim() ? content.trim().split(/\s+/).length : 0;
</script>

<div class="text-viewer">
  <!-- 툴바 -->
  <div class="toolbar-container">
    <!-- 상단: 파일 정보 및 핵심 액션 -->
    <div class="toolbar-row top">
      <div class="toolbar-left">
        <h3 class="file-name">{fileName}</h3>
        {#if isModified}
          <span class="modified-indicator" title="수정됨">●</span>
        {/if}
        <span class="language-badge">{syntaxLanguage}</span>
      </div>

      <div class="toolbar-right">
        <button
          class="btn btn-sm"
          on:click={toggleReadOnly}
          title={isReadOnly ? "편집 모드" : "읽기 전용 모드"}
        >
          {isReadOnly ? "📝" : "🔒"}
        </button>

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

    <!-- 하단: 서식 도구 및 글꼴 설정 -->
    <div class="toolbar-row bottom">
      <div class="toolbar-center">
        <!-- 스타일 -->
        <div class="format-group">
          <button
            class="btn btn-sm"
            on:click={() => insertFormatting("bold")}
            title="굵게 (Ctrl+B)"><b>B</b></button
          >
          <button
            class="btn btn-sm"
            on:click={() => insertFormatting("italic")}
            title="기울임 (Ctrl+I)"><i>I</i></button
          >
          <button
            class="btn btn-sm"
            on:click={() => insertFormatting("strikethrough")}
            title="취소선 (Ctrl+U)"><s>S</s></button
          >
        </div>

        <!-- 헤더 -->
        <div class="format-group">
          <button
            class="btn btn-sm"
            on:click={() => insertFormatting("heading", "1")}
            title="제목 1">H1</button
          >
          <button
            class="btn btn-sm"
            on:click={() => insertFormatting("heading", "2")}
            title="제목 2">H2</button
          >
          <button
            class="btn btn-sm"
            on:click={() => insertFormatting("heading", "3")}
            title="제목 3">H3</button
          >
        </div>

        <!-- 리스트 -->
        <div class="format-group">
          <button
            class="btn btn-sm"
            on:click={() => insertFormatting("list")}
            title="글머리 기호 목록">•</button
          >
          <button
            class="btn btn-sm"
            on:click={() => insertFormatting("ordered-list")}
            title="번호 매기기 목록">1.</button
          >
          <button
            class="btn btn-sm"
            on:click={() => insertFormatting("checkbox")}
            title="체크박스">☑</button
          >
        </div>

        <!-- 삽입 -->
        <div class="format-group">
          <button
            class="btn btn-sm"
            on:click={() => insertFormatting("quote")}
            title="인용">❝</button
          >
          <button
            class="btn btn-sm"
            on:click={() => insertFormatting("code")}
            title="코드 블럭">&lt;/&gt;</button
          >
          <button
            class="btn btn-sm"
            on:click={() => insertFormatting("link")}
            title="링크 (Ctrl+K)">🔗</button
          >
        </div>

        <!-- 구분선 및 글꼴 -->
        <div class="divider-vertical"></div>

        <div class="font-size-controls">
          <button
            class="btn btn-sm"
            on:click={decreaseFontSize}
            title="글꼴 크기 줄이기"
          >
            A-
          </button>
          <span class="font-size-display">{fontSize}px</span>
          <button
            class="btn btn-sm"
            on:click={increaseFontSize}
            title="글꼴 크기 늘리기"
          >
            A+
          </button>
        </div>
      </div>
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
    <!-- 텍스트 에디터 및 미리보기 -->
    <!-- 텍스트 에디터 (WYSIWYG) -->
    <div class="editor-container">
      <div
        bind:this={textareaElement}
        contenteditable={!isReadOnly}
        on:input={handleContentChange}
        on:click={updateCursorPosition}
        on:keyup={updateCursorPosition}
        class="text-editor markdown-preview"
        class:readonly={isReadOnly}
        style="font-size: {fontSize}px;"
        role="textbox"
        tabindex="0"
      ></div>
    </div>
  {/if}

  <!-- 상태바 -->
  <div class="status-bar">
    <div class="status-left">
      <span>Ln {cursorLine}, Col {cursorColumn}</span>
      <span class="divider">|</span>
      <span>{characters.toLocaleString()} 글자</span>
      <span class="divider">|</span>
      <span>UTF-8</span>
    </div>

    <div class="status-right">
      <button
        class="status-btn"
        on:click={toggleLineNumbers}
        title="줄 번호 표시/숨김"
        class:active={showLineNumbers}
      >
        {showLineNumbers ? "줄 번호: 켜짐" : "줄 번호: 꺼짐"}
      </button>
      <button
        class="status-btn"
        on:click={toggleWordWrap}
        title="자동 줄바꿈 변경"
        class:active={wordWrap}
      >
        {wordWrap ? "자동 줄바꿈" : "줄바꿈 없음"}
      </button>
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
    font-family:
      -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
  }

  .toolbar-container {
    display: flex;
    flex-direction: column;
    padding: 8px 16px;
    background: #f8f9fa;
    border-bottom: 1px solid #e9ecef;
    flex-shrink: 0;
    gap: 8px;
  }

  .toolbar-row {
    display: flex;
    align-items: center;
    width: 100%;
  }

  .toolbar-row.top {
    justify-content: space-between;
  }

  .toolbar-row.bottom {
    justify-content: flex-start; /* 또는 center */
    padding-top: 4px;
    border-top: 1px solid #e9ecef;
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

  .toolbar-center {
    display: flex;
    align-items: center;
    gap: 16px;
    margin-top: 4px; /* 약간의 간격 */
  }

  .format-group {
    display: flex;
    align-items: center;
    gap: 2px;
    padding-right: 16px;
    border-right: 1px solid #e9ecef;
  }

  .format-group:last-child {
    padding-right: 0;
    border-right: none;
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
    0% {
      transform: rotate(0deg);
    }
    100% {
      transform: rotate(360deg);
    }
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
    font-family: "Consolas", "Monaco", "Courier New", monospace;
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

  /* .text-editor.word-wrap 제거됨 - contenteditable은 기본적으로 래핑됨 */
  /* .text-editor:not(.word-wrap) 제거됨 */

  /* WYSIWYG 에디터 내부 스타일 */
  .text-editor :global(ul),
  .text-editor :global(ol) {
    margin: 8px 0;
    padding-left: 24px;
    list-style-position: inside; /* 중요: overflow:hidden 때문에 안쪽으로 배치 */
  }

  .text-editor :global(ul) {
    list-style-type: disc;
  }

  .text-editor :global(ol) {
    list-style-type: decimal;
  }

  .text-editor :global(li) {
    margin-bottom: 4px;
    display: list-item; /* 명시적 지정 */
  }

  /* 리스트 내부의 p 태그 제거 (execCommand로 인해 생길 수 있음) */
  .text-editor :global(li > p) {
    margin: 0;
    display: inline;
  }

  .text-editor :global(h1),
  .text-editor :global(h2),
  .text-editor :global(h3) {
    font-weight: bold;
    margin: 16px 0 8px 0;
    line-height: 1.3;
  }

  .text-editor :global(h1) {
    font-size: 2em;
    border-bottom: 1px solid #eaecef;
    padding-bottom: 0.3em;
  }
  .text-editor :global(h2) {
    font-size: 1.5em;
    border-bottom: 1px solid #eaecef;
    padding-bottom: 0.3em;
  }
  .text-editor :global(h3) {
    font-size: 1.25em;
  }

  .text-editor :global(blockquote) {
    border-left: 4px solid #dfe2e5;
    color: #6a737d;
    padding-left: 16px;
    margin: 8px 0;
  }

  .text-editor :global(pre) {
    background: #f6f8fa;
    padding: 16px;
    border-radius: 4px;
    overflow: auto;
    margin: 8px 0;
  }

  .text-editor :global(code) {
    background: rgba(175, 184, 193, 0.2);
    padding: 0.2em 0.4em;
    border-radius: 3px;
    font-family: monospace;
    font-size: 0.9em;
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
    align-items: center;
    gap: 8px;
  }

  .divider {
    color: #dee2e6;
  }

  .status-right {
    display: flex;
    gap: 8px;
  }

  .status-btn {
    background: none;
    border: none;
    padding: 2px 6px;
    font-size: 11px;
    color: #666;
    cursor: pointer;
    border-radius: 3px;
    display: flex;
    align-items: center;
    gap: 4px;
    transition: all 0.2s;
  }

  .status-btn:hover {
    background: #e9ecef;
    color: #333;
  }

  .status-btn.active {
    background: #e9ecef;
    color: #007bff;
    font-weight: 500;
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

  .markdown-preview {
    width: 100%;
    height: 100%;
    padding: 24px 32px;
    overflow-y: auto;
    background: white;
    color: #333;
    line-height: 1.6;
  }

  .markdown-preview :global(h1) {
    font-size: 2em;
    border-bottom: 1px solid #eaecef;
    padding-bottom: 0.3em;
    margin-top: 24px;
    margin-bottom: 16px;
    font-weight: 600;
  }

  .markdown-preview :global(h2) {
    font-size: 1.5em;
    border-bottom: 1px solid #eaecef;
    padding-bottom: 0.3em;
    margin-top: 24px;
    margin-bottom: 16px;
    font-weight: 600;
  }

  .markdown-preview :global(h3) {
    font-size: 1.25em;
    margin-top: 24px;
    margin-bottom: 16px;
    font-weight: 600;
  }

  .markdown-preview :global(ul),
  .markdown-preview :global(ol) {
    padding-left: 2em;
    margin-top: 0;
    margin-bottom: 16px;
  }

  .markdown-preview :global(li) {
    margin-top: 0.25em;
  }

  .markdown-preview :global(p) {
    margin-top: 0;
    margin-bottom: 16px;
  }

  .markdown-preview :global(blockquote) {
    color: #6a737d;
    border-left: 0.25em solid #dfe2e5;
    padding: 0 1em;
    margin: 0 0 16px 0;
  }

  .markdown-preview :global(code) {
    padding: 0.2em 0.4em;
    margin: 0;
    font-size: 85%;
    background-color: #f6f8fa;
    border-radius: 3px;
    font-family:
      "SFMono-Regular", Consolas, "Liberation Mono", Menlo, monospace;
  }

  .markdown-preview :global(pre) {
    padding: 16px;
    overflow: auto;
    font-size: 85%;
    line-height: 1.45;
    background-color: #f6f8fa;
    border-radius: 3px;
    margin-bottom: 16px;
  }

  .markdown-preview :global(pre code) {
    padding: 0;
    background-color: transparent;
  }

  .markdown-preview :global(a) {
    color: #0366d6;
    text-decoration: none;
  }

  .markdown-preview :global(a:hover) {
    text-decoration: underline;
  }

  .markdown-preview :global(hr) {
    height: 0.25em;
    padding: 0;
    margin: 24px 0;
    background-color: #e1e4e8;
    border: 0;
  }

  .markdown-preview :global(img) {
    max-width: 100%;
    box-sizing: content-box;
    background-color: #fff;
  }

  .markdown-preview :global(table) {
    border-spacing: 0;
    border-collapse: collapse;
    margin-bottom: 16px;
    width: 100%;
  }

  .markdown-preview :global(td),
  .markdown-preview :global(th) {
    padding: 6px 13px;
    border: 1px solid #dfe2e5;
  }

  .markdown-preview :global(th) {
    font-weight: 600;
    background-color: #f6f8fa;
  }

  .markdown-preview :global(tr) {
    background-color: #fff;
    border-top: 1px solid #c6cbd1;
  }

  .markdown-preview :global(tr:nth-child(2n)) {
    background-color: #f6f8fa;
  }
</style>
