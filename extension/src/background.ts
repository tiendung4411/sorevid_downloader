import {
  NATIVE_HOST,
  platformForUrl,
  requestId,
  type ResolvedMediaItem,
  type NativeRequest,
  type NativeResponse,
  type Trigger,
} from './protocol.js'

type SendUrlMessage = {
  type: 'send-url'
  url: string
  title?: string
  trigger: Trigger
}

type ScanProfileMessage = {
  type: 'scan-profile'
  tabId: number
  pageUrl: string
  title?: string
  limit?: number
}

type ScanTikTokProfileResponse = {
  ok: boolean
  error?: string
  items: ResolvedMediaItem[]
}

type ScanTikTokProfileMessage = {
  type: 'scan-tiktok-profile'
  limit?: number
}

const supportedPatterns = [
  'https://*.bilibili.com/*',
  'https://b23.tv/*',
  'https://*.douyin.com/*',
  'https://*.iesdouyin.com/*',
  'https://*.amemv.com/*',
  'https://*.tiktok.com/*',
]

chrome.runtime.onInstalled.addListener(() => {
  chrome.contextMenus.removeAll(() => {
    chrome.contextMenus.create({
      id: 'sorevid-page',
      title: 'Download page with Sorevid',
      contexts: ['page'],
      documentUrlPatterns: supportedPatterns,
    })
    chrome.contextMenus.create({
      id: 'sorevid-link',
      title: 'Download link with Sorevid',
      contexts: ['link'],
      documentUrlPatterns: supportedPatterns,
    })
    chrome.contextMenus.create({
      id: 'sorevid-video',
      title: 'Download video with Sorevid',
      contexts: ['video'],
      documentUrlPatterns: supportedPatterns,
    })
  })
})

chrome.contextMenus.onClicked.addListener((info, tab) => {
  const pageUrl = tab?.url || info.pageUrl || ''
  let targetUrl = pageUrl
  if (info.menuItemId === 'sorevid-link' && info.linkUrl) {
    targetUrl = info.linkUrl
  } else if (
    info.menuItemId === 'sorevid-video' &&
    info.srcUrl &&
    /^https?:/i.test(info.srcUrl)
  ) {
    targetUrl = info.srcUrl
  }
  if (!targetUrl) return

  sendUrl(targetUrl, tab?.title, 'context-menu')
    .then((response) => showBadge(tab?.id, response.ok))
    .catch(() => showBadge(tab?.id, false))
})

chrome.runtime.onMessage.addListener(
  (message: SendUrlMessage | ScanProfileMessage, _sender, sendResponse: (response: NativeResponse) => void) => {
    if (message?.type === 'send-url') {
      sendUrl(message.url, message.title, message.trigger)
        .then(sendResponse)
        .catch((error) => {
          sendResponse({
            version: 1,
            id: '',
            ok: false,
            code: 'app_unavailable',
            message: error instanceof Error ? error.message : String(error),
          })
        })
      return true
    }

    if (message?.type === 'scan-profile') {
      scanTikTokProfile(message.tabId, message.pageUrl, message.title, message.limit)
        .then(sendResponse)
        .catch((error) => {
          sendResponse({
            version: 1,
            id: '',
            ok: false,
            code: 'app_unavailable',
            message: error instanceof Error ? error.message : String(error),
          })
        })
      return true
    }

    return false
  },
)

function sendUrl(url: string, title: string | undefined, trigger: Trigger) {
  const request: NativeRequest = {
    version: 1,
    id: requestId(),
    action: 'import_urls',
    urls: [url],
    source: {
      pageUrl: url,
      title,
      platform: platformForUrl(url),
      trigger,
    },
  }

  return new Promise<NativeResponse>((resolve, reject) => {
    chrome.runtime.sendNativeMessage(NATIVE_HOST, request, (response: NativeResponse | undefined) => {
      const error = chrome.runtime.lastError
      if (error) {
        reject(new Error(nativeErrorMessage(error.message || 'Unknown native messaging error.')))
        return
      }
      if (!response) {
        reject(new Error('Sorevid did not return a response.'))
        return
      }
      resolve(response)
    })
  })
}

export async function scanTikTokProfile(tabId: number, pageUrl: string, title?: string, limit?: number) {
  const response = await sendTikTokScanMessage(tabId, limit)

  if (!response?.ok) {
    throw new Error(response?.error || 'TikTok scan failed.')
  }

  if (!response.items?.length) {
    throw new Error('No TikTok videos were resolved from this profile.')
  }

  const request: NativeRequest = {
    version: 1,
    id: requestId(),
    action: 'import_resolved_media',
    items: response.items,
    source: {
      pageUrl,
      title,
      platform: 'tiktok',
      trigger: 'profile-scan',
    },
  }

  return new Promise<NativeResponse>((resolve, reject) => {
    chrome.runtime.sendNativeMessage(NATIVE_HOST, request, (nativeResponse: NativeResponse | undefined) => {
      const error = chrome.runtime.lastError
      if (error) {
        reject(new Error(nativeErrorMessage(error.message || 'Unknown native messaging error.')))
        return
      }
      if (!nativeResponse) {
        reject(new Error('Sorevid did not return a response.'))
        return
      }
      resolve(nativeResponse)
    })
  })
}

async function sendTikTokScanMessage(tabId: number, limit?: number) {
  const message = { type: 'scan-tiktok-profile' as const, limit }
  try {
    return await chrome.tabs.sendMessage<ScanTikTokProfileMessage, ScanTikTokProfileResponse>(
      tabId,
      message,
    )
  } catch (error) {
    if (!isMissingContentScriptError(error)) {
      throw error
    }
    await injectContentScript(tabId)
    await wait(250)
    return chrome.tabs.sendMessage<ScanTikTokProfileMessage, ScanTikTokProfileResponse>(
      tabId,
      message,
    )
  }
}

async function injectContentScript(tabId: number) {
  await chrome.scripting.insertCSS({
    target: { tabId },
    files: ['content.css'],
  })
  await chrome.scripting.executeScript({
    target: { tabId },
    files: ['content.js'],
  })
}

function isMissingContentScriptError(error: unknown) {
  const message = error instanceof Error ? error.message : String(error)
  return message.toLowerCase().includes('receiving end does not exist')
}

function wait(ms: number) {
  return new Promise((resolve) => setTimeout(resolve, ms))
}

function nativeErrorMessage(message: string) {
  const lower = message.toLowerCase()
  if (lower.includes('not found') || lower.includes('specified native messaging host')) {
    return 'Chrome Integration is not installed. Open Sorevid and click Install.'
  }
  if (lower.includes('forbidden')) {
    return 'This extension is not authorized by the Sorevid native host.'
  }
  return `Could not connect to Sorevid: ${message}`
}

function showBadge(tabId: number | undefined, success: boolean) {
  if (tabId === undefined) return
  chrome.action.setBadgeBackgroundColor({ tabId, color: success ? '#157347' : '#b42318' })
  chrome.action.setBadgeText({ tabId, text: success ? 'OK' : '!' })
  setTimeout(() => chrome.action.setBadgeText({ tabId, text: '' }), 2500)
}
