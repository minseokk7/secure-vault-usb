<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { getCurrentWindow } from "@tauri-apps/api/window";

  // 이벤트 디스패처
  const dispatch = createEventDispatcher<{
    loginSuccess: { pin: string };
    loginFailed: { error: string };
  }>();

  // 상태 변수들
  let pin = "";
  let showPassword = false;
  let isLoading = false;
  let errorMessage = "";

  // 복구 모드 상태
  let isRecoveryMode = false;
  let recoveryKey = "";
  let showNewPin = false; // 복구 성공 후 새 PIN 설정 화면

  // 새 PIN 설정용 변수
  let newPin = "";
  let confirmPin = "";

  // 복구 처리
  async function handleRecovery() {
    if (!recoveryKey || recoveryKey.trim().length === 0) {
      errorMessage = "복구 키를 입력해주세요.";
      return;
    }

    isLoading = true;
    errorMessage = "";

    try {
      const success = await invoke("authenticate_recovery_key", {
        recoveryKey: recoveryKey.trim(),
      });

      if (success) {
        console.log("복구 키 인증 성공");
        showNewPin = true; // 새 PIN 설정 화면으로 전환
      } else {
        errorMessage = "올바르지 않은 복잡 키입니다. 다시 확인해주세요.";
      }
    } catch (error) {
      console.error("복구 키 인증 오류:", error);
      errorMessage =
        typeof error === "string" ? error : "복구 처리 중 오류가 발생했습니다.";
    } finally {
      isLoading = false;
    }
  }

  // 새 PIN 설정 처리 (복구 후)
  async function handleResetPin() {
    if (newPin.length < 4 || newPin.length > 8) {
      errorMessage = "PIN은 4~8자리여야 합니다.";
      return;
    }
    if (newPin !== confirmPin) {
      errorMessage = "PIN이 일치하지 않습니다.";
      return;
    }

    isLoading = true;
    errorMessage = "";

    try {
      await invoke("set_pin_code", { pin: newPin, complexity: "basic" });
      dispatch("loginSuccess", { pin: newPin });
    } catch (error) {
      errorMessage =
        typeof error === "string" ? error : "PIN 설정 중 오류 발생";
    } finally {
      isLoading = false;
    }
  }

  // 로그인 처리
  async function handleLogin() {
    if (!pin || pin.length < 4) {
      errorMessage = "PIN을 4자리 이상 입력해주세요.";
      return;
    }

    if (pin.length > 8) {
      errorMessage = "PIN은 8자리를 초과할 수 없습니다.";
      return;
    }

    // 숫자만 허용
    if (!/^\d+$/.test(pin)) {
      errorMessage = "PIN은 숫자만 입력 가능합니다.";
      return;
    }

    isLoading = true;
    errorMessage = "";

    try {
      // 실제 Tauri 백엔드 호출 (마스터 키 설정 필수!)
      const success = await invoke("authenticate_pin", { pin });

      if (success) {
        console.log("로그인 성공:", pin);
        dispatch("loginSuccess", { pin });
      } else {
        errorMessage = "잘못된 PIN입니다. 다시 시도해주세요.";
        dispatch("loginFailed", { error: errorMessage });
      }
    } catch (error) {
      console.error("PIN 인증 오류:", error);
      errorMessage =
        typeof error === "string" ? error : "인증 처리 중 오류가 발생했습니다.";
      dispatch("loginFailed", { error: errorMessage });
    } finally {
      isLoading = false;
    }
  }

  // 윈도우 제어 함수들
  async function minimizeWindow() {
    try {
      const window = getCurrentWindow();
      await window.minimize();
    } catch (error) {
      console.error("창 최소화 오류:", error);
    }
  }

  async function maximizeWindow() {
    try {
      const window = getCurrentWindow();
      await window.toggleMaximize();
    } catch (error) {
      console.error("창 최대화 오류:", error);
    }
  }

  async function closeWindow() {
    try {
      const window = getCurrentWindow();
      await window.close();
    } catch (error) {
      console.error("창 닫기 오류:", error);
    }
  }

  // Enter 키 처리
  function handleKeydown(event: KeyboardEvent) {
    if (event.key === "Enter" && !isLoading) {
      handleLogin();
    }
    if (event.key === "Escape") {
      pin = "";
      errorMessage = "";
    }
  }

  // PIN 입력 변경 시 에러 메시지 초기화
  $: if (pin) {
    errorMessage = "";
  }
