/**
 * 인증 상태 관리 스토어
 * 사용자 로그인 상태, 세션 정보, 브루트포스 방지 등을 관리합니다.
 */

import { writable, derived } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';

// 사용자 정보 인터페이스
export interface User {
  id: string;
  name: string;
  lastLogin: Date;
}

// 인증 상태 인터페이스
export interface AuthState {
  isAuthenticated: boolean;
  isLoading: boolean;
  user: User | null;
  sessionRemainingTime: number; // 초 단위
  failedAttempts: number;
  lastFailedAttempt: Date | null;
  lockoutEndTime: Date | null;
  hasPin: boolean;
  hasRecoveryKey: boolean;
}

// 초기 인증 상태
const initialAuthState: AuthState = {
  isAuthenticated: false,
  isLoading: false,
  user: null,
  sessionRemainingTime: 0,
  failedAttempts: 0,
  lastFailedAttempt: null,
  lockoutEndTime: null,
  hasPin: false,
  hasRecoveryKey: false,
};

// 인증 상태 스토어
export const authState = writable<AuthState>(initialAuthState);

// 브루트포스 방지 상태 (파생 스토어)
export const isBruteForceProtected = derived(
  authState,
  ($authState) => {
    if (!$authState.lockoutEndTime) return false;
    return new Date() < $authState.lockoutEndTime;
  }
);

// 남은 잠금 시간 (파생 스토어)
export const remainingLockoutTime = derived(
  authState,
  ($authState) => {
    if (!$authState.lockoutEndTime) return 0;
    const remaining = $authState.lockoutEndTime.getTime() - new Date().getTime();
    return Math.max(0, Math.floor(remaining / 1000));
  }
);

/**
 * 인증 서비스 클래스
 * 백엔드 API와 통신하여 인증 관련 작업을 수행합니다.
 */
export class AuthService {
  /**
   * 초기 인증 상태를 확인합니다.
   */
  static async checkInitialAuthState(): Promise<void> {
    try {
      authState.update(state => ({ ...state, isLoading: true }));

      // PIN 설정 여부 확인
      const hasPin = await invoke<boolean>('has_pin_set');

      // 복구 키 설정 여부 확인
      const hasRecoveryKey = await invoke<boolean>('has_recovery_key_set');

      // 현재 인증 상태 확인
      const isAuthenticated = await invoke<boolean>('check_auth_status');

      // 세션 남은 시간 확인 (인증된 경우)
      let sessionRemainingTime = 0;
      if (isAuthenticated) {
        sessionRemainingTime = await invoke<number>('get_session_remaining_time');
      }

      authState.update(state => ({
        ...state,
        isAuthenticated,
        hasPin,
        hasRecoveryKey,
        sessionRemainingTime,
        isLoading: false,
      }));

    } catch (error) {
      console.error('초기 인증 상태 확인 실패:', error);
      authState.update(state => ({
        ...state,
        isLoading: false,
        isAuthenticated: false,
      }));
    }
  }

  /**
   * PIN으로 로그인을 시도합니다.
   */
  static async authenticateWithPin(pin: string): Promise<{ success: boolean; error?: string }> {
    try {
      authState.update(state => ({ ...state, isLoading: true }));

      const result = await invoke<boolean>('authenticate_pin', { pin });

      if (result) {
        // 로그인 성공 - 상태만 업데이트하고 윈도우 크기 조정은 나중에
        const sessionRemainingTime = await invoke<number>('get_session_remaining_time');

        authState.update(state => ({
          ...state,
          isAuthenticated: true,
          isLoading: false,
          failedAttempts: 0,
          lastFailedAttempt: null,
          lockoutEndTime: null,
          sessionRemainingTime,
        }));

        return { success: true };
      } else {
        // 로그인 실패 - 실패 횟수 증가
        authState.update(state => {
          const newFailedAttempts = state.failedAttempts + 1;
          const now = new Date();
          let lockoutEndTime = null;

          // 5회 실패 시 30분 잠금
          if (newFailedAttempts >= 5) {
            lockoutEndTime = new Date(now.getTime() + 30 * 60 * 1000); // 30분 후
          }

          return {
            ...state,
            isLoading: false,
            failedAttempts: newFailedAttempts,
            lastFailedAttempt: now,
            lockoutEndTime,
          };
        });

        const remainingAttempts = 5 - (authState.subscribe(s => s.failedAttempts) as any);
        return {
          success: false,
          error: remainingAttempts > 0
            ? `PIN이 올바르지 않습니다. (${remainingAttempts}회 남음)`
            : '5회 연속 실패로 30분간 로그인이 차단됩니다.'
        };
      }

    } catch (error) {
      authState.update(state => ({ ...state, isLoading: false }));
      return {
        success: false,
        error: typeof error === 'string' ? error : '인증 중 오류가 발생했습니다.'
      };
    }
  }

