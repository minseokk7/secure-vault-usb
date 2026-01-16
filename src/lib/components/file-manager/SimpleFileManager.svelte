<!--
SecureVault 파일 매니저 컴포넌트
실제 백엔드 API와 연동하여 파일 관리 기능을 제공합니다.
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { addToast } from '$lib/stores/toast';
  import { authState, AuthService } from '$lib/stores/auth';

  // 파일 및 폴더 타입 정의
  interface FileEntry {
    id: string;
    file_name: string;
    original_file_name: string;
    file_size: number;
    file_extension: string;
    mime_type: string;
    folder_id: string | null;
    created_date: string;
    modified_date: string;
  }

  interface FolderEntry {
    id: string;
    name: string;
    parent_id: string | null;
    path: string;
    created_at: string;
    modified_at: string;
  }

  // 상태 변수들
  let isLoading = $state(true);
  let files: FileEntry[] = $state([]);
  let folders: FolderEntry[] = $state([]);
  let currentFolderId: string | null = $state(null);
  let selectedFiles: FileEntry[] = $state([]);
  let error: string | null = $state(null);
  let isAddingFile = $state(false);

  // 파일 목록을 로드합니다
  async function loadFiles(folderId: string | null = null) {
    try {
      console.log('📁 파일 목록 로드 시작:', folderId || '루트');
      isLoading = true;
      error = null;
      
      // 파일 목록 로드
      const fileList = await invoke<FileEntry[]>('get_files_in_folder', { 
        folder_id: folderId 
      });
      files = fileList || [];
      
      console.log('✅ 파일 목록 로드 완료:', files.length, '개 파일');
      
    } catch (err) {
      console.error('❌ 파일 목록 로드 실패:', err);
      error = '파일 목록을 불러올 수 없습니다.';
      files = [];
      
      addToast({
        type: 'error',
        title: '파일 목록 로드 실패',
        message: typeof err === 'string' ? err : '파일 목록을 불러올 수 없습니다.',
        duration: 5000
      });
    } finally {
      isLoading = false;
    }
  }

  // 폴더 트리를 로드합니다
  async function loadFolders() {
    try {
      console.log('📂 폴더 트리 로드 시작');
      
      const folderTree = await invoke<FolderEntry[]>('get_folder_tree');
      folders = folderTree || [];
      
      console.log('✅ 폴더 트리 로드 완료:', folders.length, '개 폴더');
      
    } catch (err) {
      console.error('❌ 폴더 트리 로드 실패:', err);
      folders = [];
    }
  }

  // 파일을 추가합니다
  async function handleAddFile() {
    try {
      isAddingFile = true;
      
      // 파일 선택 다이얼로그
      const { open } = await import('@tauri-apps/plugin-dialog');
      const selected = await open({
        multiple: false,
        filters: [{
          name: '모든 파일',
          extensions: ['*']
        }]
      });
      
      if (selected && typeof selected === 'string') {
        console.log('📄 파일 추가 시작:', selected);
        
        const result = await invoke<FileEntry>('add_file_to_vault', {
          file_path: selected,
          file_name: null,
          folder_id: currentFolderId
        });
        
        console.log('✅ 파일 추가 완료:', result.file_name);
        
        // 파일 목록 새로고침
        await loadFiles(currentFolderId);
        
        addToast({
          type: 'success',
          title: '파일 추가 완료',
          message: `${result.file_name} 파일이 추가되었습니다.`,
          duration: 4000
        });
      }
    } catch (err) {
      console.error('❌ 파일 추가 실패:', err);
      addToast({
        type: 'error',
        title: '파일 추가 실패',
        message: typeof err === 'string' ? err : '파일을 추가할 수 없습니다.',
        duration: 5000
      });
    } finally {
      isAddingFile = false;
    }
  }

  // 파일을 삭제합니다
  async function handleDeleteFile(file: FileEntry) {
    try {
      const confirmed = confirm(`'${file.file_name}' 파일을 삭제하시겠습니까?`);
      if (!confirmed) return;
      
      console.log('🗑️ 파일 삭제 시작:', file.file_name);
      
      await invoke('delete_file_from_vault', { file_id: file.id });
      
      console.log('✅ 파일 삭제 완료:', file.file_name);
      
      // 파일 목록 새로고침
      await loadFiles(currentFolderId);
      
      addToast({
        type: 'success',
        title: '파일 삭제 완료',
        message: `${file.file_name} 파일이 삭제되었습니다.`,
        duration: 4000
      });
    } catch (err) {
      console.error('❌ 파일 삭제 실패:', err);
      addToast({
        type: 'error',
        title: '파일 삭제 실패',
        message: typeof err === 'string' ? err : '파일을 삭제할 수 없습니다.',
        duration: 5000
      });
    }
  }

  // 폴더를 선택합니다
  function selectFolder(folderId: string | null) {
    currentFolderId = folderId;
    selectedFiles = [];
    loadFiles(folderId);
  }

  // 파일 크기를 포맷합니다
  function formatFileSize(bytes: number): string {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
  }

  // 파일 타입 아이콘을 반환합니다
  function getFileIcon(mimeType: string): string {
    if (mimeType.startsWith('image/')) return '🖼️';
    if (mimeType.startsWith('video/')) return '🎥';
    if (mimeType.startsWith('audio/')) return '🎵';
    if (mimeType.includes('pdf')) return '📄';
    if (mimeType.includes('text/')) return '📝';
    if (mimeType.includes('zip') || mimeType.includes('rar')) return '📦';
    return '📄';
  }

  // 컴포넌트 마운트 시 초기화
  onMount(async () => {
    console.log('=== SimpleFileManager 마운트 시작 ===');
    
    try {
      // 폴더 트리와 파일 목록을 병렬로 로드
      await Promise.all([
        loadFolders(),
        loadFiles(null) // 루트 폴더부터 시작
      ]);
      
      console.log('✅ SimpleFileManager 초기화 완료');
      
      addToast({
        type: 'success',
        title: '파일 매니저 준비 완료',
        message: `${files.length}개 파일, ${folders.length}개 폴더를 불러왔습니다.`,
        duration: 3000
      });
      
    } catch (err) {
      console.error('❌ SimpleFileManager 초기화 실패:', err);
      error = '파일 매니저를 초기화할 수 없습니다.';
    }
  });

  // 로그아웃 처리
  async function handleLogout() {
    try {
      await AuthService.logout();
      window.location.href = '/';
    } catch (error) {
      console.error('로그아웃 실패:', error);
    }
  }

  // 새 폴더 생성
  async function handleCreateFolder() {
    try {
      const folderName = prompt('새 폴더 이름을 입력하세요:');
      if (!folderName || !folderName.trim()) return;
      
      console.log('📁 폴더 생성 시작:', folderName);
      
      await invoke('create_folder', {
        name: folderName.trim(),
        parent_id: currentFolderId
      });
      
      console.log('✅ 폴더 생성 완료:', folderName);
      
      // 폴더 트리 새로고침
      await loadFolders();
      
      addToast({
        type: 'success',
        title: '폴더 생성 완료',
        message: `${folderName} 폴더가 생성되었습니다.`,
        duration: 4000
      });
    } catch (err) {
      console.error('❌ 폴더 생성 실패:', err);
      addToast({
        type: 'error',
        title: '폴더 생성 실패',
        message: typeof err === 'string' ? err : '폴더를 생성할 수 없습니다.',
        duration: 5000
      });
    }
  }
