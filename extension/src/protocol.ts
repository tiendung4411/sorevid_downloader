export const NATIVE_HOST = 'com.sorevid.downloader'

export type Trigger = 'popup' | 'context-menu' | 'player-button'
export type Platform = 'bilibili' | 'douyin' | 'other'

export type NativeRequest =
  | { version: 1; id: string; action: 'ping' }
  | {
      version: 1
      id: string
      action: 'import_urls'
      urls: string[]
      source: {
        pageUrl?: string
        title?: string
        platform?: Platform
        trigger: Trigger
      }
    }

export type NativeResponse = {
  version: 1
  id: string
  ok: boolean
  code:
    | 'ok'
    | 'invalid_request'
    | 'origin_denied'
    | 'app_unavailable'
    | 'unsupported_url'
  message: string
  acceptedUrls?: string[]
}

export function platformForUrl(url: string): Platform {
  try {
    const host = new URL(url).hostname.toLowerCase()
    if (host === 'b23.tv' || host === 'bilibili.com' || host.endsWith('.bilibili.com')) {
      return 'bilibili'
    }
    if (
      host === 'douyin.com' ||
      host.endsWith('.douyin.com') ||
      host === 'iesdouyin.com' ||
      host.endsWith('.iesdouyin.com') ||
      host === 'amemv.com' ||
      host.endsWith('.amemv.com')
    ) {
      return 'douyin'
    }
  } catch {
    // The desktop app will perform final validation.
  }
  return 'other'
}

export function requestId() {
  return crypto.randomUUID()
}
