import { writable } from 'svelte/store';

// 토스트 타입 정의
export interface ToastData {
  id: string;
  type: 'success' | 'error' | 'warning' | 'info';
  title?: string;
  message: string;
  duration?: number;
  persistent?: boolean;
  closable?: boolean;
  showIcon?: boolean;
  showProgress?: boolean;
  onClick?: (toast: ToastData) => void;
}

// 토스트 옵션 타입
export interface ToastOptions {
  title?: string;
  duration?: number;
  persistent?: boolean;
  closable?: boolean;
  showIcon?: boolean;
  showProgress?: boolean;
  onClick?: (toast: ToastData) => void;
}

// 토스트 표시 제어 스토어
export const toastEnabled = writable(true);

// 토스트 스토어 생성
function createToastStore() {
  const { subscribe, update } = writable<ToastData[]>([]);

  // 고유 ID 생성
  function generateId(): string {
    return `toast-${Date.now()}-${Math.random().toString(36).substring(2, 11)}`;
  }

  // 토스트 추가 (토스트가 활성화된 경우에만)
  function add(toast: Omit<ToastData, 'id'>): string {
    const id = generateId();
    const newToast: ToastData = {
      id,
      duration: 5000,
      persistent: false,
      closable: true,
      showIcon: true,
      showProgress: true,
      ...toast
    };

    // 토스트가 비활성화된 경우 추가하지 않음
    let enabled = true;
    toastEnabled.subscribe(value => enabled = value)();
    
    if (enabled) {
      update(toasts => [newToast, ...toasts]);
    }
    
    return id;
  }

  // 토스트 제거
  function remove(id: string) {
    update(toasts => toasts.filter(toast => toast.id !== id));
  }

  // 모든 토스트 제거
  function clear() {
    update(() => []);
  }

  // 성공 토스트
  function success(message: string, options: ToastOptions = {}): string {
    return add({
      type: 'success',
      message,
      ...options
    });
  }

  // 에러 토스트
  function error(message: string, options: ToastOptions = {}): string {
    return add({
      type: 'error',
      message,
      duration: 8000, // 에러는 더 오래 표시
      ...options
    });
  }

  // 경고 토스트
  function warning(message: string, options: ToastOptions = {}): string {
    return add({
      type: 'warning',
      message,
      duration: 6000,
      ...options
    });
  }

  // 정보 토스트
  function info(message: string, options: ToastOptions = {}): string {
    return add({
      type: 'info',
      message,
      ...options
    });
  }

  // 확인 토스트 (사용자 액션 필요)
  function confirm(
    message: string, 
    onConfirm: () => void, 
    options: ToastOptions = {}
  ): string {
    return add({
      type: 'warning',
      message,
      persistent: true,
      onClick: (toast) => {
        onConfirm();
        remove(toast.id);
      },
      ...options
    });
  }

  // 로딩 토스트
  function loading(message: string, options: ToastOptions = {}): string {
    return add({
      type: 'info',
      message,
      persistent: true,
      closable: false,
      showProgress: false,
      ...options
    });
  }

  // 로딩 토스트 업데이트
  function updateLoading(id: string, message: string, type: 'success' | 'error' = 'success') {
    update(toasts => 
      toasts.map(toast => 
        toast.id === id 
          ? { 
              ...toast, 
              message, 
              type, 
              persistent: false, 
              closable: true,
              duration: type === 'success' ? 3000 : 5000
            }
          : toast
      )
    );
  }

  return {
    subscribe,
    add,
    remove,
    clear,
    success,
    error,
    warning,
    info,
    confirm,
    loading,
    updateLoading
  };
}

// 토스트 스토어 인스턴스
export const toastStore = createToastStore();

// 토스트 활성화/비활성화 함수
export function enableToasts() {
  toastEnabled.set(true);
}

export function disableToasts() {
  toastEnabled.set(false);
  // 기존 토스트도 모두 제거
  toastStore.clear();
}

