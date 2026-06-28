## 1. Tab URL / selector

- Profile test: `https://www.tiktok.com/@vantaplay_jp`
- Tab selector thực tế:
  - `data-e2e="videos-tab"` với text `Video`
  - `data-e2e="drama-tab"` với text `Phim ngắn`
  - `data-e2e="liked-tab"` với text `Đã thích`
- Khi chuyển `Video -> Phim ngắn -> Video`, URL không đổi.
  - Trước: `https://www.tiktok.com/@vantaplay_jp`
  - Sau khi click `Phim ngắn`: vẫn là `https://www.tiktok.com/@vantaplay_jp`
  - Không có query param kiểu `?tab=series` và không có hash kiểu `#series`
- State tab đổi bằng `aria-selected`:
  - `videos-tab`: `aria-selected="true"` khi ở tab Video
  - `drama-tab`: `aria-selected="true"` khi ở tab Phim ngắn

## 2. Series card href pattern

- Grid Phim ngắn nằm trong:
  - `data-e2e="creator-drama-tab-content"`
  - `data-e2e="creator-drama-grid"`
- Mỗi card không có `<a href=...>` để trỏ sang collection/series page.
- Cấu trúc click thực tế:
  - `button.css-1s17gxg-7937d88b--ButtonCreatorDramaCard`
  - bên trong có `div[data-e2e="creator-drama-card"]`
- Card có `data-drama-id`.
  - Ví dụ card đầu tiên: `data-drama-id="7654693956337308692"`
- Metadata có sẵn ngay trong DOM card:
  - title series, ví dụ `蘇った王妃は愛を捨てる`
  - play count, ví dụ `338.1K`
  - số tập, ví dụ `73 Tập`
- Click card đầu tiên không đi tới `/collection/...` hay `/series/...`.
  - Nó mở thẳng player ở URL video:
  - `https://www.tiktok.com/@vantaplay_jp/video/7654694143851072788`

## 3. Player URL + DOM selectors cho episode panel

- Player mở dưới dạng cinema/dialog, không phải page collection riêng:
  - root dialog: `role="dialog" aria-label="Cinema mode"`
  - cinema root: `div[data-cinema-mode-open="true"]`
- URL player là video URL:
  - ví dụ mở từ card đầu tiên: `https://www.tiktok.com/@vantaplay_jp/video/7654694143851072788`
  - sau khi click tập 25: `https://www.tiktok.com/@vantaplay_jp/video/7654694172657601813`
- Side panel / drama detail:
  - `data-testid="cinema-side-panel-body"`
  - `data-testid="short-drama-detail"`
- Section tập:
  - `section.css-hr4lmo-7937d88b--DivSection.e1p2nzkw15`
  - title: `Tập`
- Tab phân trang tập:
  - container: `data-testid="tux-segmented-control"`
  - từng tab page: `button[data-testid="tux-segment-item"]`
  - giá trị quan sát được: `1-24`, `25-48`, `49-72`, `73`
- Nút episode:
  - episode active: `button[aria-current="true"].css-4cayet-7937d88b--ButtonEpisode`
  - episode thường: `button.css-14edca6-7937d88b--ButtonEpisode`
  - ví dụ page `1-24` render các nút `1..24`
  - ví dụ page `25-48` render các nút `25..48`

## 4. Page state JSON — key names và sample structure (rút gọn)

- Trong player page vẫn có script `__UNIVERSAL_DATA_FOR_REHYDRATION__`.
  - `length`: `288549`
- Nhưng trong DOM player hiện tại, script này không chứa `webapp.video-detail`.
- Root keys thực tế đọc được từ `__DEFAULT_SCOPE__`:
  - `webapp.user-detail`
  - `webapp.a-b`
- Không tìm thấy trong script này:
  - `webapp.video-detail`
  - `webapp.series`
  - `webapp.collection`
  - `seriesInfo`
  - `collectionInfo`
  - `episodeList`
  - `playList`
  - `playAddr`
  - `bitrateInfo`
- Không có:
  - `#SIGI_STATE`
  - `#sigi-persisted-data`

Kết luận thực tế ở player:

```json
{
  "__UNIVERSAL_DATA_FOR_REHYDRATION__": {
    "__DEFAULT_SCOPE__": {
      "webapp.user-detail": "...profile state...",
      "webapp.a-b": "..."
    }
  }
}
```

