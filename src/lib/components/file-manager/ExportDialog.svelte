<!--
파일 내보내기 다이얼로그 컴포넌트
선택된 파일들을 외부로 내보내기 위한 다이얼로그
-->
<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { save } from '@tauri-apps/plugin-dialog';

  // Props
  export let show = false;
  export let files = []; // 내보낼 파일들

  // 이벤트 디스패처
  const dispatch = createEventDispatcher();

  // 상태 변수
  let exportPath = '';
  let isExporting = false;

  // 다이얼로그가 열릴 때마다 초기화
  $: if (show) {
    exportPath = '';
    isExporting = false;
  }

  // 저장 위치 선택
  async function handleSelectPath() {
    try {
      let selected;
      
      if (files.length === 1) {
        // 단일 파일: 파일 저장 다이얼로그
        selected = await save({
          title: '파일 저장 위치 선택',
          defaultPath: files[0].original_file_name || files[0].file_name
        });
      } else {
        // 다중 파일: 폴더 선택 다이얼로그
        const { open } = await import('@tauri-apps/plugin-dialog');
        selected = await open({
          directory: true,
          title: '내보낼 폴더 선택'
        });
      }
      
      if (selected) {
        exportPath = selected;
      }
    } catch (error) {
      console.error('경로 선택 실패:', error);
    }
  }

  // 내보내기 실행
  async function handleExport() {
    if (!exportPath.trim()) {
      alert('내보낼 경로를 선택해주세요.');
      return;
    }

    isExporting = true;

    try {
      dispatch('exported', { exportPath: exportPath.trim() });
      // show = false; // 부모 컴포넌트에서 처리하도록 제거
    } catch (error) {
      console.error('내보내기 실패:', error);
      alert(`내보내기 실패: ${error}`);
    } finally {
      isExporting = false;
    }
  }

  // 취소 처리
  function handleCancel() {
    if (!isExporting) {
      dispatch('close'); // 닫기 이벤트 발생
      // show = false; // 부모 컴포넌트에서 처리하도록 제거
    }
  }

  // 파일 크기 포맷팅
  function formatFileSize(bytes) {
    if (!bytes || bytes === 0) return '0 B';
    
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  }

  // 총 크기 계산
  function getTotalSize() {
    return files.reduce((total, file) => total + (file.file_size || 0), 0);
  }
</script>

