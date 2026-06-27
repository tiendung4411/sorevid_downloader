# TikTok / PineDrama Short Drama Download Plan

## Van de hien tai

Mot so kenh TikTok dang Short Drama, vi du:

`https://www.tiktok.com/@teatrohoney`

tren PC nhin nhu mot profile TikTok binh thuong va co hien thi grid video. Tuy nhien khi dung yt-dlp theo cach channel/profile thong thuong, app nhan loi:

```text
This account does not have any videos posted
```

hoac:

```text
Unable to extract secondary user ID
```

Ly do: profile nay duoc TikTok mobile danh dau nhu mot Short Drama/Series profile. Feed video public ma yt-dlp doc qua `tiktok:user` bao `videoCount = 0`, trong khi giao dien TikTok web/Chrome that van render duoc cac video/tap neu profile da vuot qua challenge/session.

## Nhung gi da xac minh

### 1. yt-dlp profile extractor khong du

Lenh preview profile:

```text
yt-dlp --dump-single-json --skip-download --playlist-end 3 https://www.tiktok.com/@teatrohoney
```

khong lay duoc item tu profile extractor. Ke ca khi resolve duoc `secUid`, luong `tiktok:user:<secUid>` van co the tra `entries = []`.

### 2. Chrome profile that render duoc video links

Khi dung Chrome hien tai cua user, tab TikTok da render profile thanh cong va DOM chua nhieu link dang:

```text
https://www.tiktok.com/@teatrohoney/video/7655178346732490005
```

Sau vai lan scroll profile, da gom duoc 176 video links tu DOM.

### 3. Tab moi/browser sach co the bi TikTok chan

Khi mo video link trong tab moi sach, TikTok co the dung o man:

```text
Please wait...
```

Vi vay cach on dinh hon la thao tac tren tab/profile Chrome that da render thanh cong, hoac dung extension/content script chay trong page context da vuot challenge.

### 4. Page state co media URL that

Khi click/open video trong tab TikTok da render, TikTok nhung metadata trong:

```text
__UNIVERSAL_DATA_FOR_REHYDRATION__
```

Ben trong co:

- `webapp.video-detail.itemInfo.itemStruct.id`
- `desc`, vi du `Episodio 01`
- `video.playAddr`
- `video.downloadAddr`
- `video.bitrateInfo`
- `video.subtitleInfos`
- cover/thumbnail

Da test range request voi `playAddr` va nhan:

```text
HTTP 206
Content-Type: video/mp4
```

=> Day la media URL that, co the tai truc tiep neu con han chu ky.

## Ket luan ky thuat

Khong nen tiep tuc dua vao `tiktok:user` cho cac profile Short Drama/PineDrama.

Huong kha thi hon la them che do:

```text
Browser Capture Mode
```

Trong che do nay, Chrome extension hoac browser automation se dong vai tro "resolver":

1. Doc profile TikTok da render trong Chrome that.
2. Scroll de gom link `/video/...`.
3. Mo/click tung video trong context da vuot challenge.
4. Parse page state de lay media URL da ky san.
5. Gui danh sach media ve Sorevid desktop app.
6. App tai truc tiep bang HTTP/ffmpeg/reqwest, khong can yt-dlp profile extractor.

## Kien truc de xuat

### Thanh phan 1: Chrome extension scanner

Them action trong extension:

```text
Scan TikTok Short Drama profile
```

Nhiem vu:

- Chi chay tren `https://www.tiktok.com/@...`
- Scroll profile theo batch.
- Lay tat ca anchor co href match:

```text
/@username/video/<id>
```

- Deduplicate video links.
- Bao progress ve popup: so link da gom, so lan scroll, trang thai loading.

### Thanh phan 2: Video detail resolver

Co 2 cach:

#### Cach A: Resolver trong cung tab/profile

- Click tung video card trong profile.
- Cho TikTok update URL/detail panel.
- Doc `__UNIVERSAL_DATA_FOR_REHYDRATION__`.
- Lay `itemStruct`.
- Back ve profile hoac dong detail overlay.

Uu diem: tan dung tab da vuot challenge.

Nhuoc diem: cham hon, can thao tac UI can than.

#### Cach B: Resolver bang hidden/background tabs

- Mo video link trong tab rieng.
- Doc state JSON.
- Dong tab.

Uu diem: de code hon.

