<!--
  미디어 플레이어 컴포넌트
  오디오 및 비디오 파일을 재생하는 기능을 제공합니다.
-->
<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import Button from '$lib/components/common/Button.svelte';
  import type { FileMetadata } from '$lib/types/file-manager';

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
    media_type: 'Audio' | 'Video';
    file_size: number;
    mime_type: string;
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

  // 계산된 상태
  const progress = $derived(duration > 0 ? (currentTime / duration) * 100 : 0);
  const isAudio = $derived(metadata?.media_type === 'Audio');
  const isVideo = $derived(metadata?.media_type === 'Video');
  const formattedCurrentTime = $derived(formatTime(currentTime));
  const formattedDuration = $derived(formatTime(duration));

  // 컴포넌트 마운트 시 초기화
  onMount(async () => {
    try {
      await loadMediaMetadata();
      await loadMediaData();
    } catch (err) {
      console.error('미디어 로드 실패:', err);
      error = err instanceof Error ? err.message : '미디어를 로드할 수 없습니다.';
    } finally {
      isLoading = false;
    }
  });

  // 컴포넌트 언마운트 시 정리
  onDestroy(() => {
    if (mediaUrl) {
      URL.revokeObjectURL(mediaUrl);
    }
  });

  // 미디어 메타데이터 로드
  async function loadMediaMetadata() {
    try {
      metadata = await invoke<MediaMetadata>('get_media_metadata', {
        file_id: file.id
      });
      console.log('미디어 메타데이터 로드됨:', metadata);
    } catch (err) {
      console.error('메타데이터 로드 실패:', err);
      throw new Error('미디어 메타데이터를 로드할 수 없습니다.');
    }
  }

  // 미디어 데이터 로드
  async function loadMediaData() {
    if (!metadata) return;

    try {
      // 10MB 이하의 파일은 전체 로드, 그 이상은 스트리밍
      const maxSize = 10 * 1024 * 1024; // 10MB
      
      if (metadata.file_size <= maxSize) {
        await loadFullMedia();
      } else {
        // TODO: 스트리밍 구현 (향후 개선)
        throw new Error('큰 파일의 스트리밍은 아직 지원되지 않습니다.');
      }
    } catch (err) {
      console.error('미디어 데이터 로드 실패:', err);
      throw err;
    }
  }

  // 전체 미디어 파일 로드
  async function loadFullMedia() {
    if (!metadata) return;

    try {
      const base64Data = await invoke<string>('get_full_media_data', {
        file_id: file.id
      });

      // Base64 데이터를 Blob으로 변환
      const binaryData = atob(base64Data);
      const bytes = new Uint8Array(binaryData.length);
      for (let i = 0; i < binaryData.length; i++) {
        bytes[i] = binaryData.charCodeAt(i);
      }

      const blob = new Blob([bytes], { type: metadata.mime_type });
      mediaUrl = URL.createObjectURL(blob);
      
      console.log('미디어 URL 생성됨:', mediaUrl);
    } catch (err) {
      console.error('전체 미디어 로드 실패:', err);
      throw new Error('미디어 파일을 로드할 수 없습니다.');
    }
  }

  // 재생/일시정지 토글
  function togglePlayPause() {
    if (!mediaElement) return;

    if (isPlaying) {
      mediaElement.pause();
    } else {
      mediaElement.play().catch(err => {
        console.error('재생 실패:', err);
        error = '미디어를 재생할 수 없습니다.';
      });
    }
  }

  // 시간 이동
  function seekTo(event: Event) {
    if (!mediaElement) return;

    const target = event.target as HTMLInputElement;
    const seekTime = (parseFloat(target.value) / 100) * duration;
    mediaElement.currentTime = seekTime;
  }

  // 볼륨 조절
  function setVolume(event: Event) {
    if (!mediaElement) return;

    const target = event.target as HTMLInputElement;
    const newVolume = parseFloat(target.value) / 100;
    volume = newVolume;
    mediaElement.volume = newVolume;
  }

  // 음소거 토글
  function toggleMute() {
    if (!mediaElement) return;

    isMuted = !isMuted;
    mediaElement.muted = isMuted;
  }

  // 미디어 이벤트 핸들러들
  function handleLoadedMetadata() {
    if (mediaElement) {
      duration = mediaElement.duration || 0;
    }
  }

  function handleTimeUpdate() {
    if (mediaElement) {
      currentTime = mediaElement.currentTime || 0;
    }
  }

  function handlePlay() {
    isPlaying = true;
  }

  function handlePause() {
    isPlaying = false;
  }

  function handleEnded() {
    isPlaying = false;
    currentTime = 0;
  }

  function handleError(event: Event) {
    console.error('미디어 재생 오류:', event);
    error = '미디어 재생 중 오류가 발생했습니다.';
    isPlaying = false;
  }

  // 시간 포맷팅 (초 -> MM:SS)
  function formatTime(seconds: number): string {
    if (!isFinite(seconds)) return '00:00';
    
    const mins = Math.floor(seconds / 60);
    const secs = Math.floor(seconds % 60);
    return `${mins.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`;
  }

  // 파일 크기 포맷팅
  function formatFileSize(bytes: number): string {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
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
        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"/>
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
      <svg class="w-12 h-12 text-red-500 mb-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
              d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"/>
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
            <svg class="w-24 h-24 text-blue-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
                    d="M9 19V6l12-3v13M9 19c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zm12-3c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zM9 10l12-3"/>
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

        <!-- 재생 컨트롤 -->
        <div class="playback-controls">
          <!-- 재생/일시정지 버튼 -->
          <Button
            variant="primary"
            size="large"
            onclick={togglePlayPause}
            class="play-button"
          >
            {#if isPlaying}
              <!-- 일시정지 아이콘 -->
              <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 9v6m4-6v6"/>
              </svg>
            {:else}
              <!-- 재생 아이콘 -->
              <svg class="w-6 h-6" fill="currentColor" viewBox="0 0 24 24">
                <path d="M8 5v14l11-7z"/>
              </svg>
            {/if}
          </Button>

          <!-- 볼륨 컨트롤 -->
          <div class="volume-controls">
            <Button variant="ghost" size="small" onclick={toggleMute}>
              {#if isMuted || volume === 0}
                <!-- 음소거 아이콘 -->
                <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
                        d="M5.586 15H4a1 1 0 01-1-1v-4a1 1 0 011-1h1.586l4.707-4.707C10.923 3.663 12 4.109 12 5v14c0 .891-1.077 1.337-1.707.707L5.586 15z"/>
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2"/>
                </svg>
              {:else}
                <!-- 볼륨 아이콘 -->
                <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
                        d="M15.536 8.464a5 5 0 010 7.072m2.828-9.9a9 9 0 010 12.728M5.586 15H4a1 1 0 01-1-1v-4a1 1 0 011-1h1.586l4.707-4.707C10.923 3.663 12 4.109 12 5v14c0 .891-1.077 1.337-1.707.707L5.586 15z"/>
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
        </div>
      </div>

      <!-- 미디어 정보 -->
      <div class="media-metadata">
        <div class="metadata-grid">
          <div class="metadata-item">
            <span class="metadata-label text-korean">파일 크기:</span>
            <span class="metadata-value text-korean">{formatFileSize(metadata.file_size)}</span>
          </div>
          
          {#if metadata.duration}
            <div class="metadata-item">
              <span class="metadata-label text-korean">재생 시간:</span>
              <span class="metadata-value text-korean">{formatTime(metadata.duration)}</span>
            </div>
          {/if}
          
          {#if metadata.bitrate}
            <div class="metadata-item">
              <span class="metadata-label text-korean">비트레이트:</span>
              <span class="metadata-value text-korean">{metadata.bitrate} kbps</span>
            </div>
          {/if}
          
          {#if metadata.sample_rate}
            <div class="metadata-item">
              <span class="metadata-label text-korean">샘플레이트:</span>
              <span class="metadata-value text-korean">{metadata.sample_rate} Hz</span>
            </div>
          {/if}
          
          {#if metadata.channels}
            <div class="metadata-item">
              <span class="metadata-label text-korean">채널:</span>
              <span class="metadata-value text-korean">{metadata.channels}ch</span>
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
    @apply p-4 bg-gray-50 border-t border-gray-200;
  }

  .progress-container {
    @apply flex items-center gap-3 mb-4;
  }

  .time-display {
    @apply text-sm text-gray-600 font-mono min-w-12 text-center;
  }

  .progress-slider {
    @apply flex-1 h-2 bg-gray-200 rounded-lg appearance-none cursor-pointer;
  }

  .progress-slider::-webkit-slider-thumb {
    @apply appearance-none w-4 h-4 bg-blue-500 rounded-full cursor-pointer;
  }

  .progress-slider::-moz-range-thumb {
    @apply w-4 h-4 bg-blue-500 rounded-full cursor-pointer border-0;
  }

  .playback-controls {
    @apply flex items-center justify-between;
  }

  .play-button {
    @apply rounded-full w-12 h-12 flex items-center justify-center;
  }

  .volume-controls {
    @apply flex items-center gap-2;
  }

  .volume-slider {
    @apply w-20 h-2 bg-gray-200 rounded-lg appearance-none cursor-pointer;
  }

  .volume-slider::-webkit-slider-thumb {
    @apply appearance-none w-3 h-3 bg-blue-500 rounded-full cursor-pointer;
  }

  .volume-slider::-moz-range-thumb {
    @apply w-3 h-3 bg-blue-500 rounded-full cursor-pointer border-0;
  }

  .media-metadata {
    @apply p-4 bg-gray-50 border-t border-gray-200;
  }

  .metadata-grid {
    @apply grid grid-cols-2 gap-2 text-sm;
  }

  .metadata-item {
    @apply flex justify-between;
  }

  .metadata-label {
    @apply text-gray-600;
  }

  .metadata-value {
    @apply text-gray-800 font-medium;
  }

  /* 한국어 텍스트 최적화 */
  .text-korean {
    word-break: keep-all;
    line-height: 1.5;
  }

  /* 반응형 디자인 */
  @media (max-width: 640px) {
    .metadata-grid {
      @apply grid-cols-1;
    }
    
    .playback-controls {
      @apply flex-col gap-4;
    }
    
    .volume-controls {
      @apply justify-center;
    }
  }
</style>