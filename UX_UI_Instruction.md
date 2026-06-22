# AI UX/UI Generation Instruction: BiliBili Downloader

Use this instruction file to generate, design, or refactor the UX/UI of the **BiliBili Downloader** desktop application. The goal is to design the interface components, structural layout, interactive flows, and data bindings. You have complete freedom to choose the visual theme, colors, typography, and styling language (e.g., minimalist, brutalist, high-tech dashboard, light/dark mode) that best fits the application.

---

## 1. System Overview & Context

* **App Name:** BiliBili Downloader (yt-dlp Desktop Client)
* **Target Environment:** Tauri v2 Desktop App (runs natively on macOS, Windows, and Linux).
* **Technology Stack:** React + TypeScript + Tauri + Lucide Icons.
* **Core Value Proposition:** A local-first media downloader that handles multiple URLs, retrieves metadata previews before downloading, supports session authentication (cookies), streams live logs from the backend, and transcodes incompatible codecs (e.g., VP9/AV1) into Apple-friendly (QuickTime compatible) H.264 MP4 formats.

---

## 2. Layout Grid & Structure

The interface must implement a **Split-Screen Layout (Two Columns)** to organize the control panel and active operations:

1. **Left Side (Main Control Panel - ~60% to 65% width):**
   * **Header & App Title** + System diagnostic status badges.
   * **URL Input Area** with Metadata Preflight controls.
   * **Metadata Preflight Result List** (dynamic height, scrolling container).
   * **Configuration Panel** (Folder picker, Format presets).
   * **Session & Cookies Controls** (Source selection, manual import).
   * **Notice Alerts & Validation Error banner**.
   * **Primary Execution Row** (URL Counter + Start Download action).
2. **Right Side (Download Queue Pane - ~35% to 40% width):**
   * Persistently pinned on the side (collapsible on narrow screens).
   * Renders active, completed, or failed download jobs with live-updating log terminals.
   * Empty state placeholder when no downloads are queued.

---

## 3. Detailed Component Specs & Interactive Behaviors

### Component A: Header & System Diagnostics
* **Title Group:** App title "BiliBili Downloader" with an uppercase subhead/eyebrow: "YT-DLP DESKTOP CLIENT".
* **Check Tools Button:** An action button (`RefreshCw` or `Loader2`) that triggers tool checking. Animates/spins the icon when resolving binaries.
* **Diagnostics Badges:** Three badges side-by-side: `yt-dlp`, `ffmpeg`, and `ffprobe`.
  * **Found State:** Success indicator, checkmark icon (`CheckCircle2`), display name, and version number.
  * **Missing/Error State:** Error/Warning indicator, warning icon (`AlertTriangle`), displaying "Not found" or error details.

### Component B: URL Input & Preflight Metadata Preview
* **Textarea Input:** Large text area for pasting one or more URLs (video, playlist, or audio), separated by line breaks.
* **Metadata Preview Row:**
  * **Preview Button:** Triggers fetching details (`fetch_metadata` command) of the pasted links without starting the actual download.
  * **Preview Status:** Context text next to it (e.g., "Run preview before downloading protected links" or "X previews ready").
* **Metadata Card List:** Renders when preview metadata is available.
  * A scrollable list or a grid of compact cards.
  * Each card must show:
    * Video thumbnail (fallback placeholder container with aspect ratio `16:9` if none).
    * Title (trimmed text with ellipsis).
    * Badges for the platform (e.g., BiliBili, Douyin, YouTube).
    * Media stats: Duration (formatted `MM:SS` or `H:MM:SS`), Uploader name, and count of available download formats.
    * Codecs Row: Supported video and audio codecs (e.g., "V: avc1, vp9", "A: mp4a, opus").
    * Sensitive host warnings (if any).

### Component C: Download Configurations & Presets
* **Download Folder Picker:**
  * A label "Download folder".
  * A wide button with a folder icon (`FolderOpen`). It launches a native system dialog. Once selected, it displays the path. If empty, displays "Choose folder".
* **Format Preset Selector:**
  * A label "Format preset".
  * A **Segmented Control** (horizontal buttons grouped together) with 5 options:
    1. **MP4:** Optimized for compatibility. Forces H.264 video + AAC audio (`compatibleMp4`).
    2. **Best:** Highest possible resolution and quality, regardless of codec (`bestQuality`).
    3. **Audio:** Extract and convert to MP3 audio format only (`audioOnly`).
    4. **Video:** Download video stream only (`videoOnly`).
    5. **Original:** Retain original formats and containers without merging/transcoding (`originalCodec`).

### Component D: Authentication & Cookies Configuration
* **Cookie Mode Selector:**
  * A segmented control with options: `None` (Public URLs), `Chrome` (Extract cookies directly from Chrome), and `Manual` (Import Netscape format cookie text files).
* **Manual Import Button:**
  * Appears alongside or active only when `Manual` cookie mode is selected.
  * A button labeled "Import cookies.txt" with file icon. Opens file dialog filters for `.txt` extensions. Once selected, shows the file path.