- Dữ liệu drama/video không còn nằm trong page-state HTML kiểu cũ.
- Runtime page lấy dữ liệu episode/video từ API JSON, không phải từ script hydration HTML.

## 5. API endpoints phát hiện được (URL pattern + sample response keys)

### 5.1. Danh sách series trên profile

- Endpoint thực tế:
  - `/api/drama/user/drama_list/`
- URL quan sát được có các query param quan trọng:
  - `count=20`
  - `cursor=0`
  - `secUid=MS4wLjABAAAAeAu_ubYZWVO08dWLbjAHotCX5LU1DIp9wF7o6dQArH2DKmE1oFa94J2AO3GYnzFl`
  - `from_page=user`
  - `verifyFp=...`
  - `msToken=...`
  - `X-Bogus=...`
  - `X-Gnarly=...`
- Response top-level keys:

```json
[
  "cursor",
  "dramaList",
  "extra",
  "hasMore",
  "log_pb",
  "statusCode",
  "status_code",
  "status_msg"
]
```

- Sample item shape trong `dramaList[0]`:

```json
{
  "cover": { "urlList": ["..."] },
  "description": "ある日突然悟りを開き...",
  "dramaID": "7654556926219752469",
  "dramaName": "神の手と呼ばれた男",
  "isLimitedFree": true,
  "numVideos": 67,
  "numWatched": "316998",
  "themes": [
    { "tagID": "...", "tagKey": "...", "tagVal": "..." }
  ],
  "totalDuration": "..."
}
```

### 5.2. Danh sách episode theo drama và theo page 24 tập

- Endpoint thực tế:
  - `/api/drama/episode/item_list/`
- Khi click page `25-48`, request có:
  - `dramaID=7654693956337308692`
  - `count=24`
  - `cursor=24`
  - `from_page=video`
- Khi click page `49-72`, request có:
  - `dramaID=7654693956337308692`
  - `count=24`
  - `cursor=48`
  - `from_page=video`
- Response top-level keys:

```json
[
  "cursor",
  "extra",
  "hasMore",
  "itemList",
  "log_pb",
  "statusCode",
  "status_code",
  "status_msg",
  "totalEpisodeCount"
]
```

- `listLen` thực tế của request `cursor=48`: `24`
- Sample item shape trong `itemList[0]`:

```json
{
  "desc": "監視と試探",
  "id": "7654694128642510100",
  "dramaInfo": {
    "DramaVideoData": {
      "EpisodeNumber": 49,
      "IsFreeIntro": false,
      "IsHighlight": false,
      "IsPreview": false,
      "LinkedEpisodeID": "0",
      "PreviewType": 1
    },
    "authorUID": "7643800925870015508",
    "cover": { "urlList": ["..."] },
    "description": "永安王・謝淵は勅命により...",
    "dramaID": "7654693956337308692",
    "dramaName": "蘇った王妃は愛を捨てる",
    "isLimitedFree": true,
    "numVideos": 73,
    "numWatched": "338137",
    "themes": [
      { "tagID": "...", "tagKey": "...", "tagVal": "..." }
    ],
    "totalDuration": "8344"
  },
  "stats": {
    "collectCount": 1,
    "commentCount": 0,
    "diggCount": 3,
    "playCount": 303,
    "shareCount": 0
  },
  "video": {
    "id": "7654694128642510100",
    "duration": 135,
    "playAddr": "https://v16-webapp-prime.tiktok.com/video/...",
    "bitrate": 1401775,
    "bitrateInfo": [
      {
        "Bitrate": 1401775,
        "CodecType": "h264",
        "Format": "mp4",
        "GearName": "normal_540_0",
        "PlayAddr": {
          "Uri": "v1c025g50000d8tevb7og65ik5n65jq0",
          "UrlList": [
            "https://v16-webapp-prime.tiktok.com/video/...",
            "https://v19-webapp-prime.tiktok.com/video/...",
            "https://www.tiktok.com/aweme/v1/play/?item_id=7654694128642510100..."
          ]
        }
      }
    ],
    "subtitleInfos": [
      {
        "Format": "webvtt",
        "LanguageCodeName": "cmn-Hans-CN",
        "Url": "https://v16-webapp.tiktok.com/.../video/...webvtt..."
      }
    ]
  }
}
```

### 5.3. API cũ vẫn còn xuất hiện ở flow video

