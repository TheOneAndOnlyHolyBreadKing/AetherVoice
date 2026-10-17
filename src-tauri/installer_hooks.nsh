!macro NSIS_HOOK_PREINSTALL
  DetailPrint "Checking existing AetherVoice user configuration and data..."
  
  ; Check if previous installation data exists in either primary or backup location
  IfFileExists "$APPDATA\com.aethervoice.app\aethervoice_config.json" found_existing_data 0
  IfFileExists "$APPDATA\AetherVoice\aethervoice_config.json" found_existing_data check_done

found_existing_data:
  ; If running silent or passive, always preserve user data by default
  ${If} $PassiveMode = 1
  ${OrIf} ${Silent}
    CreateDirectory "$APPDATA\AetherVoice\backup"
    CopyFiles /SILENT "$APPDATA\com.aethervoice.app\*.json" "$APPDATA\AetherVoice\backup\"
    CopyFiles /SILENT "$APPDATA\com.aethervoice.app\*.json" "$APPDATA\AetherVoice\"
    DetailPrint "Silent/Passive mode: Preserved user data and configurations."
    Goto check_done
  ${EndIf}

  ; Ask user if they wish to preserve their data or perform a clean wipe
  MessageBox MB_YESNO|MB_ICONQUESTION "Existing AetherVoice configuration and downloaded models were detected.$\r$\n$\r$\nDo you want to keep your existing settings, dictation history, and confirmed models?$\r$\n$\r$\n(Click 'Yes' to preserve all user data, or 'No' to clear all data for a clean install)" /SD IDYES IDYES preserve_user_data IDNO wipe_user_data

wipe_user_data:
  DetailPrint "User requested clean install: Clearing existing application data..."
  Delete "$APPDATA\com.aethervoice.app\*.json"
  Delete "$APPDATA\AetherVoice\*.json"
  Delete "$APPDATA\AetherVoice\backup\*.json"
  Goto check_done

preserve_user_data:
  DetailPrint "Preserving existing settings, dictation history, and confirmed models..."
  CreateDirectory "$APPDATA\AetherVoice\backup"
  CopyFiles /SILENT "$APPDATA\com.aethervoice.app\*.json" "$APPDATA\AetherVoice\backup\"
  CopyFiles /SILENT "$APPDATA\com.aethervoice.app\*.json" "$APPDATA\AetherVoice\"
  Goto check_done

check_done:
!macroend

!macro NSIS_HOOK_POSTINSTALL
  DetailPrint "Configuring AetherVoice environment and prerequisites..."
  SetOutPath "$INSTDIR"

  ; Restore persistent user data if primary app directory was created afresh
  CreateDirectory "$APPDATA\com.aethervoice.app"
  IfFileExists "$APPDATA\com.aethervoice.app\aethervoice_config.json" skip_restore 0
    CopyFiles /SILENT "$APPDATA\AetherVoice\backup\*.json" "$APPDATA\com.aethervoice.app\"
    CopyFiles /SILENT "$APPDATA\AetherVoice\*.json" "$APPDATA\com.aethervoice.app\"
skip_restore:
  
  ; Write prerequisite helper script for automated dependency setup
  FileOpen $0 "$INSTDIR\setup_prereqs.bat" w
  FileWrite $0 "@echo off$\r$\n"
  FileWrite $0 "title AetherVoice Environment Setup$\r$\n"
  FileWrite $0 "echo ====================================================$\r$\n"
  FileWrite $0 "echo        AetherVoice: Preparing AI Dependencies$\r$\n"
  FileWrite $0 "echo ====================================================$\r$\n"
  FileWrite $0 "echo.$\r$\n"
  FileWrite $0 "echo [1/3] Checking for Python 3.10+...$\r$\n"
  FileWrite $0 "where python >nul 2>nul$\r$\n"
  FileWrite $0 "if %errorlevel% neq 0 ($\r$\n"
  FileWrite $0 "  echo Python was not found in PATH. Installing Python via winget...$\r$\n"
  FileWrite $0 "  winget install -e --id Python.Python.3.11 --accept-source-agreements --accept-package-agreements$\r$\n"
  FileWrite $0 ") else ($\r$\n"
  FileWrite $0 "  echo Python is installed.$\r$\n"
  FileWrite $0 ")$\r$\n"
  FileWrite $0 "echo.$\r$\n"
  FileWrite $0 "echo [2/3] Installing Whisper speech recognition and PyTorch...$\r$\n"
  FileWrite $0 "echo (This guarantees one-click offline voice recognition)$\r$\n"
  FileWrite $0 "python -m pip install --upgrade pip$\r$\n"
  FileWrite $0 "python -m pip install openai-whisper torch torchaudio sounddevice numpy$\r$\n"
  FileWrite $0 "if %errorlevel% neq 0 ($\r$\n"
  FileWrite $0 "  echo Pip install with standard python had an issue, trying py -m pip...$\r$\n"
  FileWrite $0 "  py -m pip install openai-whisper torch torchaudio sounddevice numpy$\r$\n"
  FileWrite $0 ")$\r$\n"
  FileWrite $0 "echo [OK] Whisper speech engine dependencies ready.$\r$\n"
  FileWrite $0 "echo.$\r$\n"
  FileWrite $0 "echo [3/3] Checking Ollama local reasoning engine...$\r$\n"
  FileWrite $0 "where ollama >nul 2>nul$\r$\n"
  FileWrite $0 "if %errorlevel% neq 0 ($\r$\n"
  FileWrite $0 "  echo Installing Ollama via winget...$\r$\n"
  FileWrite $0 "  winget install -e --id Ollama.Ollama --accept-source-agreements --accept-package-agreements$\r$\n"
  FileWrite $0 ") else ($\r$\n"
  FileWrite $0 "  echo Ollama is already installed.$\r$\n"
  FileWrite $0 ")$\r$\n"
  FileWrite $0 "echo.$\r$\n"
  FileWrite $0 "echo ====================================================$\r$\n"
  FileWrite $0 "echo        Setup Complete! Launching AetherVoice...$\r$\n"
  FileWrite $0 "echo ====================================================$\r$\n"
  FileClose $0

  DetailPrint "Installing Whisper and AI prerequisites (showing setup console window)..."
  ; Execute synchronously with visible console window so the user sees pip and winget progress
  ExecWait 'cmd.exe /c "$INSTDIR\setup_prereqs.bat"'
  
  Delete "$INSTDIR\setup_prereqs.bat"
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  ; Preserve backup of user configurations unless user explicitly deletes them
  CreateDirectory "$APPDATA\AetherVoice\backup"
  CopyFiles /SILENT "$APPDATA\com.aethervoice.app\*.json" "$APPDATA\AetherVoice\backup\"
  CopyFiles /SILENT "$APPDATA\com.aethervoice.app\*.json" "$APPDATA\AetherVoice\"
!macroend
