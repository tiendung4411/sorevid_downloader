## Muc tieu

Dieu tra truc tiep URL:

```text
https://www.tiktok.com/@vantaplay_jp/video/7654627579109854484
```

Muc tieu la tu 1 link episode bat ky lay duoc toan bo tap trong cung 1 bo, cu the bo trong anh co 52 tap chia thanh `1-24`, `25-48`, `49-52`.

## 1. Ket qua chinh

- Link dau vao la episode 2 cua bo `転生令嬢と偽りの彼`.
- Current video id: `7654627579109854484`
- TikTok player hydrate ra panel short drama dung bo:
  - drama name: `転生令嬢と偽りの彼`
  - dramaID: `7654627388341113876`
  - totalEpisodeCount: `52`
- API dung de lay episode trong bo:

```text
/api/drama/episode/item_list/
```

- Goi voi `cursor=0`, `cursor=24`, `cursor=48` lay du:
  - page 1: 24 item, episode `1..24`
  - page 2: 24 item, episode `25..48`
  - page 3: 4 item, episode `49..52`
- Tong hop:
  - `totalFetched = 52`
  - `uniqueFetched = 52`
  - episodeNumbers = `1..52`

## 2. DOM selector thuc te

Panel tap nam trong player/cinema mode, khong phai link collection rieng.

```text
role="dialog" aria-label="Cinema mode"
data-testid="short-drama-detail"
```

Section episode:

```text
section
```

Dieu kien tim section tot nhat:

```js
Array.from(document.querySelectorAll("section"))
  .find(el => /^Tập/.test((el.innerText || el.textContent || "").trim()))
```

Segment part:

```text
button[data-testid="tux-segment-item"]
```

Text segment thuc te:

```text
1-24
25-48
49-52
```

Nut tap:

```text
button.css-14edca6-7937d88b--ButtonEpisode
button.css-4cayet-7937d88b--ButtonEpisode[aria-current="true"]
```

Snapshot DOM luc mo URL dau vao:

```text
Tập
1-24
25-48
49-52
1
2
3
...
24
```

Tap dang active:

```text
2
```

## 3. API endpoint dung cho bo hien tai

Endpoint quan sat duoc trong `performance.getEntriesByType("resource")`:

```text
https://www.tiktok.com/api/drama/episode/item_list/?...&count=24&cursor=0&...&dramaID=7654627388341113876&from_page=video&...
```

Query param quan trong:

```text
aid=1988
count=24
cursor=0
dramaID=7654627388341113876
from_page=video
language=vi-VN
referer=https://www.tiktok.com/@vantaplay_jp/video/7654627579109854484
root_referer=https://www.tiktok.com/@vantaplay_jp/video/7654627579109854484
msToken=...
X-Bogus=...
X-Gnarly=...
```

Top-level response keys:

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

## 4. Pagination thuc te

### cursor=0

```json
{
  "status": 200,
  "cursorOut": "24",
  "hasMore": true,
  "totalEpisodeCount": "52",
  "count": 24,
  "firstEpisode": 1,
  "lastEpisode": 24
}
```

First item:

```json
{
  "id": "7654627572440960277",
  "desc": "01.mp4",
  "episode": 1,
  "dramaID": "7654627388341113876",
  "dramaName": "転生令嬢と偽りの彼",
  "duration": 203,
  "bitrateInfoCount": 7
}
```

Last item:

```json
{
  "id": "7654627619803221269",
  "desc": "26.mp4",
  "episode": 24,
  "dramaID": "7654627388341113876",
  "dramaName": "転生令嬢と偽りの彼",
  "duration": 107,
  "bitrateInfoCount": 7
}
```

### cursor=24

```json
{
  "status": 200,
  "cursorOut": "48",
  "hasMore": true,
  "totalEpisodeCount": "52",
  "count": 24,
  "firstEpisode": 25,
  "lastEpisode": 48
}
```

First item:

```json
{
  "id": "7654627619333229845",
  "desc": "27.mp4",
  "episode": 25,
  "dramaID": "7654627388341113876",
  "dramaName": "転生令嬢と偽りの彼",
  "duration": 80,
  "bitrateInfoCount": 7
}
```

Last item:

```json
{
  "id": "7654627663088241941",
  "desc": "50.mp4",
  "episode": 48,
  "dramaID": "7654627388341113876",
  "dramaName": "転生令嬢と偽りの彼",
  "duration": 100,
  "bitrateInfoCount": 7
}
```

