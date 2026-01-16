<script lang="ts">
  // 컴포넌트 데모 페이지
  import Button from '$lib/components/common/Button.svelte';
  import Input from '$lib/components/common/Input.svelte';
  import Modal from '$lib/components/common/Modal.svelte';
  import ToastContainer from '$lib/components/common/ToastContainer.svelte';
  import { toast, toastMessages } from '$lib/stores/toast';
  import type { ButtonProps, InputProps } from '$lib/types/ui';

  // 상태 변수들
  let textInput = '';
  let passwordInput = '';
  let numberInput = 0;
  let emailInput = '';
  let inputError = '';
  let isModalOpen = false;
  let isLoadingModalOpen = false;
  let isLoading = false;

  // 버튼 클릭 핸들러들
  function handlePrimaryClick() {
    toast.success(toastMessages.system.operationCompleted);
  }

  function handleSecondaryClick() {
    toast.info('보조 버튼이 클릭되었습니다.');
  }

  function handleDangerClick() {
    toast.error(toastMessages.system.unexpectedError);
  }

  function handleSuccessClick() {
    toast.success(toastMessages.file.addSuccess);
  }

  function handleWarningClick() {
    toast.warning('경고 메시지입니다.');
  }

  // 로딩 버튼 테스트
  async function handleLoadingTest() {
    isLoading = true;
    const loadingId = toast.loading('작업을 처리하는 중입니다...');
    
    // 3초 후 완료
    setTimeout(() => {
      isLoading = false;
      toast.updateLoading(loadingId, '작업이 성공적으로 완료되었습니다!', 'success');
    }, 3000);
  }

  // 입력 검증 함수
  function validateEmail(email: string): string {
    if (!email) return '';
    const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
    return emailRegex.test(email) ? '' : '올바른 이메일 형식을 입력해주세요.';
  }

  // 이메일 입력 핸들러
  function handleEmailInput(event: Event) {
    const target = event.target as HTMLInputElement;
    emailInput = target.value;
    inputError = validateEmail(emailInput);
  }

  // 모달 핸들러들
  function openModal() {
    isModalOpen = true;
  }

  function closeModal() {
    isModalOpen = false;
  }

  function openLoadingModal() {
    isLoadingModalOpen = true;
  }

  function closeLoadingModal() {
    isLoadingModalOpen = false;
  }

  function handleModalConfirm() {
    toast.success('모달에서 확인 버튼이 클릭되었습니다.');
    closeModal();
  }

  function handleModalCancel() {
    toast.info('모달이 취소되었습니다.');
    closeModal();
  }

  // 확인 토스트 테스트
  function showConfirmToast() {
    toast.confirm(
      '정말로 이 작업을 수행하시겠습니까?',
      () => {
        toast.success('작업이 확인되었습니다.');
      },
      { title: '작업 확인' }
    );
  }

  // 모든 토스트 지우기
  function clearAllToasts() {
    toast.clear();
  }
</script>

<svelte:head>
  <title>컴포넌트 데모 - SecureVault</title>
</svelte:head>

