# TikTok New Short Drama Layout — Playwright Investigation Brief

## Mục tiêu

TikTok vừa ra mắt giao diện mới cho các profile Short Drama / PineDrama.
Codex cần dùng Playwright để điều tra trực tiếp trên trình duyệt và trả về
một báo cáo đủ để Claude viết lại Chrome extension scanner.

Profile mục tiêu để test:

```
https://www.tiktok.com/@vantaplay_jp
```

---

## Bối cảnh — cơ chế cũ

Extension hiện tại (cần thay thế) hoạt động như sau:

1. Vào trang `/@username/` (tab Video mặc định — grid phẳng).
2. Scroll và gom tất cả `a[href*="/video/"]` → danh sách URL video lẻ.
3. `fetch` từng `/video/<id>` page HTML, parse `__UNIVERSAL_DATA_FOR_REHYDRATION__`
   → `webapp.video-detail.itemInfo.itemStruct.video.playAddr / bitrateInfo`.
4. Việc nhóm video vào bộ/tập được làm ở app sau bằng AI + heuristic.

**Vấn đề:** Giao diện mới có tab "Phim ngắn" / "Series" riêng, hiển thị
grid các bộ phim thay vì video lẻ. Cơ chế gom `a[href*="/video/"]` có thể
không còn hoạt động ở tab này, và schema page state có thể đã thay đổi.

---

## Những gì Codex cần điều tra

### 1. URL structure của tab Phim ngắn

- Khi click vào tab "Phim ngắn" (Short Drama / Series), URL thay đổi thế nào?
- Có query param hay hash mới không? Ví dụ `?tab=series` hay `#series`?
- `data-e2e` attribute của tab element là gì?

### 2. Series card — href trỏ tới đâu

- Mỗi card bộ phim trong grid có `<a>` không? `href` có dạng gì?
  - Ví dụ: `/@user/collection/<id>`, `/@user/series/<id>`, hay vẫn `/video/<id>`?
- Card có chứa thêm metadata nào (số tập, title bộ) trong DOM không?

### 3. Player bộ phim — URL và state khi mở

- Click vào một bộ phim → URL mới trông thế nào?
- DOM có panel "Tập" với lưới số tập không? `data-e2e` của các element đó là gì?
- Tab phân trang tập (ví dụ "1-24", "25-48") được render thế nào?

### 4. Page state JSON (quan trọng nhất)

Trong HTML của trang player bộ phim (sau khi click vào series):

- `__UNIVERSAL_DATA_FOR_REHYDRATION__` còn tồn tại không?
- Key nào trong JSON chứa thông tin series/collection?
  - Tìm các key: `webapp.video-detail`, `webapp.series`, `webapp.collection`,
    `seriesInfo`, `collectionInfo`, `episodeList`, `itemList`, `playList`.
- Trong JSON đó, danh sách tập có dạng array không? Mỗi item có những field gì?
  - Quan tâm: `id`, `desc`, `video.playAddr`, `video.bitrateInfo`, `cover`, `duration`
- Nếu `__UNIVERSAL_DATA_FOR_REHYDRATION__` không chứa danh sách tập đầy đủ,
  tìm `SIGI_STATE` hoặc `sigi-persisted-data` script tag và check tương tự.

### 5. Network requests — API endpoint danh sách tập

Bật network monitoring TRƯỚC khi click vào series card. Sau khi click:

- Có XHR/fetch nào đến domain `tiktok.com` trả về JSON với danh sách tập không?
- Đặc biệt tìm các endpoint có pattern:
  - `/api/series/`, `/api/collection/`, `/api/item_list/`, `/api/playlist/`
  - Response có chứa `itemList`, `episodeList`, hay array video không?
- Ghi lại: full URL (kể cả query params quan trọng như `seriesId`, `collectionId`),
  headers quan trọng, và cấu trúc response JSON (top-level keys).
- Khi bấm sang trang tập tiếp (ví dụ "25-48") có gọi thêm API không?
  Có cursor/pagination params không?

### 6. Media URL cho từng tập

- Sau khi có được video/item ID của một tập, fetch thẳng trang
  `https://www.tiktok.com/@vantaplay_jp/video/<id>` xem còn trả về
  `__UNIVERSAL_DATA_FOR_REHYDRATION__` với `webapp.video-detail.itemInfo.itemStruct` không?
- Hay giờ cần dùng endpoint API để lấy `playAddr` / `bitrateInfo` cho từng tập?

### 7. Kiểm tra tab Video gốc (regression check)

- Tab "Video" mặc định (grid video lẻ) còn chứa `a[href*="/video/"]` không?
- Cơ chế cũ có còn hoạt động cho tab này không?
- Một số profile có cả hai tab — tab Video dùng flow cũ, tab Phim ngắn
  cần flow mới?

---

## Format báo cáo trả về

Trả về file `CODEX_TIKTOK_INVESTIGATION_RESULT.md` trong cùng thư mục,
gồm các mục:

```
## 1. Tab URL / selector
## 2. Series card href pattern
## 3. Player URL + DOM selectors cho episode panel
## 4. Page state JSON — key names và sample structure (rút gọn)
## 5. API endpoints phát hiện được (URL pattern + sample response keys)
## 6. Media URL resolution — còn dùng được flow cũ không?
## 7. Tab Video cũ — còn hoạt động không?
## 8. Kết luận & đề xuất hướng implement mới
```

Mỗi mục cần **dữ liệu thực** (actual URL, actual key name, actual JSON shape)
— không phỏng đoán. Nếu không tìm được gì cho một mục thì ghi rõ "Không tìm thấy".

---

## Lưu ý khi chạy Playwright

- TikTok cần JavaScript — dùng Chromium headful hoặc với `--disable-blink-features=AutomationControlled`.
- Có thể cần đợi sau mỗi navigation (waitForLoadState `networkidle` hoặc specific element).
- Nếu TikTok yêu cầu login để xem, ghi chú lại và thử xem bao nhiêu được
  mà không cần login.
- Lưu ít nhất 1 screenshot của tab Phim ngắn và 1 screenshot của player bộ phim.
