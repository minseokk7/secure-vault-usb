<script lang="ts">
  interface Props {
    show: boolean;
    currentFile: string;
    currentIndex: number;
    totalFiles: number;
    progress: number;
    isFolder?: boolean;
  }

  let {
    show = false,
    currentFile = "",
    currentIndex = 0,
    totalFiles = 0,
    progress = 0,
    isFolder = false,
  }: Props = $props();
</script>

{#if show}
  <div class="modal-overlay">
    <div class="progress-dialog">
      <div class="progress-header">
        <h3>{isFolder ? "폴더 업로드 중..." : "파일 업로드 중..."}</h3>
        <div class="progress-count">
          {currentIndex} / {totalFiles}
        </div>
      </div>

      <div class="progress-content">
        <div class="current-file">
          <span class="file-icon">{isFolder ? "📁" : "📄"}</span>
          <span class="file-name">{currentFile}</span>
        </div>

        <div class="progress-bar-container">
          <div class="progress-bar">
            <div class="progress-fill" style="width: {progress}%"></div>
          </div>
          <div class="progress-text">{Math.round(progress)}%</div>
        </div>

        <div class="progress-info">
          <span
            >{isFolder
              ? "폴더 구조 분석 및 파일 처리 중..."
              : "압축 및 암호화 중..."}</span
          >
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

  .progress-dialog {
    background: white;
    border-radius: 8px;
    box-shadow: 0 4px 20px rgba(0, 0, 0, 0.15);
    width: 500px;
    max-width: 90vw;
  }

  .progress-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 20px 24px;
    border-bottom: 1px solid #e2e8f0;
  }

  .progress-header h3 {
    margin: 0;
    font-size: 18px;
    font-weight: 600;
    color: #1e293b;
  }

  .progress-count {
    font-size: 14px;
    font-weight: 600;
    color: #2563eb;
    background: #dbeafe;
    padding: 4px 12px;
    border-radius: 12px;
  }

  .progress-content {
    padding: 24px;
  }

  .current-file {
    display: flex;
    align-items: center;
    gap: 12px;
    margin-bottom: 20px;
    padding: 12px;
    background: #f8fafc;
    border-radius: 6px;
  }

  .file-icon {
    font-size: 24px;
  }

  .file-name {
    font-size: 14px;
    color: #334155;
    font-weight: 500;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    flex: 1;
  }

  .progress-bar-container {
    display: flex;
    align-items: center;
    gap: 12px;
    margin-bottom: 16px;
  }

  .progress-bar {
    flex: 1;
    height: 8px;
    background: #e2e8f0;
    border-radius: 4px;
    overflow: hidden;
  }

  .progress-fill {
    height: 100%;
    background: linear-gradient(90deg, #2563eb 0%, #3b82f6 100%);
    transition: width 0.3s ease;
    border-radius: 4px;
  }

  .progress-text {
    font-size: 14px;
    font-weight: 600;
    color: #2563eb;
    min-width: 45px;
    text-align: right;
  }

  .progress-info {
    text-align: center;
    font-size: 13px;
    color: #64748b;
  }
</style>