Nhuoc diem: de gap `Please wait...` neu tab moi bi challenge.

De xuat bat dau voi Cach A, vi da xac minh hoat dong trong Chrome session hien tai.

### Thanh phan 3: Native message protocol

Mo rong protocol extension -> Sorevid:

```json
{
  "action": "import_resolved_media",
  "items": [
    {
      "sourceUrl": "https://www.tiktok.com/@teatrohoney/video/...",
      "mediaUrl": "https://v45.tiktokcdn.com/...",
      "title": "Episodio 01",
      "uploader": "teatrohoney",
      "duration": 141,
      "thumbnail": "https://p16-common-sign.tiktokcdn.com/...",
      "format": "mp4",
      "codec": "h264",
      "definition": "540p",
      "subtitles": []
    }
  ]
}
```

### Thanh phan 4: Desktop app download path

Them download mode moi:

```text
Direct media URL download
```

Khac voi yt-dlp URL download:

- Input la media URL da ky san.
- Ten file lay tu metadata resolver.
- Can tai ngay vi URL co expire.
- Co the dung `reqwest` de stream file hoac van cho yt-dlp/ffmpeg tai URL truc tiep neu can.

De xuat ban dau dung HTTP stream trong Rust:

- `GET mediaUrl`
- save file `.mp4`
- emit progress theo byte
- chay ffprobe sau khi tai xong nhu luong hien tai

## Ke hoach trien khai de xuat

### Phase 1: Proof of Concept trong extension

Muc tieu:

- Tu profile TikTok dang mo, bam nut scan.
- Scroll va gom video links.
- Hien thi tong so link.
- Gui link list ve desktop app de preview.

Ket qua mong doi:

- Profile `@teatrohoney` gom duoc hang tram link thay vi 0 item.

### Phase 2: Resolver metadata cho 1 video

Muc tieu:

- Click/open 1 video link trong tab da render.
- Parse `__UNIVERSAL_DATA_FOR_REHYDRATION__`.
- Lay `playAddr`, `downloadAddr`, `bitrateInfo`.
- Gui 1 resolved item ve desktop app.

Ket qua mong doi:

- App preview duoc title, thumbnail, duration, quality.
- Tai duoc 1 file mp4 truc tiep.

### Phase 3: Batch resolver

Muc tieu:

- Resolve nhieu video links theo hang doi.
- Gioi han toc do de tranh TikTok throttle.
- Luu partial results neu bi dung giua chung.

De xuat:

- Batch size: 10-20 item/lap.
- Delay nho giua moi item.
- Retry neu state chua co `webapp.video-detail`.

### Phase 4: UI/UX trong Sorevid

Them cac trang thai:

- `Scanning Chrome profile...`
- `Found N video links`
- `Resolving signed media URLs...`
- `Media URLs expire soon - start download now`
- `Some items failed to resolve`

Them canh bao:

- "Keep the TikTok tab open while scanning."
- "Download soon after scan because TikTok media URLs expire."

### Phase 5: Fallback va bao loi

Neu khong co `playAddr/downloadAddr`:

- Bao item failed.
- Goi y user reload TikTok tab, scroll lai, hoac dang nhap/verify.

Neu tab moi bi `Please wait...`:

- Khong dung hidden tabs.
- Quay ve resolver trong tab da render.

## Rủi ro

- TikTok co the doi schema `__UNIVERSAL_DATA_FOR_REHYDRATION__`.
- Media URL co expire ngan.
- Scroll qua nhieu item co the bi throttle.
- Mot so video co the chi co HEVC/ByteVC1; nen uu tien H.264 neu co.
- Chrome extension khong nen doc cookie/local storage truc tiep; chi doc DOM/page state da render.

## Uu tien implementation

1. Them scanner trong extension de gom `/video/...` links tu profile.
2. Them protocol import danh sach link vao desktop app.
3. Them resolver cho 1 item bang Chrome page state.
4. Them direct media downloader trong Rust.
5. Batch hoa resolver va download queue.

## Tom tat

Profile Short Drama/PineDrama khong phu hop voi cach tai channel TikTok bang yt-dlp. Tuy nhien Chrome profile that da render duoc ca video links va signed media URLs. Vi vay huong tot nhat la dung Chrome extension/automation lam lop resolve, sau do de Sorevid tai truc tiep media URL da ky san.
