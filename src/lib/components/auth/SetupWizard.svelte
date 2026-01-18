<script lang="ts">
    import { createEventDispatcher } from "svelte";
    import { invoke } from "@tauri-apps/api/core";
    import { fade, slide } from "svelte/transition";

    const dispatch = createEventDispatcher<{
        setupComplete: void;
    }>();

    let step = 1; // 1: Welcome, 2: PIN Setup, 3: Recovery Key, 4: Finish
    let pin = "";
    let confirmPin = "";
    let recoveryKey = "";
    let isLoading = false;
    let errorMessage = "";
    let showPassword = false;

    async function handleSetPin() {
        if (pin.length < 4) {
            errorMessage = "PIN은 4자리 이상이어야 합니다.";
            return;
        }
        if (pin !== confirmPin) {
            errorMessage = "PIN이 일치하지 않습니다.";
            return;
        }
        if (!/^\d+$/.test(pin)) {
            errorMessage = "PIN은 숫자만 포함해야 합니다.";
            return;
        }

        isLoading = true;
        errorMessage = "";

        try {
            // PIN 설정 (중간 복잡도)
            await invoke("set_pin_code", { pin, complexity: "medium" });
            step = 3; // 다음 단계로 이동
        } catch (error) {
            console.error("PIN 설정 실패:", error);
            errorMessage =
                typeof error === "string"
                    ? error
                    : "PIN 설정 중 오류가 발생했습니다.";
        } finally {
            isLoading = false;
        }
    }

    async function handleGenerateRecoveryKey() {
        isLoading = true;
        errorMessage = "";

        try {
            // 복구 키 생성
            const key = await invoke("generate_new_recovery_key");
            recoveryKey = key as string;
        } catch (error) {
            console.error("복구 키 생성 실패:", error);
            errorMessage =
                typeof error === "string"
                    ? error
                    : "복구 키 생성 중 오류가 발생했습니다.";
        } finally {
            isLoading = false;
        }
    }

    function handleComplete() {
        dispatch("setupComplete");
    }

    function copyRecoveryKey() {
        if (recoveryKey) {
            navigator.clipboard.writeText(recoveryKey);
            alert("복구 키가 클립보드에 복사되었습니다.");
        }
    }
</script>

