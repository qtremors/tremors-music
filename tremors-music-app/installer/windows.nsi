; SPDX-License-Identifier: GPL-3.0-only
; Copyright (C) 2026 Tremors and contributors
Unicode true
Target amd64-unicode
RequestExecutionLevel user
ManifestDPIAware true
SetCompressor /SOLID lzma
SetCompressorDictSize 32

!include "MUI2.nsh"
!include "x64.nsh"
!include "WinVer.nsh"

!define APP_NAME "Tremors Music"
!define UNINSTALL_KEY "Software\Microsoft\Windows\CurrentVersion\Uninstall\TremorsMusic"
Name "${APP_NAME} ${APP_VERSION}"
OutFile "${OUTPUT_FILE}"
InstallDir "$LOCALAPPDATA\Programs\Tremors Music"
InstallDirRegKey HKCU "${UNINSTALL_KEY}" "InstallLocation"
VIProductVersion "${APP_VERSION}.0"
VIAddVersionKey "ProductName" "${APP_NAME}"
VIAddVersionKey "ProductVersion" "${APP_VERSION}"
VIAddVersionKey "FileVersion" "${APP_VERSION}.0"
VIAddVersionKey "FileDescription" "Tremors Music Windows Installer"
VIAddVersionKey "LegalCopyright" "Copyright 2026 Tremors and contributors; GPL-3.0-only"

!define MUI_ICON "${REPOSITORY_ROOT}\assets\TremorsMusic.ico"
!define MUI_UNICON "${REPOSITORY_ROOT}\assets\TremorsMusic.ico"
!define MUI_ABORTWARNING
!define MUI_WELCOMEPAGE_TEXT "Install Tremors Music ${APP_VERSION} for your Windows account.$\r$\n$\r$\nMusic files and library data are kept separately from application files."
!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_LICENSE "${PAYLOAD_DIR}\LICENSE.md"
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!define MUI_FINISHPAGE_RUN "$INSTDIR\tremors-music.exe"
!insertmacro MUI_PAGE_FINISH
!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES
!insertmacro MUI_LANGUAGE "English"

Function .onInit
    ${IfNot} ${RunningX64}
        MessageBox MB_OK|MB_ICONSTOP "Tremors Music requires 64-bit Windows." /SD IDOK
        SetErrorLevel 1
        Abort
    ${EndIf}
    SetRegView 64
    SetShellVarContext current
    ${IfNot} ${AtLeastWin10}
        MessageBox MB_OK|MB_ICONSTOP "Tremors Music requires Windows 10 version 1903 or newer." /SD IDOK
        SetErrorLevel 1
        Abort
    ${EndIf}
    ReadRegStr $0 HKLM "Software\Microsoft\Windows NT\CurrentVersion" "CurrentBuildNumber"
    IntCmp $0 18362 supported unsupported supported
    unsupported:
        MessageBox MB_OK|MB_ICONSTOP "Tremors Music requires Windows 10 version 1903 or newer." /SD IDOK
        SetErrorLevel 1
        Abort
    supported:
    ReadRegStr $0 HKCU "${UNINSTALL_KEY}" "InstallLocation"
    ${If} $0 != ""
        StrCpy $INSTDIR $0
    ${EndIf}
FunctionEnd

Section "Tremors Music"
    SetOutPath "$INSTDIR"
    File "${PAYLOAD_DIR}\tremors-music.exe"
    File "${PAYLOAD_DIR}\LICENSE.md"
    File "${PAYLOAD_DIR}\README.md"
    File "${PAYLOAD_DIR}\DEVELOPMENT.md"
    File "${PAYLOAD_DIR}\CHANGELOG.md"
    File "${PAYLOAD_DIR}\TASKS.md"
    File "${PAYLOAD_DIR}\PRIVACY.md"
    File "${PAYLOAD_DIR}\SOURCE.txt"
    File "${PAYLOAD_DIR}\NSIS-LICENSES.txt"
    WriteUninstaller "$INSTDIR\Uninstall.exe"
    CreateDirectory "$SMPROGRAMS\Tremors Music"
    CreateShortcut "$SMPROGRAMS\Tremors Music\Tremors Music.lnk" "$INSTDIR\tremors-music.exe"
    CreateShortcut "$SMPROGRAMS\Tremors Music\Uninstall Tremors Music.lnk" "$INSTDIR\Uninstall.exe"
    CreateShortcut "$DESKTOP\Tremors Music.lnk" "$INSTDIR\tremors-music.exe"
    WriteRegStr HKCU "${UNINSTALL_KEY}" "DisplayName" "${APP_NAME}"
    WriteRegStr HKCU "${UNINSTALL_KEY}" "DisplayVersion" "${APP_VERSION}"
    WriteRegStr HKCU "${UNINSTALL_KEY}" "Publisher" "Tremors"
    WriteRegStr HKCU "${UNINSTALL_KEY}" "InstallLocation" "$INSTDIR"
    WriteRegStr HKCU "${UNINSTALL_KEY}" "DisplayIcon" "$INSTDIR\tremors-music.exe,0"
    WriteRegStr HKCU "${UNINSTALL_KEY}" "UninstallString" '$\"$INSTDIR\Uninstall.exe$\"'
    WriteRegStr HKCU "${UNINSTALL_KEY}" "QuietUninstallString" '$\"$INSTDIR\Uninstall.exe$\" /S'
    WriteRegStr HKCU "${UNINSTALL_KEY}" "URLInfoAbout" "https://github.com/qtremors/tremors-music"
    WriteRegDWORD HKCU "${UNINSTALL_KEY}" "EstimatedSize" ${INSTALLED_SIZE_KB}
    WriteRegDWORD HKCU "${UNINSTALL_KEY}" "NoModify" 1
    WriteRegDWORD HKCU "${UNINSTALL_KEY}" "NoRepair" 1
SectionEnd

Function un.onInit
    SetRegView 64
    SetShellVarContext current
FunctionEnd

Section "Uninstall"
    Delete "$DESKTOP\Tremors Music.lnk"
    Delete "$SMPROGRAMS\Tremors Music\Tremors Music.lnk"
    Delete "$SMPROGRAMS\Tremors Music\Uninstall Tremors Music.lnk"
    RMDir "$SMPROGRAMS\Tremors Music"
    DeleteRegKey HKCU "${UNINSTALL_KEY}"
    Delete "$INSTDIR\tremors-music.exe"
    Delete "$INSTDIR\LICENSE.md"
    Delete "$INSTDIR\README.md"
    Delete "$INSTDIR\DEVELOPMENT.md"
    Delete "$INSTDIR\CHANGELOG.md"
    Delete "$INSTDIR\TASKS.md"
    Delete "$INSTDIR\PRIVACY.md"
    Delete "$INSTDIR\SOURCE.txt"
    Delete "$INSTDIR\NSIS-LICENSES.txt"
    Delete "$INSTDIR\Uninstall.exe"
    RMDir "$INSTDIR"
SectionEnd