<div class="demo-container">
  <div class="demo-header">
    <h1 class="demo-title">🎨 UI 컴포넌트 데모</h1>
    <p class="demo-description">
      SecureVault의 기본 UI 컴포넌트들을 테스트하고 확인할 수 있는 페이지입니다.
    </p>
  </div>

  <!-- 버튼 컴포넌트 데모 -->
  <section class="demo-section">
    <h2 class="section-title">🔘 버튼 컴포넌트</h2>
    
    <div class="demo-group">
      <h3 class="group-title">기본 변형</h3>
      <div class="button-grid">
        <Button variant="primary" on:click={handlePrimaryClick}>
          주요 버튼
        </Button>
        <Button variant="secondary" on:click={handleSecondaryClick}>
          보조 버튼
        </Button>
        <Button variant="danger" on:click={handleDangerClick}>
          위험 버튼
        </Button>
        <Button variant="success" on:click={handleSuccessClick}>
          성공 버튼
        </Button>
        <Button variant="outline">
          외곽선 버튼
        </Button>
      </div>
    </div>

    <div class="demo-group">
      <h3 class="group-title">크기 변형</h3>
      <div class="button-grid">
        <Button size="small">작은 버튼</Button>
        <Button size="medium">보통 버튼</Button>
        <Button size="large">큰 버튼</Button>
      </div>
    </div>

    <div class="demo-group">
      <h3 class="group-title">상태 변형</h3>
      <div class="button-grid">
        <Button loading={isLoading} on:click={handleLoadingTest}>
          {isLoading ? '처리 중...' : '로딩 테스트'}
        </Button>
        <Button disabled>비활성화됨</Button>
        <Button fullWidth>전체 너비 버튼</Button>
      </div>
    </div>
  </section>

  <!-- 입력 컴포넌트 데모 -->
  <section class="demo-section">
    <h2 class="section-title">📝 입력 컴포넌트</h2>
    
    <div class="demo-group">
      <h3 class="group-title">기본 입력 필드</h3>
      <div class="input-grid">
        <Input
          label="텍스트 입력"
          placeholder="텍스트를 입력하세요"
          bind:value={textInput}
          helperText="도움말 텍스트입니다."
        />
        
        <Input
          type="password"
          label="비밀번호 입력"
          placeholder="비밀번호를 입력하세요"
          bind:value={passwordInput}
          showPasswordToggle={true}
          required={true}
        />
        
        <Input
          type="number"
          label="숫자 입력"
          placeholder="숫자를 입력하세요"
          bind:value={numberInput}
          min={0}
          max={100}
        />
        
        <Input
          type="email"
          label="이메일 입력"
          placeholder="이메일을 입력하세요"
          bind:value={emailInput}
          error={inputError}
          on:input={handleEmailInput}
          required={true}
        />
      </div>
    </div>

    <div class="demo-group">
      <h3 class="group-title">크기 및 상태</h3>
      <div class="input-grid">
        <Input
          size="small"
          placeholder="작은 입력 필드"
          label="작은 크기"
        />
        <Input
          size="large"
          placeholder="큰 입력 필드"
          label="큰 크기"
        />
        <Input
          placeholder="비활성화된 입력 필드"
          label="비활성화"
          disabled={true}
        />
        <Input
          placeholder="읽기 전용 입력 필드"
          label="읽기 전용"
          value="읽기 전용 값"
          readonly={true}
        />
      </div>
    </div>
  </section>

  <!-- 모달 컴포넌트 데모 -->
  <section class="demo-section">
    <h2 class="section-title">🪟 모달 컴포넌트</h2>
    
    <div class="demo-group">
      <div class="button-grid">
        <Button on:click={openModal}>기본 모달 열기</Button>
        <Button on:click={openLoadingModal}>로딩 모달 열기</Button>
      </div>
    </div>
  </section>

  <!-- 토스트 컴포넌트 데모 -->
  <section class="demo-section">
    <h2 class="section-title">🍞 토스트 알림</h2>
    
    <div class="demo-group">
      <h3 class="group-title">기본 토스트</h3>
      <div class="button-grid">
        <Button variant="success" on:click={() => toast.success('성공 메시지입니다!')}>
          성공 토스트
        </Button>
        <Button variant="danger" on:click={() => toast.error('에러 메시지입니다!')}>
          에러 토스트
        </Button>
        <Button on:click={() => toast.warning('경고 메시지입니다!')}>
          경고 토스트
        </Button>
        <Button on:click={() => toast.info('정보 메시지입니다!')}>
          정보 토스트
        </Button>
      </div>
    </div>

    <div class="demo-group">
      <h3 class="group-title">고급 토스트</h3>
      <div class="button-grid">
        <Button on:click={showConfirmToast}>
          확인 토스트
        </Button>
        <Button on:click={() => toast.loading('로딩 중입니다...')}>
          로딩 토스트
        </Button>
        <Button variant="outline" on:click={clearAllToasts}>
          모든 토스트 지우기
        </Button>
      </div>
    </div>
  </section>

  <!-- 입력값 표시 -->
  <section class="demo-section">
    <h2 class="section-title">📊 현재 입력값</h2>
    <div class="values-display">
      <div class="value-item">
        <strong>텍스트:</strong> {textInput || '(비어있음)'}
      </div>
      <div class="value-item">
        <strong>비밀번호:</strong> {passwordInput ? '●'.repeat(passwordInput.length) : '(비어있음)'}
      </div>
      <div class="value-item">
        <strong>숫자:</strong> {numberInput}
      </div>
      <div class="value-item">
        <strong>이메일:</strong> {emailInput || '(비어있음)'}
        {#if inputError}
          <span class="error-text">- {inputError}</span>
        {/if}
      </div>
    </div>
  </section>
</div>

<!-- 모달들 -->
<Modal
  bind:isOpen={isModalOpen}
  title="기본 모달"
  size="medium"
  on:close={closeModal}
  on:confirm={handleModalConfirm}
  on:cancel={handleModalCancel}
>
  <p>이것은 기본 모달의 내용입니다.</p>
  <p>확인 또는 취소 버튼을 클릭하거나 ESC 키를 눌러 닫을 수 있습니다.</p>
</Modal>

<Modal
  bind:isOpen={isLoadingModalOpen}
  title="로딩 모달"
  size="small"
  persistent={true}
  closable={false}
  showFooter={false}
  on:close={closeLoadingModal}
>
  <div class="loading-content">
    <div class="spinner"></div>
    <p>데이터를 처리하는 중입니다...</p>
    <Button size="small" on:click={closeLoadingModal}>
      취소
    </Button>
  </div>
</Modal>

<!-- 토스트 컨테이너 -->
<ToastContainer position="top-right" maxToasts={5} />

<style>
  /* 데모 페이지 스타일 */
  .demo-container {
    @apply max-w-6xl mx-auto p-6 space-y-8;
  }

  .demo-header {
    @apply text-center mb-12;
  }

  .demo-title {
    @apply text-4xl font-bold text-gray-900 dark:text-gray-100 mb-4 text-korean;
  }

  .demo-description {
    @apply text-lg text-gray-600 dark:text-gray-400 text-korean;
  }

  .demo-section {
    @apply bg-white dark:bg-gray-800 rounded-xl shadow-lg p-8 border border-gray-200 dark:border-gray-700;
  }

  .section-title {
    @apply text-2xl font-semibold text-gray-900 dark:text-gray-100 mb-6 text-korean;
  }

  .demo-group {
    @apply mb-8 last:mb-0;
  }

  .group-title {
    @apply text-lg font-medium text-gray-800 dark:text-gray-200 mb-4 text-korean;
  }

  .button-grid {
    @apply grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-5 gap-4;
  }

  .input-grid {
    @apply grid grid-cols-1 md:grid-cols-2 gap-6;
  }

  .values-display {
    @apply bg-gray-50 dark:bg-gray-700 rounded-lg p-6 space-y-3;
  }

  .value-item {
    @apply text-sm text-gray-700 dark:text-gray-300 text-korean;
  }

  .error-text {
    @apply text-red-500 dark:text-red-400;
  }

  .loading-content {
    @apply text-center space-y-4;
  }

  .spinner {
    @apply inline-block w-8 h-8 border-4 border-blue-200 border-t-blue-500 rounded-full animate-spin mx-auto;
  }

  /* 반응형 디자인 */
  @media (max-width: 640px) {
    .demo-container {
      @apply p-4 space-y-6;
    }

    .demo-section {
      @apply p-6;
    }

    .button-grid {
      @apply grid-cols-1 gap-3;
    }

    .input-grid {
      @apply grid-cols-1 gap-4;
    }
  }

  /* 한국어 텍스트 최적화 */
  .text-korean {
    word-break: keep-all;
    line-height: 1.7;
  }

  /* 다크 모드 추가 스타일 */
  @media (prefers-color-scheme: dark) {
    .demo-section {
      @apply bg-gray-800 border-gray-700;
    }

    .values-display {
      @apply bg-gray-700;
    }
  }
</style>