<script lang="ts">
  // 입력 컴포넌트 속성 정의
  interface Props {
    type?: 'text' | 'password' | 'email' | 'number' | 'tel' | 'url';
    value?: string | number;
    placeholder?: string;
    disabled?: boolean;
    readonly?: boolean;
    required?: boolean;
    autofocus?: boolean;
    error?: string;
    label?: string;
    helperText?: string;
    maxlength?: number;
    minlength?: number;
    min?: number;
    max?: number;
    step?: number;
    pattern?: string;
    autocomplete?: string;
    size?: 'small' | 'medium' | 'large';
    fullWidth?: boolean;
    leftIcon?: string;
    rightIcon?: string;
    showPasswordToggle?: boolean;
    onkeydown?: (event: KeyboardEvent) => void;
  }

  let {
    type = 'text',
    value = $bindable(''),
    placeholder = '',
    disabled = false,
    readonly = false,
    required = false,
    autofocus = false,
    error = '',
    label = '',
    helperText = '',
    maxlength,
    minlength,
    min,
    max,
    step,
    pattern,
    autocomplete,
    size = 'medium',
    fullWidth = false,
    leftIcon = '',
    rightIcon = '',
    showPasswordToggle = false,
    onkeydown
  }: Props = $props();

  // 내부 상태
  let inputElement: HTMLInputElement = $state()!;
  let showPassword = $state(false);
  let isFocused = $state(false);

  // inputElement를 외부에서 바인딩할 수 있도록 export
  export { inputElement };

  // 고유 ID 생성
  const inputId = `input-${Math.random().toString(36).substr(2, 9)}`;

  // 패스워드 표시/숨김 토글
  function togglePasswordVisibility() {
    showPassword = !showPassword;
  }

  // 포커스 이벤트 핸들러
  function handleFocus() {
    isFocused = true;
  }

  function handleBlur() {
    isFocused = false;
  }

  // 입력 이벤트 핸들러
  function handleInput(event: Event) {
    const target = event.target as HTMLInputElement;
    if (type === 'number') {
      value = target.valueAsNumber || 0;
    } else {
      value = target.value;
    }
  }

  // 동적 클래스 계산
  const inputClasses = $derived([
    'input-field',
    `input-${size}`,
    error && 'input-error',
    disabled && 'input-disabled',
    readonly && 'input-readonly',
    isFocused && 'input-focused',
    fullWidth && 'input-full-width',
    leftIcon && 'input-with-left-icon',
    (rightIcon || (type === 'password' && showPasswordToggle)) && 'input-with-right-icon'
  ].filter(Boolean).join(' '));

  const containerClasses = $derived([
    'input-container',
    fullWidth && 'input-container-full-width'
  ].filter(Boolean).join(' '));

  const actualType = $derived((type === 'password' && showPassword) ? 'text' : type);
</script>

