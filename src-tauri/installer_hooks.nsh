!macro customInstall
  DetailPrint "Configuring AetherVoice environment and prerequisites..."
  SetOutPath "$INSTDIR"
  
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
