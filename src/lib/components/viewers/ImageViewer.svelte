<script lang="ts">
    import { onMount, onDestroy, createEventDispatcher } from "svelte";
    import { invoke } from "@tauri-apps/api/core";
    import type { FileMetadata } from "$lib/types/file-manager";
    import { addToast } from "$lib/stores/toast";

    export let file: FileMetadata;
    export let onClose: () => void;

    const dispatch = createEventDispatcher<{
        export: void;
    }>();

    let imageUrl: string | null = null;
    let isLoading = true;
    let error: string | null = null;

    // 이미지 로드
    async function loadImage() {
        try {
            isLoading = true;
            error = null;

            // 파일 내용 가져오기 (바이너리)
            const content = await invoke<number[]>("get_file_content", {
                fileId: file.id,
            });

            const uint8Array = new Uint8Array(content);

            // Blob 생성
            const blob = new Blob([uint8Array], {
                type: file.mime_type || "image/png",
            });

            // 기존 URL 해제
            if (imageUrl) {
                URL.revokeObjectURL(imageUrl);
            }

            // 새 URL 생성
            imageUrl = URL.createObjectURL(blob);
        } catch (err) {
            console.error("이미지 로드 실패:", err);
            error = "이미지를 불러오는데 실패했습니다.";
            addToast({
                type: "error",
                message: `이미지 로드 실패: ${err}`,
            });
        } finally {
            isLoading = false;
        }
    }

    function handleExport() {
        dispatch("export");
    }

    onMount(() => {
        loadImage();
    });

    onDestroy(() => {
        if (imageUrl) {
            URL.revokeObjectURL(imageUrl);
        }
    });
</script>

<div class="image-viewer-container">
    {#if isLoading}
        <div class="loading">
            <div class="spinner"></div>
            <p>이미지 불러오는 중...</p>
        </div>
    {:else if error}
        <div class="error">
            <div class="error-icon">⚠️</div>
            <p>{error}</p>
            <button class="btn" on:click={loadImage}>다시 시도</button>
        </div>
    {:else if imageUrl}
        <div class="image-wrapper">
            <img src={imageUrl} alt={file.file_name} />
        </div>
    {/if}

    <div class="viewer-actions">
        <button class="btn btn-primary" on:click={handleExport}>
            파일 내보내기
        </button>
        <button class="btn" on:click={onClose}>닫기</button>
    </div>
</div>

<style>
    .image-viewer-container {
        display: flex;
        flex-direction: column;
        height: 100%;
        width: 100%;
        position: relative;
        background-color: #000; /* 어두운 배경 */
    }

    .image-wrapper {
        flex: 1;
        display: flex;
        align-items: center;
        justify-content: center;
        overflow: auto;
        padding: 20px;
    }

    img {
        max-width: 100%;
        max-height: 100%;
        object-fit: contain;
        box-shadow: 0 4px 6px rgba(0, 0, 0, 0.3);
    }

    .loading,
    .error {
        flex: 1;
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        color: white;
    }

    .spinner {
        width: 40px;
        height: 40px;
        border: 4px solid rgba(255, 255, 255, 0.3);
        border-radius: 50%;
        border-top-color: white;
        animation: spin 1s ease-in-out infinite;
        margin-bottom: 16px;
    }

    @keyframes spin {
        to {
            transform: rotate(360deg);
        }
    }

    .error-icon {
        font-size: 48px;
        margin-bottom: 16px;
    }

    .viewer-actions {
        display: flex;
        justify-content: center;
        gap: 12px;
        padding: 16px;
        background-color: rgba(0, 0, 0, 0.8);
        backdrop-filter: blur(5px);
    }

    .btn {
        padding: 8px 16px;
        border: 1px solid rgba(255, 255, 255, 0.2);
        background: rgba(255, 255, 255, 0.1);
        color: white;
        border-radius: 4px;
        cursor: pointer;
        transition: all 0.2s;
        font-size: 14px;
    }

    .btn:hover {
        background: rgba(255, 255, 255, 0.2);
    }

    .btn-primary {
        background: #007bff;
        border-color: #007bff;
    }

    .btn-primary:hover {
        background: #0056b3;
    }
</style>