// 편의 함수들 (전역에서 사용 가능)
export const toast = {
  success: toastStore.success,
  error: toastStore.error,
  warning: toastStore.warning,
  info: toastStore.info,
  confirm: toastStore.confirm,
  loading: toastStore.loading,
  updateLoading: toastStore.updateLoading,
  clear: toastStore.clear
};

// addToast 함수 (호환성을 위해 추가)
export function addToast(toastData: Omit<ToastData, 'id'>): string {
  return toastStore.add(toastData);
}

// 한국어 메시지 템플릿
export const toastMessages = {
  // 인증 관련
  auth: {
    loginSuccess: '로그인에 성공했습니다.',
    loginFailed: '로그인에 실패했습니다. PIN을 확인해주세요.',
    logoutSuccess: '로그아웃되었습니다.',
    pinChanged: 'PIN이 성공적으로 변경되었습니다.',
    recoveryKeyGenerated: '복구 키가 생성되었습니다. 안전한 곳에 보관해주세요.',
    sessionExpired: '세션이 만료되었습니다. 다시 로그인해주세요.'
  },

  // 파일 관리 관련
  file: {
    addSuccess: '파일이 성공적으로 추가되었습니다.',
    addFailed: '파일 추가에 실패했습니다.',
    deleteSuccess: '파일이 성공적으로 삭제되었습니다.',
    deleteFailed: '파일 삭제에 실패했습니다.',
    exportSuccess: '파일이 성공적으로 내보내졌습니다.',
    exportFailed: '파일 내보내기에 실패했습니다.',
    encryptionFailed: '파일 암호화에 실패했습니다.',
    decryptionFailed: '파일 복호화에 실패했습니다.',
    fileTooLarge: '파일 크기가 너무 큽니다. (최대 1GB)',
    invalidFileType: '지원하지 않는 파일 형식입니다.'
  },

  // 폴더 관리 관련
  folder: {
    createSuccess: '폴더가 성공적으로 생성되었습니다.',
    createFailed: '폴더 생성에 실패했습니다.',
    deleteSuccess: '폴더가 성공적으로 삭제되었습니다.',
    deleteFailed: '폴더 삭제에 실패했습니다.',
    renameSuccess: '폴더 이름이 성공적으로 변경되었습니다.',
    renameFailed: '폴더 이름 변경에 실패했습니다.',
    duplicateName: '같은 이름의 폴더가 이미 존재합니다.'
  },

  // 시스템 관련
  system: {
    saveSuccess: '설정이 저장되었습니다.',
    saveFailed: '설정 저장에 실패했습니다.',
    loadFailed: '데이터 로드에 실패했습니다.',
    networkError: '네트워크 연결을 확인해주세요.',
    unexpectedError: '예상치 못한 오류가 발생했습니다.',
    operationCancelled: '작업이 취소되었습니다.',
    operationCompleted: '작업이 완료되었습니다.'
  },

  // 보안 관련
  security: {
    bruteForceProtection: '보안을 위해 잠시 후 다시 시도해주세요.',
    invalidPin: 'PIN 형식이 올바르지 않습니다.',
    weakPin: 'PIN이 너무 단순합니다. 더 복잡한 PIN을 사용해주세요.',
    recoveryKeyInvalid: '복구 키가 올바르지 않습니다.',
    encryptionError: '암호화 처리 중 오류가 발생했습니다.',
    integrityCheckFailed: '파일 무결성 검증에 실패했습니다.'
  }
};

// 사용 예시:
// import { toast, toastMessages } from '$lib/stores/toast';
// 
// toast.success(toastMessages.auth.loginSuccess);
// toast.error(toastMessages.file.addFailed);
// 
// const loadingId = toast.loading('파일을 처리하는 중...');
// // 작업 완료 후
// toast.updateLoading(loadingId, '파일 처리가 완료되었습니다.', 'success');