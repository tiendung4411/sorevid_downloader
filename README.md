# BiliBili Downloader

Desktop video/audio downloader powered by `yt-dlp`, built with Tauri v2, React, TypeScript, and Rust.

## Phase 1 Features

- Paste one or more URLs.
- Choose a download folder.
- Download as best video, audio-only MP3, or video-only.
- Use Chrome cookies with `--cookies-from-browser chrome`.
- Import a local `cookies.txt` file with `--cookies`.
- Stream yt-dlp progress, speed, ETA, logs, completion/failure state.
- Cancel active download jobs.
- Resolve `yt-dlp`, `ffmpeg`, and `ffprobe` from bundled sidecars first, then PATH.

## Setup

Install dependencies:

```bash
npm install
```

Fetch macOS sidecars:

```bash
npm run fetch:sidecars:mac
```

Fetch Windows sidecars on Windows:

```powershell
npm run fetch:sidecars:win
```

Run the desktop app:

```bash
npm run dev
```

Run checks:

```bash
npm run build
npm run lint
cd src-tauri && cargo test
```

## Cookie Modes

- `No cookies`: public URLs only.
- `Chrome`: passes `--cookies-from-browser chrome`; sign in with Chrome first.
- `Manual`: choose a local `cookies.txt`; the file is passed to local `yt-dlp` and is not uploaded or synced.

## Sidecar Layout

macOS binaries are expected here:

```text
src-tauri/resources/bin/macos/yt-dlp
src-tauri/resources/bin/macos/ffmpeg
src-tauri/resources/bin/macos/ffprobe
```

The resolver is OS-bucketed so Windows can later use:

```text
src-tauri/resources/bin/windows/yt-dlp.exe
src-tauri/resources/bin/windows/ffmpeg.exe
src-tauri/resources/bin/windows/ffprobe.exe
```
# sorevid_downloader