<div class={containerClasses}>
  {#if label}
    <label for={inputId} class="input-label">
      {label}
      {#if required}
        <span class="input-required" aria-label="필수 입력">*</span>
      {/if}
    </label>
  {/if}

  <div class="input-wrapper">
    {#if leftIcon}
      <div class="input-icon input-icon-left" aria-hidden="true">
        <i class={leftIcon}></i>
      </div>
    {/if}

    <input
      bind:this={inputElement}
      id={inputId}
      type={actualType}
      class={inputClasses}
      {placeholder}
      {disabled}
      {readonly}
      {required}
      {autofocus}
      {maxlength}
      {minlength}
      {min}
      {max}
      {step}
      {pattern}
      {autocomplete}
      value={type === 'number' ? value : value}
      oninput={handleInput}
      onfocus={handleFocus}
      onblur={handleBlur}
      onkeydown={onkeydown}
      aria-invalid={error ? 'true' : 'false'}
      aria-describedby={error ? `${inputId}-error` : helperText ? `${inputId}-helper` : undefined}
    />

    {#if rightIcon || (type === 'password' && showPasswordToggle)}
      <div class="input-icon input-icon-right">
        {#if type === 'password' && showPasswordToggle}
          <button
            type="button"
            class="input-password-toggle"
            onclick={togglePasswordVisibility}
            aria-label={showPassword ? '비밀번호 숨기기' : '비밀번호 보기'}
            tabindex="-1"
          >
            {#if showPassword}
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M17.94 17.94A10.07 10.07 0 0 1 12 20c-7 0-11-8-11-8a18.45 18.45 0 0 1 5.06-5.94M9.9 4.24A9.12 9.12 0 0 1 12 4c7 0 11 8 11 8a18.5 18.5 0 0 1-2.16 3.19m-6.72-1.07a3 3 0 1 1-4.24-4.24"/>
                <line x1="1" y1="1" x2="23" y2="23"/>
              </svg>
            {:else}
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"/>
                <circle cx="12" cy="12" r="3"/>
              </svg>
            {/if}
          </button>
        {:else if rightIcon}
          <i class={rightIcon} aria-hidden="true"></i>
        {/if}
      </div>
    {/if}
  </div>

  {#if error}
    <p id="{inputId}-error" class="input-error-message" role="alert">
      {error}
    </p>
  {:else if helperText}
    <p id="{inputId}-helper" class="input-helper-text">
      {helperText}
    </p>
  {/if}
</div>

<style>
  /* 컨테이너 스타일 */
  .input-container {
    @apply flex flex-col gap-1;
  }

  .input-container-full-width {
    @apply w-full;
  }

  /* 라벨 스타일 */
  .input-label {
    @apply text-sm font-medium text-gray-700 dark:text-gray-300;
  }

  .input-required {
    @apply text-red-500 ml-1;
  }

  /* 입력 래퍼 */
  .input-wrapper {
    @apply relative flex items-center;
  }

  /* 기본 입력 필드 스타일 */
  .input-field {
    @apply w-full border border-gray-300 rounded-lg bg-white text-gray-900 transition-all duration-200;
    @apply focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500;
    @apply disabled:bg-gray-50 disabled:text-gray-500 disabled:cursor-not-allowed;
  }

  .input-readonly {
    @apply bg-gray-50 cursor-default;
  }

  /* 크기 변형 */
  .input-small {
    @apply px-3 py-1.5 text-sm;
  }

  .input-medium {
    @apply px-3 py-2 text-sm;
  }

  .input-large {
    @apply px-4 py-3 text-base;
  }

  /* 아이콘이 있는 경우 패딩 조정 */
  .input-with-left-icon.input-small {
    @apply pl-9;
  }

  .input-with-left-icon.input-medium {
    @apply pl-10;
  }

  .input-with-left-icon.input-large {
    @apply pl-12;
  }

  .input-with-right-icon.input-small {
    @apply pr-9;
  }

  .input-with-right-icon.input-medium {
    @apply pr-10;
  }

  .input-with-right-icon.input-large {
    @apply pr-12;
  }

  /* 상태별 스타일 */
  .input-error {
    @apply border-red-500 focus:ring-red-500 focus:border-red-500;
  }

  .input-focused {
    @apply ring-2 ring-blue-500 border-blue-500;
  }

  .input-disabled {
    @apply opacity-50 cursor-not-allowed;
  }

  .input-readonly {
    @apply bg-gray-50 cursor-default;
  }

  .input-full-width {
    @apply w-full;
  }

  /* 아이콘 스타일 */
  .input-icon {
    @apply absolute flex items-center justify-center text-gray-400 pointer-events-none;
  }

  .input-icon-left {
    @apply left-3;
  }

  .input-icon-right {
    @apply right-3;
  }

  /* 패스워드 토글 버튼 */
  .input-password-toggle {
    @apply flex items-center justify-center text-gray-400 hover:text-gray-600 transition-colors;
    @apply focus:outline-none focus:text-blue-500 pointer-events-auto;
  }

  /* 메시지 스타일 */
  .input-error-message {
    @apply text-sm text-red-600 dark:text-red-400;
  }

  .input-helper-text {
    @apply text-sm text-gray-500 dark:text-gray-400;
  }

  /* 다크 모드 지원 */
  @media (prefers-color-scheme: dark) {
    .input-field {
      @apply bg-gray-800 border-gray-600 text-gray-100;
      @apply focus:ring-blue-400 focus:border-blue-400;
      @apply disabled:bg-gray-700 disabled:text-gray-400;
    }

    .input-readonly {
      @apply bg-gray-700;
    }

    .input-focused {
      @apply ring-blue-400 border-blue-400;
    }

    .input-error {
      @apply border-red-400 focus:ring-red-400 focus:border-red-400;
    }
  }

  /* 한국어 텍스트 최적화 */
  .input-field {
    word-break: keep-all;
  }

  /* PIN 입력 특별 스타일 */
  .input-field[type="password"].pin-input {
    @apply text-center text-xl font-mono tracking-widest;
  }

  /* 숫자 입력 스피너 숨김 */
  .input-field[type="number"]::-webkit-outer-spin-button,
  .input-field[type="number"]::-webkit-inner-spin-button {
    -webkit-appearance: none;
    margin: 0;
  }

  .input-field[type="number"] {
    -moz-appearance: textfield;
  }
</style>