<div class="setup-wizard">
    <div class="wizard-container">
        <div class="header">
            <div class="logo">🔒</div>
            <h1>SecureVault 시작하기</h1>
            <div class="progress-bar">
                <div class="step {step >= 1 ? 'active' : ''}">1</div>
                <div class="line {step >= 2 ? 'active' : ''}"></div>
                <div class="step {step >= 2 ? 'active' : ''}">2</div>
                <div class="line {step >= 3 ? 'active' : ''}"></div>
                <div class="step {step >= 3 ? 'active' : ''}">3</div>
                <div class="line {step >= 4 ? 'active' : ''}"></div>
                <div class="step {step >= 4 ? 'active' : ''}">4</div>
            </div>
        </div>

        <div class="content">
            {#if step === 1}
                <div class="step-content" in:fade>
                    <h2>환영합니다!</h2>
                    <p>
                        SecureVault는 개인 파일을 안전하게 보호하는 암호화
                        금고입니다.<br />
                        시작하기 전에 몇 가지 보안 설정을 진행합니다.
                    </p>
                    <div class="features">
                        <div class="feature-item">
                            <span class="icon">🛡️</span>
                            <span>강력한 AES-256 암호화</span>
                        </div>
                        <div class="feature-item">
                            <span class="icon">🔑</span>
                            <span>PIN 기반 간편 접근</span>
                        </div>
                        <div class="feature-item">
                            <span class="icon">💾</span>
                            <span>로컬 저장소 전용 (클라우드 없음)</span>
                        </div>
                    </div>
                    <button class="primary-button" on:click={() => (step = 2)}>
                        시작하기
                    </button>
                </div>
            {:else if step === 2}
                <div class="step-content" in:slide>
                    <h2>PIN 설정</h2>
                    <p>
                        앱 잠금을 해제할 때 사용할 PIN 번호를 설정해주세요.
                        (4-8자리 숫자)
                    </p>

                    <div class="input-group">
                        <label for="pin">PIN 입력</label>
                        <div class="password-wrapper">
                            <input
                                id="pin"
                                type={showPassword ? "text" : "password"}
                                bind:value={pin}
                                maxlength="8"
                                placeholder="4-8자리 숫자"
                                inputmode="numeric"
                            />
                            <button
                                class="toggle-visibility"
                                on:click={() => (showPassword = !showPassword)}
                            >
                                {showPassword ? "숨기기" : "보기"}
                            </button>
                        </div>
                    </div>

                    <div class="input-group">
                        <label for="confirm-pin">PIN 확인</label>
                        <input
                            id="confirm-pin"
                            type={showPassword ? "text" : "password"}
                            bind:value={confirmPin}
                            maxlength="8"
                            placeholder="PIN 다시 입력"
                            inputmode="numeric"
                        />
                    </div>

                    {#if errorMessage}
                        <p class="error">{errorMessage}</p>
                    {/if}

                    <button
                        class="primary-button"
                        disabled={isLoading || !pin || !confirmPin}
                        on:click={handleSetPin}
                    >
                        {isLoading ? "설정 중..." : "다음"}
                    </button>
                </div>
            {:else if step === 3}
                <div class="step-content" in:slide>
                    <h2>복구 키 발급</h2>
                    <p>
                        PIN을 잊어버렸을 때를 대비해 복구 키를 발급받으세요.<br
                        />
                        <strong
                            >이 키는 다시 조회할 수 없으므로 안전한 곳에
                            보관하세요.</strong
                        >
                    </p>

                    {#if !recoveryKey}
                        <div class="generate-section">
                            <p>복구 키를 생성하려면 아래 버튼을 누르세요.</p>
                            <button
                                class="secondary-button"
                                on:click={handleGenerateRecoveryKey}
                                disabled={isLoading}
                            >
                                {isLoading ? "생성 중..." : "복구 키 생성"}
                            </button>
                        </div>
                    {:else}
                        <div class="key-display">
                            <code class="recovery-key">{recoveryKey}</code>
                            <button
                                class="copy-button"
                                on:click={copyRecoveryKey}
                            >
                                복사하기
                            </button>
                        </div>
                        <p class="warning">
                            ⚠️ 주의: 이 화면을 벗어나면 복구 키를 다시 볼 수
                            없습니다.
                        </p>
                        <button
                            class="primary-button"
                            on:click={() => (step = 4)}
                        >
                            저장했습니다 (다음)
                        </button>
                    {/if}
                    {#if errorMessage}
                        <p class="error">{errorMessage}</p>
                    {/if}
                </div>
            {:else if step === 4}
                <div class="step-content" in:fade>
                    <h2>설정 완료!</h2>
                    <div class="success-icon">🎉</div>
                    <p>
                        모든 설정이 완료되었습니다.<br />이제 파일을 안전하게
                        보관할 준비가 되었습니다.
                    </p>
                    <button class="primary-button" on:click={handleComplete}>
                        앱 시작하기
                    </button>
                </div>
            {/if}
        </div>
    </div>
</div>

<style>
    .setup-wizard {
        position: fixed;
        top: 0;
        left: 0;
        width: 100vw;
        height: 100vh;
        background: linear-gradient(135deg, #f5f7fa 0%, #c3cfe2 100%);
        display: flex;
        justify-content: center;
        align-items: center;
        font-family:
            -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
    }

    .wizard-container {
        background: white;
        width: 400px;
        padding: 40px;
        border-radius: 16px;
        box-shadow: 0 10px 25px rgba(0, 0, 0, 0.1);
        display: flex;
        flex-direction: column;
        align-items: center;
    }

    .header {
        text-align: center;
        margin-bottom: 30px;
        width: 100%;
    }

    .logo {
        font-size: 48px;
        margin-bottom: 10px;
    }

    h1 {
        font-size: 24px;
        margin: 0 0 20px 0;
        color: #333;
    }

    .progress-bar {
        display: flex;
        align-items: center;
        justify-content: center;
        gap: 5px;
    }

    .step {
        width: 24px;
        height: 24px;
        border-radius: 50%;
        background: #e0e0e0;
        color: #fff;
        font-size: 12px;
        display: flex;
        align-items: center;
        justify-content: center;
        font-weight: bold;
    }

    .step.active {
        background: #4f7cff;
    }

    .line {
        width: 30px;
        height: 2px;
        background: #e0e0e0;
    }

    .line.active {
        background: #4f7cff;
    }

    .content {
        width: 100%;
        min-height: 250px; /* 고정 높이 확보 */
        display: flex;
        flex-direction: column;
        justify-content: flex-start; /* 상단 정렬 */
    }

    .step-content {
        display: flex;
        flex-direction: column;
        align-items: center;
        text-align: center;
        width: 100%;
    }

    h2 {
        margin: 0 0 10px 0;
        font-size: 20px;
        color: #333;
    }

    p {
        font-size: 14px;
        color: #666;
        margin-bottom: 24px;
        line-height: 1.5;
    }

    .features {
        display: flex;
        flex-direction: column;
        gap: 12px;
        margin-bottom: 30px;
        align-items: flex-start;
        width: 100%;
        padding: 0 20px;
    }

    .feature-item {
        display: flex;
        align-items: center;
        gap: 10px;
        font-size: 14px;
        color: #444;
    }

    .input-group {
        width: 100%;
        margin-bottom: 16px;
        text-align: left;
    }

    label {
        display: block;
        font-size: 13px;
        font-weight: 600;
        color: #444;
        margin-bottom: 6px;
    }

    input {
        width: 100%;
        padding: 12px;
        border: 1px solid #ddd;
        border-radius: 8px;
        font-size: 16px;
    }

    input:focus {
        border-color: #4f7cff;
        outline: none;
    }

    .password-wrapper {
        position: relative;
    }

    .toggle-visibility {
        position: absolute;
        right: 10px;
        top: 50%;
        transform: translateY(-50%);
        background: none;
        border: none;
        font-size: 12px;
        color: #666;
        cursor: pointer;
        padding: 4px;
    }

    .primary-button {
        width: 100%;
        padding: 14px;
        background: #4f7cff;
        color: white;
        border: none;
        border-radius: 8px;
        font-size: 16px;
        font-weight: 600;
        cursor: pointer;
        transition: background 0.2s;
        margin-top: auto; /* 하단 배치 */
    }

    .primary-button:hover:not(:disabled) {
        background: #3a63e0;
    }

    .primary-button:disabled {
        background: #ccc;
        cursor: not-allowed;
    }

    .secondary-button {
        padding: 12px 24px;
        background: white;
        border: 1px solid #4f7cff;
        color: #4f7cff;
        border-radius: 8px;
        font-size: 14px;
        font-weight: 600;
        cursor: pointer;
    }

    .key-display {
        width: 100%;
        background: #f8f9fa;
        border: 1px dashed #ccc;
        padding: 16px;
        border-radius: 8px;
        margin-bottom: 16px;
        display: flex;
        flex-direction: column;
        gap: 10px;
    }

    .recovery-key {
        word-break: break-all;
        font-family: monospace;
        font-size: 14px;
        color: #333;
    }

    .copy-button {
        align-self: flex-end;
        font-size: 12px;
        padding: 4px 8px;
        cursor: pointer;
    }

    .warning {
        color: #e67e22;
        font-size: 13px;
        font-weight: 600;
    }

    .error {
        color: #e74c3c;
        font-size: 13px;
        margin-bottom: 10px;
    }

    .success-icon {
        font-size: 64px;
        margin-bottom: 20px;
        animation: bounce 1s infinite;
    }

    @keyframes bounce {
        0%,
        100% {
            transform: translateY(0);
        }
        50% {
            transform: translateY(-10px);
        }
    }
</style>
