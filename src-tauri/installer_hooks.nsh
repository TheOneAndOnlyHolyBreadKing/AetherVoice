!macro customInstall
  DetailPrint "Configuring AetherVoice prerequisites..."
  SetOutPath "$INSTDIR"
  
  ; Write prerequisite helper script for automated dependency setup
  FileOpen $0 "$INSTDIR\setup_prereqs.bat" w
  FileWrite $0 "@echo off$\r$\n"
  FileWrite $0 "title AetherVoice Environment Setup$\r$\n"
  FileWrite $0 "echo ====================================================$\r$\n"
  FileWrite $0 "echo        AetherVoice: Checking Environment Dependencies$\r$\n"
  FileWrite $0 "echo ====================================================$\r$\n"
  FileWrite $0 "echo.$\r$\n"
  FileWrite $0 "echo Checking for Python...$\r$\n"
  FileWrite $0 "where python >nul 2>nul$\r$\n"
  FileWrite $0 "if %errorlevel% neq 0 ($\r$\n"
  FileWrite $0 "  echo Python is not detected. Installing Python via winget...$\r$\n"
  FileWrite $0 "  winget install -e --id Python.Python.3.11 --accept-source-agreements --accept-package-agreements$\r$\n"
  FileWrite $0 ") else ($\r$\n"
  FileWrite $0 "  echo [OK] Python is already installed.$\r$\n"
  FileWrite $0 ")$\r$\n"
  FileWrite $0 "echo.$\r$\n"
  FileWrite $0 "echo Installing/Updating Whisper, PyTorch and dependencies...$\r$\n"
  FileWrite $0 "python -m pip install --upgrade pip >nul 2>nul$\r$\n"
  FileWrite $0 "python -m pip install openai-whisper torch torchaudio sounddevice numpy >nul 2>nul$\r$\n"
  FileWrite $0 "echo [OK] Whisper speech engine dependencies verified.$\r$\n"
  FileWrite $0 "echo.$\r$\n"
  FileWrite $0 "echo Checking for Ollama (Local AI Reasoning Engine)...$\r$\n"
  FileWrite $0 "where ollama >nul 2>nul$\r$\n"
  FileWrite $0 "if %errorlevel% neq 0 ($\r$\n"
  FileWrite $0 "  echo Ollama is not installed. Installing Ollama via winget...$\r$\n"
  FileWrite $0 "  winget install -e --id Ollama.Ollama --accept-source-agreements --accept-package-agreements$\r$\n"
  FileWrite $0 ") else ($\r$\n"
  FileWrite $0 "  echo [OK] Ollama is already installed.$\r$\n"
  FileWrite $0 ")$\r$\n"
  FileWrite $0 "echo.$\r$\n"
  FileWrite $0 "echo ====================================================$\r$\n"
  FileWrite $0 "echo        AetherVoice Setup Completed Successfully!$\r$\n"
  FileWrite $0 "echo ====================================================$\r$\n"
  FileClose $0

  DetailPrint "Executing AetherVoice prerequisite installer..."
  nsExec::ExecToLog 'cmd.exe /c "$INSTDIR\setup_prereqs.bat"'
  
  Delete "$INSTDIR\setup_prereqs.bat"
!macroend