  /**
   * 복구 키로 로그인을 시도합니다.
   */
  static async authenticateWithRecoveryKey(recoveryKey: string): Promise<{ success: boolean; error?: string }> {
    try {
      authState.update(state => ({ ...state, isLoading: true }));

      const result = await invoke<boolean>('authenticate_with_recovery_key', { recoveryKey });

      if (result) {
        // 로그인 성공 - 상태만 업데이트하고 윈도우 크기 조정은 나중에
        const sessionRemainingTime = await invoke<number>('get_session_remaining_time');

        authState.update(state => ({
          ...state,
          isAuthenticated: true,
          isLoading: false,
          failedAttempts: 0,
          lastFailedAttempt: null,
          lockoutEndTime: null,
          sessionRemainingTime,
        }));

        return { success: true };
      } else {
        authState.update(state => ({ ...state, isLoading: false }));
        return { success: false, error: '복구 키가 올바르지 않습니다.' };
      }

    } catch (error) {
      authState.update(state => ({ ...state, isLoading: false }));
      return {
        success: false,
        error: typeof error === 'string' ? error : '복구 키 인증 중 오류가 발생했습니다.'
      };
    }
  }

  /**
   * 로그아웃을 수행합니다.
   */
  static async logout(): Promise<void> {
    try {
      await invoke('logout');

      authState.update(state => ({
        ...state,
        isAuthenticated: false,
        sessionRemainingTime: 0,
      }));

    } catch (error) {
      console.error('로그아웃 실패:', error);
      // 로그아웃은 실패해도 클라이언트 상태는 초기화
      authState.update(state => ({
        ...state,
        isAuthenticated: false,
        sessionRemainingTime: 0,
      }));
    }
  }

  /**
   * PIN을 변경합니다.
   */
  static async changePin(currentPin: string, newPin: string): Promise<{ success: boolean; error?: string }> {
    try {
      await invoke('change_pin', { currentPin, newPin });
      return { success: true };
    } catch (error) {
      return {
        success: false,
        error: typeof error === 'string' ? error : 'PIN 변경 중 오류가 발생했습니다.'
      };
    }
  }

  /**
   * 세션 남은 시간을 업데이트합니다.
   */
  static async updateSessionTime(): Promise<void> {
    try {
      const sessionRemainingTime = await invoke<number>('get_session_remaining_time');
      authState.update(state => ({ ...state, sessionRemainingTime }));
    } catch (error) {
      console.error('세션 시간 업데이트 실패:', error);
    }
  }

  /**
   * 파일 매니저 모드로 창 크기를 조정합니다.
   */
  static async resizeWindowForFileManager(): Promise<void> {
    try {
      const { getCurrentWindow } = await import('@tauri-apps/api/window');
      const { LogicalSize } = await import('@tauri-apps/api/window');

      const appWindow = getCurrentWindow();

      // 파일 매니저에 적합한 크기로 조정 (1400x800)
      await Promise.all([
        appWindow.setSize(new LogicalSize(1400, 800)),
        appWindow.setResizable(true),
        appWindow.setMaximizable(true)
      ]);

      // 창을 화면 중앙으로 이동
      await appWindow.center();

      console.log('창 크기가 파일 매니저 모드로 조정되었습니다: 1400x800');
    } catch (error) {
      console.error('창 크기 조정 실패:', error);
    }
  }

  /**
   * 로그인 모드로 창 크기를 조정합니다.
   */
  static async resizeWindowForLogin(): Promise<void> {
    try {
      const { getCurrentWindow } = await import('@tauri-apps/api/window');
      const { LogicalSize } = await import('@tauri-apps/api/window');

      const appWindow = getCurrentWindow();

      // 로그인에 적합한 크기로 조정
      await Promise.all([
        appWindow.setSize(new LogicalSize(370, 650)),
        appWindow.setResizable(false),
        appWindow.setMaximizable(false)
      ]);

      // 창을 화면 중앙으로 이동
      await appWindow.center();

      console.log('창 크기가 로그인 모드로 조정되었습니다: 370x650 (크기 조절 비활성화)');
    } catch (error) {
      console.error('창 크기 조정 실패:', error);
    }
  }
}

// 세션 타이머 (1초마다 업데이트)
let sessionTimer: number | null = null;

authState.subscribe(($authState) => {
  if ($authState.isAuthenticated && !sessionTimer) {
    // 인증된 상태에서 세션 타이머 시작
    sessionTimer = window.setInterval(() => {
      AuthService.updateSessionTime();
    }, 1000);
  } else if (!$authState.isAuthenticated && sessionTimer) {
    // 인증 해제 시 타이머 정리
    window.clearInterval(sessionTimer);
    sessionTimer = null;
  }
});

// 브라우저 종료 시 정리
if (typeof window !== 'undefined') {
  window.addEventListener('beforeunload', () => {
    if (sessionTimer) {
      window.clearInterval(sessionTimer);
    }
  });
}