{#if show}
  <div class="modal-overlay" onclick={handleCancel}>
    <div class="modal-content" onclick={(e) => e.stopPropagation()}>
      <div class="modal-header">
        <h2>📤 파일 내보내기</h2>
        <button class="close-btn" onclick={handleCancel} disabled={isExporting}>
          ✕
        </button>
      </div>
      
      <div class="modal-body">
        {#if !isExporting}
          <div class="export-setup">
            <div class="files-summary">
              <h3>내보낼 파일 ({files.length}개)</h3>
              <div class="files-list">
                {#each files.slice(0, 5) as file}
                  <div class="file-row">
                    <span class="file-icon">📄</span>
                    <span class="file-name">{file.file_name}</span>
                    <span class="file-size">{formatFileSize(file.file_size)}</span>
                  </div>
                {/each}
                
                {#if files.length > 5}
                  <div class="more-files">
                    ... 외 {files.length - 5}개 파일
                  </div>
                {/if}
              </div>
              
              <div class="total-info">
                총 크기: {formatFileSize(getTotalSize())}
              </div>
            </div>
            
            <div class="path-selection">
              <label for="export-path">
                {files.length === 1 ? '저장 위치:' : '내보낼 폴더:'}
              </label>
              <div class="path-input-group">
                <input
                  id="export-path"
                  type="text"
                  bind:value={exportPath}
                  placeholder={files.length === 1 ? '파일 저장 경로' : '폴더 경로'}
                  class="path-input"
                />
                <button class="select-path-btn" onclick={handleSelectPath}>
                  선택
                </button>
              </div>
            </div>
            
            <div class="button-group">
              <button class="cancel-btn" onclick={handleCancel}>
                취소
              </button>
              <button 
                class="export-btn" 
                onclick={handleExport}
                disabled={!exportPath.trim()}
              >
                내보내기 ({files.length}개 파일)
              </button>
            </div>
          </div>
        {:else}
          <div class="export-progress">
            <div class="progress-icon">📤</div>
            <h3>파일 내보내기 중...</h3>
            <p>잠시만 기다려주세요.</p>
          </div>
        {/if}
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
    min-width: 500px;
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

  .close-btn:hover:not(:disabled) {
    background: #e9ecef;
    color: #495057;
  }

  .close-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .modal-body {
    padding: 1.5rem;
  }

  .export-setup {
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
  }

  .files-summary h3 {
    margin: 0 0 1rem 0;
    font-size: 1.1rem;
    font-weight: 600;
    color: #495057;
  }

  .files-list {
    background: #f8f9fa;
    border: 1px solid #e9ecef;
    border-radius: 6px;
    padding: 1rem;
    max-height: 200px;
    overflow-y: auto;
    margin-bottom: 0.8rem;
  }

  .file-row {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.3rem 0;
    font-size: 0.9rem;
  }

  .file-icon {
    width: 20px;
    text-align: center;
  }

  .file-name {
    flex: 1;
    color: #495057;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .file-size {
    color: #6c757d;
    font-size: 0.8rem;
    min-width: 60px;
    text-align: right;
  }

  .more-files {
    text-align: center;
    color: #6c757d;
    font-style: italic;
    padding: 0.5rem 0;
    border-top: 1px solid #e9ecef;
    margin-top: 0.5rem;
  }

  .total-info {
    text-align: right;
    font-weight: 600;
    color: #495057;
    border-top: 1px solid #e9ecef;
    padding-top: 0.5rem;
  }

  .path-selection label {
    display: block;
    margin-bottom: 0.5rem;
    font-weight: 500;
    color: #495057;
    font-size: 0.9rem;
  }

  .path-input-group {
    display: flex;
    gap: 0.5rem;
  }

  .path-input {
    flex: 1;
    padding: 0.8rem;
    border: 1px solid #ced4da;
    border-radius: 6px;
    font-size: 0.9rem;
    transition: all 0.2s ease;
  }

  .path-input:focus {
    outline: none;
    border-color: #007bff;
    box-shadow: 0 0 0 3px rgba(0, 123, 255, 0.1);
  }

  .select-path-btn {
    padding: 0.8rem 1.2rem;
    background: #6c757d;
    color: white;
    border: none;
    border-radius: 6px;
    cursor: pointer;
    font-size: 0.9rem;
    transition: all 0.2s ease;
  }

  .select-path-btn:hover {
    background: #545b62;
  }

  .button-group {
    display: flex;
    gap: 0.8rem;
    justify-content: flex-end;
    padding-top: 1rem;
    border-top: 1px solid #e9ecef;
  }

  .cancel-btn, .export-btn {
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

  .export-btn {
    background: #28a745;
    color: white;
  }

  .export-btn:hover:not(:disabled) {
    background: #218838;
    transform: translateY(-1px);
    box-shadow: 0 2px 8px rgba(40, 167, 69, 0.3);
  }

  .export-btn:disabled {
    background: #6c757d;
    cursor: not-allowed;
    opacity: 0.6;
  }

  .export-progress {
    text-align: center;
    padding: 2rem 1rem;
  }

  .progress-icon {
    font-size: 3rem;
    margin-bottom: 1rem;
    animation: bounce 1s infinite;
  }

  .export-progress h3 {
    margin: 0 0 0.5rem 0;
    font-size: 1.2rem;
    font-weight: 600;
    color: #495057;
  }

  .export-progress p {
    margin: 0;
    color: #6c757d;
  }

  @keyframes bounce {
    0%, 20%, 50%, 80%, 100% {
      transform: translateY(0);
    }
    40% {
      transform: translateY(-10px);
    }
    60% {
      transform: translateY(-5px);
    }
  }

  /* 스크롤바 스타일 */
  .files-list::-webkit-scrollbar {
    width: 6px;
  }

  .files-list::-webkit-scrollbar-track {
    background: #f1f3f4;
  }

  .files-list::-webkit-scrollbar-thumb {
    background: #cbd5e1;
    border-radius: 3px;
  }

  .files-list::-webkit-scrollbar-thumb:hover {
    background: #94a3b8;
  }
</style>