### cursor=48

```json
{
  "status": 200,
  "cursorOut": "72",
  "hasMore": false,
  "totalEpisodeCount": "52",
  "count": 4,
  "firstEpisode": 49,
  "lastEpisode": 52
}
```

First item:

```json
{
  "id": "7654627650882833685",
  "desc": "51.mp4",
  "episode": 49,
  "dramaID": "7654627388341113876",
  "dramaName": "転生令嬢と偽りの彼",
  "duration": 108,
  "bitrateInfoCount": 7
}
```

Last item:

```json
{
  "id": "7654627640648682773",
  "desc": "54.mp4",
  "episode": 52,
  "dramaID": "7654627388341113876",
  "dramaName": "転生令嬢と偽りの彼",
  "duration": 145,
  "bitrateInfoCount": 7
}
```

## 5. Shape item can lay de tai video

Moi item trong `itemList[]` co cac field can cho downloader:

```json
{
  "id": "7654627579109854484",
  "desc": "03.mp4",
  "dramaInfo": {
    "DramaVideoData": {
      "EpisodeNumber": 2,
      "IsFreeIntro": false,
      "IsHighlight": false,
      "IsPreview": false,
      "LinkedEpisodeID": "0",
      "PreviewType": 1
    },
    "dramaID": "7654627388341113876",
    "dramaName": "転生令嬢と偽りの彼",
    "numVideos": 52
  },
  "video": {
    "duration": 141,
    "playAddr": "https://v16-webapp-prime.tiktok.com/video/...",
    "bitrateInfo": [
      {
        "Bitrate": 992677,
        "CodecType": "h264",
        "GearName": "normal_540_0",
        "PlayAddr": {
          "UrlList": [
            "https://v16-webapp-prime.tiktok.com/video/...",
            "https://v19-webapp-prime.tiktok.com/video/...",
            "https://www.tiktok.com/aweme/v1/play/?item_id=7654627579109854484..."
          ]
        }
      }
    ],
    "cover": "https://p16-common-sign.tiktokcdn.com/..."
  }
}
```

Field nen dung:

```text
item.id
item.desc
item.dramaInfo.dramaID
item.dramaInfo.dramaName
item.dramaInfo.DramaVideoData.EpisodeNumber
item.video.duration
item.video.playAddr
item.video.bitrateInfo[]
item.video.bitrateInfo[].PlayAddr.UrlList[]
item.video.cover
```

## 6. Cach lay dung bo tu mot video URL

De tranh tai nham bo dau tien/random:

1. Mo dung URL video nguoi dung dua vao, vi du `/@vantaplay_jp/video/7654627579109854484`.
2. Doi player render `data-testid="short-drama-detail"` va section `Tập`.
3. Trong browser context, tim request:

```js
const episodeApi = performance
  .getEntriesByType("resource")
  .map(r => r.name)
  .find(u => u.includes("/api/drama/episode/item_list/"));
```

4. Parse `dramaID` tu URL request:

```js
const dramaID = new URL(episodeApi).searchParams.get("dramaID");
```

5. Goi lai endpoint nay theo cursor:

```js
for (let cursor = 0; ; cursor += 24) {
  const url = new URL(episodeApi);
  url.searchParams.set("count", "24");
  url.searchParams.set("cursor", String(cursor));
  url.searchParams.set("dramaID", dramaID);

  const json = await fetch(url.toString(), { credentials: "include" }).then(r => r.json());
  collect(json.itemList);

  if (!json.hasMore) break;
}
```

Trong lan test nay, viec thay `cursor` tren URL API da capture van tra ve 200 va gom du 52 tap.

## 7. Luu y implement

- Khong dung logic "lay bo dau tien tren profile".
- Khong can click tung episode de lay media URL.
- Dung `dramaID` phat sinh tu player cua video dau vao.
- Dung `/api/drama/episode/item_list/` paginate den khi `hasMore=false`.
- Media URL co query `expire=...`, nen can tai ngay hoac refresh lai API truoc khi tai neu de lau.
- Neu khong capture duoc endpoint trong `performance`, fallback co the click segment `25-48` hoac `49-52` de buoc TikTok tao request `/api/drama/episode/item_list/`.
