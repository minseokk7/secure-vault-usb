; SecureVault USB Installer
; Custom NSIS script for USB drive installation with Korean support

!include "MUI2.nsh"
!include "LogicLib.nsh"
!include "nsDialogs.nsh"
!include "WinMessages.nsh"

; Basic settings
!define PRODUCT_NAME "SecureVault"
!define PRODUCT_VERSION "1.0.0"

Name "${PRODUCT_NAME} ${PRODUCT_VERSION}"
OutFile "SecureVault_USB_Installer.exe"
InstallDir ""
RequestExecutionLevel admin
Unicode True

; Korean font settings
!define MUI_FONT_TITLE "Malgun Gothic"
!define MUI_FONT_TITLE_SIZE "12"
!define MUI_FONT "Malgun Gothic"
!define MUI_FONT_SIZE "9"

; Variables
Var Dialog
Var UsbDriveCombo
Var UsbInfoLabel
Var RefreshBtn
Var TitleLabel
Var DescLabel
Var SelectedDrive
Var DriveCount
Var KoreanFont

; UI settings
!define MUI_ABORTWARNING

; Language - Korean

; Pages
Page custom UsbSelectPage UsbSelectPageLeave
!insertmacro MUI_PAGE_INSTFILES

; Finish Page Settings
!define MUI_FINISHPAGE_NOAUTOCLOSE
!define MUI_FINISHPAGE_RUN "$INSTDIR\SecureVault.exe"
!define MUI_FINISHPAGE_RUN_TEXT "Run SecureVault"
!define MUI_FINISHPAGE_SHOWREADME ""
!define MUI_FINISHPAGE_SHOWREADME_TEXT "Open Install Folder"
!define MUI_FINISHPAGE_SHOWREADME_FUNCTION OpenInstallFolder

!insertmacro MUI_PAGE_FINISH

; Language Settings - must be after page macros
!insertmacro MUI_LANGUAGE "English"

Function OpenInstallFolder
  ExecShell "open" "$INSTDIR"
FunctionEnd

; USB Drive Selection Page
Function UsbSelectPage
  !insertmacro MUI_HEADER_TEXT "USB $(^Name)" "USB $(^Name)"

  ; Create Korean font
  CreateFont $KoreanFont "Malgun Gothic" 9 400

  nsDialogs::Create 1018
  Pop $Dialog
  ${If} $Dialog == error
    Abort
  ${EndIf}

  ; Title
  ${NSD_CreateLabel} 0 0 100% 25u "SecureVault USB Install"
  Pop $TitleLabel
  CreateFont $0 "Malgun Gothic" 14 700
  SendMessage $TitleLabel ${WM_SETFONT} $0 1

  ; Description
  ${NSD_CreateLabel} 0 30u 100% 20u "Install SecureVault to USB drive"
  Pop $DescLabel
  SendMessage $DescLabel ${WM_SETFONT} $KoreanFont 1

  ; Drive selection group
  ${NSD_CreateGroupBox} 0 55u 100% 75u "Select USB Drive"
  Pop $0
  SendMessage $0 ${WM_SETFONT} $KoreanFont 1

  ${NSD_CreateLabel} 10u 72u 55u 12u "Drive:"
  Pop $0
  SendMessage $0 ${WM_SETFONT} $KoreanFont 1

  ; Dropdown
  ${NSD_CreateDropList} 70u 70u 50% 100u ""
  Pop $UsbDriveCombo
  SendMessage $UsbDriveCombo ${WM_SETFONT} $KoreanFont 1

  ; Refresh button
  ${NSD_CreateButton} 75% 69u 23% 15u "Refresh"
  Pop $RefreshBtn
  SendMessage $RefreshBtn ${WM_SETFONT} $KoreanFont 1
  ${NSD_OnClick} $RefreshBtn OnRefreshClick

  ; Drive info
  ${NSD_CreateLabel} 10u 92u 88% 25u ""
  Pop $UsbInfoLabel
  SendMessage $UsbInfoLabel ${WM_SETFONT} $KoreanFont 1

  ; Notes
  ${NSD_CreateLabel} 0 140u 100% 35u "* Install to SecureVault folder$\r$\n* Requires 50MB free space"
  Pop $0
  SendMessage $0 ${WM_SETFONT} $KoreanFont 1

  ; Populate drives
  Call PopulateUsbDrives

  ; Dropdown change event
  ${NSD_OnChange} $UsbDriveCombo OnDriveChange

  nsDialogs::Show
