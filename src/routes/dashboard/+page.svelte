<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { goto } from "$app/navigation";
  import { authState, AuthService } from "$lib/stores/auth";
  import { enableToasts, addToast } from "$lib/stores/toast";
  import { invoke } from "@tauri-apps/api/core";
  import { open, save } from "@tauri-apps/plugin-dialog";

  // 다이얼로그 컴포넌트들
  import SimpleCreateFolderDialog from "$lib/components/file-manager/SimpleCreateFolderDialog.svelte";
  import SimpleNewFileDialog from "$lib/components/file-manager/SimpleNewFileDialog.svelte";
  import DeleteConfirmDialog from "$lib/components/file-manager/DeleteConfirmDialog.svelte";
  import ExportDialog from "$lib/components/file-manager/ExportDialog.svelte";
  import SimpleRenameDialog from "$lib/components/file-manager/SimpleRenameDialog.svelte";

  let isInitializing = $state(true);

  // 파일 매니저 상태
  let files = $state<any[]>([]);
  let folders = $state<any[]>([]);
  let selectedFiles = $state<Set<string>>(new Set());
  let selectedFolders = $state<Set<string>>(new Set());
  let currentFolder = $state<any>(null);

  // 검색 및 정렬
  let searchQuery = $state("");
  let sortBy = $state("name");
  let sortOrder = $state<"asc" | "desc">("asc");
  let viewMode = $state<"list" | "grid" | "detail">("list");

  // 다이얼로그 상태
  let showCreateFolderDialog = $state(false);
  let showNewFileDialog = $state(false);
  let showDeleteDialog = $state(false);
  let showExportDialog = $state(false);
  let showRenameDialog = $state(false);
  let showFileViewer = $state(false);

  // 컨텍스트 메뉴 상태
  let showContextMenu = $state(false);
  let contextMenuX = $state(0);
  let contextMenuY = $state(0);
  let contextMenuTarget = $state<any>(null);
  let contextMenuType = $state<"file" | "folder" | "empty">("empty");

  // 다이얼로그 데이터
  let deleteItems = $state<any[]>([]);
  let exportFiles = $state<any[]>([]);
  let renameItem = $state<any>(null);
  let renameItemType = $state<"file" | "folder">("file");
  let viewerFile = $state<any>(null);
  let viewerContent = $state("");
  let isViewerLoading = $state(false);

  // 시간 상태
  let currentTime = $state(new Date());
  let sessionTime = $state(3600);
  let timeInterval: number | null = null;

  // 선택된 항목 수
  let selectedCount = $derived(selectedFiles.size + selectedFolders.size);

  // 필터링된 파일 목록
  let filteredFiles = $derived.by(() => {
    let result = [...files];

    // 검색 필터
    if (searchQuery.trim()) {
      const query = searchQuery.toLowerCase();
      result = result.filter(
        (f) =>
          f.file_name?.toLowerCase().includes(query) ||
          f.original_file_name?.toLowerCase().includes(query),
      );
    }

    // 정렬
    result.sort((a, b) => {
      let compare = 0;
      switch (sortBy) {
        case "name":
          compare = (a.file_name || "").localeCompare(b.file_name || "");
          break;
        case "date":
          compare =
            new Date(a.modified_date || 0).getTime() -
            new Date(b.modified_date || 0).getTime();
          break;
        case "size":
          compare = (a.file_size || 0) - (b.file_size || 0);
          break;
        case "type":
          compare = (a.file_extension || "").localeCompare(
            b.file_extension || "",
          );
          break;
      }
      return sortOrder === "asc" ? compare : -compare;
    });

    return result;
  });

  // 필터링된 폴더 목록 (현재 폴더의 하위 폴더만)
  let filteredFolders = $derived.by(() => {
    // 현재 폴더의 하위 폴더만 필터링
    let result = folders.filter((f) => {
      const parentId = f.parent_id || null;
      const currentId = currentFolder?.id || null;
      return parentId === currentId;
    });

    if (searchQuery.trim()) {
      const query = searchQuery.toLowerCase();
      result = result.filter((f) => f.name?.toLowerCase().includes(query));
    }

    result.sort((a, b) => (a.name || "").localeCompare(b.name || ""));
    return result;
  });

  // 파일 추가 핸들러
  async function handleAddFile() {
    try {
      console.log("파일 추가 시작...");
      const selected = await open({
        multiple: true,
        filters: [{ name: "모든 파일", extensions: ["*"] }],
      });

      console.log("선택된 파일:", selected);

      if (selected) {
        const paths = Array.isArray(selected) ? selected : [selected];
        let successCount = 0;
        let lastError = null;

        for (const path of paths) {
          try {
            console.log("파일 추가 invoke:", path);
            await invoke("add_file_to_vault", {
              filePath: path,
              fileName: null,
              folderId: currentFolder?.id || null,
            });
            console.log("파일 추가 성공:", path);
            successCount++;
          } catch (e) {
            console.error("파일 추가 실패:", path, e);
            lastError = e;
          }
        }

        if (successCount > 0) {
          addToast({
            type: "success",
            title: "파일 추가 완료",
            message: `${successCount}개 파일이 추가되었습니다.`,
            duration: 3000,
          });
          await loadFiles();
        } else if (lastError) {
          addToast({
            type: "error",
            title: "파일 추가 실패",
            message:
              typeof lastError === "string" ? lastError : String(lastError),
            duration: 5000,
          });
        }
      }
    } catch (error) {
      console.error("파일 추가 실패:", error);
      addToast({
        type: "error",
        title: "파일 추가 실패",
        message: typeof error === "string" ? error : String(error),
        duration: 5000,
      });
    }
  }

  // 폴더 추가 핸들러
  async function handleAddFolder() {
    try {
      console.log("폴더 추가 시작...");
      const selected = await open({ directory: true, multiple: false });
      console.log("선택된 폴더:", selected);

      if (selected && typeof selected === "string") {
        console.log("폴더 추가 invoke:", selected);
        const result = await invoke<{
          folder_count: number;
          file_count: number;
        }>("add_folder_to_vault", {
          folderPath: selected,
          targetFolderId: currentFolder?.id || null,
        });
        console.log("폴더 추가 결과:", result);

        addToast({
          type: "success",
          title: "폴더 추가 완료",
          message: `폴더 ${result.folder_count}개, 파일 ${result.file_count}개가 추가되었습니다.`,
          duration: 3000,
        });
        await loadFolders();
        await loadFiles();
      }
    } catch (error) {
      console.error("폴더 추가 실패:", error);
      addToast({
        type: "error",
        title: "폴더 추가 실패",
        message: typeof error === "string" ? error : String(error),
        duration: 5000,
      });
    }
  }

  function handleCreateFile() {
    showNewFileDialog = true;
  }
  function handleCreateFolder() {
    showCreateFolderDialog = true;
  }

  function handleRename() {
    if (selectedFiles.size === 1) {
      const fileId = Array.from(selectedFiles)[0];
      const file = files.find((f) => f.id === fileId);
      if (file) {
        renameItem = file;
        renameItemType = "file";
        showRenameDialog = true;
      }
    } else if (selectedFolders.size === 1) {
      const folderId = Array.from(selectedFolders)[0];
      const folder = folders.find((f) => f.id === folderId);
      if (folder) {
        renameItem = folder;
        renameItemType = "folder";
        showRenameDialog = true;
      }
    }
  }

  function handleDelete() {
    const selectedFilesList = Array.from(selectedFiles)
      .map((id) => files.find((f) => f.id === id))
      .filter(Boolean);
    const selectedFoldersList = Array.from(selectedFolders)
      .map((id) => folders.find((f) => f.id === id))
      .filter(Boolean);
    deleteItems = [...selectedFilesList, ...selectedFoldersList];
    showDeleteDialog = true;
  }

  function handleExport() {
    const selectedFilesList = Array.from(selectedFiles)
      .map((id) => files.find((f) => f.id === id))
      .filter(Boolean);
    exportFiles = selectedFilesList;
    showExportDialog = true;
  }

  async function handleRefresh() {
    try {
      await Promise.all([loadFiles(), loadFolders()]);
      addToast({
        type: "success",
        title: "새로고침 완료",
        message: "파일 목록이 업데이트되었습니다.",
        duration: 2000,
      });
    } catch (error) {
      console.error("새로고침 실패:", error);
    }
  }

  // 검색 핸들러
  function handleSearch() {
    // 이미 derived에서 필터링되므로 별도 처리 불필요
    if (searchQuery.trim()) {
      addToast({
        type: "info",
        title: "검색 결과",
        message: `${filteredFiles.length}개 파일, ${filteredFolders.length}개 폴더`,
        duration: 2000,
      });
    }
  }

  // 정렬 변경
  function toggleSortOrder() {
    sortOrder = sortOrder === "asc" ? "desc" : "asc";
  }

  // 파일 목록 로드
  async function loadFiles() {
    try {
      const fileList = await invoke<any[]>("get_files_in_folder", {
        folderId: currentFolder?.id || null,
      });
      files = Array.isArray(fileList) ? fileList : [];
    } catch (error) {
      console.error("파일 목록 로드 실패:", error);
      files = [];
    }
  }

  // 폴더 목록 로드
  async function loadFolders() {
    try {
      const folderTree = await invoke<any[]>("get_folder_tree");
      folders = Array.isArray(folderTree) ? folderTree : [];
    } catch (error) {
      console.error("폴더 목록 로드 실패:", error);
      folders = [];
    }
  }

  function selectFolder(folder: any | null) {
    currentFolder = folder;
    selectedFiles = new Set();
    selectedFolders = new Set();
    loadFiles();
  }

  function toggleFileSelection(fileId: string, event: MouseEvent) {
    if (event.ctrlKey || event.metaKey) {
      if (selectedFiles.has(fileId)) selectedFiles.delete(fileId);
      else selectedFiles.add(fileId);
      selectedFiles = new Set(selectedFiles);
    } else if (event.shiftKey && selectedFiles.size > 0) {
      // 범위 선택
      const fileIds = filteredFiles.map((f) => f.id);
      const lastSelected = Array.from(selectedFiles).pop();
      const lastIdx = fileIds.indexOf(lastSelected!);
      const currentIdx = fileIds.indexOf(fileId);
      const [start, end] =
        lastIdx < currentIdx ? [lastIdx, currentIdx] : [currentIdx, lastIdx];
      for (let i = start; i <= end; i++) {
        selectedFiles.add(fileIds[i]);
      }
      selectedFiles = new Set(selectedFiles);
    } else {
      selectedFiles = new Set([fileId]);
      selectedFolders = new Set();
    }
  }

  function toggleFolderSelection(folderId: string, event: MouseEvent) {
    if (event.ctrlKey || event.metaKey) {
      if (selectedFolders.has(folderId)) selectedFolders.delete(folderId);
      else selectedFolders.add(folderId);
      selectedFolders = new Set(selectedFolders);
    } else {
      selectedFolders = new Set([folderId]);
      selectedFiles = new Set();
    }
  }

  // 파일 뷰어 열기
  async function openFile(file: any) {
    viewerFile = file;
    isViewerLoading = true;
    showFileViewer = true;

    try {
      // 텍스트 파일인지 확인
      const textExtensions = [
        "txt",
        "md",
        "json",
        "js",
        "ts",
        "html",
        "css",
        "xml",
        "yaml",
        "yml",
        "ini",
        "cfg",
        "log",
        "py",
        "rs",
        "go",
        "java",
        "c",
        "cpp",
        "h",
        "sh",
        "bat",
        "ps1",
      ];
      const ext = (file.file_extension || "").toLowerCase();

      if (textExtensions.includes(ext)) {
        const content = await invoke<string>("get_text_file_content", {
          fileId: file.id,
        });
        viewerContent = content;
      } else if (file.mime_type?.startsWith("image/")) {
        // 이미지 파일은 base64로 로드
        const data = await invoke<number[]>("get_file_content", {
          fileId: file.id,
        });
        const base64 = btoa(String.fromCharCode(...new Uint8Array(data)));
        viewerContent = `data:${file.mime_type};base64,${base64}`;
      } else {
        viewerContent = "";
      }
    } catch (error) {
      console.error("파일 로드 실패:", error);
      addToast({
        type: "error",
        title: "파일 열기 실패",
        message: typeof error === "string" ? error : "파일을 열 수 없습니다.",
        duration: 3000,
      });
      showFileViewer = false;
      viewerFile = null;
    } finally {
      isViewerLoading = false;
    }
  }

  function closeViewer() {
    showFileViewer = false;
    viewerFile = null;
    viewerContent = "";
  }

  // 컨텍스트 메뉴
  function showContextMenuHandler(
    event: MouseEvent,
    target: any | null,
    type: "file" | "folder" | "empty",
  ) {
    event.preventDefault();
    contextMenuX = event.clientX;
    contextMenuY = event.clientY;
    contextMenuTarget = target;
    contextMenuType = type;
    showContextMenu = true;

    if (type === "file" && target && !selectedFiles.has(target.id)) {
      selectedFiles = new Set([target.id]);
      selectedFolders = new Set();
    } else if (type === "folder" && target && !selectedFolders.has(target.id)) {
      selectedFolders = new Set([target.id]);
      selectedFiles = new Set();
    }
  }

  function hideContextMenu() {
    showContextMenu = false;
  }

  function handleContextMenuAction(action: string) {
    hideContextMenu();
    switch (action) {
      case "open":
        if (contextMenuTarget) openFile(contextMenuTarget);
        break;
      case "rename":
        handleRename();
        break;
      case "delete":
        handleDelete();
        break;
      case "export":
        handleExport();
        break;
      case "newFile":
        handleCreateFile();
        break;
      case "newFolder":
        handleCreateFolder();
        break;
      case "refresh":
        handleRefresh();
        break;
    }
  }

  // 다이얼로그 핸들러
  async function onFolderCreated(event: CustomEvent<{ name: string }>) {
    showCreateFolderDialog = false;
    try {
      console.log(
        "폴더 생성 시작:",
        event.detail.name,
        "부모:",
        currentFolder?.id,
      );
      await invoke("create_folder", {
        name: event.detail.name,
        parentId: currentFolder?.id || null,
      });
      console.log("폴더 생성 성공");
      addToast({
        type: "success",
        title: "폴더 생성 완료",
        message: `'${event.detail.name}' 폴더가 생성되었습니다.`,
        duration: 3000,
      });
      await loadFolders();
    } catch (error) {
      console.error("폴더 생성 실패:", error);
      addToast({
        type: "error",
        title: "폴더 생성 실패",
        message: typeof error === "string" ? error : String(error),
        duration: 5000,
      });
    }
  }

  async function onFileCreated(
    event: CustomEvent<{ fileName: string; content: string }>,
  ) {
    showNewFileDialog = false;
    try {
      await invoke("create_new_file_in_vault", {
        folderId: currentFolder?.id || null,
        fileName: event.detail.fileName,
        content: event.detail.content,
      });
      addToast({
        type: "success",
        title: "파일 생성 완료",
        message: `'${event.detail.fileName}' 파일이 생성되었습니다.`,
        duration: 3000,
      });
      await loadFiles();
    } catch (error) {
      addToast({
        type: "error",
        title: "파일 생성 실패",
        message:
          typeof error === "string" ? error : "파일을 생성할 수 없습니다.",
        duration: 5000,
      });
    }
  }

  async function onDeleteConfirmed() {
    showDeleteDialog = false;

    // 현재 폴더가 삭제 목록에 포함되어 있는지 확인
    const currentDeleted =
      currentFolder && deleteItems.some((item) => item.id === currentFolder.id);
    const targetParentId = currentFolder?.parent_id;

    try {
      for (const file of deleteItems.filter(
        (item) => item.file_name !== undefined,
      )) {
        await invoke("delete_file_from_vault", { fileId: file.id });
      }
      for (const folder of deleteItems.filter(
        (item) => item.file_name === undefined,
      )) {
        await invoke("delete_folder", {
          folderId: folder.id,
          recursive: true,
        });
      }
      addToast({
        type: "success",
        title: "삭제 완료",
        message: `${deleteItems.length}개 항목이 삭제되었습니다.`,
        duration: 3000,
      });

      // 데이터 갱신
      await Promise.all([loadFiles(), loadFolders()]);

      if (currentDeleted) {
        // 상위 폴더로 이동
        const parent = folders.find((f) => f.id === targetParentId);
        selectFolder(parent || null);
      } else {
        selectedFiles = new Set();
        selectedFolders = new Set();
      }
    } catch (error) {
      addToast({
        type: "error",
        title: "삭제 실패",
        message:
          typeof error === "string" ? error : "항목을 삭제할 수 없습니다.",
        duration: 5000,
      });
    }
  }

  async function onExported(event: CustomEvent<{ exportPath: string }>) {
    showExportDialog = false;
    try {
      for (const file of exportFiles) {
        await invoke("export_file_from_vault", {
          fileId: file.id,
          exportPath: event.detail.exportPath,
        });
      }
      addToast({
        type: "success",
        title: "내보내기 완료",
        message: `${exportFiles.length}개 파일이 내보내졌습니다.`,
        duration: 3000,
      });
      selectedFiles = new Set();
    } catch (error) {
      addToast({
        type: "error",
        title: "내보내기 실패",
        message: "파일을 내보낼 수 없습니다.",
        duration: 5000,
      });
    }
  }

  async function onRenamed(event: CustomEvent<{ newName: string }>) {
    showRenameDialog = false;
    try {
      if (renameItemType === "file") {
        await invoke("rename_file_in_vault", {
          fileId: renameItem.id,
          newName: event.detail.newName,
        });
      } else {
        await invoke("rename_folder", {
          folderId: renameItem.id,
          newName: event.detail.newName,
        });
      }
      addToast({
        type: "success",
        title: "이름 변경 완료",
        message: `'${event.detail.newName}'으로 변경되었습니다.`,
        duration: 3000,
      });
      selectedFiles = new Set();
      selectedFolders = new Set();
      await Promise.all([loadFiles(), loadFolders()]);
    } catch (error) {
      addToast({
        type: "error",
        title: "이름 변경 실패",
        message:
          typeof error === "string" ? error : "이름을 변경할 수 없습니다.",
        duration: 5000,
      });
    }
  }

  function formatFileSize(bytes: number): string {
    if (bytes === 0) return "0 B";
    const k = 1024;
    const sizes = ["B", "KB", "MB", "GB"];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + " " + sizes[i];
  }

  function formatSessionTime(seconds: number): string {
    const mins = Math.floor(seconds / 60);
    const secs = seconds % 60;
    return `${mins}:${secs.toString().padStart(2, "0")}`;
  }

  function getFileIcon(file: any): string {
    const ext = (file.file_extension || "").toLowerCase();
    if (file.mime_type?.startsWith("image/")) return "🖼️";
    if (file.mime_type?.startsWith("video/")) return "🎥";
    if (file.mime_type?.startsWith("audio/")) return "🎵";
    if (ext === "pdf") return "📄";
    if (["doc", "docx"].includes(ext)) return "📝";
    if (["xls", "xlsx"].includes(ext)) return "📊";
    if (["ppt", "pptx"].includes(ext)) return "📽️";
    if (["zip", "rar", "7z", "tar", "gz"].includes(ext)) return "📦";
    if (["js", "ts", "py", "rs", "go", "java", "c", "cpp", "h"].includes(ext))
      return "💻";
    if (["html", "css", "xml", "json", "yaml", "yml"].includes(ext))
      return "🌐";
    if (["txt", "md", "log"].includes(ext)) return "📄";
    return "📄";
  }

  // 폴더 깊이 계산 함수
  function getFolderDepth(folder: any): number {
    let depth = 0;
    let currentParentId = folder.parent_id;
    while (currentParentId) {
      depth++;
      const parent = folders.find((f) => f.id === currentParentId);
      if (parent) {
        currentParentId = parent.parent_id;
      } else {
        break;
      }
    }
    return depth;
  }

  // 폴더 트리 접두사 생성 함수
  function getFolderTreePrefix(folder: any): string {
    const depth = getFolderDepth(folder);
    if (depth === 0) return "";
    return "　".repeat(depth - 1) + "└ ";
  }

  // 폴더를 계층 구조대로 정렬하는 함수
  function getSortedFolders(): any[] {
    const result: any[] = [];

    // 재귀적으로 폴더를 추가하는 함수
    function addFolderAndChildren(parentId: string | null) {
      const children = folders.filter(
        (f) => (f.parent_id || null) === parentId,
      );
      children.sort((a, b) => (a.name || "").localeCompare(b.name || ""));
      for (const child of children) {
        result.push(child);
        addFolderAndChildren(child.id);
      }
    }

    // 루트 폴더부터 시작
    addFolderAndChildren(null);
    return result;
  }

  // 정렬된 폴더 목록
  let sortedFolders = $derived(getSortedFolders());

  onMount(async () => {
    if (!$authState.isAuthenticated) {
      await goto("/");
      return;
    }

    try {
      // 볼트 초기화 (데이터베이스 생성)
      await invoke("initialize_vault");

      await AuthService.resizeWindowForFileManager();
      enableToasts();
      await Promise.all([loadFiles(), loadFolders()]);

      timeInterval = window.setInterval(() => {
        currentTime = new Date();
        if (sessionTime > 0) sessionTime--;
      }, 1000);

      isInitializing = false;
    } catch (error) {
      console.error("초기화 실패:", error);
      isInitializing = false;
    }
  });

  onDestroy(() => {
    if (timeInterval) clearInterval(timeInterval);
  });

  function handleKeyDown(event: KeyboardEvent) {
    if (
      showFileViewer ||
      showCreateFolderDialog ||
      showNewFileDialog ||
      showDeleteDialog ||
      showExportDialog ||
      showRenameDialog
    )
      return;

    if (event.key === "Delete" && selectedCount > 0) handleDelete();
    if (event.key === "F2" && selectedCount === 1) handleRename();
    if (event.key === "F5") {
      event.preventDefault();
      handleRefresh();
    }
    if ((event.ctrlKey || event.metaKey) && event.key === "a") {
      event.preventDefault();
      selectedFiles = new Set(filteredFiles.map((f) => f.id));
    }
    if (event.key === "Escape") {
      selectedFiles = new Set();
      selectedFolders = new Set();
      hideContextMenu();
    }
    if (event.key === "Enter" && selectedFiles.size === 1) {
      const fileId = Array.from(selectedFiles)[0];
      const file = files.find((f) => f.id === fileId);
      if (file) openFile(file);
    }
  }