</script>

<svelte:window on:keydown={handleKeydown} />

<div class="login-screen">
  <!-- 파란색 헤더 (30% = 180px) -->
  <div class="header" data-tauri-drag-region>
    <!-- 타이틀바 버튼들 -->
    <div class="titlebar-buttons">
      <button
        class="titlebar-button minimize"
        on:click={minimizeWindow}
        title="최소화"
        type="button"
      >
        <svg width="10" height="10" viewBox="0 0 10 10">
          <path d="M0,5 L10,5" stroke="currentColor" stroke-width="1" />
        </svg>
      </button>
      <button
        class="titlebar-button close"
        on:click={closeWindow}
        title="닫기"
        type="button"
      >
        <svg width="10" height="10" viewBox="0 0 10 10">
          <path
            d="M0,0 L10,10 M0,10 L10,0"
            stroke="currentColor"
            stroke-width="1"
          />
        </svg>
      </button>
    </div>

    <!-- 헤더 콘텐츠 -->
    <div class="header-content">
      <div class="lock-icon">🔒</div>
      <h1 class="app-title">SecureVault</h1>
      <p class="app-subtitle">보안 파일 매니저</p>
    </div>
  </div>

  <!-- 흰색 로그인 폼 (70% = 420px) -->
  <div class="form-area">
    <div class="form-content">
      <!-- 볼트 정보 -->
      <div class="vault-info">
        <div class="vault-icon">📁</div>
        <div class="vault-details">
          <h3>볼트 접근</h3>
          <p>PIN을 입력하여 보안 볼트에 접근하세요</p>
        </div>
      </div>

      <!-- 새 PIN 설정 모드 (복구 성공 후) -->
      {#if showNewPin}
        <div class="reset-pin-form">
          <div class="input-group">
            <label for="new-pin">새 PIN (4~8자리 숫자)</label>
            <div class="input-wrapper">
              <input
                id="new-pin"
                type="password"
                bind:value={newPin}
                placeholder="새 PIN"
                maxlength="8"
                inputmode="numeric"
                pattern="[0-9]*"
              />
            </div>
          </div>
          <div class="input-group">
            <label for="confirm-pin">PIN 확인</label>
            <div class="input-wrapper">
              <input
                id="confirm-pin"
                type="password"
                bind:value={confirmPin}
                placeholder="PIN 확인"
                maxlength="8"
                inputmode="numeric"
                pattern="[0-9]*"
              />
            </div>
            {#if errorMessage}
              <div class="error-message">{errorMessage}</div>
            {/if}
          </div>

          <button
            type="button"
            class="login-button recovery-btn"
            disabled={isLoading || newPin.length < 4}
            on:click={handleResetPin}
          >
            {isLoading ? "설정 중..." : "PIN 재설정 완료"}
          </button>
        </div>

        <!-- 일반 로그인 모드 -->
      {:else if !isRecoveryMode}
        <!-- PIN 입력 폼 -->
        <form on:submit|preventDefault={handleLogin}>
          <!-- PIN 입력 -->
          <div class="input-group">
            <label for="pin">PIN</label>
            <div class="input-wrapper">
              <input
                id="pin"
                type={showPassword ? "text" : "password"}
                bind:value={pin}
                placeholder="PIN을 입력하세요"
                maxlength="8"
                disabled={isLoading}
                autocomplete="current-password"
                inputmode="numeric"
                pattern="[0-9]*"
              />
              <button
                type="button"
                class="password-toggle"
                on:click={() => (showPassword = !showPassword)}
                title={showPassword ? "PIN 숨기기" : "PIN 보기"}
                disabled={isLoading}
              >
                {showPassword ? "👁️" : "👁️‍🗨️"}
              </button>
            </div>
            {#if errorMessage}
              <div class="error-message">{errorMessage}</div>
            {/if}
          </div>

          <!-- 로그인 버튼 -->
          <button
            type="submit"
            class="login-button"
            disabled={isLoading || pin.length < 4}
          >
            {isLoading ? "로그인 중..." : "로그인"}
          </button>
        </form>

        <!-- 복구 키 옵션 -->
        <div class="recovery-option">
          <button
            type="button"
            class="recovery-link"
            disabled={isLoading}
            on:click={() => {
              isRecoveryMode = true;
              errorMessage = "";
              pin = "";
            }}
          >
            PIN을 잊으셨나요? 복구 키 사용하기
          </button>
        </div>

        <!-- 복구 모드 -->
      {:else}
        <div class="recovery-form">
          <div class="input-group">
            <label for="recovery-key">복구 키</label>
            <textarea
              id="recovery-key"
              bind:value={recoveryKey}
              placeholder="32자리 복구 키를 입력하세요"
              disabled={isLoading}
              rows="3"
            ></textarea>
            {#if errorMessage}
              <div class="error-message">{errorMessage}</div>
            {/if}
          </div>

          <button
            type="button"
            class="login-button recovery-btn"
            disabled={isLoading || !recoveryKey}
            on:click={handleRecovery}
          >
            {isLoading ? "확인 중..." : "복구 및 PIN 재설정"}
          </button>

          <button
            type="button"
            class="back-link"
            disabled={isLoading}
            on:click={() => {
              isRecoveryMode = false;
              errorMessage = "";
              recoveryKey = "";
            }}
          >
            취소하고 돌아가기
          </button>
        </div>
      {/if}
    </div>
  </div>
</div>

<style>
  /* 전역 스타일 초기화 */
  :global(html, body) {
    margin: 0 !important;
    padding: 0 !important;
    overflow: hidden !important;
    height: 100vh !important;
    width: 100vw !important;
  }

  :global(*) {
    box-sizing: border-box !important;
  }

  :global(::-webkit-scrollbar) {
    display: none !important;
  }

  /* 로그인 화면 컨테이너 */
  .login-screen {
    position: fixed;
    top: 0;
    left: 0;
    width: 100vw;
    height: 100vh;
    display: flex;
    flex-direction: column;
    font-family:
      -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
    margin: 0;
    padding: 0;
    overflow: hidden;
  }

  /* 파란색 헤더 (30%) */
  .header {
    background: linear-gradient(135deg, #4f7cff 0%, #6b5fff 100%);
    width: 100%;
    height: 180px; /* 30% of 600px */
    position: relative;
    display: flex;
    align-items: center;
    justify-content: center;
    color: white;
  }

  /* 타이틀바 버튼들 */
  .titlebar-buttons {
    position: absolute;
    top: 12px;
    right: 12px;
    display: flex;
    gap: 8px;
    z-index: 10;
  }

  .titlebar-button {
    width: 28px;
    height: 28px;
    border: none;
    border-radius: 6px;
    background: rgba(255, 255, 255, 0.1);
    color: white;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: background-color 0.2s;
  }

  .titlebar-button:hover {
    background: rgba(255, 255, 255, 0.2);
  }

  .titlebar-button.close:hover {
    background: #ff5f57;
  }

  /* 헤더 콘텐츠 */
  .header-content {
    text-align: center;
  }

  .lock-icon {
    font-size: 48px;
    margin-bottom: 8px;
  }

  .app-title {
    font-size: 28px;
    font-weight: 600;
    margin: 0 0 4px 0;
    letter-spacing: -0.5px;
  }

  .app-subtitle {
    font-size: 14px;
    margin: 0;
    opacity: 0.9;
    font-weight: 400;
  }

  /* 흰색 폼 영역 (70%) */
  .form-area {
    background: white;
    width: 100%;
    height: 420px; /* 70% of 600px */
    padding: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    user-select: none;
    -webkit-user-drag: none;
  }

  .form-content {
    width: 100%;
    max-width: 360px;
    padding: 0 32px;
  }

  /* 볼트 정보 */
  .vault-info {
    display: flex;
    align-items: center;
    gap: 12px;
    margin-bottom: 32px;
    padding: 16px;
    background: #f8f9fa;
    border-radius: 12px;
    border: 1px solid #e9ecef;
  }

  .vault-icon {
    font-size: 24px;
  }

  .vault-details h3 {
    margin: 0 0 4px 0;
    font-size: 16px;
    font-weight: 600;
    color: #212529;
  }

  .vault-details p {
    margin: 0;
    font-size: 14px;
    color: #6c757d;
  }

  /* 입력 그룹 */
  .input-group {
    margin-bottom: 24px;
  }

  .input-group label {
    display: block;
    margin-bottom: 8px;
    font-size: 14px;
    font-weight: 500;
    color: #374151;
  }

  .input-wrapper {
    position: relative;
  }

  .input-wrapper input {
    width: 100%;
    height: 48px;
    padding: 0 48px 0 16px;
    border: 2px solid #e5e7eb;
    border-radius: 8px;
    font-size: 16px;
    background: white;
    transition: border-color 0.2s;
  }

  .input-wrapper input:focus {
    outline: none;
    border-color: #4f7cff;
    box-shadow: 0 0 0 3px rgba(79, 124, 255, 0.1);
  }

  .input-wrapper input:disabled {
    background: #f9fafb;
    color: #9ca3af;
  }

  .password-toggle {
    position: absolute;
    right: 12px;
    top: 50%;
    transform: translateY(-50%);
    background: none;
    border: none;
    cursor: pointer;
    font-size: 16px;
    padding: 4px;
    border-radius: 4px;
    transition: background-color 0.2s;
  }

  .password-toggle:hover:not(:disabled) {
    background: #f3f4f6;
  }

  .password-toggle:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  /* 에러 메시지 */
  .error-message {
    margin-top: 8px;
    font-size: 14px;
    color: #dc2626;
    font-weight: 500;
  }

  /* 로그인 버튼 */
  .login-button {
    width: 100%;
    height: 48px;
    background: linear-gradient(135deg, #4f7cff 0%, #6b5fff 100%);
    color: white;
    border: none;
    border-radius: 8px;
    font-size: 16px;
    font-weight: 600;
    cursor: pointer;
    transition:
      transform 0.2s,
      box-shadow 0.2s;
    margin-bottom: 16px;
  }

  .login-button:hover:not(:disabled) {
    transform: translateY(-1px);
    box-shadow: 0 4px 12px rgba(79, 124, 255, 0.3);
  }

  .login-button:active:not(:disabled) {
    transform: translateY(0);
  }

  .login-button:disabled {
    opacity: 0.6;
    cursor: not-allowed;
    transform: none;
  }

  /* 복구 옵션 */
  .recovery-option {
    text-align: center;
  }

  .recovery-link {
    background: none;
    border: none;
    color: #4f7cff;
    font-size: 14px;
    cursor: pointer;
    text-decoration: underline;
    padding: 8px;
    border-radius: 4px;
    transition: background-color 0.2s;
  }

  .recovery-link:hover:not(:disabled) {
    background: #f0f4ff;
  }

  .recovery-link:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  /* 복구 키 입력창 스타일 */
  .input-wrapper textarea {
    width: 100%;
    padding: 12px 16px;
    border: 2px solid #e5e7eb;
    border-radius: 8px;
    font-size: 14px;
    background: white;
    resize: none;
    font-family: monospace;
    transition: border-color 0.2s;
  }

  .input-wrapper textarea:focus {
    outline: none;
    border-color: #4f7cff;
    box-shadow: 0 0 0 3px rgba(79, 124, 255, 0.1);
  }

  .input-wrapper textarea:disabled {
    background: #f9fafb;
    color: #9ca3af;
  }

  /* 뒤로가기 링크 */
  .back-link {
    width: 100%;
    margin-top: 8px;
    background: none;
    border: none;
    color: #6c757d;
    font-size: 14px;
    cursor: pointer;
    text-decoration: underline;
    padding: 8px;
  }

  .back-link:hover:not(:disabled) {
    color: #343a40;
  }
</style>
