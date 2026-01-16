// UI 컴포넌트 공통 타입 정의

// 기본 크기 타입
export type Size = 'small' | 'medium' | 'large';

// 색상 변형 타입
export type Variant = 'primary' | 'secondary' | 'danger' | 'success' | 'warning' | 'info' | 'outline' | 'ghost';

// 위치 타입
export type Position = 
  | 'top-left' 
  | 'top-center' 
  | 'top-right' 
  | 'bottom-left' 
  | 'bottom-center' 
  | 'bottom-right';

// 입력 필드 타입
export type InputType = 
  | 'text' 
  | 'password' 
  | 'email' 
  | 'number' 
  | 'tel' 
  | 'url' 
  | 'search'
  | 'date'
  | 'time'
  | 'datetime-local';

// 버튼 타입
export type ButtonType = 'button' | 'submit' | 'reset';

// 모달 크기 타입
export type ModalSize = 'small' | 'medium' | 'large' | 'full';

// 토스트 타입
export type ToastType = 'success' | 'error' | 'warning' | 'info';

// 컴포넌트 기본 속성 인터페이스
export interface BaseComponentProps {
  /** 컴포넌트 ID */
  id?: string;
  /** CSS 클래스명 */
  class?: string;
  /** 인라인 스타일 */
  style?: string;
  /** 비활성화 여부 */
  disabled?: boolean;
  /** 접근성 라벨 */
  ariaLabel?: string;
  /** 접근성 설명 */
  ariaDescribedby?: string;
}

// 버튼 컴포넌트 속성
export interface ButtonProps extends BaseComponentProps {
  /** 버튼 변형 */
  variant?: Variant;
  /** 버튼 크기 */
  size?: Size;
  /** 로딩 상태 */
  loading?: boolean;
  /** 전체 너비 사용 */
  fullWidth?: boolean;
  /** 버튼 타입 */
  type?: ButtonType;
  /** 왼쪽 아이콘 */
  leftIcon?: string;
  /** 오른쪽 아이콘 */
  rightIcon?: string;
}

// 입력 필드 컴포넌트 속성
export interface InputProps extends BaseComponentProps {
  /** 입력 타입 */
  type?: InputType;
  /** 입력값 */
  value?: string | number;
  /** 플레이스홀더 */
  placeholder?: string;
  /** 읽기 전용 */
  readonly?: boolean;
  /** 필수 입력 */
  required?: boolean;
  /** 에러 메시지 */
  error?: string;
  /** 라벨 */
  label?: string;
  /** 도움말 텍스트 */
  helperText?: string;
  /** 최대 길이 */
  maxlength?: number;
  /** 최소 길이 */
  minlength?: number;
  /** 최솟값 (숫자) */
  min?: number;
  /** 최댓값 (숫자) */
  max?: number;
  /** 단계 (숫자) */
  step?: number;
  /** 패턴 (정규식) */
  pattern?: string;
  /** 자동완성 */
  autocomplete?: string;
  /** 크기 */
  size?: Size;
  /** 전체 너비 사용 */
  fullWidth?: boolean;
  /** 왼쪽 아이콘 */
  leftIcon?: string;
  /** 오른쪽 아이콘 */
  rightIcon?: string;
  /** 패스워드 표시/숨김 토글 버튼 표시 */
  showPasswordToggle?: boolean;
}

// 모달 컴포넌트 속성
export interface ModalProps extends BaseComponentProps {
  /** 모달 열림 상태 */
  isOpen?: boolean;
  /** 모달 제목 */
  title?: string;
  /** 모달 크기 */
  size?: ModalSize;
  /** 닫기 가능 여부 */
  closable?: boolean;
  /** 백드롭 클릭으로 닫기 */
  closeOnBackdrop?: boolean;
  /** ESC 키로 닫기 */
  closeOnEscape?: boolean;
  /** 헤더 표시 */
  showHeader?: boolean;
  /** 푸터 표시 */
  showFooter?: boolean;
  /** 지속적 모달 (자동 닫힘 방지) */
  persistent?: boolean;
  /** Z-index 값 */
  zIndex?: number;
}

// 토스트 컴포넌트 속성
export interface ToastProps extends BaseComponentProps {
  /** 토스트 타입 */
  type?: ToastType;
  /** 제목 */
  title?: string;
  /** 메시지 */
  message?: string;
  /** 표시 시간 (밀리초) */
  duration?: number;
  /** 지속적 표시 */
  persistent?: boolean;
  /** 닫기 가능 여부 */
  closable?: boolean;
  /** 위치 */
  position?: Position;
  /** 아이콘 표시 */
  showIcon?: boolean;
  /** 프로그레스 바 표시 */
  showProgress?: boolean;
}

// 이벤트 핸들러 타입
export interface ComponentEvents {
  /** 클릭 이벤트 */
  click?: (event: MouseEvent) => void;
  /** 포커스 이벤트 */
  focus?: (event: FocusEvent) => void;
  /** 블러 이벤트 */
  blur?: (event: FocusEvent) => void;
  /** 입력 이벤트 */
  input?: (event: Event) => void;
  /** 변경 이벤트 */
  change?: (event: Event) => void;
  /** 키다운 이벤트 */
  keydown?: (event: KeyboardEvent) => void;
  /** 키업 이벤트 */
  keyup?: (event: KeyboardEvent) => void;
  /** 키프레스 이벤트 */
  keypress?: (event: KeyboardEvent) => void;
}

