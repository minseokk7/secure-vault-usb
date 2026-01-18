<!--
  파일 목록 컴포넌트
  C# MainForm의 ListView 기능을 포팅
-->
<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import {
    fileManagerState,
    FileManagerService,
    filteredFiles,
    sortedFolders,
  } from "$lib/stores/file-manager";
  import type { FileMetadata, FolderEntry } from "$lib/types/file-manager";
  import RenameDialog from "./RenameDialog.svelte";
  import FileViewer from "../viewers/FileViewer.svelte";
  import type { FileEntry } from "$lib/types/file-manager";
  import { addToast } from "$lib/stores/toast";

  // 반응형 상태
  const viewMode = $derived($fileManagerState.viewMode);
  const selection = $derived($fileManagerState.selection);
  const files = $derived($filteredFiles);
  const folders = $derived($sortedFolders);

  // 파일 이름 변경 다이얼로그 상태
  let isRenameDialogOpen = $state(false);
  let fileToRename: FileEntry | null = $state(null);

  // 파일 뷰어 상태
  let isViewerOpen = $state(false);
  let viewerFile: FileMetadata | null = $state(null);

  // 파일 크기 포맷팅 (압축 정보 포함)
  function formatFileSize(bytes: number): string {
    if (bytes === 0) return "0 B";

    const k = 1024;
    const sizes = ["B", "KB", "MB", "GB", "TB"];
    const i = Math.floor(Math.log(bytes) / Math.log(k));

    return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + " " + sizes[i];
  }

  // 압축 정보를 포함한 파일 크기 표시
  function formatFileSizeWithCompression(
    file: FileMetadata,
    showCompressionInfo: boolean = false,
  ): string {
    const displaySize = formatFileSize(file.file_size);

    if (!showCompressionInfo || !file.is_compressed) {
      return displaySize;
    }

    const compressedSize = formatFileSize(file.compressed_size);
    const savedPercent = (
      ((file.file_size - file.compressed_size) / file.file_size) *
      100
    ).toFixed(1);

    return `${displaySize} (압축: ${compressedSize}, ${savedPercent}% 절약)`;
  }

  // 압축 상태 표시 아이콘
  function getCompressionIcon(file: FileMetadata): string {
    return file.is_compressed ? "📦" : "";
  }

  // 날짜 포맷팅
  function formatDate(dateString: string): string {
    const date = new Date(dateString);
    return date.toLocaleDateString("ko-KR", {
      year: "numeric",
      month: "2-digit",
      day: "2-digit",
      hour: "2-digit",
      minute: "2-digit",
    });
  }

  // 파일 타입 아이콘 가져오기
  function getFileIcon(file: FileMetadata): string {
    // mime_type이 undefined이거나 null인 경우 기본값 사용
    const mimeType = (file.mime_type || "").toLowerCase();

    if (mimeType.startsWith("image/")) return "🖼️";
    if (mimeType.startsWith("video/")) return "🎥";
    if (mimeType.startsWith("audio/")) return "🎵";
    if (mimeType.includes("pdf")) return "📄";
    if (
      mimeType.includes("text/") ||
      mimeType.includes("json") ||
      mimeType.includes("xml")
    )
      return "📝";
    if (
      mimeType.includes("zip") ||
      mimeType.includes("rar") ||
      mimeType.includes("7z")
    )
      return "📦";
    if (mimeType.includes("word") || mimeType.includes("document")) return "📄";
    if (mimeType.includes("excel") || mimeType.includes("spreadsheet"))
      return "📊";
    if (mimeType.includes("powerpoint") || mimeType.includes("presentation"))
      return "📈";

    return "📄"; // 기본 파일 아이콘
  }

  // 폴더 아이콘
  function getFolderIcon(): string {
    return "📁";
  }

  // 항목 선택 처리
  function handleItemClick(
    itemId: string,
    itemType: "file" | "folder",
    event: MouseEvent,
  ) {
    const multiSelect = event.ctrlKey || event.metaKey;
    FileManagerService.selectItem(itemId, itemType, multiSelect);
  }

  // 폴더 더블클릭 처리
  async function handleFolderDoubleClick(folder: FolderEntry) {
    await FileManagerService.navigateToFolder(folder.id);
  }

  // 파일 더블클릭 처리 (파일 뷰어 열기)
  function handleFileDoubleClick(file: FileMetadata) {
    console.log(
      "파일 더블클릭 이벤트 발생:",
      file.original_file_name || file.file_name,
    );
    console.log("파일 ID:", file.id);
    console.log("현재 viewerFile:", viewerFile);
    console.log("현재 isViewerOpen:", isViewerOpen);

    viewerFile = file;
    isViewerOpen = true;

    console.log("설정 후 viewerFile:", viewerFile);
    console.log("설정 후 isViewerOpen:", isViewerOpen);
    console.log("파일 뷰어 열기:", file.original_file_name || file.file_name);
  }

  // 항목이 선택되었는지 확인
  function isSelected(itemId: string, itemType: "file" | "folder"): boolean {
    if (itemType === "file") {
      return selection.selectedFiles.has(itemId);
    } else {
      return selection.selectedFolders.has(itemId);
    }
  }

  // 컨텍스트 메뉴 처리 (우클릭) - 현재 비활성화
  function handleContextMenu(
    event: MouseEvent,
    itemId: string,
    itemType: "file" | "folder",
  ) {
    event.preventDefault();
    // 컨텍스트 메뉴 기능 비활성화 - 툴바 삭제 버튼 사용
    console.log("항목 우클릭:", itemType, itemId, "- 컨텍스트 메뉴 비활성화됨");
  }

  // 키보드 이벤트 처리
  function handleKeydown(event: KeyboardEvent) {
    // F2 키로 파일/폴더 이름 변경
    if (event.key === "F2") {
      event.preventDefault();

      // 선택된 항목이 하나만 있을 때만 이름 변경 가능
      const totalSelected =
        selection.selectedFiles.size + selection.selectedFolders.size;

      if (totalSelected === 1) {
        if (selection.selectedFiles.size === 1) {
          // 파일 이름 변경
          const selectedFileId = Array.from(selection.selectedFiles)[0];
          const selectedFile = files.find((f) => f.id === selectedFileId);

          if (selectedFile) {
            startRename(selectedFile);
          }
        } else if (selection.selectedFolders.size === 1) {
          // 폴더 이름 변경 - FileManager의 handleRenameSelected 호출
          const selectedFolderId = Array.from(selection.selectedFolders)[0];
          const selectedFolder = folders.find((f) => f.id === selectedFolderId);

          if (selectedFolder) {
            // 커스텀 이벤트 발생시켜 FileManager에서 처리하도록 함
            window.dispatchEvent(
              new CustomEvent("requestFolderRename", {
                detail: { folderId: selectedFolderId },
              }),
            );
          }
        }
      }
    }
  }

  // 파일 이름 변경 시작
  function startRename(file: FileMetadata) {
    // FileMetadata를 FileEntry로 변환
    fileToRename = {
      id: file.id,
      file_name: file.original_file_name,
      original_file_name: file.original_file_name,
      file_size: file.file_size,
      file_extension: file.file_extension || "",
      mime_type: file.mime_type || "application/octet-stream",
      checksum: file.checksum || "",
      created_date: file.created_date,
      modified_date: file.modified_date,
      last_access_date: file.last_access_date || file.modified_date,
      folder_id: file.folder_id,
      encrypted_file_name: file.encrypted_file_name || "",
      encrypted_size: file.encrypted_size || file.file_size,
      tags: file.tags || [],
      description: file.description || "",
      version: file.version || 1,
      is_favorite: file.is_favorite || false,
      is_deleted: false,
      deleted_date: null,
      custom_properties: file.custom_properties || {},
      access_count: file.access_count || 0,
      security_level: file.security_level || { Normal: null },
    };

    isRenameDialogOpen = true;
  }

  // 파일 이름 변경 완료 처리
  function handleRenameComplete(
    event: CustomEvent<{ file: FileEntry; newName: string }>,
  ) {
    const { file, newName } = event.detail;

    // 파일 목록 새로고침
    FileManagerService.refresh();

    // 선택 상태 유지
    FileManagerService.selectItem(file.id, "file", false);
  }

  // 다이얼로그 닫기
  function handleRenameDialogClose() {
    isRenameDialogOpen = false;
    fileToRename = null;
  }

  // 파일 뷰어 닫기
  function handleViewerClose() {
    isViewerOpen = false;
    viewerFile = null;
  }

  // 파일 저장 처리
  function handleViewerSave(
    event: CustomEvent<{ fileId: string; content: string }>,
  ) {
    const { fileId, content } = event.detail;
    addToast("파일이 저장되었습니다.", "success");

    // 파일 목록 새로고침 (파일 크기나 수정 시간이 변경될 수 있음)
    FileManagerService.refresh();
  }

  // 컴포넌트 마운트 시 이벤트 리스너 등록
  let handleRenameRequest: (event: CustomEvent) => void;

  onMount(() => {
    handleRenameRequest = (event: CustomEvent) => {
      const { fileId } = event.detail;
      const file = files.find((f) => f.id === fileId);
      if (file) {
        startRename(file);
      }
    };

    window.addEventListener(
      "requestRename",
      handleRenameRequest as EventListener,
    );

    // 전역 드래그 방지 이벤트 리스너 추가
    const preventDrag = (e: DragEvent) => {
      e.preventDefault();
      e.stopPropagation();
      return false;
    };

    const preventDragStart = (e: DragEvent) => {
      e.preventDefault();
      e.stopPropagation();
      return false;
    };

    const preventSelectStart = (e: Event) => {
      const target = e.target as HTMLElement;
      // HTMLElement인지 확인하고 텍스트 에디터나 입력 필드가 아닌 경우에만 선택 방지
      if (target && typeof target.matches === "function") {
        if (!target.matches('input, textarea, [contenteditable="true"]')) {
          e.preventDefault();
          return false;
        }
      }
    };

    // 드래그 관련 이벤트 완전 차단
    document.addEventListener("dragstart", preventDragStart, true);
    document.addEventListener("drag", preventDrag, true);
    document.addEventListener("dragenter", preventDrag, true);
    document.addEventListener("dragover", preventDrag, true);
    document.addEventListener("dragleave", preventDrag, true);
    document.addEventListener("drop", preventDrag, true);
    document.addEventListener("dragend", preventDrag, true);
    document.addEventListener("selectstart", preventSelectStart, true);

    // 컨텍스트 메뉴도 방지 (우클릭 드래그 방지)
    document.addEventListener(
      "contextmenu",
      (e) => {
        const target = e.target as HTMLElement;
        if (target.closest(".file-list, .folder-tree")) {
          e.preventDefault();
          return false;
        }
      },
      true,
    );
  });

  onDestroy(() => {
    if (handleRenameRequest) {
      window.removeEventListener(
        "requestRename",
        handleRenameRequest as EventListener,
      );
    }

    // 드래그 방지 이벤트 리스너 제거
    const preventDrag = (e: DragEvent) => {
      e.preventDefault();
      e.stopPropagation();
      return false;
    };

    const preventDragStart = (e: DragEvent) => {
      e.preventDefault();
      e.stopPropagation();
      return false;
    };

    const preventSelectStart = (e: Event) => {
      const target = e.target as HTMLElement;
      if (!target.matches('input, textarea, [contenteditable="true"]')) {
        e.preventDefault();
        return false;
      }
    };

    document.removeEventListener("dragstart", preventDragStart, true);
    document.removeEventListener("drag", preventDrag, true);
    document.removeEventListener("dragenter", preventDrag, true);
    document.removeEventListener("dragover", preventDrag, true);
    document.removeEventListener("dragleave", preventDrag, true);
    document.removeEventListener("drop", preventDrag, true);
    document.removeEventListener("dragend", preventDrag, true);
    document.removeEventListener("selectstart", preventSelectStart, true);
  });
