type NativeResponse = {
  ok: boolean
  message: string
}

const hostMarker = 'sorevidPlayerButton'
let scanTimer: number | undefined

scanPlayers()
const observer = new MutationObserver(scheduleScan)
observer.observe(document.documentElement, { childList: true, subtree: true })
window.addEventListener('popstate', scheduleScan)
window.addEventListener('hashchange', scheduleScan)
setInterval(scanPlayers, 3000)

function scheduleScan() {
  window.clearTimeout(scanTimer)
  scanTimer = window.setTimeout(scanPlayers, 180)
}

function scanPlayers() {
  for (const video of document.querySelectorAll<HTMLVideoElement>('video')) {
    const container = findPlayerContainer(video)
    if (!container || container.dataset[hostMarker] === 'true') continue
    attachButton(container)
  }
}

function findPlayerContainer(video: HTMLVideoElement): HTMLElement | null {
  return (
    video.closest<HTMLElement>(
      '[class*="player-container"], [class*="video-player"], [class*="bpx-player"], [class*="xgplayer"], [class*="player"]',
    ) ||
    video.parentElement
  )
}

function attachButton(container: HTMLElement) {
  container.dataset[hostMarker] = 'true'
  if (getComputedStyle(container).position === 'static') {
    container.style.position = 'relative'
  }
  const host = document.createElement('div')
  host.className = 'sorevid-player-host'
  const button = document.createElement('button')
  button.type = 'button'
  button.className = 'sorevid-player-button'
  button.textContent = '↓ Sorevid'
  button.addEventListener('click', (event) => {
    event.preventDefault()
    event.stopPropagation()
    sendCurrentPage(button)
  })
  host.append(button)
  container.append(host)
}

function sendCurrentPage(button: HTMLButtonElement) {
  if (button.disabled) return
  button.disabled = true
  button.textContent = 'Sending…'
  chrome.runtime.sendMessage(
    {
      type: 'send-url',
      url: location.href,
      title: document.title,
      trigger: 'player-button',
    },
    (response: NativeResponse | undefined) => {
      const error = chrome.runtime.lastError
      button.disabled = false
      button.textContent = '↓ Sorevid'
      if (error) {
        showToast(error.message || 'Could not connect to Sorevid.', true)
        return
      }
      showToast(response?.message || 'Sorevid did not return a response.', !response?.ok)
    },
  )
}

function showToast(message: string, isError: boolean) {
  document.querySelector('.sorevid-toast')?.remove()
  const toast = document.createElement('div')
  toast.className = `sorevid-toast${isError ? ' error' : ''}`
  toast.textContent = message
  document.documentElement.append(toast)
  window.setTimeout(() => toast.remove(), 3500)
}
