<script lang="ts">
    import { createEventDispatcher, onMount } from "svelte";
    import { invoke } from "@tauri-apps/api/core";
    import { addToast } from "$lib/stores/toast";

    export let show = false;

    const dispatch = createEventDispatcher();

    let activeTab: "general" | "pin" | "recovery" = "general";
    let isLoading = false;

    // 자동 로그아웃 상태
    let autoLogoutTime = 3600; // 기본 1시간

    onMount(() => {
        console.log("SettingsModal mounted. Show:", show);
        if (show) {
            loadAutoLogoutTime();
        }
    });

    async function loadAutoLogoutTime() {
        try {
            console.log("Fetching auto logout time...");
            const time = await invoke<number>("get_auto_logout_time");
            console.log("Fetched auto logout time:", time);
            autoLogoutTime = time;
        } catch (error) {
            console.error("자동 로그아웃 시간 로드 실패:", error);
        }
    }

    const autoLogoutOptions = [
        { value: 60, label: "1분" },
        { value: 300, label: "5분" },
        { value: 600, label: "10분" },
        { value: 1800, label: "30분" },
        { value: 3600, label: "1시간" },
        { value: 7200, label: "2시간" },
    ];

    // PIN 변경 상태
    let oldPin = "";
    let newPin = "";
    let confirmNewPin = "";

    // 복구 키 상태
    let currentPinForRecovery = "";
    let recoveryKeyInfo: any = null;
    let showRecoveryKey = false; // 실제 키 표시 여부
    let recoveryKeyFull = ""; // 받아온 전체 키

    async function loadRecoveryInfo() {
        try {
            recoveryKeyInfo = await invoke("get_recovery_key_info");
        } catch (error) {
            console.error("복구 키 정보 로드 실패:", error);
        }
    }

    async function handleAutoLogoutChange() {
        try {
            await invoke("set_auto_logout_time", {
                seconds: parseInt(autoLogoutTime.toString()),
            });
            addToast({
                type: "success",
                message: "자동 로그아웃 시간이 변경되었습니다.",
            });
            dispatch("sessionTimeUpdated");
        } catch (error) {
            addToast({ type: "error", message: "설정 변경 실패" });
            console.error(error);
        }
    }

    // 모달 열릴 때 초기화
    $: if (show) {
        activeTab = "general"; // 기본 탭으로 리셋
        oldPin = "";
        newPin = "";
        confirmNewPin = "";
        currentPinForRecovery = "";
        showRecoveryKey = false;
        loadRecoveryInfo();
        loadAutoLogoutTime();
        console.log("SettingsModal initialized. Loading auto logout time...");
    }

    function close() {
        dispatch("close");
    }

    async function handleChangePin() {
        if (newPin.length < 4 || newPin.length > 8) {
            addToast({
                type: "error",
                message: "새 PIN은 4~8자리여야 합니다.",
            });
            return;
        }
        if (newPin !== confirmNewPin) {
            addToast({ type: "error", message: "새 PIN이 일치하지 않습니다." });
            return;
        }

        isLoading = true;
        try {
            await invoke("change_pin", {
                oldPin,
                newPin,
                complexity: "basic",
            });
            addToast({
                type: "success",
                message: "PIN이 성공적으로 변경되었습니다.",
            });
            oldPin = "";
            newPin = "";
            confirmNewPin = "";
        } catch (error) {
            addToast({
                type: "error",
                message: typeof error === "string" ? error : "PIN 변경 실패",
            });
        } finally {
            isLoading = false;
        }
    }

    async function handleViewRecoveryKey() {
        if (!currentPinForRecovery) {
            addToast({ type: "error", message: "PIN을 입력해주세요." });
            return;
        }

        isLoading = true;
        try {
            // PIN 검증 먼저 수행 (보안)
            const isValid = await invoke("authenticate_pin", {
                pin: currentPinForRecovery,
            });
            if (!isValid) {
                throw "잘못된 PIN입니다.";
            }

            // 새 복구 키 생성 (기존 키를 보여주는 건 보안상 위험하거나 불가능할 수 있음 - 정책에 따라 다름)
            // 요구사항: "View Recovery Key" -> 보통은 생성된 키를 다시 보여주진 않음 (해시만 저장되므로).
            // 하지만 여기서는 "재생성" 기능을 제공하는 것이 일반적임.
            // 혹은 "현재 키 상태 확인" + "재설정"

            // *수정*: 사용자가 "설정" 기능을 원하므로, 복구 키를 새로 발급받는 기능을 제공.
            const newKey = await invoke<string>("generate_recovery_key");
            recoveryKeyFull = newKey;
            showRecoveryKey = true;
            currentPinForRecovery = ""; // PIN 클리어

            // 저장 로직 (백엔드에서 generate_recovery_key가 저장을 수행하는지 확인 필요 -> 확인 결과: 안함. generate_recovery_key는 생성만 함)
            // auth.rs의 generate_new_recovery_key가 저장까지 수행함.

            recoveryKeyFull = await invoke<string>("generate_new_recovery_key");
            showRecoveryKey = true;
            loadRecoveryInfo();
        } catch (error) {
            addToast({
                type: "error",
                message:
                    typeof error === "string" ? error : "복구 키 조회 실패",
            });
        } finally {
            isLoading = false;
        }
    }

    function copyToClipboard(text: string) {
        navigator.clipboard.writeText(text);
        addToast({ type: "success", message: "복사되었습니다." });
    }
