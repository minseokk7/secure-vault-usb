<!-- 파일 뷰어 메인 컴포넌트 -->
<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { getFileViewerType } from "$lib/utils/file-type-detector";
  import type { FileMetadata } from "$lib/types/file-manager";
  import Modal from "$lib/components/common/Modal.svelte";
  import TextViewer from "./TextViewer.svelte";
  import MediaPlayer from "./MediaPlayer.svelte";
  import ImageViewer from "./ImageViewer.svelte";
  import { addToast } from "$lib/stores/toast";

  // Props
  export let file: FileMetadata;
  export let isOpen = false;

  // 이벤트 디스패처
  const dispatch = createEventDispatcher<{
    close: void;
    save: { fileId: string; content: string };
  }>();

  // 파일명 추출 (original_file_name 우선, 없으면 file_name 사용)
  $: fileName = file?.original_file_name || file?.file_name || "";

  // 파일 타입 감지 - 안전한 체크 추가
  $: viewerType = fileName
    ? getFileViewerType(fileName, file?.mime_type)
    : "unsupported";

  // 미디어 파일 크기에 따른 모달 크기 결정
  $: modalSize =
    viewerType === "media"
      ? file?.file_size && file.file_size > 50 * 1024 * 1024
        ? "large"
        : "medium"
      : "large";

  function handleClose() {
    dispatch("close");
  }

  async function handleSave(content: string) {
    try {
      console.log("파일 저장 시작:", {
        file_id: file.id,
        contentLength: content.length,
      });

      await invoke("save_text_file", {
        fileId: file.id,
        content: content,
      });

      console.log("파일 저장 완료:", file.id);
      dispatch("save", { file_id: file.id, content });
      addToast({
        type: "success",
        message: `파일 '${fileName}'이 성공적으로 저장되었습니다.`,
      });
    } catch (error) {
      console.error("파일 저장 오류:", error);
      addToast({
        type: "error",
        message: `파일 '${fileName}' 저장에 실패했습니다: ${error}`,
      });
      throw error;
    }
  }

  function handleExport() {
    // TODO: 파일 내보내기 기능 구현
    addToast({
      type: "info",
      message: "파일 내보내기 기능은 곧 구현될 예정입니다.",
    });
  }
</script>

<Modal
  {isOpen}
  title={`파일 뷰어 - ${fileName}`}
  size={modalSize}
  showFooter={false}
  onClose={handleClose}
>
  <div class="file-viewer-content">
    {#if viewerType === "text"}
      <TextViewer {file} onClose={handleClose} onSave={handleSave} />
    {:else if viewerType === "image"}
      <!-- 이미지 뷰어 -->
      <ImageViewer {file} onClose={handleClose} on:export={handleExport} />
    {:else if viewerType === "media"}
      <!-- 미디어 플레이어 -->
      <MediaPlayer {file} />
    {:else}
      <!-- 지원되지 않는 파일 형식 -->
      <div class="unsupported-file">
        <div class="unsupported-icon">📄</div>
        <h3>지원되지 않는 파일 형식</h3>
        <p>'{fileName}' 파일을 미리 볼 수 없습니다.</p>
        <p>파일을 내보내기하여 외부 프로그램으로 열어보세요.</p>

        <div class="file-details">
          <div class="detail-item">
            <span class="label">파일명:</span>
            <span class="value">{fileName}</span>
          </div>
          <div class="detail-item">
            <span class="label">크기:</span>
            <span class="value">{(file.file_size / 1024).toFixed(1)} KB</span>
          </div>
          {#if file.mime_type}
            <div class="detail-item">
              <span class="label">타입:</span>
              <span class="value">{file.mime_type}</span>
            </div>
          {/if}
        </div>

        <div class="unsupported-actions">
          <button class="btn btn-primary" on:click={handleExport}>
            파일 내보내기
          </button>
          <button class="btn" on:click={handleClose}> 닫기 </button>
        </div>
      </div>
    {/if}
  </div>
</Modal>

<style>
  .file-viewer-content {
    height: 70vh;
    min-height: 500px;
    display: flex;
    flex-direction: column;
  }

  .unsupported-file {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    text-align: center;
    padding: 40px;
  }

  .unsupported-icon {
    font-size: 64px;
    margin-bottom: 16px;
    opacity: 0.6;
  }

  .unsupported-file h3 {
    margin: 0 0 8px 0;
    color: #333;
    font-size: 24px;
  }

  .unsupported-file p {
    margin: 4px 0;
    color: #666;
    line-height: 1.5;
  }

  .file-details {
    margin: 24px 0;
    text-align: left;
    background: #f8f9fa;
    padding: 16px;
    border-radius: 8px;
    min-width: 300px;
  }

  .detail-item {
    display: flex;
    justify-content: space-between;
    margin-bottom: 8px;
  }

  .detail-item:last-child {
    margin-bottom: 0;
  }

  .label {
    font-weight: 600;
    color: #495057;
  }

  .value {
    color: #6c757d;
    font-family: "Consolas", "Monaco", "Courier New", monospace;
  }

  .unsupported-actions {
    display: flex;
    gap: 12px;
    margin-top: 24px;
  }

  .btn {
    padding: 8px 16px;
    border: 1px solid #ddd;
    background: white;
    border-radius: 4px;
    cursor: pointer;
    transition: all 0.2s;
    font-size: 14px;
  }

  .btn:hover {
    background: #f8f9fa;
  }

  .btn-primary {
    background: #007bff;
    color: white;
    border-color: #007bff;
  }

  .btn-primary:hover {
    background: #0056b3;
  }
</style>
