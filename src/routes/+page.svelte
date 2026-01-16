<script lang="ts">
  import { goto } from '$app/navigation';
  import LoginScreen from '$lib/components/auth/LoginScreen.svelte';
  import { authState } from '$lib/stores/auth';

  // 로그인 성공 처리
  async function handleLoginSuccess(event: CustomEvent<{ pin: string }>) {
    console.log('로그인 성공:', event.detail.pin);
    
    // auth 스토어 업데이트
    authState.update(state => ({
      ...state,
      isAuthenticated: true,
      user: {
        id: 'user1',
        name: '사용자',
        lastLogin: new Date()
      }
    }));
    
    // 대시보드로 이동
    await goto('/dashboard');
  }

  // 로그인 실패 처리
  function handleLoginFailed(event: CustomEvent<{ error: string }>) {
    console.error('로그인 실패:', event.detail.error);
    // 에러는 LoginScreen 컴포넌트에서 처리됨
  }
</script>

<svelte:head>
  <title>SecureVault - 보안 파일 매니저</title>
</svelte:head>

<LoginScreen 
  on:loginSuccess={handleLoginSuccess}
  on:loginFailed={handleLoginFailed}
/>