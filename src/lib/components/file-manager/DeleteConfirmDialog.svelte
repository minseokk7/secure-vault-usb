<!--
삭제 확인 다이얼로그 컴포넌트
선택된 파일/폴더 삭제 전 사용자 확인을 받는 모달
-->
<script lang="ts">
  import { createEventDispatcher } from 'svelte';

  // Props
  export let show = false;
  export let items = []; // 삭제할 항목들 (파일과 폴더 혼합)

  // 이벤트 디스패처
  const dispatch = createEventDispatcher();

  // 삭제 확인 처리
  function handleConfirm() {
    dispatch('confirmed');
    // show = false; // 부모 컴포넌트에서 처리하도록 제거
  }

  // 취소 처리
  function handleCancel() {
    dispatch('close'); // 닫기 이벤트 발생
    // show = false; // 부모 컴포넌트에서 처리하도록 제거
  }

  // 파일 크기 포맷팅
  function formatFileSize(bytes) {
    if (!bytes || bytes === 0) return '0 B';
    
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  }

  // 항목 타입 확인
  function isFile(item) {
    return item.file_name !== undefined;
  }

  // 총 크기 계산
  function getTotalSize() {
    return items.reduce((total, item) => {
      if (isFile(item)) {
        return total + (item.file_size || 0);
      } else {
        return total + (item.total_size || 0);
      }
    }, 0);
  }
</script>

{#if show}
  <div class="modal-overlay" onclick={handleCancel}>
    <div class="modal-content" onclick={(e) => e.stopPropagation()}>
      <div class="modal-header">
        <h2>⚠️ 항목 삭제 확인</h2>
        <button class="close-btn" onclick={handleCancel}>
          ✕
        </button>
      </div>
      
      <div class="modal-body">
        <div class="delete-confirm-area">
          <div class="warning-icon">⚠️</div>
          
          <h3>선택된 항목을 삭제하시겠습니까?</h3>
          <p class="warning-text">이 작업은 되돌릴 수 없습니다.</p>
          
          <div class="items-summary">
            <h4>삭제될 항목 ({items.length}개)</h4>
            <div class="items-list">
              {#each items.slice(0, 5) as item}
                <div class="item-row">
                  <span class="item-icon">
                    {isFile(item) ? '📄' : '📂'}
                  </span>
                  <span class="item-name">
                    {isFile(item) ? item.file_name : item.name}
                  </span>
                  <span class="item-size">
                    {formatFileSize(isFile(item) ? item.file_size : item.total_size)}
                  </span>
                </div>
              {/each}
              
              {#if items.length > 5}
                <div class="more-items">
                  ... 외 {items.length - 5}개 항목
                </div>
              {/if}
            </div>
            
            <div class="total-size">
              총 크기: {formatFileSize(getTotalSize())}
            </div>
          </div>
          
          <div class="button-group">
            <button class="cancel-btn" onclick={handleCancel}>
              취소
            </button>
            <button class="delete-btn" onclick={handleConfirm}>
              삭제 ({items.length}개 항목)
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
    color: #dc3545;
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

  .delete-confirm-area {
    text-align: center;
  }

  .warning-icon {
    font-size: 3rem;
    margin-bottom: 1rem;
  }

  .delete-confirm-area h3 {
    margin: 0 0 0.5rem 0;
    font-size: 1.2rem;
    font-weight: 600;
    color: #495057;
  }

  .warning-text {
    margin: 0 0 1.5rem 0;
    color: #dc3545;
    font-weight: 500;
  }

  .items-summary {
    background: #f8f9fa;
    border: 1px solid #e9ecef;
    border-radius: 6px;
    padding: 1rem;
    margin-bottom: 1.5rem;
    text-align: left;
  }

  .items-summary h4 {
    margin: 0 0 0.8rem 0;
    font-size: 1rem;
    font-weight: 600;
    color: #495057;
  }

  .items-list {
    max-height: 150px;
    overflow-y: auto;
    margin-bottom: 0.8rem;
  }

  .item-row {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.3rem 0;
    font-size: 0.9rem;
  }

  .item-icon {
    width: 20px;
    text-align: center;
  }

  .item-name {
    flex: 1;
    color: #495057;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .item-size {
    color: #6c757d;
    font-size: 0.8rem;
    min-width: 60px;
    text-align: right;
  }

  .more-items {
    text-align: center;
    color: #6c757d;
    font-style: italic;
    padding: 0.5rem 0;
    border-top: 1px solid #e9ecef;
    margin-top: 0.5rem;
  }

  .total-size {
    text-align: right;
    font-weight: 600;
    color: #495057;
    border-top: 1px solid #e9ecef;
    padding-top: 0.5rem;
  }

  .button-group {
    display: flex;
    gap: 0.8rem;
    justify-content: center;
  }

  .cancel-btn, .delete-btn {
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

  .delete-btn {
    background: #dc3545;
    color: white;
  }

  .delete-btn:hover {
    background: #c82333;
    transform: translateY(-1px);
    box-shadow: 0 2px 8px rgba(220, 53, 69, 0.3);
  }

  /* 스크롤바 스타일 */
  .items-list::-webkit-scrollbar {
    width: 6px;
  }

  .items-list::-webkit-scrollbar-track {
    background: #f1f3f4;
  }

  .items-list::-webkit-scrollbar-thumb {
    background: #cbd5e1;
    border-radius: 3px;
  }

  .items-list::-webkit-scrollbar-thumb:hover {
    background: #94a3b8;
  }
</style>