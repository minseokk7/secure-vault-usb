<!--
  미디어 플레이어 컴포넌트
  오디오 및 비디오 파일을 재생하는 기능을 제공합니다.
-->
<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { invoke, convertFileSrc } from "@tauri-apps/api/core";
  import Button from "$lib/components/common/Button.svelte";
  import type { FileMetadata } from "$lib/types/file-manager";

  // Props
  interface Props {
    file: FileMetadata;
    onClose?: () => void;
  }

  const { file, onClose }: Props = $props();

  // 미디어 메타데이터 타입
  interface MediaMetadata {
    title?: string;
    artist?: string;
    album?: string;
    duration?: number;
    bitrate?: number;
    sample_rate?: number;
    channels?: number;
    media_type: "Audio" | "Video";
    file_size: number;
    mime_type: string;
    file_path: string;
  }

  // 상태 변수들
  let mediaElement: HTMLAudioElement | HTMLVideoElement | null = $state(null);
  let metadata: MediaMetadata | null = $state(null);
  let isLoading = $state(true);
  let isPlaying = $state(false);
  let currentTime = $state(0);
  let duration = $state(0);
  let volume = $state(1.0);
  let isMuted = $state(false);
  let error = $state<string | null>(null);
  let mediaUrl = $state<string | null>(null);

  // 추가 상태
  let playbackRate = $state(1.0);
  let isLooping = $state(false);

  // 계산된 상태
  const progress = $derived(duration > 0 ? (currentTime / duration) * 100 : 0);
  const isAudio = $derived(metadata?.media_type === "Audio");
  const isVideo = $derived(metadata?.media_type === "Video");
  const formattedCurrentTime = $derived(formatTime(currentTime));
  const formattedDuration = $derived(formatTime(duration));

  // 컴포넌트 마운트 시 초기화
  // 컴포넌트 마운트 시 초기화
  onMount(async () => {
    console.log("MediaPlayer: 마운트됨", file.id);
    try {
      await loadMediaMetadata();

      // 스트리밍 준비 (임시 파일 복호화)
      console.log("MediaPlayer: 스트리밍 준비 요청");
      const tempFilePath = await invoke<string>("prepare_media_stream", {
        fileId: file.id,
      });
      console.log("MediaPlayer: 스트리밍 준비 완료 (임시 경로):", tempFilePath);

      // Tauri의 convertFileSrc를 사용하여 올바른 URL 생성
      // convertFileSrc는 로컬 파일 경로를 Tauri가 이해할 수 있는 asset:// 프로토콜로 변환
      mediaUrl = convertFileSrc(tempFilePath);
      console.log("MediaPlayer: Tauri URL 생성됨", mediaUrl);
    } catch (err) {
      console.error("미디어 로드 실패:", err);
      // 에러 메시지 간소화
      error = "미디어 파일을 준비하는 중 오류가 발생했습니다. (복호화 실패 등)";
      if (err instanceof Error) {
        error += " " + err.message;
      } else if (typeof err === "string") {
        error += " " + err;
      }
    } finally {
      isLoading = false;
    }
  });

  onDestroy(() => {
    console.log("MediaPlayer: 언마운트");
    // URL.revokeObjectURL은 필요 없음 (convertFileSrc는 브라우저 관리)
  });

  // 미디어 메타데이터 로드
  async function loadMediaMetadata() {
    try {
      metadata = await invoke<MediaMetadata>("get_media_metadata", {
        fileId: file.id,
      });
      console.log("미디어 메타데이터:", metadata);
    } catch (err) {
      console.error("메타데이터 로드 실패:", err);
      throw new Error("미디어 메타데이터를 로드할 수 없습니다.");
    }
  }

  // 재생 속도 조절
  function cyclePlaybackRate() {
    if (!mediaElement) return;

    const rates = [0.5, 1.0, 1.25, 1.5, 2.0];
    const currentIndex = rates.indexOf(playbackRate);
    const nextIndex = (currentIndex + 1) % rates.length;
    playbackRate = rates[nextIndex];
    mediaElement.playbackRate = playbackRate;
  }

  // 반복 재생 토글
  function toggleLoop() {
    if (!mediaElement) return;
    isLooping = !isLooping;
    mediaElement.loop = isLooping;
  }

  // 시간 건너뛰기
  function skipTime(seconds: number) {
    if (!mediaElement) return;
    const newTime = Math.min(
      Math.max(mediaElement.currentTime + seconds, 0),
      duration,
    );
    mediaElement.currentTime = newTime;
  }

  // 전체화면 토글
  function toggleFullscreen() {
    if (!mediaElement) return;

    if (document.fullscreenElement) {
      document.exitFullscreen();
    } else {
      mediaElement.requestFullscreen().catch((err) => {
        console.error("전체화면 진입 실패:", err);
      });
    }
  }

  // 재생/일시정지 토글
  function togglePlayPause() {
    if (!mediaElement) return;

    if (isPlaying) {
      mediaElement.pause();
    } else {
      mediaElement.play().catch((err) => {
        console.error("재생 실패:", err);
        error = "미디어를 재생할 수 없습니다.";
      });
    }
  }

  // 음소거 토글
  function toggleMute() {
    if (!mediaElement) return;
    isMuted = !isMuted;
    mediaElement.muted = isMuted;
  }

  // 볼륨 설정
  function setVolume(e: Event) {
    if (!mediaElement) return;
    const target = e.target as HTMLInputElement;
    volume = parseFloat(target.value) / 100;
    mediaElement.volume = volume;
    if (volume > 0 && isMuted) {
      isMuted = false;
      mediaElement.muted = false;
    }
  }

  // 탐색 (Seek)
  function seekTo(e: Event) {
    if (!mediaElement) return;
    const target = e.target as HTMLInputElement;
    const percent = parseFloat(target.value);
    const time = (percent / 100) * duration;
    mediaElement.currentTime = time;
  }

  // 미디어 이벤트 핸들러
  function handleLoadedMetadata() {
    if (!mediaElement) return;
    console.log("미디어 로드 완료 (duration):", mediaElement.duration);
    duration = mediaElement.duration;
    mediaElement.volume = volume;
    mediaElement.playbackRate = playbackRate;
    mediaElement.loop = isLooping;

    // 자동 재생
    mediaElement.play().catch((e) => console.warn("자동 재생 실패:", e));
  }

  function handleTimeUpdate() {
    if (!mediaElement) return;
    currentTime = mediaElement.currentTime;
  }

  function handlePlay() {
    isPlaying = true;
  }

  function handlePause() {
    isPlaying = false;
  }

  function handleEnded() {
    isPlaying = false;
    if (!isLooping) {
      // currentTime = 0; // 필요 시 주석 해제
    }
  }

  function handleError(event: Event) {
    console.error("미디어 에러 이벤트:", event);
    if (mediaElement && mediaElement.error) {
      console.error("미디어 에러 코드:", mediaElement.error.code);
      console.error("미디어 에러 메시지:", mediaElement.error.message);

      let msg = "알 수 없는 오류";
      switch (mediaElement.error.code) {
        case 1:
          msg = "사용자 취소";
          break;
        case 2:
          msg = "네트워크 오류";
          break;
        case 3:
          msg = "디코딩 오류";
          break;
        case 4:
          msg = "소스 지원 안됨";
          break;
      }
      error = msg;
    } else {
      error = "재생 중 오류 발생";
    }
    isPlaying = false;
  }

  // 시간 포맷팅 (초 -> MM:SS)
  function formatTime(seconds: number): string {
    if (!isFinite(seconds)) return "00:00";

    const mins = Math.floor(seconds / 60);
    const secs = Math.floor(seconds % 60);
    return `${mins.toString().padStart(2, "0")}:${secs.toString().padStart(2, "0")}`;
  }

  // 파일 크기 포맷팅
  function formatFileSize(bytes: number): string {
    if (bytes === 0) return "0 B";
    const k = 1024;
    const sizes = ["B", "KB", "MB", "GB"];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + " " + sizes[i];
  }
