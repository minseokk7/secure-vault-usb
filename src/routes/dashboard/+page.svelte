<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { goto } from "$app/navigation";
  import { authState, AuthService } from "$lib/stores/auth";
  import { enableToasts, addToast } from "$lib/stores/toast";
  import FileViewer from "$lib/components/viewers/FileViewer.svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import { open, save } from "@tauri-apps/plugin-dialog";
  import { FileManagerService } from "$lib/stores/file-manager";

  // 다이얼로그 컴포넌트들
  import SimpleCreateFolderDialog from "$lib/components/file-manager/SimpleCreateFolderDialog.svelte";
  import SimpleNewFileDialog from "$lib/components/file-manager/SimpleNewFileDialog.svelte";
  import DeleteConfirmDialog from "$lib/components/file-manager/DeleteConfirmDialog.svelte";
  import ExportDialog from "$lib/components/file-manager/ExportDialog.svelte";
  import SimpleRenameDialog from "$lib/components/file-manager/SimpleRenameDialog.svelte";
  import UploadProgressDialog from "$lib/components/file-manager/UploadProgressDialog.svelte";
  import SettingsModal from "$lib/components/settings/SettingsModal.svelte";

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
  let showUploadProgress = $state(false);
  let showSettingsModal = $state(false);

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

  // 업로드 진행률 상태
  let uploadCurrentFile = $state("");
  let uploadCurrentIndex = $state(0);
  let uploadTotalFiles = $state(0);
  let uploadProgress = $state(0);
  let uploadIsFolder = $state(false);

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

  // 필터링된 폴더 목록
  let filteredFolders = $derived.by(() => {
    let result = [];

    // 검색어가 있으면 모든 폴더 대상, 없으면 현재 폴더의 하위 폴더만
    if (searchQuery.trim()) {
      result = [...folders];
    } else {
      result = folders.filter((f) => {
        const parentId = f.parent_id || null;
        const currentId = currentFolder?.id || null;
        return parentId === currentId;
      });
    }

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
      uploadIsFolder = false;
      console.log("파일 추가 시작...");
      const selected = await open({
        multiple: true,
        filters: [{ name: "모든 파일", extensions: ["*"] }],
      });

      console.log("선택된 파일:", selected);

      if (selected) {
        const paths = Array.isArray(selected) ? selected : [selected];

        // 진행률 다이얼로그 표시
        uploadTotalFiles = paths.length;
        uploadCurrentIndex = 0;
        uploadProgress = 0;
        showUploadProgress = true;

        let successCount = 0;
        let lastError = null;
        let currentJobId: string | null = null;
        let unlistenProgress: UnlistenFn | null = null;
        let unlistenComplete: UnlistenFn | null = null;
        let unlistenError: UnlistenFn | null = null;

        for (let i = 0; i < paths.length; i++) {
          const path = paths[i];
          try {
            // 파일명 추출
            const fileName = path.split(/[\\/]/).pop() || "파일";
            uploadCurrentFile = fileName;
            uploadCurrentIndex = i + 1;
            uploadProgress = 0;

            console.log("스트리밍 업로드 시작:", path);

            // 업로드 시작 (먼저 Job ID 획득)
            currentJobId = await invoke<string>("start_file_upload", {
              filePath: path,
              fileName: null,
              folderId: currentFolder?.id || null,
            });
            console.log("업로드 Job ID:", currentJobId);

            // 완료/에러 Promise 설정
            let uploadResolve!: (value: boolean) => void;
            let uploadReject!: (reason: any) => void;
            const uploadPromise = new Promise<boolean>((resolve, reject) => {
              uploadResolve = resolve;
              uploadReject = reject;
            });

            const jobIdToMatch = currentJobId;

            // 진행률 이벤트 리스너
            unlistenProgress = await listen<{
              job_id: string;
              progress: number;
              bytes_processed: number;
              total_bytes: number;
            }>("upload://progress", (event) => {
              if (event.payload.job_id === jobIdToMatch) {
                uploadProgress = event.payload.progress * 100;
                console.log(
                  `업로드 진행률: ${uploadProgress.toFixed(1)}% (${event.payload.bytes_processed}/${event.payload.total_bytes})`,
                );
              }
            });

            // 완료 이벤트 리스너
            unlistenComplete = await listen<{
              job_id: string;
              file_id: string;
            }>("upload://complete", (event) => {
              if (event.payload.job_id === jobIdToMatch) {
                console.log("업로드 완료:", event.payload.file_id);
                uploadResolve(true);
              }
            });

            // 에러 이벤트 리스너
            unlistenError = await listen<{ job_id: string; error: string }>(
              "upload://error",
              (event) => {
                if (event.payload.job_id === jobIdToMatch) {
                  console.error("업로드 오류:", event.payload.error);
                  uploadReject(new Error(event.payload.error));
                }
              },
            );

            // 완료 대기
            await uploadPromise;

            // 완료 진행률
            uploadProgress = 100;
            console.log("파일 추가 성공:", path);
            successCount++;

            // 잠시 대기 (사용자가 진행률을 볼 수 있도록)
            await new Promise((resolve) => setTimeout(resolve, 200));
          } catch (e) {
            console.error("파일 추가 실패:", path, e);
            lastError = e;
          } finally {
            // 이벤트 리스너 정리
            if (unlistenProgress) {
              unlistenProgress();
              unlistenProgress = null;
            }
            if (unlistenComplete) {
              unlistenComplete();
              unlistenComplete = null;
            }
            if (unlistenError) {
              unlistenError();
              unlistenError = null;
            }
            currentJobId = null;
          }
        }

        // 진행률 다이얼로그 닫기
        showUploadProgress = false;

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
      showUploadProgress = false;
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
      uploadIsFolder = true;
      console.log("폴더 추가 시작...");
      const selected = await open({ directory: true, multiple: false });
      console.log("선택된 폴더:", selected);

      if (selected && typeof selected === "string") {
        // 폴더명 추출
        const folderName = selected.split(/[\\/]/).pop() || "폴더";

        // 진행률 다이얼로그 표시
        uploadCurrentFile = folderName;
        uploadCurrentIndex = 1;
        uploadTotalFiles = 1;
        uploadProgress = 0;
        showUploadProgress = true;

        console.log("폴더 추가 invoke:", selected);
        const result = await invoke<{
          folder_count: number;
          file_count: number;
        }>("add_folder_to_vault", {
          folderPath: selected,
          targetFolderId: currentFolder?.id || null,
        });

        uploadProgress = 100;
        console.log("폴더 추가 결과:", result);

        // 잠시 대기
        await new Promise((resolve) => setTimeout(resolve, 300));
        showUploadProgress = false;

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
      showUploadProgress = false;
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
  async function handleSearch() {
    if (!searchQuery.trim()) {
      // 검색어가 비어있으면 현재 폴더로 리셋
      if (currentFolder) {
        await loadFiles();
      } else {
        await Promise.all([loadFiles(), loadFolders()]);
      }
      return;
    }

    try {
      console.log("검색 시작:", searchQuery);
      const result = await invoke<{ files: any[]; folders: any[] }>(
        "search_files",
        {
          query: searchQuery,
          folderId: null, // 전체 검색
        },
      );

      console.log("검색 결과:", result);
      files = result.files || [];
      folders = result.folders || [];

      // 검색 모드 알림
      addToast({
        type: "info",
        title: "검색 완료",
        message: `${files.length}개 파일, ${folders.length}개 폴더를 찾았습니다.`,
        duration: 2000,
      });
    } catch (error) {
      console.error("검색 실패:", error);
      addToast({
        type: "error",
        title: "검색 실패",
        message:
          typeof error === "string" ? error : "검색 중 오류가 발생했습니다.",
        duration: 3000,
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

  // 세션 시간 로드
  async function loadSessionTime() {
    try {
      const time = await invoke<number>("get_session_remaining_time");
      sessionTime = time;

      if (time <= 0) {
        console.log("세션 만료. 로그아웃 처리...");
        await handleLogout();
      }
    } catch (error) {
      console.error("세션 시간 로드 실패:", error);
    }
  }

  // 로그아웃 처리
  async function handleLogout() {
    try {
      console.log("로그아웃 실행...");
      await AuthService.logout();
      await AuthService.resizeWindowForLogin();
      await goto("/");
    } catch (error) {
      console.error("로그아웃 실패:", error);
      // 실패하더라도 강제 이동 시도
      await goto("/");
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
    try {
      isViewerLoading = true;
      viewerFile = file;
      showFileViewer = true;
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

  async function handleFileSaved(event: CustomEvent) {
    console.log("파일 저장됨:", event.detail);
    await loadFiles();
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
        handleFileExportAction();
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

  // 삭제 확인
  async function onDeleteConfirmed() {
    showDeleteDialog = false;
    try {
      let currentDeleted = false;
      const targetParentId = currentFolder?.parent_id;

      for (const item of deleteItems) {
        if (item.file_name) {
          // 파일 삭제
          await invoke("delete_file_from_vault", { fileId: item.id });
        } else {
          // 폴더 삭제
          await invoke("delete_folder", {
            folderId: item.id,
            recursive: true,
          });
          if (currentFolder?.id === item.id) {
            currentDeleted = true;
          }
        }
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

  function handleFileExportAction() {
    console.log("export action triggered", {
      selectedFiles: selectedFiles.size,
      selectedFolders: selectedFolders.size,
    });

    if (selectedFiles.size === 0 && selectedFolders.size === 0) {
      console.log("no items selected");
      addToast({
        type: "warning",
        title: "선택된 항목 없음",
        message: "내보낼 파일이나 폴더를 선택해주세요.",
        duration: 3000,
      });
      return;
    }

    const filesToExport = filteredFiles
      .filter((f) => selectedFiles.has(f.id))
      .map((f) => ({ ...f, type: "file" }));

    // 수정: filteredFolders 대신 전체 folders에서 검색하여 사이드바 등에서 선택된 폴더도 포함
    const foldersToExport = folders
      .filter((f) => selectedFolders.has(f.id))
      .map((f) => ({ ...f, type: "folder", file_name: f.name })); // 폴더는 name을 file_name으로 매핑

    const allItems = [...filesToExport, ...foldersToExport];
    console.log("items to export:", allItems);

    if (allItems.length === 0) {
      console.log("export list empty");
      return;
    }

    exportFiles = allItems as any[]; // 타입 호환성을 위해 any 캐스팅
    showExportDialog = true;
    console.log("showExportDialog set to true");
  }

  async function onExported(event: CustomEvent<{ exportPath: string }>) {
    console.log("onExported event received:", event.detail);
    showExportDialog = false;
    try {
      const targetPath = event.detail.exportPath;

      if (exportFiles.length === 1) {
        const item = exportFiles[0];
        console.log(`Exporting single item (${item.type}) to:`, targetPath);

        if (item.type === "folder") {
          await invoke("export_folder", {
            folderId: item.id,
            exportPath: targetPath,
          });
        } else {
          await invoke("export_file", {
            fileId: item.id,
            exportPath: targetPath,
          });
        }
      } else {
        // 다중 파일인 경우: targetPath는 디렉토리 경로임 (open 다이얼로그)
        console.log("Exporting multiple items to directory:", targetPath);
        const separator = targetPath.includes("\\") ? "\\" : "/";

        const items = exportFiles as any[];
        for (const item of items) {
          // item.file_name은 위에서 매핑함 (폴더인 경우 name)
          const name = item.file_name || item.name;
          const fullPath = `${targetPath}${targetPath.endsWith(separator) ? "" : separator}${name}`;
          console.log(`Exporting ${item.type} ${name} to:`, fullPath);

          if (item.type === "folder") {
            // 폴더 내보내기 (재귀적)
            await invoke("export_folder", {
              folderId: item.id,
              exportPath: fullPath, // 전체 경로 전달
            });
          } else {
            // 파일 내보내기
            await invoke("export_file", {
              fileId: item.id,
              exportPath: fullPath,
            });
          }
        }
      }

      addToast({
        type: "success",
        title: "내보내기 완료",
        message: `${exportFiles.length}개 항목이 내보내졌습니다.`,
        duration: 3000,
      });
      selectedFiles = new Set();
      selectedFolders = new Set();
    } catch (error) {
      console.error("Export failed:", error);
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
      // 데이터 갱신
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

  onMount(() => {
    let unlistenDrop: UnlistenFn | undefined;

    const init = async () => {
      if (!$authState.isAuthenticated) {
        await goto("/");
        return;
      }

      try {
        // 파일 드롭 리스너 등록
        unlistenDrop = await listen("tauri://drop", async (event: any) => {
          const paths = event.payload.paths as string[];
          if (paths && paths.length > 0) {
            console.log("파일 드롭 감지:", paths);
            const targetFolderId = currentFolder?.id || undefined;
            await FileManagerService.uploadFiles(paths, targetFolderId);
          }
        });

        // 볼트 초기화 (데이터베이스 생성)
        await invoke("initialize_vault");

        await AuthService.resizeWindowForFileManager();
        enableToasts();
        await Promise.all([loadFiles(), loadFolders(), loadSessionTime()]);

        timeInterval = window.setInterval(async () => {
          currentTime = new Date();
          // 백엔드와 세션 시간 동기화 (10초마다 또는 로컬 카운트가 0일 때)
          if (sessionTime % 10 === 0 || sessionTime <= 0) {
            await loadSessionTime();
          } else {
            if (sessionTime > 0) sessionTime--;
          }
        }, 1000);

        isInitializing = false;
      } catch (error) {
        console.error("초기화 실패:", error);
        // 오류 발생 시에도 기본 UI 로드 시도
        try {
          await Promise.all([loadFiles(), loadFolders()]);
        } catch (e) {
          console.error("기본 로드 실패:", e);
        }
        isInitializing = false;
      }
    };

    init();

    return () => {
      if (unlistenDrop) unlistenDrop();
      if (timeInterval) clearInterval(timeInterval);
    };
  });

  onDestroy(() => {
    // onDestroy cleanup is now handled in onMount return, but keeping existing strict cleanup if needed.
    // However, onMount return is the preferred way for component lifecycle cleanup in Svelte.
    // We can keep onDestroy for safety regarding timeInterval if it was defined outside.
    if (timeInterval) clearInterval(timeInterval);
  });

  // 윈도우 컨트롤 함수들
  async function minimizeWindow() {
    const { getCurrentWindow } = await import("@tauri-apps/api/window");
    await getCurrentWindow().minimize();
  }

  async function toggleMaximize() {
    const { getCurrentWindow } = await import("@tauri-apps/api/window");
    await getCurrentWindow().toggleMaximize();
  }

  async function closeWindow() {
    const { getCurrentWindow } = await import("@tauri-apps/api/window");
    await getCurrentWindow().close();
  }

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
        <span
          class="logo"
          onclick={handleLogout}
          onkeydown={(e) => e.key === "Enter" && handleLogout()}
          role="button"
          tabindex="0"
          title="로그아웃"
          style="cursor: pointer;">🔒</span
        >
        <div class="header-text">
          <h1>SecureVault</h1>
          <span class="subtitle">파일 매니저</span>
        </div>
      </div>
      <div class="window-controls">
        <button class="win-btn minimize" onclick={minimizeWindow} title="최소화"
          >─</button
        >
        <button class="win-btn maximize" onclick={toggleMaximize} title="최대화"
          >□</button
        >
        <button class="win-btn close" onclick={closeWindow} title="닫기"
          >×</button
        >
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
          onclick={handleFileExportAction}
          disabled={selectedCount === 0}
        >
          <span class="icon">📤</span>
          <span class="label">내보내기</span>
        </button>
        <div class="separator"></div>
        <button class="tool-btn" onclick={handleRefresh}>
          <span class="icon">🔄</span>
          <span class="label">새로고침</span>
        </button>
        <div class="separator"></div>
        <button class="tool-btn" onclick={() => (showSettingsModal = true)}>
          <span class="icon">⚙️</span>
          <span class="label">설정</span>
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
                      ·
                      <span class="mime-type"
                        >{file.mime_type || "Unknown"}</span
                      >
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
    <FileViewer
      file={viewerFile}
      isOpen={showFileViewer}
      on:close={closeViewer}
      on:save={handleFileSaved}
    />
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

  <!-- 업로드 진행률 다이얼로그 -->
  <UploadProgressDialog
    show={showUploadProgress}
    currentFile={uploadCurrentFile}
    currentIndex={uploadCurrentIndex}
    totalFiles={uploadTotalFiles}
    progress={uploadProgress}
    isFolder={uploadIsFolder}
  />
  {#if showSettingsModal}
    <SettingsModal
      show={showSettingsModal}
      on:close={() => (showSettingsModal = false)}
      on:sessionTimeUpdated={loadSessionTime}
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
    background: #f0f4f8;
    gap: 16px;
  }

  .loading-spinner {
    width: 40px;
    height: 40px;
    border: 3px solid #d0d8e0;
    border-top-color: #2563eb;
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
    background: #f0f4f8;
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
    background: linear-gradient(135deg, #1e40af 0%, #3b82f6 100%);
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
    background: #ffffff;
    border-bottom: 1px solid #d1d5db;
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
    border: 1px solid #cbd5e1;
    background: #f8fafc;
    border-radius: 4px;
    cursor: pointer;
    font-size: 11px;
    color: #334155;
    min-width: 56px;
    transition: all 0.15s;
  }
  .tool-btn:hover:not(:disabled) {
    background: #e2e8f0;
    border-color: #94a3b8;
  }
  .tool-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
  .tool-btn.primary {
    background: #2563eb;
    border-color: #1d4ed8;
    color: white;
  }
  .tool-btn.primary:hover {
    background: #1d4ed8;
  }
  .tool-btn.danger {
    background: #ef4444;
    border-color: #dc2626;
    color: white;
  }
  .tool-btn.danger:hover:not(:disabled) {
    background: #dc2626;
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
    background: #cbd5e1;
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
    border: 1px solid #cbd5e1;
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
    background: #f1f5f9;
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
    border: 1px solid #cbd5e1;
    border-radius: 4px;
    font-size: 12px;
  }
  .sort-order-btn {
    padding: 4px 8px;
    border: 1px solid #cbd5e1;
    background: white;
    border-radius: 4px;
    cursor: pointer;
    font-size: 12px;
  }
  .view-modes {
    display: flex;
    border: 1px solid #cbd5e1;
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
    background: #2563eb;
    color: white;
  }

  .main-content {
    flex: 1;
    display: flex;
    overflow: hidden;
  }
  .sidebar {
    width: 200px;
    background: #ffffff;
    border-right: 1px solid #d1d5db;
    display: flex;
    flex-direction: column;
  }
  .sidebar-header {
    padding: 12px 16px;
    font-size: 12px;
    font-weight: 600;
    color: #64748b;
    border-bottom: 1px solid #e2e8f0;
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
    color: #334155;
    transition: background 0.15s;
  }
  .folder-item:hover {
    background: #f1f5f9;
  }
  .folder-item.active {
    background: #dbeafe;
    color: #1e40af;
    border-left: 3px solid #2563eb;
    padding-left: 13px;
  }
  .folder-item.selected {
    background: #fef3c7;
  }
  .folder-icon {
    font-size: 14px;
  }
  .folder-empty {
    padding: 16px;
    text-align: center;
    font-size: 12px;
    color: #94a3b8;
  }

  .file-area {
    flex: 1;
    display: flex;
    flex-direction: column;
    background: #ffffff;
  }
  .breadcrumb {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 12px 16px;
    font-size: 13px;
    color: #64748b;
    border-bottom: 1px solid #f1f5f9;
  }
  .breadcrumb-icon {
    font-size: 14px;
  }
  .search-indicator {
    margin-left: auto;
    color: #2563eb;
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
    color: #64748b;
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
    color: #334155;
  }
  .empty-state p {
    font-size: 13px;
    color: #94a3b8;
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
    overflow: hidden;
    user-select: none;
    -webkit-user-select: none;
  }
  .file-item:hover {
    background: #f8fafc;
  }
  .file-item.selected {
    background: #dbeafe;
    outline: 1px solid #93c5fd;
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
    color: #334155;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .file-meta {
    font-size: 11px;
    color: #94a3b8;
    white-space: nowrap !important;
    overflow: hidden !important;
    text-overflow: ellipsis !important;
  }
  .file-ext {
    font-size: 10px;
    color: #64748b;
    background: #f1f5f9;
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
    width: 100%;
    overflow: hidden;
    position: relative;
  }
  .file-list.grid .file-name {
    text-align: center;
    width: 100%;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    display: block;
  }

  /* 그리드 뷰에서 hover 시 텍스트 말줄임표 유지 (스크롤 효과 제거) */
  .file-list.grid .file-item:hover .file-name {
    text-overflow: ellipsis;
    display: block;
    white-space: nowrap;
    overflow: hidden;
  }

  .status-bar {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 6px 16px;
    background: #f8fafc;
    border-top: 1px solid #e2e8f0;
    font-size: 11px;
    color: #64748b;
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
    background: #cbd5e1;
  }
  .dot.active {
    background: #10b981;
  }
  .status-item.secure {
    color: #10b981;
  }
  .selected-count {
    color: #2563eb;
    margin-left: 4px;
  }

  /* 컨텍스트 메뉴 */
  .context-menu {
    position: fixed;
    background: white;
    border: 1px solid #cbd5e1;
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
    background: #f1f5f9;
  }
  .context-item.danger {
    color: #ef4444;
  }
  .context-separator {
    height: 1px;
    background: #e2e8f0;
    margin: 4px 0;
  }

  /* 파일 뷰어 */
</style>
