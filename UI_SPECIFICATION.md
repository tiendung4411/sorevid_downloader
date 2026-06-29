# BiliBili Downloader - Functional UI Specification

> Tài liệu mô tả chức năng giao diện cho team Design. Không bao gồm chi tiết về màu sắc, font chữ, khoảng cách - hãy tự do sáng tạo!

---

## 📐 Cấu trúc tổng thể

### Layout chính
- **Workspace Area** (trái): Nơi làm việc chính, chứa tabs Download và Settings
- **Jobs Pane** (phải): Sidebar hiển thị danh sách downloads đang chạy
- **Bottom Action Bar** (dưới cùng): Thanh cố định với các nút hành động chính

### Responsive
- Desktop: 2 cột (workspace + jobs pane)
- Mobile: 1 cột dọc, jobs pane xuống dưới workspace

---

## 🎯 Màn hình Download Tab

### 1. Topbar
**Chức năng:**
- Hiển thị tên app: "BiliBili Downloader"
- Hiển thị mô tả: "yt-dlp desktop client"
- Navigation tabs: "Download" | "Settings"
- Button: "Check tools" (refresh tool versions)

### 2. Tool Status Strip
**Chức năng:**
- Hiển thị 3 tools: yt-dlp, ffmpeg, ffprobe
- Mỗi tool có:
  - Icon trạng thái (✓ hoặc ✗)
  - Tên tool
  - Version number hoặc error message

### 3. URL Input Section
**Chức năng:**
- Label: "URLs"
- Textarea lớn cho phép paste nhiều URLs
- Placeholder: "Paste one or more video, playlist, or audio URLs..."
- Hiển thị số preview đã ready (ví dụ: "12 preview ready")

### 4. Metadata Preview List
**Khi nào hiển thị:** Sau khi click "Preview"

**Selection Bar:**
- Hiển thị: "X/Y selected"
- Button: "Select all"
- Button: "Select none"

**Metadata Card** (1 card cho mỗi video):
- Checkbox để chọn
- Thumbnail preview
- Thông tin video:
  - Title
  - Platform (BiliBili/TikTok/Douyin)
  - Duration
  - Uploader
  - Resolution
  - Format count
  - Video codecs
  - Audio codecs
  - Recommended preset
- Actions:
  - Button: "View cover"
  - Button: "Save cover"
  - Label: "Saved" (nếu đã lưu)
- Playlist info (nếu là playlist):
  - Playlist title
  - Part number (ví dụ: "Part 3/15")
- Warning messages (nếu có)

### 5. Batch Organizer
**Khi nào hiển thị:** Khi có nhiều videos trong preview

**Heading:**
- Title: "Batch organizer"
- Hiển thị: "X series groups from Y preview items"
- Dropdown: "Files: 1.mp4, 2.mp4" | "Files: full episode title"
- Dropdown: "As scanned" | "Reverse order" | "Sort by episode"
- Button: "Organize with AI" (có trạng thái loading)
- Button: "Copy AI prompt"

**Series Card** (1 card cho mỗi bộ phim):
- Checkbox chọn cả series
- Input: Series title (có thể edit)
- Hiển thị: "X/Y selected"
- Button: "Select"
- Button: "Clear"
- **Episode Table:**
  - Mỗi episode có:
    - Thumbnail nhỏ
    - Input: Episode number
    - Input: Episode title
    - Preview output filename
  - Hiển thị tối đa 8 episodes
  - Nếu > 8 episodes: hiển thị "+ X more episodes in this series"
- **Visual indicators:**
  - Episode bị pinned: highlight đặc biệt
  - Episode out-of-order: warning indicator

### 6. Notice Messages
**Các loại:**
- Warning: Khi có channel/profile URL
- Error: Hiển thị lỗi (màu đỏ)

---

## ⚙️ Màn hình Settings Tab

### 1. Audio Notification Settings
**Chức năng:**
- Heading: "Audio notifications"
- Description: "Play short sounds when downloads start, complete, fail, or need attention."
- Checkbox: "Enable download sounds"
- Button: "Test sound"