</script>

{#if show}
    <div class="modal-overlay" on:click={close}>
        <div class="modal-content" on:click|stopPropagation>
            <div class="modal-header">
                <h2>설정</h2>
                <button class="close-btn" on:click={close}>&times;</button>
            </div>

            <div class="modal-body">
                <div class="tabs">
                    <button
                        class:active={activeTab === "general"}
                        on:click={() => (activeTab = "general")}>일반</button
                    >
                    <button
                        class:active={activeTab === "pin"}
                        on:click={() => (activeTab = "pin")}>PIN 변경</button
                    >
                    <button
                        class:active={activeTab === "recovery"}
                        on:click={() => (activeTab = "recovery")}
                        >복구 키 관리</button
                    >
                </div>

                <div class="tab-content">
                    {#if activeTab === "general"}
                        <div class="form-group">
                            <label>자동 로그아웃 시간</label>
                            <select bind:value={autoLogoutTime}>
                                {#each autoLogoutOptions as option}
                                    <option
                                        value={option.value}
                                        selected={autoLogoutTime ===
                                            option.value}>{option.label}</option
                                    >
                                {/each}
                            </select>
                            <p class="help-text">
                                설정된 시간 동안 활동이 없으면 자동으로
                                로그아웃됩니다.
                            </p>
                            <button
                                class="action-btn primary"
                                on:click={handleAutoLogoutChange}
                            >
                                적용
                            </button>
                        </div>
                    {:else if activeTab === "pin"}
                        <div class="form-group">
                            <label>현재 PIN</label>
                            <input
                                type="password"
                                bind:value={oldPin}
                                placeholder="현재 PIN 입력"
                                maxlength="8"
                                disabled={isLoading}
                            />
                        </div>
                        <div class="form-group">
                            <label>새 PIN (4~8자리)</label>
                            <input
                                type="password"
                                bind:value={newPin}
                                placeholder="새 PIN 입력"
                                maxlength="8"
                                disabled={isLoading}
                            />
                        </div>
                        <div class="form-group">
                            <label>새 PIN 확인</label>
                            <input
                                type="password"
                                bind:value={confirmNewPin}
                                placeholder="새 PIN 다시 입력"
                                maxlength="8"
                                disabled={isLoading}
                            />
                        </div>
                        <button
                            class="action-btn primary"
                            disabled={isLoading}
                            on:click={handleChangePin}
                        >
                            {#if isLoading}변경 중...{:else}PIN 변경하기{/if}
                        </button>
                    {:else if activeTab === "recovery"}
                        <div class="recovery-section">
                            <div class="status-box">
                                <span class="label">현재 상태:</span>
                                <span
                                    class="status {recoveryKeyInfo
                                        ? 'active'
                                        : 'inactive'}"
                                >
                                    {recoveryKeyInfo ? "설정됨" : "미설정"}
                                </span>
                                {#if recoveryKeyInfo}
                                    <span class="date"
                                        >({new Date(
                                            recoveryKeyInfo.created_at * 1000,
                                        ).toLocaleDateString()})</span
                                    >
                                {/if}
                            </div>

                            {#if !showRecoveryKey}
                                <div class="generate-section">
                                    <p class="warning-text">
                                        ⚠️ 복구 키를 새로 생성하면 기존 키는
                                        무효화됩니다.
                                    </p>
                                    <label
                                        >본인 확인을 위해 PIN을 입력하세요</label
                                    >
                                    <input
                                        type="password"
                                        bind:value={currentPinForRecovery}
                                        placeholder="PIN 입력"
                                        maxlength="8"
                                    />
                                    <button
                                        class="action-btn warning"
                                        disabled={!currentPinForRecovery ||
                                            isLoading}
                                        on:click={handleViewRecoveryKey}
                                    >
                                        복구 키 새로 발급
                                    </button>
                                </div>
                            {:else}
                                <div class="key-display">
                                    <p class="success-text">
                                        새 복구 키가 발급되었습니다. 안전한 곳에
                                        보관하세요!
                                    </p>
                                    <div class="key-box">
                                        <textarea
                                            readonly
                                            value={recoveryKeyFull}
                                        ></textarea>
                                        <button
                                            class="copy-btn"
                                            on:click={() =>
                                                copyToClipboard(
                                                    recoveryKeyFull,
                                                )}>복사</button
                                        >
                                    </div>
                                    <button
                                        class="action-btn secondary"
                                        on:click={() =>
                                            (showRecoveryKey = false)}
                                        >닫기</button
                                    >
                                </div>
                            {/if}
                        </div>
                    {/if}
                </div>
            </div>
        </div>
    </div>
{/if}

<style>
    .modal-overlay {
        position: fixed;
        top: 0;
        left: 0;
        width: 100%;
        height: 100%;
        background: rgba(0, 0, 0, 0.5);
        display: flex;
        justify-content: center;
        align-items: center;
        z-index: 1000;
    }

    .modal-content {
        background: white;
        width: 90%;
        max-width: 400px;
        border-radius: 12px;
        box-shadow: 0 4px 20px rgba(0, 0, 0, 0.15);
        overflow: hidden;
    }

    .modal-header {
        padding: 16px 20px;
        border-bottom: 1px solid #eee;
        display: flex;
        justify-content: space-between;
        align-items: center;
    }

    .modal-header h2 {
        margin: 0;
        font-size: 18px;
        font-weight: 600;
    }

    .close-btn {
        background: none;
        border: none;
        font-size: 24px;
        cursor: pointer;
        color: #666;
    }

    .modal-body {
        padding: 20px;
    }

    .tabs {
        display: flex;
        border-bottom: 2px solid #eee;
        margin-bottom: 20px;
    }

    .tabs button {
        flex: 1;
        background: none;
        border: none;
        padding: 10px;
        font-size: 14px;
        font-weight: 500;
        color: #888;
        cursor: pointer;
        border-bottom: 2px solid transparent;
        margin-bottom: -2px;
    }

    .tabs button.active {
        color: #4f7cff;
        border-bottom-color: #4f7cff;
    }

    .form-group {
        margin-bottom: 16px;
    }

    .form-group label {
        display: block;
        margin-bottom: 6px;
        font-size: 13px;
        color: #555;
        font-weight: 500;
    }

    input[type="password"],
    select {
        width: 100%;
        padding: 10px;
        border: 1px solid #ddd;
        border-radius: 6px;
        font-size: 14px;
        background: white;
    }

    .help-text {
        font-size: 12px;
        color: #888;
        margin-top: 6px;
    }

    .action-btn {
        width: 100%;
        padding: 12px;
        border: none;
        border-radius: 6px;
        font-size: 14px;
        font-weight: 600;
        cursor: pointer;
        margin-top: 10px;
    }

    .action-btn.primary {
        background: #4f7cff;
        color: white;
    }

    .action-btn.warning {
        background: #ff9800;
        color: white;
    }

    .action-btn.secondary {
        background: #f1f3f5;
        color: #333;
    }

    .action-btn:disabled {
        opacity: 0.6;
        cursor: not-allowed;
    }

    .status-box {
        background: #f8f9fa;
        padding: 12px;
        border-radius: 8px;
        margin-bottom: 20px;
        text-align: center;
        font-size: 14px;
    }

    .status.active {
        color: #28a745;
        font-weight: bold;
    }
    .status.inactive {
        color: #dc3545;
        font-weight: bold;
    }

    .warning-text {
        color: #ff9800;
        font-size: 13px;
        margin: 0 0 10px 0;
    }
    .success-text {
        color: #28a745;
        font-size: 13px;
        margin: 0 0 10px 0;
        font-weight: 600;
    }

    .key-box {
        position: relative;
        margin-bottom: 16px;
    }

    textarea {
        width: 100%;
        height: 80px;
        padding: 10px;
        border: 1px solid #ddd;
        border-radius: 6px;
        background: #f8f9fa;
        resize: none;
        font-family: monospace;
        font-size: 13px;
        color: #333;
    }

    .copy-btn {
        position: absolute;
        bottom: 8px;
        right: 8px;
        background: white;
        border: 1px solid #ddd;
        padding: 4px 8px;
        border-radius: 4px;
        font-size: 12px;
        cursor: pointer;
    }
</style>