### Component E: Notice, Security, & Error Alert Banners
* **Context Warning Banner:**
  * Displays warning notices about session validation.
  * If a URL containing BiliBili or Douyin is pasted, it automatically warns: *"BiliBili/Douyin detected. Use Chrome cookies or import cookies.txt before downloading."*
  * Otherwise, shows a generic privacy assurance message: *"Cookie files are sensitive. This app passes them only to local yt-dlp and does not upload or sync them."*
* **Error Banner:**
  * Displayed right above the main download button.
  * Highlights API errors, missing directories, or authentication failures in an alert container with a warning symbol.

### Component F: Main Action Bar
* **Left:** Count of prepared URLs (e.g., "3 URLs ready").
* **Right:** Action trigger button **"Start download"** with a download icon (`Download`). Clicking this pushes jobs into the Queue Pane.

### Component G: Download Queue Pane & Job Items
* **Header:** Icon (`Terminal`) + Title "Queue" or "Download Monitor".
* **Empty State:** Text ("No downloads yet.") inside a dashed layout placeholder container.
* **Job Item Cards:** Cards representing current downloads. Each card must visually distinguish its state:
  * **Visual State Mapping:** Differentiate states (`queued`, `starting`, `running`, `warning`, `completed`, `failed`, `canceled`) using distinct visual boundaries, icons, or subtle accents.
  * **Header:** Shows the primary URL (with a tag "+ X more" if downloading in batches), alongside the textual status indicator.
  * **Progress Tracker:**
    * A progress bar track showing download status.
    * Metadata displaying `% percentage`, speed (`MB/s` or `KB/s`), and `ETA`.
  * **Log Terminal:** A monospace console shell emulator (`pre` tag) displaying the last 8 lines of active output logs from `yt-dlp`.
  * **Media Report Summary:** Renders for finished downloads. Shows container details, resolution (e.g., `1920x1080`), file size (e.g., `85.4 MB`), and audio/video codecs.
    * **QuickTime Compatibility Check:**
      * **Compatible State:** Shows video tag/codec + audio tag/codec (e.g. `avc1 / mp4a`) with a success indicator.
      * **Incompatible Warning State:** Highlights codecs not supported by native macOS QuickLook/QuickTime (e.g., `vp9 / opus`), prompting the user that it may not play natively on Mac.

### Component H: Post-Download Context Actions
Inside the completed Job Card, show these buttons:
1. **Open File (`FileText` icon):** Launches/plays the downloaded file in the OS default player.
2. **Open Folder (`FolderOpen` icon):** Opens the parent folder containing the file.
3. **Convert H.264 Button:**
   * Only appears if the completed job's media report indicates `quicktimeCompatible` is **false**.
   * Clicking this runs a background FFmpeg transcode (`convert_to_h264`) converting it to H.264 AVC1 + AAC MP4.
   * During conversion, display a loading indicator. Once done, it updates the media report card to "Compatible" status and updates the file path.

---

## 4. UI States & Transitions

Ensure smooth states handling:
* **Idle:** Clean states with placeholders.
* **Loading/Checking:** Loading indicators on tool badge refreshing, metadata preflighting, and conversion processes.
* **Active Progress:** The progress bar must transition widths smoothly.
* **Interactive Feedback:** Consistent hover, focus, and active states for all clickable elements.
* **Responsive Layout:**
  * Desktop width: Two-column side-by-side layout.
  * Tablet/Mobile width: Vertically stacked layout (Main console top, Queue pane bottom). Grid columns collapse to 1 column.

---

## 5. Frontend Models / Data Schema

These are the exact TypeScript models driving the UI. AI UX/UI generation tools should model fields around these types:

```typescript
type DownloadPreset = 'compatibleMp4' | 'bestQuality' | 'audioOnly' | 'videoOnly' | 'originalCodec';
type CookieMode = 'none' | 'chrome' | 'manual';
type JobStatus = 'queued' | 'starting' | 'running' | 'warning' | 'completed' | 'failed' | 'canceled';

type ToolStatus = {
  found: boolean;
  path?: string;
  version?: string;
  error?: string;
};

type ToolVersions = {
  ytDlp: ToolStatus;
  ffmpeg: ToolStatus;
  ffprobe: ToolStatus;
};

type MetadataPreview = {
  url: string;
  title?: string;
  thumbnail?: string;
  duration?: number;
  uploader?: string;
  platform: string;
  webpageUrl?: string;
  formatCount: number;
  videoCodecs: string[];
  audioCodecs: string[];
  requiresSession: boolean;
  warning?: string;
};

type MediaReport = {
  path: string;
  fileSize?: number;
  container?: string;
  duration?: number;
  videoCodec?: string;
  videoTag?: string;
  audioCodec?: string;
  audioTag?: string;
  width?: number;
  height?: number;
  quicktimeCompatible: boolean;
  warning?: string;
};

type DownloadJob = {
  id: string;
  urls: string[];
  status: JobStatus;
  percent?: number;
  speed?: string;
  eta?: string;
  logs: string[];
  outputPath?: string;
  mediaReport?: MediaReport;
  converting?: boolean;
};
```