</script>

<!-- 파일 목록 -->
<div class="file-list {viewMode}" tabindex="0" onkeydown={handleKeydown}>
  {#if folders.length === 0 && files.length === 0}
    <!-- 빈 폴더 상태 -->
    <div class="empty-state">
      <div class="empty-icon">📂</div>
      <h3 class="empty-title text-korean">폴더가 비어있습니다</h3>
      <p class="empty-description text-korean">
        파일을 추가하거나 새 폴더를 생성해보세요.
      </p>
    </div>
  {:else}
    <!-- 목록 보기 -->
    {#if viewMode === "list"}
      <div class="list-view">
        <!-- 폴더들 -->
        {#each folders as folder}
          <div
            class="list-item folder no-drag {isSelected(folder.id, 'folder')
              ? 'selected'
              : ''}"
            onclick={(e) => handleItemClick(folder.id, "folder", e)}
            ondblclick={() => handleFolderDoubleClick(folder)}
            oncontextmenu={(e) => handleContextMenu(e, folder.id, "folder")}
            draggable="false"
          >
            <div class="item-icon no-drag" draggable="false">
              {getFolderIcon()}
            </div>
            <div class="item-info no-drag">
              <div class="item-name text-korean no-drag">{folder.name}</div>
              <div class="item-details no-drag">
                <span class="text-korean">{folder.file_count}개 파일</span>
                <span class="separator">•</span>
                <span>{formatFileSize(folder.total_size)}</span>
              </div>
            </div>
            <div class="item-date no-drag">
              {formatDate(folder.modified_at)}
            </div>
          </div>
        {/each}

        <!-- 파일들 -->
        {#each files as file}
          <div
            class="list-item file no-drag {isSelected(file.id, 'file')
              ? 'selected'
              : ''}"
            onclick={(e) => handleItemClick(file.id, "file", e)}
            ondblclick={() => handleFileDoubleClick(file)}
            oncontextmenu={(e) => handleContextMenu(e, file.id, "file")}
            draggable="false"
          >
            <div class="item-icon no-drag" draggable="false">
              {getFileIcon(file)}
            </div>
            <div class="item-info no-drag">
              <div class="item-name text-korean no-drag">
                {file.original_file_name}
              </div>
              <div class="item-details no-drag">
                <span>{formatFileSize(file.file_size)}</span>
                {#if file.is_compressed}
                  <span class="compression-info"
                    >📦 {(
                      ((file.file_size - file.compressed_size) /
                        file.file_size) *
                      100
                    ).toFixed(1)}% 절약</span
                  >
                {/if}
                <span class="separator">•</span>
                <span>{file.mime_type}</span>
                {#if file.is_favorite}
                  <span class="favorite-badge">⭐</span>
                {/if}
              </div>
            </div>
            <div class="item-date no-drag">
              {formatDate(file.modified_date)}
            </div>
          </div>
        {/each}
      </div>
    {/if}

    <!-- 격자 보기 -->
    {#if viewMode === "grid"}
      <div class="grid-view">
        <!-- 폴더들 -->
        {#each folders as folder}
          <div
            class="grid-item folder {isSelected(folder.id, 'folder')
              ? 'selected'
              : ''}"
            onclick={(e) => handleItemClick(folder.id, "folder", e)}
            ondblclick={() => handleFolderDoubleClick(folder)}
            oncontextmenu={(e) => handleContextMenu(e, folder.id, "folder")}
          >
            <div class="grid-icon">{getFolderIcon()}</div>
            <div class="grid-name text-korean">{folder.name}</div>
            <div class="grid-details text-korean">
              {folder.file_count}개 파일
            </div>
          </div>
        {/each}

        <!-- 파일들 -->
        {#each files as file}
          <div
            class="grid-item file {isSelected(file.id, 'file')
              ? 'selected'
              : ''}"
            onclick={(e) => handleItemClick(file.id, "file", e)}
            ondblclick={() => handleFileDoubleClick(file)}
            oncontextmenu={(e) => handleContextMenu(e, file.id, "file")}
          >
            <div class="grid-icon">
              {getFileIcon(file)}
              {#if file.is_compressed}
                <div class="compression-overlay">📦</div>
              {/if}
              {#if file.encrypted_file_name}
                <div class="encryption-overlay">🔒</div>
              {/if}
            </div>
            <div class="grid-name text-korean">{file.original_file_name}</div>
            <div class="grid-details">
              {formatFileSize(file.file_size)}
              {#if file.is_compressed}
                <div class="compression-badge">압축됨</div>
              {/if}
            </div>
          </div>
        {/each}
      </div>
    {/if}

    <!-- 자세히 보기 -->
    {#if viewMode === "details"}
      <div class="details-view">
        <!-- 헤더 -->
        <div class="details-header">
          <div class="header-cell name text-korean">이름</div>
          <div class="header-cell size text-korean">크기</div>
          <div class="header-cell compression text-korean">압축</div>
          <div class="header-cell type text-korean">형식</div>
          <div class="header-cell modified text-korean">수정일</div>
        </div>

        <!-- 폴더들 -->
        {#each folders as folder}
          <div
            class="details-row folder {isSelected(folder.id, 'folder')
              ? 'selected'
              : ''}"
            onclick={(e) => handleItemClick(folder.id, "folder", e)}
            ondblclick={() => handleFolderDoubleClick(folder)}
            oncontextmenu={(e) => handleContextMenu(e, folder.id, "folder")}
          >
            <div class="details-cell name">
              <span class="cell-icon">{getFolderIcon()}</span>
              <span class="cell-text text-korean">{folder.name}</span>
            </div>
            <div class="details-cell size">
              {formatFileSize(folder.total_size)}
            </div>
            <div class="details-cell compression">-</div>
            <div class="details-cell type text-korean">폴더</div>
            <div class="details-cell modified">
              {formatDate(folder.modified_at)}
            </div>
          </div>
        {/each}

        <!-- 파일들 -->
        {#each files as file}
          <div
            class="details-row file {isSelected(file.id, 'file')
              ? 'selected'
              : ''}"
            onclick={(e) => handleItemClick(file.id, "file", e)}
            ondblclick={() => handleFileDoubleClick(file)}
            oncontextmenu={(e) => handleContextMenu(e, file.id, "file")}
          >
            <div class="details-cell name">
              <span class="cell-icon">{getFileIcon(file)}</span>
              <span class="cell-text text-korean"
                >{file.original_file_name}</span
              >
              {#if file.encrypted_file_name}
                <span class="encryption-badge">🔒</span>
              {/if}
              {#if file.is_compressed}
                <span class="compression-badge">📦</span>
              {/if}
            </div>
            <div class="details-cell size">
              {formatFileSize(file.file_size)}
            </div>
            <div class="details-cell compression">
              {#if file.is_compressed}
                <span class="compression-info">
                  {formatFileSize(file.compressed_size)}
                  ({(
                    ((file.file_size - file.compressed_size) / file.file_size) *
                    100
                  ).toFixed(1)}% 절약)
                </span>
              {:else}
                <span class="no-compression">압축 안됨</span>
              {/if}
            </div>
            <div class="details-cell type">{file.mime_type}</div>
            <div class="details-cell modified">
              {formatDate(file.modified_date)}
            </div>
          </div>
        {/each}
      </div>
    {/if}
  {/if}
</div>

<!-- 파일 이름 변경 다이얼로그 -->
<RenameDialog
  bind:isOpen={isRenameDialogOpen}
  file={fileToRename}
  on:close={handleRenameDialogClose}
  on:renamed={handleRenameComplete}
/>

<!-- 파일 뷰어 -->
{#if viewerFile}
  <FileViewer
    file={viewerFile}
    bind:isOpen={isViewerOpen}
    on:close={handleViewerClose}
    on:save={handleViewerSave}
  />
{/if}

<style>
  .file-list {
    height: 100%;
    overflow-y: auto;
    padding: 1rem;
    outline: none; /* 포커스 아웃라인 제거 */
  }

  /* 빈 상태 */
  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    text-align: center;
    color: #6b7280;
  }

  .empty-icon {
    font-size: 4rem;
    margin-bottom: 1rem;
    opacity: 0.5;
  }

  .empty-title {
    font-size: 1.25rem;
    font-weight: 600;
    margin-bottom: 0.5rem;
    color: #374151;
  }

  .empty-description {
    font-size: 0.875rem;
    max-width: 300px;
  }

  /* 목록 보기 */
  .list-view {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  .list-item {
    display: flex;
    align-items: center;
    padding: 0.75rem;
    border-radius: 0.5rem;
    cursor: pointer;
    transition: background-color 0.15s ease;
    user-select: none;
  }

  .list-item:hover {
    background-color: #f3f4f6;
  }

  .list-item.selected {
    background-color: #dbeafe;
    border: 1px solid #93c5fd;
  }

  .item-icon {
    font-size: 1.5rem;
    margin-right: 0.75rem;
    flex-shrink: 0;
  }

  .item-info {
    flex: 1;
    min-width: 0;
  }

  .item-name {
    font-weight: 500;
    color: #111827;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .item-details {
    font-size: 0.75rem;
    color: #6b7280;
    margin-top: 0.25rem;
    display: flex;
    align-items: center;
    gap: 0.5rem;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .separator {
    color: #d1d5db;
  }

  .item-date {
    font-size: 0.75rem;
    color: #6b7280;
    flex-shrink: 0;
    margin-left: 1rem;
  }

  /* 격자 보기 */
  .grid-view {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(120px, 1fr));
    gap: 1rem;
  }

  .grid-item {
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 1rem;
    border-radius: 0.5rem;
    cursor: pointer;
    transition: background-color 0.15s ease;
    user-select: none;
    text-align: center;
  }

  .grid-item:hover {
    background-color: #f3f4f6;
  }

  .grid-item.selected {
    background-color: #dbeafe;
    border: 1px solid #93c5fd;
  }

  .grid-icon {
    font-size: 2.5rem;
    margin-bottom: 0.5rem;
    position: relative;
  }

  .grid-name {
    font-weight: 500;
    color: #111827;
    font-size: 0.875rem;
    margin-bottom: 0.25rem;
    word-break: break-word;
    line-height: 1.2;
  }

  .grid-details {
    font-size: 0.75rem;
    color: #6b7280;
  }

  /* 자세히 보기 */
  .details-view {
    display: flex;
    flex-direction: column;
  }

  .details-header {
    display: grid;
    grid-template-columns: 2fr 1fr 1fr 1fr 1fr;
    gap: 1rem;
    padding: 0.75rem;
    background-color: #f9fafb;
    border-bottom: 1px solid #e5e7eb;
    font-weight: 600;
    font-size: 0.875rem;
    color: #374151;
  }

  .details-row {
    display: grid;
    grid-template-columns: 2fr 1fr 1fr 1fr 1fr;
    gap: 1rem;
    padding: 0.75rem;
    border-bottom: 1px solid #f3f4f6;
    cursor: pointer;
    transition: background-color 0.15s ease;
    user-select: none;
  }

  .details-row:hover {
    background-color: #f9fafb;
  }

  .details-row.selected {
    background-color: #dbeafe;
  }

  .details-cell.name {
    display: flex;
    align-items: center;
    min-width: 0;
  }

  .cell-icon {
    font-size: 1.25rem;
    margin-right: 0.5rem;
    flex-shrink: 0;
  }

  .cell-text {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    font-weight: 500;
    color: #111827;
  }

  .details-cell {
    display: flex;
    align-items: center;
    font-size: 0.875rem;
    color: #6b7280;
  }

  /* 암호화 표시 */
  .encryption-badge {
    margin-left: 0.5rem;
    font-size: 0.75rem;
  }

  .encryption-overlay {
    position: absolute;
    bottom: -2px;
    right: -2px;
    font-size: 0.75rem;
    background-color: white;
    border-radius: 50%;
    padding: 1px;
  }

  /* 압축 정보 표시 */
  .compression-info {
    color: #059669;
    font-size: 0.75rem;
    font-weight: 500;
  }

  .compression-badge {
    background-color: #d1fae5;
    color: #065f46;
    font-size: 0.625rem;
    padding: 0.125rem 0.25rem;
    border-radius: 0.25rem;
    margin-top: 0.125rem;
  }

  .compression-overlay {
    position: absolute;
    top: -2px;
    left: -2px;
    font-size: 0.75rem;
    background-color: white;
    border-radius: 50%;
    padding: 1px;
  }

  .no-compression {
    color: #6b7280;
    font-size: 0.75rem;
  }

  .favorite-badge {
    color: #f59e0b;
    margin-left: 0.25rem;
  }

  /* 한국어 텍스트 최적화 */
  .text-korean {
    word-break: keep-all;
    line-height: 1.4;
  }

  /* 스크롤바 스타일링 */
  .file-list::-webkit-scrollbar {
    width: 8px;
  }

  .file-list::-webkit-scrollbar-track {
    background: #f1f5f9;
  }

  .file-list::-webkit-scrollbar-thumb {
    background: #cbd5e1;
    border-radius: 4px;
  }

  .file-list::-webkit-scrollbar-thumb:hover {
    background: #94a3b8;
  }

  /* 반응형 디자인 */
  @media (max-width: 768px) {
    .file-list {
      padding: 0.5rem;
    }

    .grid-view {
      grid-template-columns: repeat(auto-fill, minmax(100px, 1fr));
      gap: 0.75rem;
    }

    .details-header,
    .details-row {
      grid-template-columns: 2fr 1fr 1fr;
      gap: 0.5rem;
    }

    .details-cell.compression,
    .details-cell.type,
    .details-cell.modified {
      display: none;
    }

    .header-cell.compression,
    .header-cell.type,
    .header-cell.modified {
      display: none;
    }
  }
</style>