</script>

<div class="media-player">
  <!-- 헤더 -->
  <div class="media-header">
    <div class="media-info">
      <h3 class="media-title text-korean">
        {metadata?.title || file.name}
      </h3>
      {#if metadata?.artist}
        <p class="media-artist text-korean">{metadata.artist}</p>
      {/if}
      {#if metadata?.album}
        <p class="media-album text-korean">{metadata.album}</p>
      {/if}
    </div>

    {#if onClose}
      <Button variant="ghost" size="small" onclick={onClose}>
        <svg
          class="w-5 h-5"
          fill="none"
          stroke="currentColor"
          viewBox="0 0 24 24"
        >
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            stroke-width="2"
            d="M6 18L18 6M6 6l12 12"
          />
        </svg>
      </Button>
    {/if}
  </div>

  <!-- 로딩 상태 -->
  {#if isLoading}
    <div class="loading-container">
      <div class="loading-spinner"></div>
      <p class="text-korean">미디어 로드 중...</p>
    </div>
  {:else if error}
    <!-- 에러 상태 -->
    <div class="error-container">
      <svg
        class="w-12 h-12 text-red-500 mb-4"
        fill="none"
        stroke="currentColor"
        viewBox="0 0 24 24"
      >
        <path
          stroke-linecap="round"
          stroke-linejoin="round"
          stroke-width="2"
          d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
        />
      </svg>
      <p class="text-red-600 text-korean">{error}</p>
    </div>
  {:else if mediaUrl && metadata}
    <!-- 미디어 플레이어 -->
    <div class="media-content">
      <!-- 비디오 플레이어 -->
      {#if isVideo}
        <video
          bind:this={mediaElement}
          src={mediaUrl}
          class="video-element"
          controls={false}
          onloadedmetadata={handleLoadedMetadata}
          ontimeupdate={handleTimeUpdate}
          onplay={handlePlay}
          onpause={handlePause}
          onended={handleEnded}
          onerror={handleError}
        >
          <track kind="captions" />
        </video>
      {:else if isAudio}
        <!-- 오디오 플레이어 (시각적 표현) -->
        <div class="audio-visualizer">
          <audio
            bind:this={mediaElement}
            src={mediaUrl}
            onloadedmetadata={handleLoadedMetadata}
            ontimeupdate={handleTimeUpdate}
            onplay={handlePlay}
            onpause={handlePause}
            onended={handleEnded}
            onerror={handleError}
          >
          </audio>

          <!-- 오디오 시각화 -->
          <div class="audio-artwork">
            <svg
              class="w-24 h-24 text-blue-400"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M9 19V6l12-3v13M9 19c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zm12-3c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zM9 10l12-3"
              />
            </svg>
          </div>
        </div>
      {/if}

      <!-- 컨트롤 패널 -->
      <div class="media-controls">
        <!-- 진행률 바 -->
        <div class="progress-container">
          <span class="time-display text-korean">{formattedCurrentTime}</span>
          <input
            type="range"
            min="0"
            max="100"
            value={progress}
            class="progress-slider"
            oninput={seekTo}
          />
          <span class="time-display text-korean">{formattedDuration}</span>
        </div>

        <!-- 메인 컨트롤 -->
        <div class="main-controls">
          <!-- 추가 컨트롤: 반복 재생 -->
          <Button
            variant="ghost"
            size="small"
            onclick={toggleLoop}
            class={isLooping ? "text-blue-500" : "text-gray-500"}
            title="반복 재생"
          >
            <svg
              class="w-5 h-5"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"
              />
            </svg>
          </Button>

          <div class="playback-buttons">
            <!-- 10초 뒤로 -->
            <Button
              variant="ghost"
              size="small"
              onclick={() => skipTime(-10)}
              title="10초 뒤로"
            >
              <svg
                class="w-6 h-6"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width="2"
                  d="M12.066 11.2a1 1 0 000 1.6l5.334 4A1 1 0 0019 16V8a1 1 0 00-1.6-.8l-5.333 4zM4.066 11.2a1 1 0 000 1.6l5.334 4A1 1 0 0011 16V8a1 1 0 00-1.6-.8l-5.334 4z"
                />
              </svg>
            </Button>

            <!-- 재생/일시정지 버튼 -->
            <Button
              variant="primary"
              size="large"
              onclick={togglePlayPause}
              class="play-button"
            >
              {#if isPlaying}
                <svg
                  class="w-8 h-8"
                  fill="none"
                  stroke="currentColor"
                  viewBox="0 0 24 24"
                >
                  <path
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    stroke-width="2"
                    d="M10 9v6m4-6v6"
                  />
                </svg>
              {:else}
                <svg class="w-8 h-8" fill="currentColor" viewBox="0 0 24 24">
                  <path d="M8 5v14l11-7z" />
                </svg>
              {/if}
            </Button>

            <!-- 10초 앞으로 -->
            <Button
              variant="ghost"
              size="small"
              onclick={() => skipTime(10)}
              title="10초 앞으로"
            >
              <svg
                class="w-6 h-6"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width="2"
                  d="M13 10V3L4 14h7v7l9-11h-7z"
                  style="display:none"
                />
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width="2"
                  d="M11.933 12.8a1 1 0 000-1.6L6.6 7.2A1 1 0 005 8v8a1 1 0 001.6.8l5.333-4zM19.933 12.8a1 1 0 000-1.6l-5.333-4A1 1 0 0013 8v8a1 1 0 001.6.8l5.333-4z"
                />
              </svg>
            </Button>
          </div>

          <!-- 재생 속도 -->
          <div class="speed-control relative">
            <button
              class="text-xs font-bold text-gray-600 hover:text-blue-600 w-10 text-center"
              onclick={cyclePlaybackRate}
              title="재생 속도"
            >
              {playbackRate}x
            </button>
          </div>
        </div>

        <div class="bottom-controls">
          <!-- 볼륨 컨트롤 -->
          <div class="volume-controls">
            <Button variant="ghost" size="small" onclick={toggleMute}>
              {#if isMuted || volume === 0}
                <svg
                  class="w-5 h-5"
                  fill="none"
                  stroke="currentColor"
                  viewBox="0 0 24 24"
                >
                  <path
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    stroke-width="2"
                    d="M5.586 15H4a1 1 0 01-1-1v-4a1 1 0 011-1h1.586l4.707-4.707C10.923 3.663 12 4.109 12 5v14c0 .891-1.077 1.337-1.707.707L5.586 15z"
                  />
                  <path
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    stroke-width="2"
                    d="M17 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2"
                  />
                </svg>
              {:else}
                <svg
                  class="w-5 h-5"
                  fill="none"
                  stroke="currentColor"
                  viewBox="0 0 24 24"
                >
                  <path
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    stroke-width="2"
                    d="M15.536 8.464a5 5 0 010 7.072m2.828-9.9a9 9 0 010 12.728M5.586 15H4a1 1 0 01-1-1v-4a1 1 0 011-1h1.586l4.707-4.707C10.923 3.663 12 4.109 12 5v14c0 .891-1.077 1.337-1.707.707L5.586 15z"
                  />
                </svg>
              {/if}
            </Button>

            <input
              type="range"
              min="0"
              max="100"
              value={volume * 100}
              class="volume-slider"
              oninput={setVolume}
            />
          </div>

          <!-- 전체화면 (비디오인 경우) -->
          {#if isVideo}
            <Button
              variant="ghost"
              size="small"
              onclick={toggleFullscreen}
              title="전체화면"
            >
              <svg
                class="w-5 h-5"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width="2"
                  d="M4 8V4m0 0h4M4 4l5 5m11-1V4m0 0h-4m4 0l-5 5M4 16v4m0 0h4m-4 0l5-5m11 5l-5-5m5 5v-4m0 4h-4"
                />
              </svg>
            </Button>
          {/if}
        </div>
      </div>

      <!-- 미디어 정보 -->
      <div class="media-metadata">
        <div class="metadata-grid">
          <div class="metadata-item">
            <span class="metadata-label text-korean">파일 크기:</span>
            <span class="metadata-value text-korean"
              >{formatFileSize(metadata.file_size)}</span
            >
          </div>

          {#if metadata.duration}
            <div class="metadata-item">
              <span class="metadata-label text-korean">재생 시간:</span>
              <span class="metadata-value text-korean"
                >{formatTime(metadata.duration)}</span
              >
            </div>
          {/if}

          {#if metadata.bitrate}
            <div class="metadata-item">
              <span class="metadata-label text-korean">비트레이트:</span>
              <span class="metadata-value text-korean"
                >{metadata.bitrate} kbps</span
              >
            </div>
          {/if}

          {#if metadata.sample_rate}
            <div class="metadata-item">
              <span class="metadata-label text-korean">샘플레이트:</span>
              <span class="metadata-value text-korean"
                >{metadata.sample_rate} Hz</span
              >
            </div>
          {/if}

          {#if metadata.channels}
            <div class="metadata-item">
              <span class="metadata-label text-korean">채널:</span>
              <span class="metadata-value text-korean"
                >{metadata.channels}ch</span
              >
            </div>
          {/if}
        </div>
      </div>
    </div>
  {/if}
</div>

<style>
  .media-player {
    @apply w-full h-full flex flex-col bg-white;
  }

  .media-header {
    @apply flex items-start justify-between p-4 border-b border-gray-200;
  }

  .media-info {
    @apply flex-1;
  }

  .media-title {
    @apply text-lg font-semibold text-gray-800 mb-1;
  }

  .media-artist {
    @apply text-sm text-gray-600 mb-1;
  }

  .media-album {
    @apply text-xs text-gray-500;
  }

  .loading-container {
    @apply flex flex-col items-center justify-center py-12;
  }

  .loading-spinner {
    @apply w-8 h-8 border-4 border-blue-200 border-t-blue-500 rounded-full animate-spin mb-4;
  }

  .error-container {
    @apply flex flex-col items-center justify-center py-12 text-center;
  }

  .media-content {
    @apply flex-1 flex flex-col;
  }

  .video-element {
    @apply w-full flex-1 bg-black;
  }

  .audio-visualizer {
    @apply flex-1 flex items-center justify-center bg-gradient-to-br from-blue-50 to-indigo-100;
  }

  .audio-artwork {
    @apply flex items-center justify-center w-32 h-32 bg-white bg-opacity-50 rounded-full;
  }

  .media-controls {
    @apply p-4 bg-gray-50 border-t border-gray-200 flex flex-col gap-4;
  }

  .progress-container {
    @apply flex items-center gap-3 w-full;
  }

  .time-display {
    @apply text-xs text-gray-500 font-mono min-w-[40px] text-center;
  }

  .progress-slider {
    @apply flex-1 h-1.5 bg-gray-200 rounded-lg appearance-none cursor-pointer;
  }

  .progress-slider::-webkit-slider-thumb {
    @apply appearance-none w-3.5 h-3.5 bg-blue-500 rounded-full cursor-pointer hover:scale-110 transition-transform;
  }

  /* 메인 컨트롤 영역 (반복, 재생버튼 그룹, 속도) */
  .main-controls {
    @apply flex items-center justify-between px-2;
  }

  .playback-buttons {
    @apply flex items-center gap-6;
  }

  .play-button {
    @apply rounded-full w-14 h-14 flex items-center justify-center shadow-md hover:shadow-lg hover:scale-105 transition-all text-white;
  }

  /* 하단 컨트롤 (볼륨, 전체화면) */
  .bottom-controls {
    @apply flex items-center justify-between;
  }

  .volume-controls {
    @apply flex items-center gap-2;
  }

  .volume-slider {
    @apply w-24 h-1.5 bg-gray-200 rounded-lg appearance-none cursor-pointer;
  }

  .volume-slider::-webkit-slider-thumb {
    @apply appearance-none w-3 h-3 bg-gray-400 rounded-full cursor-pointer hover:bg-blue-500 transition-colors;
  }

  .media-metadata {
    @apply p-4 bg-white border-t border-gray-100;
  }

  .metadata-grid {
    @apply grid grid-cols-2 gap-x-4 gap-y-2 text-xs;
  }

  .metadata-item {
    @apply flex justify-between items-center;
  }

  .metadata-label {
    @apply text-gray-400;
  }

  .metadata-value {
    @apply text-gray-700 font-medium;
  }

  /* 한국어 텍스트 최적화 */
  .text-korean {
    word-break: keep-all;
    line-height: 1.5;
  }

  /* 반응형 디자인 */
  @media (max-width: 640px) {
    .main-controls {
      @apply justify-center gap-4;
    }

    .playback-buttons {
      gap: 4;
    }

    .speed-control,
    .bottom-controls {
      display: none; /* 모바일에서는 일부 숨김 */
    }
  }
</style>
