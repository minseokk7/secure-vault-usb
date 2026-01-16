<!--
파일 추가 다이얼로그 컴포넌트
외부 파일을 볼트에 추가하기 위한 다이얼로그
-->
<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { open } from '@tauri-apps/plugin-dialog';
  import { addToast } from '$lib/stores/toast';

  // Props
  export let show = false;
  export let currentFolderId = null;

  // 이벤트 디스패처
  const dispatch = createEventDispatcher();

  // 상태 변수
  let isUploading = false;
  let uploadProgress = 0;

  // 파일 선택 및 업로드
  async function handleFileSelect() {
    try {
      // 파일 선택 다이얼로그 열기
      const selected = await open({
        multiple: true,
        filters: [{
          name: '모든 파일',
          extensions: ['*']
        }]
      });

      if (!selected || (Array.isArray(selected) && selected.length === 0)) {
        return;
      }

      const filePaths = Array.isArray(selected) ? selected : [selected];
      
      isUploading = true;
      uploadProgress = 0;

      const totalFiles = filePaths.length;
      let completedFiles = 0;

      for (const filePath of filePaths) {
        try {
          console.log('파일 추가 중:', filePath);
          
          await invoke('add_file_to_vault', {
            filePath: filePath,
            folderId: currentFolderId
          });
          
          completedFiles++;
          uploadProgress = (completedFiles / totalFiles) * 100;
          
        } catch (error) {
          console.error('파일 추가 실패:', filePath, error);
          
          addToast({
            type: 'error',
            title: '파일 추가 실패',
            message: `${filePath} 추가 중 오류가 발생했습니다: ${error}`,
            duration: 5000
          });
        }
      }

      // 성공 메시지
      if (completedFiles > 0) {
        addToast({
          type: 'success',
          title: '파일 추가 완료',
          message: `${completedFiles}개 파일이 성공적으로 추가되었습니다.`,
          duration: 3000
        });

        // 파일 추가 완료 이벤트 발생
        dispatch('filesAdded', { count: completedFiles });
      }

      // 다이얼로그 닫기
      show = false;

    } catch (error) {
      console.error('파일 선택 실패:', error);
      
      addToast({
        type: 'error',
        title: '파일 선택 실패',
        message: `파일 선택 중 오류가 발생했습니다: ${error}`,
        duration: 5000
      });
    } finally {
      isUploading = false;
      uploadProgress = 0;
    }
  }

  // 다이얼로그 닫기
  function handleClose() {
    if (!isUploading) {
      show = false;
    }
  }
</script>

{#if show}
  <div class="modal-overlay" onclick={handleClose}>
    <div class="modal-content" onclick={(e) => e.stopPropagation()}>
      <div class="modal-header">
        <h2>📁 파일 추가</h2>
        <button class="close-btn" onclick={handleClose} disabled={isUploading}>
          ✕
        </button>
      </div>
      
      <div class="modal-body">
        {#if isUploading}
          <div class="upload-progress">
            <div class="progress-info">
              <span>파일 업로드 중...</span>
              <span>{Math.round(uploadProgress)}%</span>
            </div>
            <div class="progress-bar">
              <div class="progress-fill" style="width: {uploadProgress}%"></div>
            </div>
          </div>
        {:else}
          <div class="file-select-area">
            <div class="file-icon">📁</div>
            <h3>컴퓨터에서 파일을 선택하세요</h3>
            <p>하나 이상의 파일을 선택하여 볼트에 추가할 수 있습니다.</p>
            
            <button class="select-btn" onclick={handleFileSelect}>
              파일 선택
            </button>
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

  .file-select-area {
    text-align: center;
    padding: 2rem 1rem;
  }

  .file-icon {
    font-size: 3rem;
    margin-bottom: 1rem;
  }

  .file-select-area h3 {
    margin: 0 0 0.5rem 0;
    font-size: 1.1rem;
    font-weight: 600;
    color: #495057;
  }

  .file-select-area p {
    margin: 0 0 1.5rem 0;
    color: #6c757d;
    font-size: 0.9rem;
    line-height: 1.4;
  }

  .select-btn {
    padding: 0.8rem 2rem;
    background: #007bff;
    color: white;
    border: none;
    border-radius: 6px;
    font-size: 1rem;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .select-btn:hover {
    background: #0056b3;
    transform: translateY(-1px);
    box-shadow: 0 2px 8px rgba(0, 123, 255, 0.3);
  }

  .upload-progress {
    padding: 1rem;
  }

  .progress-info {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 0.8rem;
    font-size: 0.9rem;
    color: #495057;
  }

  .progress-bar {
    width: 100%;
    height: 8px;
    background: #e9ecef;
    border-radius: 4px;
    overflow: hidden;
  }

  .progress-fill {
    height: 100%;
    background: linear-gradient(90deg, #007bff, #0056b3);
    transition: width 0.3s ease;
  }
</style>