</script>

<svelte:head>
  <title>파일 매니저 - SecureVault</title>
</svelte:head>

<svelte:window onclick={hideContextMenu} />

{#if isInitializing}
  <div class="loading-screen">
    <div class="loading-spinner"></div>
    <p>파일 매니저 준비 중...</p>
  </div>
{:else}
  <div
    class="file-manager"
    onkeydown={handleKeyDown}
    tabindex="0"
    role="application"
  >
    <!-- 헤더 -->
    <header class="header">
      <div class="header-left">
        <span class="logo">🔒</span>
        <div class="header-text">
          <h1>SecureVault</h1>
          <span class="subtitle">포일 매니저</span>
        </div>
      </div>
      <div class="window-controls">
        <button class="win-btn minimize">─</button>
        <button class="win-btn maximize">□</button>
        <button class="win-btn close">×</button>
      </div>
    </header>

    <!-- 툴바 -->
    <div class="toolbar">
      <div class="toolbar-buttons">
        <button class="tool-btn primary" onclick={handleAddFile}>
          <span class="icon">📁</span>
          <span class="label">파일 추가</span>
        </button>
        <button class="tool-btn primary" onclick={handleAddFolder}>
          <span class="icon">📂</span>
          <span class="label">폴더 추가</span>
        </button>
        <div class="separator"></div>
        <button class="tool-btn" onclick={handleCreateFile}>
          <span class="icon">📄</span>
          <span class="label">파일 생성</span>
        </button>
        <button class="tool-btn" onclick={handleCreateFolder}>
          <span class="icon">📁</span>
          <span class="label">폴더 생성</span>
        </button>
        <div class="separator"></div>
        <button
          class="tool-btn"
          onclick={handleRename}
          disabled={selectedCount !== 1}
        >
          <span class="icon">✏️</span>
          <span class="label">이름 변경</span>
        </button>
        <button
          class="tool-btn danger"
          onclick={handleDelete}
          disabled={selectedCount === 0}
        >
          <span class="icon">🗑️</span>
          <span class="label">삭제</span>
        </button>
        <button
          class="tool-btn"
          onclick={handleExport}
          disabled={selectedFiles.size === 0}
        >
          <span class="icon">📤</span>
          <span class="label">내보내기</span>
        </button>
        <div class="separator"></div>
        <button class="tool-btn" onclick={handleRefresh}>
          <span class="icon">🔄</span>
          <span class="label">새로고침</span>
        </button>
      </div>

      <div class="toolbar-right">
        <div class="search-box">
          <input
            type="text"
            placeholder="파일 검색..."
            bind:value={searchQuery}
            onkeydown={(e) => e.key === "Enter" && handleSearch()}
          />
          <button class="search-btn" onclick={handleSearch}>🔍</button>
        </div>

        <div class="sort-box">
          <span>정렬:</span>
          <select bind:value={sortBy}>
            <option value="name">이름</option>
            <option value="date">날짜</option>
            <option value="size">크기</option>
            <option value="type">유형</option>
          </select>
          <button
            class="sort-order-btn"
            onclick={toggleSortOrder}
            title={sortOrder === "asc" ? "오름차순" : "내림차순"}
          >
            {sortOrder === "asc" ? "↑" : "↓"}
          </button>
        </div>

        <div class="view-modes">
          <button
            class="view-btn"
            class:active={viewMode === "list"}
            onclick={() => (viewMode = "list")}
            title="목록 보기">≡</button
          >
          <button
            class="view-btn"
            class:active={viewMode === "grid"}
            onclick={() => (viewMode = "grid")}
            title="그리드 보기">⊞</button
          >
          <button
            class="view-btn"
            class:active={viewMode === "detail"}
            onclick={() => (viewMode = "detail")}
            title="상세 보기">☰</button
          >
        </div>
      </div>
    </div>

    <!-- 메인 컨텐츠 -->
    <div class="main-content">
      <!-- 사이드바 -->
      <aside class="sidebar">
        <div class="sidebar-header">폴더</div>
        <div class="folder-tree">
          <div
            class="folder-item"
            class:active={currentFolder === null}
            onclick={() => selectFolder(null)}
            oncontextmenu={(e) => showContextMenuHandler(e, null, "empty")}
            role="button"
            tabindex="0"
          >
            <span class="folder-icon">🏠</span>
            <span>루트 폴더</span>
          </div>

          {#each sortedFolders as folder}
            <div
              class="folder-item"
              class:active={currentFolder?.id === folder.id}
              class:selected={selectedFolders.has(folder.id)}
              onclick={(e) => {
                e.stopPropagation();
                selectFolder(folder);
                // 폴더 트리에서도 삭제 가능하도록 선택 상태 추가
                selectedFolders.add(folder.id);
                selectedFolders = new Set(selectedFolders);
              }}
              oncontextmenu={(e) => showContextMenuHandler(e, folder, "folder")}
              role="button"
              tabindex="0"
            >
              <span class="folder-prefix">{getFolderTreePrefix(folder)}</span>
              <span class="folder-icon">📁</span>
              <span>{folder.name}</span>
            </div>
          {/each}

          {#if folders.length === 0}
            <div class="folder-empty">폴더가 없습니다</div>
          {/if}
        </div>
      </aside>

      <!-- 파일 영역 -->
      <main
        class="file-area"
        oncontextmenu={(e) => {
          if (
            e.target === e.currentTarget ||
            (e.target as HTMLElement).closest(".file-content")
          )
            showContextMenuHandler(e, null, "empty");
        }}
      >
        <div class="breadcrumb">
          <span class="breadcrumb-icon">📁</span>
          <span>현재 위치: {currentFolder ? currentFolder.name : "/"}</span>
          {#if searchQuery}
            <span class="search-indicator">🔍 "{searchQuery}" 검색 중</span>
          {/if}
        </div>

        <div
          class="file-content"
          class:grid-view={viewMode === "grid"}
          class:detail-view={viewMode === "detail"}
        >
          {#if filteredFiles.length === 0 && filteredFolders.length === 0}
            <div class="empty-state">
              <div class="empty-icon">📂</div>
              <h2>
                {searchQuery ? "검색 결과가 없습니다" : "폴더가 비어있습니다"}
              </h2>
              <p>
                {searchQuery
                  ? "다른 검색어를 시도해보세요."
                  : "파일을 추가하거나 새 폴더를 생성해보세요."}
              </p>
            </div>
          {:else}
            <div
              class="file-list"
              class:grid={viewMode === "grid"}
              class:detail={viewMode === "detail"}
            >
              {#each filteredFolders as folder}
                <div
                  class="file-item folder-item"
                  class:selected={selectedFolders.has(folder.id)}
                  onclick={(e) => toggleFolderSelection(folder.id, e)}
                  ondblclick={() => selectFolder(folder)}
                  oncontextmenu={(e) =>
                    showContextMenuHandler(e, folder, "folder")}
                  role="button"
                  tabindex="0"
                >
                  <span class="file-icon">📂</span>
                  <div class="file-info">
                    <span class="file-name" title={folder.name}
                      >{folder.name}</span
                    >
                    <span class="file-meta">폴더</span>
                  </div>
                </div>
              {/each}
              {#each filteredFiles as file}
                <div
                  class="file-item"
                  class:selected={selectedFiles.has(file.id)}
                  onclick={(e) => toggleFileSelection(file.id, e)}
                  ondblclick={() => openFile(file)}
                  oncontextmenu={(e) => showContextMenuHandler(e, file, "file")}
                  role="button"
                  tabindex="0"
                >
                  <span class="file-icon">{getFileIcon(file)}</span>
                  <div class="file-info">
                    <span class="file-name" title={file.file_name}
                      >{file.file_name}</span
                    >
                    {#if viewMode === "detail"}
                      <span class="file-ext"
                        >{file.file_extension?.toUpperCase() || "-"}</span
                      >
                    {/if}
                    <span class="file-meta">
                      {formatFileSize(file.file_size || 0)}
                      {#if viewMode !== "grid"}
                        · {new Date(file.modified_date).toLocaleDateString(
                          "ko-KR",
                        )}
                      {/if}
                    </span>
                  </div>
                </div>
              {/each}
            </div>
          {/if}
        </div>
      </main>
    </div>

    <!-- 상태바 -->
    <footer class="status-bar">
      <div class="status-left">
        <span class="status-item">
          <span
            class="dot"
            class:active={filteredFiles.length + filteredFolders.length > 0}
          ></span>
          {filteredFiles.length + filteredFolders.length > 0
            ? `${filteredFolders.length}개 폴더, ${filteredFiles.length}개 파일`
            : "항목 없음"}
          {#if selectedCount > 0}
            <span class="selected-count">({selectedCount}개 선택)</span>
          {/if}
        </span>
      </div>
      <div class="status-right">
        <span class="status-item"
          >⏱️ 세션: {formatSessionTime(sessionTime)}</span
        >
        <span class="status-item"
          >🕐 {currentTime.toLocaleTimeString("ko-KR", { hour12: true })}</span
        >
        <span class="status-item secure">🔒 암호화 활성</span>
      </div>
    </footer>
  </div>

  <!-- 컨텍스트 메뉴 -->
  {#if showContextMenu}
    <div
      class="context-menu"
      style="left: {contextMenuX}px; top: {contextMenuY}px;"
    >
      {#if contextMenuType === "file"}
        <button
          class="context-item"
          onclick={() => handleContextMenuAction("open")}>📂 열기</button
        >
        <button
          class="context-item"
          onclick={() => handleContextMenuAction("export")}>📤 내보내기</button
        >
        <div class="context-separator"></div>
        <button
          class="context-item"
          onclick={() => handleContextMenuAction("rename")}>✏️ 이름 변경</button
        >
        <button
          class="context-item danger"
          onclick={() => handleContextMenuAction("delete")}>🗑️ 삭제</button
        >
      {:else if contextMenuType === "folder"}
        <button
          class="context-item"
          onclick={() => selectFolder(contextMenuTarget)}>📂 폴더 열기</button
        >
        <div class="context-separator"></div>
        <button
          class="context-item"
          onclick={() => handleContextMenuAction("rename")}>✏️ 이름 변경</button
        >
        <button
          class="context-item danger"
          onclick={() => handleContextMenuAction("delete")}>🗑️ 삭제</button
        >
      {:else}
        <button
          class="context-item"
          onclick={() => handleContextMenuAction("newFile")}>📄 새 파일</button
        >
        <button
          class="context-item"
          onclick={() => handleContextMenuAction("newFolder")}
          >📁 새 폴더</button
        >
        <div class="context-separator"></div>
        <button
          class="context-item"
          onclick={() => handleContextMenuAction("refresh")}>🔄 새로고침</button
        >
      {/if}
    </div>
  {/if}

  <!-- 파일 뷰어 -->
  {#if showFileViewer && viewerFile}
    <div class="viewer-overlay" onclick={closeViewer}>
      <div class="viewer-container" onclick={(e) => e.stopPropagation()}>
        <div class="viewer-header">
          <h2>{viewerFile.file_name}</h2>
          <button class="viewer-close" onclick={closeViewer}>✕</button>
        </div>
        <div class="viewer-content">
          {#if isViewerLoading}
            <div class="viewer-loading">
              <div class="loading-spinner"></div>
              <p>파일 로드 중...</p>
            </div>
          {:else if viewerFile.mime_type?.startsWith("image/")}
            <img
              src={viewerContent}
              alt={viewerFile.file_name}
              class="viewer-image"
            />
          {:else if viewerContent}
            <pre class="viewer-text">{viewerContent}</pre>
          {:else}
            <div class="viewer-unsupported">
              <span class="icon">{getFileIcon(viewerFile)}</span>
              <p>이 파일 형식은 미리보기가 지원되지 않습니다.</p>
              <button
                class="btn-export"
                onclick={() => {
                  closeViewer();
                  handleExport();
                }}>📤 내보내기</button
              >
            </div>
          {/if}
        </div>
      </div>
    </div>
  {/if}

  <!-- 다이얼로그들 -->
  {#if showCreateFolderDialog}
    <SimpleCreateFolderDialog
      show={showCreateFolderDialog}
      on:folderCreated={onFolderCreated}
      on:close={() => (showCreateFolderDialog = false)}
    />
  {/if}
  {#if showNewFileDialog}
    <SimpleNewFileDialog
      show={showNewFileDialog}
      on:fileCreated={onFileCreated}
      on:close={() => (showNewFileDialog = false)}
    />
  {/if}
  {#if showDeleteDialog}
    <DeleteConfirmDialog
      show={showDeleteDialog}
      items={deleteItems}
      on:confirmed={onDeleteConfirmed}
      on:close={() => (showDeleteDialog = false)}
    />
  {/if}
  {#if showExportDialog}
    <ExportDialog
      show={showExportDialog}
      files={exportFiles}
      on:exported={onExported}
      on:close={() => (showExportDialog = false)}
    />
  {/if}
  {#if showRenameDialog}
    <SimpleRenameDialog
      show={showRenameDialog}
      itemType={renameItemType}
      currentName={renameItemType === "file"
        ? renameItem?.file_name
        : renameItem?.name}
      on:renamed={onRenamed}
      on:close={() => (showRenameDialog = false)}
    />
  {/if}
{/if}

<style>
  * {
    box-sizing: border-box;
    margin: 0;
    padding: 0;
  }

  .loading-screen {
    height: 100vh;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    background: #f5f5f5;
    gap: 16px;
  }

  .loading-spinner {
    width: 40px;
    height: 40px;
    border: 3px solid #e0e0e0;
    border-top-color: #4a90d9;
    border-radius: 50%;
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  .file-manager {
    height: 100vh;
    display: flex;
    flex-direction: column;
    background: #f5f5f5;
    font-family:
      "Segoe UI",
      -apple-system,
      sans-serif;
    outline: none;
  }

  .header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 8px 16px;
    background: linear-gradient(135deg, #4a7fb5 0%, #5a8fc5 100%);
    color: white;
    -webkit-app-region: drag;
  }

  .header-left {
    display: flex;
    align-items: center;
    gap: 12px;
  }
  .logo {
    font-size: 24px;
  }
  .header-text h1 {
    font-size: 16px;
    font-weight: 600;
    margin: 0;
  }
  .header-text .subtitle {
    font-size: 11px;
    opacity: 0.9;
  }
  .window-controls {
    display: flex;
    gap: 8px;
    -webkit-app-region: no-drag;
  }
  .win-btn {
    width: 32px;
    height: 32px;
    border: none;
    background: rgba(255, 255, 255, 0.1);
    color: white;
    border-radius: 4px;
    cursor: pointer;
    font-size: 14px;
  }
  .win-btn:hover {
    background: rgba(255, 255, 255, 0.2);
  }
  .win-btn.close:hover {
    background: #e81123;
  }

  .toolbar {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 8px 12px;
    background: white;
    border-bottom: 1px solid #e0e0e0;
    gap: 16px;
    flex-wrap: wrap;
  }
  .toolbar-buttons {
    display: flex;
    align-items: center;
    gap: 4px;
  }
  .tool-btn {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 2px;
    padding: 6px 10px;
    border: 1px solid #d0d0d0;
    background: #f8f8f8;
    border-radius: 4px;
    cursor: pointer;
    font-size: 11px;
    color: #333;
    min-width: 56px;
    transition: all 0.15s;
  }
  .tool-btn:hover:not(:disabled) {
    background: #e8e8e8;
    border-color: #b0b0b0;
  }
  .tool-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
  .tool-btn.primary {
    background: #4a90d9;
    border-color: #3a80c9;
    color: white;
  }
  .tool-btn.primary:hover {
    background: #3a80c9;
  }
  .tool-btn.danger {
    background: #e57373;
    border-color: #d56363;
    color: white;
  }
  .tool-btn.danger:hover:not(:disabled) {
    background: #d56363;
  }
  .tool-btn .icon {
    font-size: 16px;
  }
  .tool-btn .label {
    font-size: 10px;
  }
  .separator {
    width: 1px;
    height: 32px;
    background: #d0d0d0;
    margin: 0 4px;
  }

  .toolbar-right {
    display: flex;
    align-items: center;
    gap: 12px;
  }
  .search-box {
    display: flex;
    align-items: center;
    border: 1px solid #d0d0d0;
    border-radius: 4px;
    overflow: hidden;
  }
  .search-box input {
    padding: 6px 10px;
    border: none;
    outline: none;
    width: 160px;
    font-size: 12px;
  }
  .search-btn {
    padding: 6px 10px;
    border: none;
    background: #f0f0f0;
    cursor: pointer;
  }
  .sort-box {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 12px;
  }
  .sort-box select {
    padding: 4px 8px;
    border: 1px solid #d0d0d0;
    border-radius: 4px;
    font-size: 12px;
  }
  .sort-order-btn {
    padding: 4px 8px;
    border: 1px solid #d0d0d0;
    background: white;
    border-radius: 4px;
    cursor: pointer;
    font-size: 12px;
  }
  .view-modes {
    display: flex;
    border: 1px solid #d0d0d0;
    border-radius: 4px;
    overflow: hidden;
  }
  .view-btn {
    padding: 6px 10px;
    border: none;
    background: white;
    cursor: pointer;
    font-size: 14px;
  }
  .view-btn.active {
    background: #4a90d9;
    color: white;
  }

  .main-content {
    flex: 1;
    display: flex;
    overflow: hidden;
  }
  .sidebar {
    width: 200px;
    background: white;
    border-right: 1px solid #e0e0e0;
    display: flex;
    flex-direction: column;
  }
  .sidebar-header {
    padding: 12px 16px;
    font-size: 12px;
    font-weight: 600;
    color: #666;
    border-bottom: 1px solid #e0e0e0;
  }
  .folder-tree {
    flex: 1;
    overflow-y: auto;
    padding: 8px 0;
  }
  .folder-item {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 16px;
    cursor: pointer;
    font-size: 13px;
    color: #333;
    transition: background 0.15s;
  }
  .folder-item:hover {
    background: #f0f0f0;
  }
  .folder-item.active {
    background: #e3f2fd;
    color: #1976d2;
    border-left: 3px solid #1976d2;
    padding-left: 13px;
  }
  .folder-item.selected {
    background: #fff3cd;
  }
  .folder-icon {
    font-size: 14px;
  }
  .folder-empty {
    padding: 16px;
    text-align: center;
    font-size: 12px;
    color: #999;
  }

  .file-area {
    flex: 1;
    display: flex;
    flex-direction: column;
    background: white;
  }
  .breadcrumb {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 12px 16px;
    font-size: 13px;
    color: #666;
    border-bottom: 1px solid #f0f0f0;
  }
  .breadcrumb-icon {
    font-size: 14px;
  }
  .search-indicator {
    margin-left: auto;
    color: #4a90d9;
    font-size: 12px;
  }
  .file-content {
    flex: 1;
    overflow-y: auto;
    padding: 16px;
  }

  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    text-align: center;
    color: #666;
  }
  .empty-icon {
    font-size: 64px;
    opacity: 0.5;
    margin-bottom: 16px;
  }
  .empty-state h2 {
    font-size: 18px;
    font-weight: 500;
    margin-bottom: 8px;
    color: #333;
  }
  .empty-state p {
    font-size: 13px;
    color: #888;
  }

  .file-list {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .file-list.grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(120px, 1fr));
    gap: 12px;
  }
  .file-list.detail {
    gap: 2px;
  }

  .file-item {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 10px 12px;
    border-radius: 4px;
    cursor: pointer;
    transition: background 0.15s;
  }
  .file-item:hover {
    background: #f5f5f5;
  }
  .file-item.selected {
    background: #e3f2fd;
    outline: 1px solid #90caf9;
  }
  .file-icon {
    font-size: 24px;
  }
  .file-info {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
    flex: 1;
  }
  .file-name {
    font-size: 13px;
    font-weight: 500;
    color: #333;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .file-meta {
    font-size: 11px;
    color: #888;
  }
  .file-ext {
    font-size: 10px;
    color: #666;
    background: #f0f0f0;
    padding: 2px 6px;
    border-radius: 3px;
    align-self: flex-start;
  }

  .file-list.grid .file-item {
    flex-direction: column;
    text-align: center;
    padding: 16px 8px;
  }
  .file-list.grid .file-icon {
    font-size: 36px;
  }
  .file-list.grid .file-info {
    align-items: center;
  }
  .file-list.grid .file-name {
    text-align: center;
    max-width: 100%;
  }

  .status-bar {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 6px 16px;
    background: #f8f8f8;
    border-top: 1px solid #e0e0e0;
    font-size: 11px;
    color: #666;
  }
  .status-left,
  .status-right {
    display: flex;
    align-items: center;
    gap: 16px;
  }
  .status-item {
    display: flex;
    align-items: center;
    gap: 4px;
  }
  .dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: #ccc;
  }
  .dot.active {
    background: #4caf50;
  }
  .status-item.secure {
    color: #4caf50;
  }
  .selected-count {
    color: #4a90d9;
    margin-left: 4px;
  }

  /* 컨텍스트 메뉴 */
  .context-menu {
    position: fixed;
    background: white;
    border: 1px solid #d0d0d0;
    border-radius: 4px;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
    min-width: 160px;
    z-index: 1000;
    padding: 4px 0;
  }
  .context-item {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    padding: 8px 16px;
    border: none;
    background: none;
    cursor: pointer;
    font-size: 13px;
    text-align: left;
  }
  .context-item:hover {
    background: #f0f0f0;
  }
  .context-item.danger {
    color: #dc3545;
  }
  .context-separator {
    height: 1px;
    background: #e0e0e0;
    margin: 4px 0;
  }

  /* 파일 뷰어 */
  .viewer-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.7);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 2000;
  }
  .viewer-container {
    background: white;
    border-radius: 8px;
    width: 90%;
    max-width: 1000px;
    max-height: 90vh;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }
  .viewer-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 16px 20px;
    border-bottom: 1px solid #e0e0e0;
  }
  .viewer-header h2 {
    font-size: 16px;
    font-weight: 600;
    margin: 0;
  }
  .viewer-close {
    width: 32px;
    height: 32px;
    border: none;
    background: #f0f0f0;
    border-radius: 4px;
    cursor: pointer;
    font-size: 18px;
  }
  .viewer-close:hover {
    background: #e0e0e0;
  }
  .viewer-content {
    flex: 1;
    overflow: auto;
    padding: 20px;
    min-height: 300px;
  }
  .viewer-loading {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 200px;
    gap: 16px;
  }
  .viewer-text {
    white-space: pre-wrap;
    word-break: break-word;
    font-family: "Consolas", monospace;
    font-size: 13px;
    line-height: 1.5;
    background: #f8f8f8;
    padding: 16px;
    border-radius: 4px;
    max-height: 60vh;
    overflow: auto;
  }
  .viewer-image {
    max-width: 100%;
    max-height: 60vh;
    object-fit: contain;
    margin: 0 auto;
    display: block;
  }
  .viewer-unsupported {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 200px;
    gap: 16px;
    color: #666;
  }
  .viewer-unsupported .icon {
    font-size: 48px;
  }
  .btn-export {
    padding: 8px 16px;
    background: #4a90d9;
    color: white;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    font-size: 13px;
  }
  .btn-export:hover {
    background: #3a80c9;
  }
</style>