### 2. AI Organizer Settings
**Chức năng:**
- Heading: "AI organizer"
- Description: "Used by the Batch organizer button on the Download tab."
- Input: "Google AI Studio API key" (type password)
- Input: "Model" (default: gemini-3.1-flash-lite)

### 3. Output Settings
**Chức năng:**
- Heading: "Output"
- Description: "Folder and media format used when starting downloads."
- **Download folder:**
  - Button hiển thị path hoặc "Choose folder"
  - Click để mở folder picker
- **Format preset** (chọn 1 trong 5):
  - MP4 (compatible)
  - Best (quality)
  - Audio (only)
  - Video (only)
  - Original (codec)

### 4. Subtitles & Danmaku Settings
**Khi nào hiển thị:** Khi có BiliBili URL

**Chức năng:**
- Heading: "Subtitles & Danmaku"
- Description: chi tiết về subtitle và danmaku
- **Subtitles mode** (chọn 1 trong 4):
  - Off
  - Subs
  - Auto
  - Both
- **Subtitle format** (chọn 1 trong 2):
  - SRT
  - VTT
- **Danmaku format** (chọn 1 trong 3):
  - Off
  - XML
  - ASS
- Checkbox: "Embed subtitles into MP4 when possible" (disabled khi subtitle off)

### 5. Platform Cookie Profiles
**Chức năng:**
- Heading: "Platform cookie profiles"
- Description: "Detected protected platforms from the pasted URLs."
- Hiển thị platforms cần cookie: "Missing: BiliBili, Douyin"

**Cookie Profile Card** (1 card cho mỗi platform: BiliBili, Douyin, TikTok):
- Platform name
- Platform hosts
- Status: "Ready" hoặc "Needs session"
- **Cookie mode** (chọn 1 trong 3):
  - None
  - Chrome
  - Manual
- **Actions:**
  - Button: "Import [Platform] cookies.txt" (hoặc hiển thị path nếu đã import)
  - Button: "Export from Chrome"
  - Button: "Validate"
  - Button: "Delete" (danger style)
- **Cookie status** (nếu có file):
  - "Valid cookie file" hoặc "Invalid cookie file"
  - Số cookies
  - File size
  - Modified date

### 6. Chrome Integration Bridge
**Chức năng:**
- Heading: "Chrome Integration Bridge"
- Icon trạng thái: Installed | Invalid | Not installed
- Description: Status message
- Status label: "Installed" | "Invalid" | "Not installed"
- **Actions:**
  - Button: "Register bridge"
  - Button: "Test desktop bridge"
  - Button: "Remove bridge"
- **Instructions panel:**
  - Hướng dẫn cài Chrome extension
  - Hiển thị path: `chrome://extensions` và `dist-extension`

### 7. Warning Notice
- Icon warning
- Text: Thông báo về cookie security

---

## 📊 Jobs Pane (Queue Sidebar)

### Heading
- Icon: Terminal
- Title: "Queue"
- Button: "Clear all" (disabled khi queue rỗng)

### Empty State
- Message: "No downloads yet."

### Job Item Card
**Hiển thị cho mỗi download job:**

**Main info:**
- Job title (từ video title hoặc URL)
- "+ X more" (nếu nhiều videos)
- Status label: "queued" | "starting" | "running" | "warning" | "completed" | "failed" | "canceled"

**Batch Summary:**
- Số clips: "X/Y clips"
- Playlist title hoặc active title
- Progress bar (nếu đang chạy)

**Progress indicators:**
- Progress bar chính
- Percent: "45.3%"
- Speed: "2.5 MB/s"
- ETA: "ETA 3m 15s"

**Activity Log:**
- Heading: "Activity log" + "X lines"
- Console output (hiển thị 6 dòng cuối)

**Media Report** (sau khi completed):
- Video codec / Audio codec
- Resolution (width x height)
- File size
- QuickTime compatible: Yes/No
- Warning message (nếu có)

