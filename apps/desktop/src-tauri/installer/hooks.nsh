!macro NSIS_HOOK_POSTINSTALL
  CreateShortCut "$SENDTO\Muniko Share.lnk" "$INSTDIR\${MAINBINARYNAME}.exe" "--send"
!macroend

!macro NSIS_HOOK_POSTUNINSTALL
  Delete "$SENDTO\Muniko Share.lnk"
!macroend