</script>

<div class="simple-file-manager">
  <!-- 헤더 -->
  <div class="header">
    <div class="header-left">
      <h1>🔒 SecureVault</h1>
      <span class="subtitle">보안 파일 매니저</span>
    </div>
    <div class="header-right">
      <button onclick={handleLogout} class="logout-btn">
        🚪 로그아웃
      </button>
    </div>
  </div>

  <!-- 메인 콘텐츠 -->
  <div class="main-content">
    {#if isLoading}
      <div class="loading">
        <div class="loading-spinner"></div>
        <p>파일 매니저를 로딩 중입니다...</p>
      </div>
    {:else}
      <!-- 도구 모음 -->
      <div class="toolbar">
        <button 
          class="btn-primary" 
          onclick={handleAddFile}
          disabled={isAddingFile}
        >
          {isAddingFile ? '📁 추가 중...' : '📁 파일 추가'}
        </button>
        <button 
          class="btn-secondary"
          onclick={handleCreateFolder}
        >
          📂 새 폴더
        </button>
        <div class="toolbar-info">
          <span>📁 {folders.length}개 폴더</span>
          <span>📄 {files.length}개 파일</span>
          {#if error}
            <span class="error-indicator">⚠️ 오류 발생</span>
          {/if}
        </div>
      </div>

      <!-- 콘텐츠 영역 -->
      <div class="content-area">
        <!-- 사이드바 -->
        <div class="sidebar">
          <div class="sidebar-header">
            <h3>📁 폴더 구조</h3>
          </div>
          
          <!-- 루트 폴더 -->
          <div 
            class="folder-item"
            class:active={currentFolderId === null}
            onclick={() => selectFolder(null)}
          >
            <span class="folder-icon">🏠</span>
            <span class="folder-name">볼트 루트</span>
          </div>
          
          <!-- 폴더 목록 -->
          {#each folders as folder}
            <div 
              class="folder-item"
              class:active={currentFolderId === folder.id}
              onclick={() => selectFolder(folder.id)}
            >
              <span class="folder-icon">📁</span>
              <span class="folder-name">{folder.name}</span>
            </div>
          {/each}
          
          {#if folders.length === 0 && !isLoading}
            <div class="empty-folders">
              <p>폴더가 없습니다.</p>
              <p class="empty-subtitle">새 폴더를 만들어보세요.</p>
            </div>
          {/if}
        </div>

        <!-- 파일 영역 -->
        <div class="file-area">
          <div class="file-area-header">
            <h3>파일 목록</h3>
            <p class="current-path">
              📂 {currentFolderId ? 
                folders.find(f => f.id === currentFolderId)?.name || '알 수 없는 폴더' : 
                '루트 폴더'
              }
            </p>
          </div>
          
          <div class="file-list">
            {#if isLoading}
              <div class="loading-files">
                <div class="loading-spinner"></div>
                <p>파일 목록을 불러오는 중...</p>
              </div>
            {:else if error}
              <div class="error-state">
                <div class="error-icon">⚠️</div>
                <p>파일 목록을 불러올 수 없습니다.</p>
                <p class="error-message">{error}</p>
                <button onclick={() => loadFiles(currentFolderId)} class="btn-secondary">
                  다시 시도
                </button>
              </div>
            {:else if files.length === 0}
              <div class="empty-state">
                <div class="empty-icon">📂</div>
                <p>파일이 없습니다.</p>
                <p class="empty-subtitle">파일을 추가하여 시작해보세요.</p>
              </div>
            {:else}
              <!-- 파일 목록 -->
              <div class="file-grid">
                {#each files as file}
                  <div class="file-item">
                    <div class="file-icon">
                      {getFileIcon(file.mime_type)}
                    </div>
                    <div class="file-info">
                      <div class="file-name" title={file.original_file_name}>
                        {file.file_name}
                      </div>
                      <div class="file-details">
                        <span class="file-size">{formatFileSize(file.file_size)}</span>
                        <span class="file-date">
                          {new Date(file.modified_date).toLocaleDateString('ko-KR')}
                        </span>
                      </div>
                    </div>
                    <div class="file-actions">
                      <button 
                        class="action-btn delete-btn"
                        onclick={() => handleDeleteFile(file)}
                        title="파일 삭제"
                      >
                        🗑️
                      </button>
                    </div>
                  </div>
                {/each}
              </div>
            {/if}
          </div>
        </div>
      </div>
    {/if}
  </div>
</div>

<style>
  .simple-file-manager {
    display: flex;
    flex-direction: column;
    height: 100vh;
    background: #f8f9fa;
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
  }

  /* 헤더 스타일 */
  .header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 1rem 1.5rem;
    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
    color: white;
    box-shadow: 0 2px 4px rgba(0,0,0,0.1);
  }

  .header-left {
    display: flex;
    align-items: center;
    gap: 1rem;
  }

  .header h1 {
    margin: 0;
    font-size: 1.5rem;
    font-weight: 600;
  }

  .subtitle {
    font-size: 0.9rem;
    opacity: 0.9;
  }

  .logout-btn {
    padding: 0.5rem 1rem;
    background: rgba(255, 255, 255, 0.2);
    color: white;
    border: 1px solid rgba(255, 255, 255, 0.3);
    border-radius: 6px;
    cursor: pointer;
    font-size: 0.9rem;
    transition: all 0.2s ease;
  }

  .logout-btn:hover {
    background: rgba(255, 255, 255, 0.3);
    transform: translateY(-1px);
  }

  /* 메인 콘텐츠 */
  .main-content {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  /* 로딩 상태 */
  .loading {
    display: flex;
    flex-direction: column;
    justify-content: center;
    align-items: center;
    height: 100%;
    gap: 1rem;
  }

  .loading-spinner {
    width: 40px;
    height: 40px;
    border: 4px solid #e3e3e3;
    border-top: 4px solid #667eea;
    border-radius: 50%;
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    0% { transform: rotate(0deg); }
    100% { transform: rotate(360deg); }
  }

  /* 도구 모음 */
  .toolbar {
    display: flex;
    align-items: center;
    gap: 1rem;
    padding: 1rem 1.5rem;
    background: white;
    border-bottom: 1px solid #e9ecef;
    box-shadow: 0 1px 3px rgba(0,0,0,0.05);
  }

  .btn-primary {
    padding: 0.6rem 1.2rem;
    background: #007bff;
    color: white;
    border: none;
    border-radius: 6px;
    cursor: pointer;
    font-size: 0.9rem;
    font-weight: 500;
    transition: all 0.2s ease;
  }

  .btn-primary:hover {
    background: #0056b3;
    transform: translateY(-1px);
    box-shadow: 0 2px 8px rgba(0,123,255,0.3);
  }

  .btn-secondary {
    padding: 0.6rem 1.2rem;
    background: #6c757d;
    color: white;
    border: none;
    border-radius: 6px;
    cursor: pointer;
    font-size: 0.9rem;
    font-weight: 500;
    transition: all 0.2s ease;
  }

  .btn-secondary:hover {
    background: #545b62;
    transform: translateY(-1px);
    box-shadow: 0 2px 8px rgba(108,117,125,0.3);
  }

  .toolbar-info {
    margin-left: auto;
    display: flex;
    gap: 1rem;
    font-size: 0.9rem;
    color: #6c757d;
    align-items: center;
  }

  .error-indicator {
    color: #dc3545 !important;
    font-weight: 500;
  }

  /* 콘텐츠 영역 */
  .content-area {
    flex: 1;
    display: flex;
    overflow: hidden;
  }

  /* 사이드바 */
  .sidebar {
    width: 280px;
    background: white;
    border-right: 1px solid #e9ecef;
    padding: 1.5rem;
    overflow-y: auto;
  }

  .sidebar-header {
    margin-bottom: 1rem;
    padding-bottom: 0.5rem;
    border-bottom: 1px solid #e9ecef;
  }

  .sidebar h3 {
    margin: 0;
    color: #495057;
    font-size: 1rem;
    font-weight: 600;
  }

  .folder-item {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.7rem;
    cursor: pointer;
    border-radius: 6px;
    margin-bottom: 0.3rem;
    transition: all 0.2s ease;
    font-size: 0.9rem;
  }

  .folder-item:hover {
    background: #f8f9fa;
  }

  .folder-item.active {
    background: #e3f2fd;
    color: #1976d2;
    font-weight: 500;
  }

  .folder-icon {
    font-size: 1rem;
  }

  .folder-name {
    flex: 1;
  }

  .empty-folders {
    text-align: center;
    padding: 2rem 1rem;
    color: #6c757d;
  }

  .empty-folders p {
    margin: 0.5rem 0;
    font-size: 0.9rem;
  }

  /* 파일 영역 */
  .file-area {
    flex: 1;
    display: flex;
    flex-direction: column;
    background: white;
    overflow: hidden;
  }

  .file-area-header {
    padding: 1.5rem;
    border-bottom: 1px solid #e9ecef;
  }

  .file-area-header h3 {
    margin: 0 0 0.5rem 0;
    color: #495057;
    font-size: 1rem;
    font-weight: 600;
  }

  .current-path {
    margin: 0;
    font-size: 0.85rem;
    color: #6c757d;
  }

  .file-list {
    flex: 1;
    padding: 1.5rem;
    overflow-y: auto;
  }

  /* 로딩 상태 */
  .loading-files {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 200px;
    gap: 1rem;
  }

  .loading-files .loading-spinner {
    width: 32px;
    height: 32px;
    border: 3px solid #e3e3e3;
    border-top: 3px solid #667eea;
    border-radius: 50%;
    animation: spin 1s linear infinite;
  }

  /* 오류 상태 */
  .error-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 200px;
    text-align: center;
    color: #6c757d;
  }

  .error-icon {
    font-size: 3rem;
    margin-bottom: 1rem;
    color: #dc3545;
  }

  .error-message {
    font-size: 0.85rem !important;
    color: #dc3545 !important;
    margin-bottom: 1rem !important;
  }

  /* 빈 상태 */
  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    text-align: center;
    color: #6c757d;
  }

  .empty-icon {
    font-size: 4rem;
    margin-bottom: 1rem;
    opacity: 0.5;
  }

  .empty-state p {
    margin: 0.5rem 0;
    font-size: 1.1rem;
  }

  .empty-subtitle {
    font-size: 0.9rem !important;
    opacity: 0.7;
  }

  /* 파일 그리드 */
  .file-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
    gap: 1rem;
  }

  .file-item {
    display: flex;
    align-items: center;
    gap: 1rem;
    padding: 1rem;
    border: 1px solid #e9ecef;
    border-radius: 8px;
    background: white;
    transition: all 0.2s ease;
    cursor: pointer;
  }

  .file-item:hover {
    border-color: #667eea;
    box-shadow: 0 2px 8px rgba(102, 126, 234, 0.1);
    transform: translateY(-1px);
  }

  .file-icon {
    font-size: 2rem;
    flex-shrink: 0;
  }

  .file-info {
    flex: 1;
    min-width: 0;
  }

  .file-name {
    font-weight: 500;
    color: #495057;
    margin-bottom: 0.25rem;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .file-details {
    display: flex;
    gap: 1rem;
    font-size: 0.8rem;
    color: #6c757d;
  }

  .file-actions {
    display: flex;
    gap: 0.5rem;
    flex-shrink: 0;
  }

  .action-btn {
    padding: 0.5rem;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    font-size: 1rem;
    transition: all 0.2s ease;
    background: transparent;
  }

  .action-btn:hover {
    background: #f8f9fa;
    transform: scale(1.1);
  }

  .delete-btn:hover {
    background: #f8d7da;
    color: #721c24;
  }
</style>