FunctionEnd

; Populate USB Drives
Function PopulateUsbDrives
  SendMessage $UsbDriveCombo ${CB_RESETCONTENT} 0 0
  StrCpy $DriveCount 0
  StrCpy $SelectedDrive ""

  StrCpy $R0 65

  ${DoWhile} $R0 < 91
    IntFmt $R1 "%c" $R0

    System::Call "kernel32::GetDriveType(t '$R1:\') i .r2"

    ${If} $2 == 2
    ${OrIf} $2 == 3
      System::Call "kernel32::GetDiskFreeSpaceEx(t '$R1:\', *l .r3, *l .r4, *l .r5) i .r6"

      ${If} $6 != 0
        System::Call "kernel32::GetVolumeInformation(t '$R1:\', t .r7, i ${NSIS_MAX_STRLEN}, *i, *i, *i, t, i) i .r8"

        System::Int64Op $4 / 1073741824
        Pop $R2
        System::Int64Op $3 / 1073741824
        Pop $R3

        ${If} $7 == ""
          StrCpy $7 "USB"
        ${EndIf}

        StrCpy $R4 "$R1: ($7) - $R3GB / $R2GB"
        SendMessage $UsbDriveCombo ${CB_ADDSTRING} 0 "STR:$R4"

        IntOp $DriveCount $DriveCount + 1

        ${If} $DriveCount == 1
          StrCpy $SelectedDrive "$R1:"
        ${EndIf}
      ${EndIf}
    ${EndIf}

    IntOp $R0 $R0 + 1
  ${Loop}

  ${If} $DriveCount == 0
    SendMessage $UsbDriveCombo ${CB_ADDSTRING} 0 "STR:No USB found"
    ${NSD_SetText} $UsbInfoLabel "Connect USB and click Refresh"
    EnableWindow $UsbDriveCombo 0
    GetDlgItem $0 $HWNDPARENT 1
    EnableWindow $0 0
  ${Else}
    EnableWindow $UsbDriveCombo 1
    SendMessage $UsbDriveCombo ${CB_SETCURSEL} 0 0
    Call OnDriveChange
    GetDlgItem $0 $HWNDPARENT 1
    EnableWindow $0 1
  ${EndIf}
FunctionEnd

Function OnRefreshClick
  Call PopulateUsbDrives
FunctionEnd

Function OnDriveChange
  SendMessage $UsbDriveCombo ${CB_GETCURSEL} 0 0 $R0
  ${If} $R0 != -1
    System::Call "user32::SendMessage(p $UsbDriveCombo, i ${CB_GETLBTEXT}, i $R0, t .r1)"
    StrCpy $SelectedDrive $1 2
    ${NSD_SetText} $UsbInfoLabel "Install Path: $SelectedDrive\"
    StrCpy $INSTDIR "$SelectedDrive\"
  ${EndIf}
FunctionEnd

Function UsbSelectPageLeave
  ${If} $DriveCount == 0
    MessageBox MB_OK|MB_ICONEXCLAMATION "No USB drive found. Please connect and refresh."
    Abort
  ${EndIf}

  ${If} $SelectedDrive == ""
    MessageBox MB_OK|MB_ICONEXCLAMATION "Please select a USB drive."
    Abort
  ${EndIf}
FunctionEnd

Section "SecureVault" SecMain
  SetOutPath "$INSTDIR"
  
  ; Remove existing .securevault folder (reset attributes first)
  IfFileExists "$INSTDIR\.securevault" 0 +3
    SetFileAttributes "$INSTDIR\.securevault" NORMAL
    RMDir /r "$INSTDIR\.securevault"
  
  ; Install executable
  File "release\SecureVault.exe"
SectionEnd

Function .onInstSuccess
  MessageBox MB_OK "Installation Complete!$\r$\n$\r$\nPath: $INSTDIR"
  ExecShell "open" "$INSTDIR"
FunctionEnd
