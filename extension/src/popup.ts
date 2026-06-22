import { platformForUrl, type NativeResponse } from './protocol.js'

const titleElement = document.querySelector<HTMLElement>('#title')!
const urlElement = document.querySelector<HTMLElement>('#url')!
const platformElement = document.querySelector<HTMLElement>('#platform')!
const statusElement = document.querySelector<HTMLElement>('#status')!
const sendButton = document.querySelector<HTMLButtonElement>('#send')!

let activeTab: chrome.tabs.Tab | undefined

chrome.tabs.query({ active: true, currentWindow: true }, ([tab]) => {
  activeTab = tab
  const url = tab?.url || ''
  titleElement.textContent = tab?.title || 'Current page'
  urlElement.textContent = url || 'This Chrome page cannot be sent.'
  urlElement.title = url
  const platform = platformForUrl(url)
  platformElement.textContent =
    platform === 'bilibili' ? 'BiliBili detected' : platform === 'douyin' ? 'Douyin detected' : 'Web page'
  sendButton.disabled = !/^https?:/i.test(url)
  if (sendButton.disabled) {
    setStatus('This Chrome page does not provide a downloadable URL.', 'error')
  }
})

sendButton.addEventListener('click', () => {
  const url = activeTab?.url
  if (!url) return
  sendButton.disabled = true
  sendButton.textContent = 'Opening Sorevid…'
  setStatus('Connecting to the desktop app…', 'idle')

  chrome.runtime.sendMessage(
    {
      type: 'send-url',
      url,
      title: activeTab?.title,
      trigger: 'popup',
    },
    (response: NativeResponse | undefined) => {
      const error = chrome.runtime.lastError
      sendButton.disabled = false
      sendButton.textContent = 'Open in Sorevid'
      if (error) {
        setStatus(error.message || 'Could not contact the extension service worker.', 'error')
        return
      }
      if (!response) {
        setStatus('Sorevid did not return a response.', 'error')
        return
      }
      setStatus(response.message, response.ok ? 'success' : 'error')
    },
  )
})

function setStatus(message: string, state: 'idle' | 'success' | 'error') {
  statusElement.textContent = message
  statusElement.className = `status ${state}`
}
