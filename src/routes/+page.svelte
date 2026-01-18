<script lang="ts">
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import { invoke } from "@tauri-apps/api/core";
  import LoginScreen from "$lib/components/auth/LoginScreen.svelte";
  import SetupWizard from "$lib/components/auth/SetupWizard.svelte";
  import { authState } from "$lib/stores/auth";

  let isChecking = true;
  let showSetupWizard = false;

  onMount(async () => {
    try {
      // Check if PIN is set (First Run Check)
      const hasPin = await invoke("has_pin_set");
      showSetupWizard = !hasPin;
    } catch (error) {
      console.error("Failed to check PIN status:", error);
      // Fallback to LoginScreen just in case
      showSetupWizard = false;
    } finally {
      isChecking = false;
    }
  });

  async function handleLoginSuccess(event: CustomEvent<{ pin: string }>) {
    console.log("로그인 성공:", event.detail.pin);

    authState.update((state) => ({
      ...state,
      isAuthenticated: true,
      user: {
        id: "user1",
        name: "사용자",
        lastLogin: new Date(),
      },
    }));

    await goto("/dashboard");
  }

  function handleLoginFailed(event: CustomEvent<{ error: string }>) {
    console.error("로그인 실패:", event.detail.error);
  }

  function handleSetupComplete() {
    // Setup complete -> Switch to Login Screen
    showSetupWizard = false;
  }
</script>

<svelte:head>
  <title>SecureVault - 보안 파일 매니저</title>
</svelte:head>

{#if isChecking}
  <!-- Loading Splash (Optional) -->
  <div class="splash-screen">
    <div class="spinner"></div>
    <p>Loading SecureVault...</p>
  </div>
{:else if showSetupWizard}
  <SetupWizard on:setupComplete={handleSetupComplete} />
{:else}
  <LoginScreen
    on:loginSuccess={handleLoginSuccess}
    on:loginFailed={handleLoginFailed}
  />
{/if}

<style>
  .splash-screen {
    height: 100vh;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    background: linear-gradient(135deg, #f5f7fa 0%, #c3cfe2 100%);
    color: #333;
    font-family:
      -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
  }

  .spinner {
    width: 40px;
    height: 40px;
    border: 4px solid #f3f3f3;
    border-top: 4px solid #4f7cff;
    border-radius: 50%;
    animation: spin 1s linear infinite;
    margin-bottom: 20px;
  }

  @keyframes spin {
    0% {
      transform: rotate(0deg);
    }
    100% {
      transform: rotate(360deg);
    }
  }
</style>