**Actions:**
- Button: "Open file"
- Button: "Open folder"
- Button: "Convert H.264" (nếu không compatible)
- Button: "Cancel" (khi đang chạy)

**Visual states:**
- Completed: border màu xanh
- Failed: border màu đỏ
- Running: progress animation

---

## 🎬 Bottom Action Bar

### Left Side
- Hiển thị: "X/Y selected" hoặc "X URLs ready"
- Hiển thị: Download folder path hoặc "No output folder selected"

### Right Side
- Button: "Settings" (secondary)
- Button: "Preview" (secondary, có loading state)
- Button: "Re-scan selected" (chỉ hiển thị khi có TikTok direct items)
- Button: "Start download" (primary, có loading state)

---

## 🖼️ Modal & Overlays

### Cover Viewer Modal
**Khi nào hiển thị:** Click "View cover" trên metadata card

**Chức năng:**
- Backdrop tối (click để đóng)
- Dialog chính:
  - Top bar:
    - Cover title
    - File path (nếu đã save)
    - Button: Close (X)
  - Image stage: Hiển thị cover ảnh to
- Keyboard: ESC để đóng

---

## 🔄 States & Interactions

### Loading States
- Button với spinner icon khi processing
- "Checking..." | "Starting..." | "Organizing..." text

### Disabled States
- Buttons disabled khi:
  - Không đủ điều kiện (ví dụ: chưa chọn folder)
  - Đang process
  - Dependencies chưa ready (ví dụ: ffmpeg missing)

### Error States
- Error panel màu đỏ hiển thị error message
- Có thể là validation error hoặc runtime error

### Success States
- Completed jobs: visual indicator xanh
- "Saved" label khi đã save cover
- "Valid cookie file" status

### Focus States
- Tất cả interactive elements cần có focus indicator rõ ràng

---

## 📱 Responsive Behavior

### Mobile (< 980px)
- Grid layout chuyển thành 1 cột dọc
- Jobs pane di chuyển xuống dưới workspace
- Bottom action bar:
  - Controls thành grid 2 cột
  - Summary stack dọc
- Forms & grids collapse thành 1 cột
- Batch organizer heading stack dọc

---

## 🎨 Design Freedom

### Bạn có thể tự do quyết định:
- Color scheme (light, dark, colorful, minimal)
- Typography (font family, sizes, weights)
- Spacing & padding
- Border radius & shadows
- Button styles (flat, gradient, 3D, ghost)
- Card styles (bordered, elevated, flat)
- Animation styles & transitions
- Icon style (outline, filled, duotone)
- Layout density (compact, comfortable, spacious)

### Hãy sáng tạo với:
- Glassmorphism
- Neumorphism
- Brutalism
- Material Design
- Fluent Design
- macOS style
- iOS style
- Gaming UI style
- Retro/vintage style
- Futuristic style
- Minimalist
- Maximalist

### Chỉ cần đảm bảo:
✅ Tất cả chức năng được liệt kê ở trên có đủ
✅ User có thể tìm thấy và sử dụng mọi controls
✅ States (loading, error, success) được phân biệt rõ ràng
✅ Responsive trên mobile vẫn usable
✅ Accessibility (contrast, focus, touch targets)

---

## 💡 Gợi ý thử nghiệm

1. **Thử color schemes khác nhau:**
   - Dark mode với accent màu neon
   - Pastel palette nhẹ nhàng
   - High contrast cho accessibility
   - Gradient backgrounds

2. **Thử layouts khác nhau:**
   - Jobs pane ở dưới thay vì bên phải
   - Tabs dọc thay vì ngang
   - Card grid thay vì list
   - Collapsible sections

3. **Thử typography styles:**
   - Monospace cho tech vibe
   - Rounded fonts cho friendly look
   - Serif fonts cho elegant feel

4. **Thử button & control styles:**
   - Icon-only buttons
   - Floating action buttons
   - Segmented controls khác
   - Toggle switches thay vì checkboxes

Chúc team Design vui vẻ và sáng tạo! 🎨✨