- Endpoint thực tế:
  - `/api/post/item_list/`
- Response top-level keys:

```json
[
  "cursor",
  "extra",
  "hasMore",
  "itemList",
  "log_pb",
  "statusCode",
  "status_code",
  "status_msg"
]
```

- `itemList[0]` vẫn có:
  - `dramaInfo`
  - `video.playAddr`
  - `video.bitrateInfo`
  - `subtitleInfos`

### 5.4. Sau khi click 1 episode trong page đã load

- Sau khi click episode `25` trong page `25-48`:
  - URL đổi sang `https://www.tiktok.com/@vantaplay_jp/video/7654694172657601813`
  - Không thấy thêm JSON API mới cho episode list
  - Chủ yếu chỉ tải media video/subtitle (`video` / `xmlhttprequest` tới CDN video/caption)

## 6. Media URL resolution — còn dùng được flow cũ không?

- Với direct `/@user/video/<id>` page trong browser hiện tại:
  - `__UNIVERSAL_DATA_FOR_REHYDRATION__` không còn chứa `webapp.video-detail.itemInfo.itemStruct`
  - Không có `playAddr` / `bitrateInfo` trong HTML page-state
- Nhưng API runtime vẫn trả media URL đầy đủ:
  - `/api/drama/episode/item_list/` trả `itemList[].video.playAddr`
  - `/api/drama/episode/item_list/` trả `itemList[].video.bitrateInfo[]`
  - `/api/post/item_list/` cũng vẫn có `itemList[].video.playAddr` và `bitrateInfo[]`

Kết luận:

- Flow cũ kiểu:
  - lấy `/video/<id>` HTML
  - parse `__UNIVERSAL_DATA_FOR_REHYDRATION__`
- không còn đáng tin cho Short Drama player mới.
- Flow mới nên lấy media trực tiếp từ API JSON.

## 7. Tab Video cũ — còn hoạt động không?

- Có.
- Trong lần kiểm tra chuyển tab đầu tiên:
  - `videos-tab` active
  - `videoLinks = 22`
  - `dramaCards = 0`
- Trong cùng phiên đó, tab `Phim ngắn` cho:
  - `videoLinks = 0`
  - `dramaCards = 20`

Kết luận:

- Profile này có cả hai flow:
  - tab `Video`: flow video lẻ
  - tab `Phim ngắn`: flow drama mới
- Scanner cần tách 2 nhánh rõ ràng, không dùng chung chiến lược.

## 8. Kết luận & đề xuất hướng implement mới

### Những gì thay đổi

- Tab `Phim ngắn` không đổi URL và không expose `<a href="/series/...">`.
- Series card là button + `data-drama-id`, click mở thẳng `video/<episodeId>`.
- Player không nhúng `webapp.video-detail` vào `__UNIVERSAL_DATA_FOR_REHYDRATION__`.
- Episode list được nạp qua API thật:
  - series list: `/api/drama/user/drama_list/`
  - episodes theo page: `/api/drama/episode/item_list/`

### Hướng implement mới đề xuất

1. Vào profile `/@username`.
2. Detect tab `data-e2e="drama-tab"`.
3. Nếu có:
   - click tab `drama-tab`
   - gọi hoặc hook request `/api/drama/user/drama_list/`
   - lấy `dramaList[].dramaID`, `dramaName`, `numVideos`, `cover`, `themes`
4. Với mỗi `dramaID`:
   - paginate `/api/drama/episode/item_list/`
   - `count=24`
   - `cursor=0,24,48,...`
   - dừng khi `hasMore=false`
5. Từ `itemList[]` lấy trực tiếp:
   - `id`
   - `desc`
   - `dramaInfo.DramaVideoData.EpisodeNumber`
   - `video.playAddr`
   - `video.bitrateInfo`
   - `video.subtitleInfos`
   - `video.cover / dynamicCover / originCover`
   - `video.duration`
6. Với tab `Video` cũ:
   - giữ flow riêng cho grid video lẻ
   - không trộn với flow drama

### Ảnh chứng cứ

- Profile tab Phim ngắn: [profile-drama-grid.png](/Users/macbook/Documents/BiliBili%20Downloader/artifacts/tiktok-layout-investigation/profile-drama-grid.png)
- Player side panel: [player-panel.png](/Users/macbook/Documents/BiliBili%20Downloader/artifacts/tiktok-layout-investigation/player-panel.png)
