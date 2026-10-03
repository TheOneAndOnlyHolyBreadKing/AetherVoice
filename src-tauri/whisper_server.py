import sys
import os
import json
import torch
import whisper
import numpy as np

def main():
    device = "cuda" if torch.cuda.is_available() else "cpu"
    sys.stderr.write(f"[whisper_server] Using device: {device}\n")
    sys.stderr.flush()

    # Default to turbo model (already cached and tested)
    model_name = sys.argv[1] if len(sys.argv) > 1 else "turbo"
    sys.stderr.write(f"[whisper_server] Loading model '{model_name}'...\n")
    sys.stderr.flush()

    try:
        model = whisper.load_model(model_name, device=device)
        sys.stderr.write("[whisper_server] Model loaded successfully.\n")
        sys.stderr.flush()
    except Exception as e:
        sys.stderr.write(f"[whisper_server] Error loading model: {e}\n")
        sys.stderr.flush()
        # Fallback to small or base if turbo fails
        try:
            model = whisper.load_model("small", device=device)
        except Exception as e2:
            model = whisper.load_model("base", device="cpu")

    # Signal ready to Rust parent process
    print("READY", flush=True)

    # Process audio transcription requests over stdin line by line
    for line in sys.stdin:
        line = line.strip()
        if not line:
            continue
        if line == "PING":
            print("PONG", flush=True)
            continue
        if line == "QUIT":
            break

        wav_path = line
        try:
            if not os.path.exists(wav_path):
                resp = json.dumps({"status": "error", "error": f"File not found: {wav_path}"})
                print(resp, flush=True)
                continue

            # Run transcription with fp16 when on CUDA for speed
            result = model.transcribe(
                wav_path,
                fp16=(device == "cuda"),
                language="en",
                temperature=0.0
            )

            text = result.get("text", "").strip()
            resp = json.dumps({"status": "ok", "text": text})
            print(resp, flush=True)
        except Exception as err:
            sys.stderr.write(f"[whisper_server] Transcription error: {err}\n")
            sys.stderr.flush()
            resp = json.dumps({"status": "error", "error": str(err)})
            print(resp, flush=True)

if __name__ == "__main__":
    main()
