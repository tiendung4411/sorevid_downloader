# AI UX/UI Generation Instruction: BiliBili Downloader (CapCut-Inspired Theme)

Use this instruction file to generate, design, or refactor the UX/UI of the **BiliBili Downloader** desktop application with a visual identity heavily inspired by the **CapCut Desktop Client**.

---

## 1. Visual Theme & Aesthetic Direction (CapCut System)

The UI must feel **ultra-sleek, professional, dark-mode first, and content-centric**, matching the native CapCut desktop application.

### A. Color Palette
* **Main Canvas Background:** Deep Charcoal / Pitch Black (`#0B0B0C`).
* **Section / Card Background:** Slate-Grey (`#18181A` or `#1E1E20`).
* **Input / Inner Controls Background:** Jet Black (`#121213` or `#141415`).
* **Primary Text:** High-contrast pure White (`#FFFFFF`).
* **Secondary / Muted Text:** Soft Slate-Grey (`#8E8E93` or `#A1A1A6`).
* **Success State:** Neon Teal/Mint (`#00C4B4` or `#10B981`).
* **Warning State:** Sunset Orange/Gold (`#FF9F0A` or `#E6A23C`).
* **Error / Destructive State:** Ruby Red (`#FF453A` or `#F56C6C`).

### B. The Signature CapCut Gradient Accents
* **Hero Banner / Key Accents:** A smooth horizontal linear gradient stretching from **Teal/Dark Cyan** (`#00A29A`) on the left to **Crimson/Burgundy** (`#B50E3E` or `#C91D4C`) on the right.
* Use this gradient for:
  * A prominent top title banner card (similar to CapCut's "New Project / Dự án mới" button block).
  * Hover states of critical action triggers.

### C. Controls & Typography
* **Buttons:**
  * **Primary Buttons:** Pure White solid backgrounds with Black text (highly readable, pill-shaped, rounded-full or rounded-lg).
  * **Secondary / Path Buttons:** Solid Charcoal backgrounds (`#2C2C2E` or `#28282A`) with White text.
  * **Hover States:** Slight brightness increase (e.g., `#2C2C2E` transitions to `#3A3A3C` on hover) with crisp scale transitions.
* **Badges:** Compact, dark solid backgrounds with colored borders or tiny solid color dots for statuses.
* **Typography:** Clean, geometric sans-serif (Inter, Outfit, or system-ui). Monospace family (JetBrains Mono or SFMono) reserved strictly for terminal logs.

---

## 2. Layout Grid & Structure

The interface must implement a **Split-Screen Desktop Layout (Two Columns)**:

1. **Left Side (Main Control Panel - ~60% to 65% width):**
   * **Hero Header Card:** A prominent banner with the App Title "BiliBili Downloader" and subhead "YT-DLP DESKTOP CLIENT" overlaid on a **Teal-to-Crimson Gradient** background. Includes the diagnostic badges and the "Check Tools" button.
   * **URL Input Area:** Seamless textarea embedded in a dark container (`#121213`) with no borders, only active focus rings.
   * **Metadata Preflight Result Grid:** Renders video previews using a CapCut-like thumbnail grid.
   * **Configuration Panel:** Segmented format presets and destination folder picker.
   * **Session & Cookies Controls:** Input buttons to import files and toggle browser cookies.
   * **Notice Alerts & Validation Error banner.**
   * **Primary Execution Row:** URL Counter and the main "Start Download" trigger.
2. **Right Side (Download Queue Pane - ~35% to 40% width):**
   * Persistently pinned on the right side.
   * Separated visually by a clean, dark divider line (`#222224`).
   * Renders active, completed, or failed download jobs with live-updating log terminals.
   * Empty state placeholder (e.g., "No downloads yet") rendered with a minimalist icon when no downloads are queued.

---

## 3. Detailed Component Specs & Interactive Behaviors

### Component A: Header & System Diagnostics (Hero Banner)
* **Visual Card:** Renders as a wide, rounded horizontal banner featuring the Teal-to-Crimson gradient background.
* **Content:** App title "BiliBili Downloader" in bold white text, alongside an uppercase subtitle: "YT-DLP DESKTOP CLIENT".
* **Check Tools Button:** Compact icon button (`RefreshCw` or `Loader2`) aligned on the right. Spins when checking binaries.
* **Diagnostics Badges:** Three badges (yt-dlp, ffmpeg, ffprobe) with subtle transparent-dark backgrounds:
  * **Found State:** Displays version/path with a Teal checkmark dot.
  * **Missing State:** Displays error info with a Red warning dot.

### Component B: URL Input & Preflight Metadata Preview
* **Textarea Input:** Large text area for pasting one or more URLs, styled to blend into the dark container background (`#121213`).
* **Metadata Preview Row:**
  * **Preview Button:** Triggers fetching details (`fetch_metadata` command).
  * **Preview Status:** Context text next to it ("Run preview before downloading protected links" or "X previews ready").
* **Metadata Card List (CapCut Project Grid Style):**
  * Displays previewed media as cards matching the CapCut project list.
  * Each card has:
    * **16:9 Thumbnail:** Aspect-ratio locked image with rounded corners.
    * **Metadata Overlays:** Duration badge (e.g., `12:43`) in the corner of the thumbnail.
    * **Title Block:** Muted white text below the thumbnail, with uploader name and download format count shown in small grey text underneath.
    * Codecs details shown on hover.

### Component C: Download Configurations & Presets
* **Download Folder Picker:**
  * Label: "Download folder" in Slate-Grey.
  * Selector: Solid Charcoal button (`#28282A`) displaying a folder icon (`FolderOpen`) and the selected path.
* **Format Preset Selector:**
  * Label: "Format preset".
  * **Segmented Control:** Grouped buttons inside a dark track (`#121213`). The active preset highlights in solid light-grey (`#3A3A3C`) or white, displaying five options: MP4, Best, Audio, Video, and Original.

### Component D: Authentication & Cookies Configuration
* **Cookie Mode Selector:**
  * Segmented buttons inside a dark track (`#121213`). Options: None, Chrome, Manual.
* **Manual Import Button:**
  * Renders a Charcoal button (`#28282A`) labeled "Import cookies.txt" with a file icon. Shows the file path once loaded.

### Component E: Notices & Error Alert Banners
* **Security Notice:**
  * Styled as a flat notice block with a warning symbol.
  * Automatically switches text depending on whether sensitive hosts (BiliBili/Douyin) are input.
* **Error Banner:**
  * Red outline container displaying application errors with high-contrast warning text.

### Component F: Main Action Bar
* **Left:** Count of prepared URLs (e.g., "3 URLs ready").
* **Right:** Large pill-shaped **"Start download"** button (Solid White background, Black text, bold typography).

### Component G: Download Queue Pane & Job Items
* **Header:** Icon (`Terminal`) + Title "Queue" or "Download Monitor" in White.
* **Empty State:** Minimalist text and icon aligned in a dark, dashed layout card.
* **Job Item Cards:**
  * Styled as clean dark cards (`#18181A`) with thin subtle borders reflecting status (Teal for completed, Red for failed, Charcoal for running/queued).
  * **Progress Tracker:**
    * A progress bar track with a solid Teal progress fill.
    * Live percentage, download speed, and ETA shown in soft slate-grey text.
  * **Log Terminal:** Monospace console shell emulator (`pre` tag) displaying white text on a pitch black background (`#000000`).
  * **Media Report Summary:** Renders file size, resolution, and audio/video codecs.
    * **QuickTime Compatibility Check:**
      * **Compatible State:** Success icon + Teal text indicating native macOS compatibility.
      * **Incompatible Warning State:** Warning icon + Gold text advising that the video requires conversion to play natively on Mac.

### Component H: Post-Download Context Actions
* Displayed inside the completed Job Card as a horizontal group of pill buttons:
  1. **Open File:** Charcoal pill (`#28282A`) with a document icon.
  2. **Open Folder:** Charcoal pill (`#28282A`) with a folder icon.
  3. **Convert H.264:** Solid White pill button with Black text. When clicked, displays a loading spinner and converts incompatible files to macOS-compatible formats.

---

## 4. UI States & Transitions

* **Active Progress:** The progress bar must fill smoothly (`transition: width 180ms ease`).
* **Animations:** Spin animations on loaders; hover scaling triggers on all interactive cards.
* **Responsive Layout:**
  * Desktop widths: Two-column grid (60/40 Split).
  * Tablet/Mobile widths: Single vertical column, collapsing side panels.

---

## 5. Frontend Models / Data Schema

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