// 모달 이벤트
export interface ModalEvents {
  /** 모달 열림 */
  open?: () => void;
  /** 모달 닫힘 */
  close?: () => void;
  /** 확인 버튼 클릭 */
  confirm?: () => void;
  /** 취소 버튼 클릭 */
  cancel?: () => void;
}

// 토스트 이벤트
export interface ToastEvents {
  /** 토스트 닫힘 */
  close?: () => void;
  /** 토스트 클릭 */
  click?: () => void;
}

// 폼 검증 결과
export interface ValidationResult {
  /** 검증 성공 여부 */
  isValid: boolean;
  /** 에러 메시지 */
  error?: string;
  /** 경고 메시지 */
  warning?: string;
}

// 폼 필드 상태
export interface FieldState {
  /** 필드값 */
  value: string | number;
  /** 에러 메시지 */
  error?: string;
  /** 터치됨 여부 */
  touched: boolean;
  /** 검증됨 여부 */
  validated: boolean;
  /** 포커스됨 여부 */
  focused: boolean;
}

// 테마 관련 타입
export interface ThemeColors {
  /** 주요 색상 */
  primary: string;
  /** 보조 색상 */
  secondary: string;
  /** 성공 색상 */
  success: string;
  /** 경고 색상 */
  warning: string;
  /** 에러 색상 */
  error: string;
  /** 정보 색상 */
  info: string;
  /** 배경 색상 */
  background: string;
  /** 표면 색상 */
  surface: string;
  /** 텍스트 색상 */
  text: string;
}

// 반응형 브레이크포인트
export interface Breakpoints {
  /** 모바일 (0-639px) */
  mobile: string;
  /** 태블릿 (640-1023px) */
  tablet: string;
  /** 데스크톱 (1024px+) */
  desktop: string;
}

// 애니메이션 설정
export interface AnimationConfig {
  /** 지속 시간 (밀리초) */
  duration: number;
  /** 이징 함수 */
  easing: string;
  /** 지연 시간 (밀리초) */
  delay?: number;
}

// 컴포넌트 상태 타입
export type ComponentState = 'idle' | 'loading' | 'success' | 'error' | 'disabled';

// 드래그 앤 드롭 관련 타입
export interface DragDropData {
  /** 드래그되는 데이터 */
  data: any;
  /** 드래그 타입 */
  type: string;
  /** 드래그 소스 */
  source?: string;
}

// 키보드 단축키 타입
export interface KeyboardShortcut {
  /** 키 조합 */
  key: string;
  /** Ctrl 키 필요 여부 */
  ctrl?: boolean;
  /** Alt 키 필요 여부 */
  alt?: boolean;
  /** Shift 키 필요 여부 */
  shift?: boolean;
  /** 핸들러 함수 */
  handler: () => void;
  /** 설명 */
  description?: string;
}

// 접근성 관련 타입
export interface AccessibilityProps {
  /** ARIA 역할 */
  role?: string;
  /** ARIA 라벨 */
  ariaLabel?: string;
  /** ARIA 설명 */
  ariaDescribedby?: string;
  /** ARIA 확장됨 */
  ariaExpanded?: boolean;
  /** ARIA 선택됨 */
  ariaSelected?: boolean;
  /** ARIA 비활성화됨 */
  ariaDisabled?: boolean;
  /** 탭 인덱스 */
  tabindex?: number;
}

// 한국어 UI 텍스트 타입
export interface KoreanUITexts {
  /** 공통 텍스트 */
  common: {
    confirm: string;
    cancel: string;
    close: string;
    save: string;
    delete: string;
    edit: string;
    add: string;
    remove: string;
    search: string;
    loading: string;
    error: string;
    success: string;
    warning: string;
    info: string;
  };
  
  /** 인증 관련 텍스트 */
  auth: {
    login: string;
    logout: string;
    pin: string;
    recoveryKey: string;
    authenticate: string;
  };
  
  /** 파일 관리 텍스트 */
  file: {
    addFile: string;
    deleteFile: string;
    renameFile: string;
    exportFile: string;
    fileSize: string;
    fileName: string;
    fileType: string;
  };
  
  /** 폴더 관리 텍스트 */
  folder: {
    createFolder: string;
    deleteFolder: string;
    renameFolder: string;
    folderName: string;
  };
}

// 기본 한국어 UI 텍스트
export const defaultKoreanTexts: KoreanUITexts = {
  common: {
    confirm: '확인',
    cancel: '취소',
    close: '닫기',
    save: '저장',
    delete: '삭제',
    edit: '편집',
    add: '추가',
    remove: '제거',
    search: '검색',
    loading: '로딩 중...',
    error: '오류',
    success: '성공',
    warning: '경고',
    info: '정보'
  },
  auth: {
    login: '로그인',
    logout: '로그아웃',
    pin: 'PIN',
    recoveryKey: '복구 키',
    authenticate: '인증'
  },
  file: {
    addFile: '파일 추가',
    deleteFile: '파일 삭제',
    renameFile: '파일 이름 변경',
    exportFile: '파일 내보내기',
    fileSize: '파일 크기',
    fileName: '파일명',
    fileType: '파일 형식'
  },
  folder: {
    createFolder: '폴더 생성',
    deleteFolder: '폴더 삭제',
    renameFolder: '폴더 이름 변경',
    folderName: '폴더명'
  }
};