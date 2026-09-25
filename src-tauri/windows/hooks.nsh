!macro NSIS_HOOK_PREUNINSTALL
  nsExec::ExecToLog 'taskkill /IM Adam.exe /T /F'
  Sleep 1000
!macroend

!macro NSIS_HOOK_POSTUNINSTALL
  ; AppData can still be briefly locked by WebView2/child processes after Adam.exe exits.
  ; Retry removal before the NSIS uninstaller terminates.
  StrCpy $0 0
loop:
  RMDir /r "$APPDATA\\com.saeedhub.adam"
  RMDir /r "$LOCALAPPDATA\\com.saeedhub.adam"
  RMDir /r "$LOCALAPPDATA\\Adam"
  IntOp $0 $0 + 1
  ${If} $0 < 5
    Sleep 1000
    Goto loop
  ${EndIf}
!macroend
