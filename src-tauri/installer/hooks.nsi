; SecureVault 커스텀 NSIS 인스톨러 훅
; USB 드라이브 자동 감지 및 선택 기능
; Tauri v2 호환 형식

!include "LogicLib.nsh"
!include "nsDialogs.nsh"
!include "WinMessages.nsh"

; 변수 선언
Var UsbDriveCombo
Var UsbDriveLabel
Var UsbInfoLabel
Var RefreshBtn
Var SelectedDriveLetter

; ============================================================
; NSIS_HOOK_PREINSTALL - 설치 시작 전 호출
; ============================================================
!macro NSIS_HOOK_PREINSTALL
  ; USB 드라이브 선택 페이지 표시
  Call ShowUsbDrivePage
!macroend

; ============================================================
; USB 드라이브 선택 페이지
; ============================================================
Function ShowUsbDrivePage
  ; 커스텀 페이지 생성
  nsDialogs::Create 1018
  Pop $0
  ${If} $0 == error
    Abort
  ${EndIf}

  ; 제목 라벨
  ${NSD_CreateLabel} 0 0 100% 25u "🔒 SecureVault USB 설치"
  Pop $0
  CreateFont $1 "맑은 고딕" 14 700
  SendMessage $0 ${WM_SETFONT} $1 0

  ; 설명 라벨
  ${NSD_CreateLabel} 0 30u 100% 20u "SecureVault를 설치할 USB 드라이브를 선택하세요:"
  Pop $UsbDriveLabel

  ; USB 드라이브 드롭다운
  ${NSD_CreateDropList} 0 55u 80% 120u ""
  Pop $UsbDriveCombo

  ; 새로고침 버튼
  ${NSD_CreateButton} 82% 54u 18% 22u "새로고침"
  Pop $RefreshBtn
  ${NSD_OnClick} $RefreshBtn OnRefreshClick

  ; USB 드라이브 목록 채우기
  Call PopulateUsbDrives

  ; 정보 라벨
  ${NSD_CreateLabel} 0 85u 100% 40u ""
  Pop $UsbInfoLabel

  ; 드롭다운 선택 변경 이벤트
  ${NSD_OnChange} $UsbDriveCombo OnDriveSelect

  ; 안내 메시지
  ${NSD_CreateLabel} 0 130u 100% 30u "※ 선택한 드라이브의 'SecureVault' 폴더에 설치됩니다.$\r$\n※ 최소 50MB의 여유 공간이 필요합니다."
  Pop $0

  ; 페이지 표시
  nsDialogs::Show
FunctionEnd

; ============================================================
; USB 드라이브 목록 채우기
; ============================================================
Function PopulateUsbDrives
  ; 드롭다운 초기화
  SendMessage $UsbDriveCombo ${CB_RESETCONTENT} 0 0

  ; 드라이브 순회 (C-Z)
  StrCpy $R0 67 ; 'C'
  
  ${DoWhile} $R0 < 91 ; 'Z' + 1
    IntFmt $R1 "%c" $R0 ; 드라이브 문자
    
    ; 드라이브 타입 확인
    System::Call "kernel32::GetDriveType(t '$R1:\') i .r2"
    
    ${If} $2 == 2 ; DRIVE_REMOVABLE (이동식 드라이브)
      ; 드라이브 정보 가져오기
      System::Call "kernel32::GetDiskFreeSpaceEx(t '$R1:\', *l .r3, *l .r4, *l .r5) i .r6"
      
      ${If} $6 != 0 ; 드라이브 접근 가능
        ; 볼륨 라벨 가져오기
        System::Call "kernel32::GetVolumeInformation(t '$R1:\', t .r7, i ${NSIS_MAX_STRLEN}, *i, *i, *i, t, i) i .r8"
        
        ; 용량 계산 (GB)
        System::Int64Op $4 / 1073741824
        Pop $R2 ; 총 용량
        System::Int64Op $3 / 1073741824
        Pop $R3 ; 여유 공간
        
        ; 라벨이 비어있으면 기본값
        ${If} $7 == ""
          StrCpy $7 "이동식 디스크"
        ${EndIf}
        
        ; 드롭다운에 추가: "E: (라벨) - 4GB / 8GB"
        StrCpy $R4 "$R1: ($7) - $R3GB / $R2GB 사용 가능"
        SendMessage $UsbDriveCombo ${CB_ADDSTRING} 0 "STR:$R4"
      ${EndIf}
    ${EndIf}
    
    IntOp $R0 $R0 + 1
  ${Loop}

  ; 드라이브가 없으면 안내 메시지
  SendMessage $UsbDriveCombo ${CB_GETCOUNT} 0 0 $R0
  ${If} $R0 == 0
    SendMessage $UsbDriveCombo ${CB_ADDSTRING} 0 "STR:USB 드라이브를 찾을 수 없습니다"
    EnableWindow $UsbDriveCombo 0
  ${Else}
    EnableWindow $UsbDriveCombo 1
    ; 첫 번째 항목 선택
    SendMessage $UsbDriveCombo ${CB_SETCURSEL} 0 0
    Call OnDriveSelect
  ${EndIf}
FunctionEnd

; ============================================================
; 새로고침 버튼 클릭
; ============================================================
Function OnRefreshClick
  Call PopulateUsbDrives
FunctionEnd

; ============================================================
; 드라이브 선택 변경
; ============================================================
Function OnDriveSelect
  ; 선택된 항목 가져오기
  SendMessage $UsbDriveCombo ${CB_GETCURSEL} 0 0 $R0
  ${If} $R0 != -1
    System::Call "user32::SendMessage(p $UsbDriveCombo, i ${CB_GETLBTEXT}, i $R0, t .r1)"
    
    ; 드라이브 문자 추출 (첫 2글자)
    StrCpy $SelectedDriveLetter $1 2
    
    ; 설치 경로 설정
    StrCpy $INSTDIR "$SelectedDriveLetter\SecureVault"
    
    ; 정보 라벨 업데이트
    ${NSD_SetText} $UsbInfoLabel "설치 경로: $INSTDIR"
  ${EndIf}
FunctionEnd

; ============================================================
; 페이지 떠날 때 검증
; ============================================================
Function ValidateUsbSelection
  SendMessage $UsbDriveCombo ${CB_GETCOUNT} 0 0 $R0
  ${If} $R0 == 0
    MessageBox MB_OK|MB_ICONEXCLAMATION "USB 드라이브가 연결되어 있지 않습니다.$\r$\nUSB를 연결한 후 '새로고침' 버튼을 클릭하세요."
    Abort
  ${EndIf}
  
  ${If} $SelectedDriveLetter == ""
    MessageBox MB_OK|MB_ICONEXCLAMATION "USB 드라이브를 선택해주세요."
    Abort
  ${EndIf}
FunctionEnd
