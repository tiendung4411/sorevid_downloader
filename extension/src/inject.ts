interface XMLHttpRequest {
  __sorevidCaptureUrl?: string
}

const captureSource = 'sorevid-drama-capture'
const captureTargets = [
  '/api/drama/user/drama_list/',
  '/api/drama/episode/item_list/',
  '/api/post/item_list/',
]

const originalFetch = window.fetch
const originalXhrOpen = XMLHttpRequest.prototype.open
const originalXhrSend = XMLHttpRequest.prototype.send

console.debug('[sorevid] inject.ts active in MAIN world')

window.fetch = async function (...args) {
  const response = await originalFetch.apply(this, args)
  queueCapture(response)
  return response
}

XMLHttpRequest.prototype.open = function (method: string, url: string | URL, async?: boolean, username?: string | null, password?: string | null) {
  this.__sorevidCaptureUrl = typeof url === 'string' ? url : url.toString()
  return originalXhrOpen.call(this, method, url, async ?? true, username ?? null, password ?? null)
}

XMLHttpRequest.prototype.send = function (body?: Document | XMLHttpRequestBodyInit | null) {
  this.addEventListener('load', function () {
    const url = this.responseURL || this.__sorevidCaptureUrl
    if (!url || shouldCapture(url) === false || this.responseType && this.responseType !== 'text') return
    try {
      const payload = JSON.parse(this.responseText)
      console.debug('[sorevid] captured xhr', url)
      window.postMessage({ source: captureSource, url, payload }, '*')
    } catch {
      // Ignore non-JSON or transient parse failures.
    }
  })
  return originalXhrSend.call(this, body)
}

function queueCapture(response: Response) {
  const url = response.url
  if (!shouldCapture(url)) return
  response
    .clone()
    .json()
    .then((payload) => {
      console.debug('[sorevid] captured fetch', url)
      window.postMessage({ source: captureSource, url, payload }, '*')
    })
    .catch(() => {
      // Ignore non-JSON or transient parse failures.
    })
}

function shouldCapture(url: string) {
  return captureTargets.some((target) => url.includes(target))
}
