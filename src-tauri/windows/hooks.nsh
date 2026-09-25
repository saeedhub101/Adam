!macro NSIS_HOOK_PREUNINSTALL
  nsExec::ExecToLog 'taskkill /IM Adam.exe /T /F'
!macroend

!macro NSIS_HOOK_POSTUNINSTALL
  RMDir /r "$APPDATA\\com.saeedhub.adam"
  RMDir /r "$LOCALAPPDATA\\com.saeedhub.adam"
  RMDir /r "$LOCALAPPDATA\\Adam"
!macroend
