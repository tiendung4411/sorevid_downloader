import { useEffect, useMemo, useState } from 'react'
import { convertFileSrc, invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { open } from '@tauri-apps/plugin-dialog'
import {
  CheckCircle2,
  Cable,
  Download,
  FileText,
  FolderOpen,
  Loader2,
  RefreshCw,
  ShieldAlert,
  Square,
  Terminal,
  Trash2,
  Unplug,
  Upload,
  X,
} from 'lucide-react'
import './App.css'

type DownloadPreset =
  | 'compatibleMp4'
  | 'bestQuality'
  | 'audioOnly'
  | 'videoOnly'
  | 'originalCodec'
type CookieMode = 'none' | 'chrome' | 'manual'
type PlatformKey = 'bilibili' | 'douyin'
type SubtitleMode = 'off' | 'subtitles' | 'auto' | 'both'
type SubtitleFormat = 'srt' | 'vtt'
type DanmakuFormat = 'none' | 'xml' | 'ass'
type JobStatus =
  | 'queued'
  | 'starting'
  | 'running'
  | 'warning'
  | 'completed'
  | 'failed'
  | 'canceled'

type ToolStatus = {
  found: boolean
  path?: string
  version?: string
  error?: string
}

type ToolVersions = {
  ytDlp: ToolStatus
  ffmpeg: ToolStatus
  ffprobe: ToolStatus
}

type DownloadEvent = {
  jobId: string
  status: JobStatus
  percent?: number
  speed?: string
  eta?: string
  line?: string
  outputPath?: string
  mediaReport?: MediaReport
}

type MetadataPreview = {
  url: string
  sourceUrl: string
  title?: string
  thumbnail?: string
  duration?: number
  uploader?: string
  platform: string
  webpageUrl?: string
  playlistTitle?: string
  playlistIndex?: number
  playlistCount?: number
  formatCount: number
  bestWidth?: number
  bestHeight?: number
  recommendedPreset: string
  videoCodecs: string[]
  audioCodecs: string[]
  requiresSession: boolean
  warning?: string
}

type MediaReport = {
  path: string
  fileSize?: number
  container?: string
  duration?: number
  videoCodec?: string
  videoTag?: string
  audioCodec?: string
  audioTag?: string
  width?: number
  height?: number
  quicktimeCompatible: boolean
  warning?: string
}

type CoverResult = {
  path: string
}

type CoverViewer = {
  src: string
  title: string
  path?: string
}

type CookieProfile = {
  mode: CookieMode
  manualCookiePath: string
}

type AppSettings = {
  downloadDir: string
  cookieMode: CookieMode
  manualCookiePath: string
  downloadPreset: DownloadPreset
  subtitleMode: SubtitleMode
  subtitleFormat: SubtitleFormat
  embedSubtitles: boolean
  danmakuFormat: DanmakuFormat
  cookieProfiles: Record<string, CookieProfile>
}

type DownloadJob = {
  id: string
  urls: string[]
  titles?: string[]
  status: JobStatus
  percent?: number
  speed?: string
  eta?: string
  logs: string[]
  outputPath?: string
  mediaReport?: MediaReport
  converting?: boolean
}

type ChromeIntegrationStatus = {
  state: 'installed' | 'notInstalled' | 'invalid'
  message: string
  manifestPath?: string
  extensionId: string
}

type CookieRequestGroup = {
  key: string
  urls: string[]
  titles: string[]
  cookieMode: CookieMode
  manualCookiePath: string
}

type CookieFileStatus = {
  path: string
  valid: boolean
  cookieCount: number
  fileSize: number
  modifiedAt?: number
  message: string
}

const defaultTools: ToolVersions = {
  ytDlp: { found: false },
  ffmpeg: { found: false },
  ffprobe: { found: false },
}

const defaultSettings: AppSettings = {
  downloadDir: '',
  cookieMode: 'none',
  manualCookiePath: '',
  downloadPreset: 'compatibleMp4',
  subtitleMode: 'off',
  subtitleFormat: 'srt',
  embedSubtitles: false,
  danmakuFormat: 'none',
  cookieProfiles: {},
}

const isTauriRuntime = () =>
  typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window

const platformConfigs: Record<PlatformKey, { label: string; hosts: string[] }> = {
  bilibili: {
    label: 'BiliBili',
    hosts: ['bilibili.com', 'b23.tv', 'space.bilibili.com'],
  },
  douyin: {
    label: 'Douyin',
    hosts: ['douyin.com', 'iesdouyin.com', 'amemv.com'],
  },
}

const defaultCookieProfile: CookieProfile = {
  mode: 'none',
  manualCookiePath: '',
}

function getPlatformForUrl(url: string): PlatformKey | undefined {
  const value = normalizeUrlCandidate(url)?.toLowerCase() || url.trim().toLowerCase()

  try {
    const parsed = new URL(value)
    return (Object.entries(platformConfigs) as [PlatformKey, typeof platformConfigs[PlatformKey]][])
      .find(([, config]) =>
        config.hosts.some(
          (host) => parsed.hostname === host || parsed.hostname.endsWith(`.${host}`),
        ),
      )?.[0]
  } catch {
    return (Object.entries(platformConfigs) as [PlatformKey, typeof platformConfigs[PlatformKey]][])
      .find(([, config]) => config.hosts.some((host) => value.includes(host)))?.[0]
  }
}

function isBilibiliChannelUrl(url: string) {
  const value = normalizeUrlCandidate(url)?.toLowerCase() || url.trim().toLowerCase()
  return (
    value.includes('space.bilibili.com') ||
    value.includes('bilibili.com/space/') ||
    value.includes('/space.bilibili.com/')
  )
}

function App() {
  const [urlsText, setUrlsText] = useState('')
  const [downloadDir, setDownloadDir] = useState('')
  const [downloadPreset, setDownloadPreset] = useState<DownloadPreset>('compatibleMp4')
  const [subtitleMode, setSubtitleMode] = useState<SubtitleMode>('off')
  const [subtitleFormat, setSubtitleFormat] = useState<SubtitleFormat>('srt')
  const [embedSubtitles, setEmbedSubtitles] = useState(false)
  const [danmakuFormat, setDanmakuFormat] = useState<DanmakuFormat>('none')
  const [cookieMode, setCookieMode] = useState<CookieMode>('none')
  const [manualCookiePath, setManualCookiePath] = useState('')
  const [cookieProfiles, setCookieProfiles] = useState<Record<string, CookieProfile>>({})
  const [tools, setTools] = useState<ToolVersions>(defaultTools)
  const [checkingTools, setCheckingTools] = useState(false)
  const [metadata, setMetadata] = useState<MetadataPreview[]>([])
  const [selectedPreviewKeys, setSelectedPreviewKeys] = useState<Set<string>>(new Set())
  const [checkingMetadata, setCheckingMetadata] = useState(false)
  const [coverPaths, setCoverPaths] = useState<Record<string, string>>({})
  const [savingCoverUrl, setSavingCoverUrl] = useState('')
  const [coverViewer, setCoverViewer] = useState<CoverViewer | null>(null)
  const [jobs, setJobs] = useState<DownloadJob[]>([])
  const [error, setError] = useState('')
  const [settingsLoaded, setSettingsLoaded] = useState(false)
  const [startingDownload, setStartingDownload] = useState(false)
  const [chromeIntegration, setChromeIntegration] = useState<ChromeIntegrationStatus | null>(null)
  const [chromeIntegrationBusy, setChromeIntegrationBusy] = useState(false)
  const [chromeIntegrationMessage, setChromeIntegrationMessage] = useState('')
  const [cookieStatuses, setCookieStatuses] = useState<Record<string, CookieFileStatus>>({})
  const [cookieBusyPlatform, setCookieBusyPlatform] = useState('')
  const [cookieMessage, setCookieMessage] = useState('')

  const urls = useMemo(
    () => collectUrlsFromText(urlsText),
    [urlsText],
  )

  const requiredPlatformKeys = useMemo(() => {
    const keys = urls.map(getPlatformForUrl).filter(Boolean) as PlatformKey[]
    return Array.from(new Set(keys))
  }, [urls])

  const hasBilibiliUrl = useMemo(
    () => urls.some((url) => getPlatformForUrl(url) === 'bilibili'),
    [urls],
  )

  const hasBilibiliChannelUrl = useMemo(
    () => urls.some(isBilibiliChannelUrl),
    [urls],
  )

  const missingCookiePlatforms = useMemo(
    () =>
      requiredPlatformKeys.filter((key) => !cookieProfileReady(getCookieProfile(cookieProfiles, key))),
    [cookieProfiles, requiredPlatformKeys],
  )

  const selectedMetadata = useMemo(
    () => metadata.filter((item) => selectedPreviewKeys.has(previewKey(item))),
    [metadata, selectedPreviewKeys],
  )

  const effectiveDanmakuFormat = hasBilibiliUrl ? danmakuFormat : 'none'

  useEffect(() => {
    refreshTools()
    loadSavedSettings()
    refreshChromeIntegration()

    if (!isTauriRuntime()) {
      return
    }

    const importExtensionUrls = async () => {
      try {
        const imported = await invoke<string[]>('drain_extension_imports')
        if (imported.length > 0) {
          setUrlsText((current) => mergeUrlText(current, imported))
          setMetadata([])
          setSelectedPreviewKeys(new Set())
          setChromeIntegrationMessage(
            `${imported.length} URL${imported.length === 1 ? '' : 's'} received from Chrome.`,
          )
        }
      } catch (err) {
        setError(String(err))
      }
    }

    importExtensionUrls()
    const unlisten = listen<DownloadEvent>('download-event', ({ payload }) => {
      setJobs((currentJobs) =>
        currentJobs.map((job) => {
          if (job.id !== payload.jobId) return job

          return {
            ...job,
            status: payload.status,
            percent: payload.percent ?? job.percent,
            speed: payload.speed ?? job.speed,
            eta: payload.eta ?? job.eta,
            outputPath: payload.outputPath ?? job.outputPath,
            mediaReport: payload.mediaReport ?? job.mediaReport,
            logs: payload.line
              ? [...job.logs.slice(-80), payload.line]
              : job.logs,
          }
        }),
      )
    })
    const unlistenExtension = listen<string[]>('extension-import', () => {
      importExtensionUrls()
    })

    return () => {
      unlisten.then((off) => off())
      unlistenExtension.then((off) => off())
    }
  }, [])

  useEffect(() => {
    if (!settingsLoaded) return

    const settings: AppSettings = {
      downloadDir,
      cookieMode,
      manualCookiePath,
      downloadPreset,
      subtitleMode,
      subtitleFormat,
      embedSubtitles,
      danmakuFormat: effectiveDanmakuFormat,
      cookieProfiles,
    }

    if (!isTauriRuntime()) {
      localStorage.setItem('bilibili-downloader-settings', JSON.stringify(settings))
      return
    }

    invoke('save_settings', { settings }).catch((err) => {
      setError(String(err))
    })
  }, [
    cookieMode,
    cookieProfiles,
    effectiveDanmakuFormat,
    downloadDir,
    downloadPreset,
    embedSubtitles,
    manualCookiePath,
    settingsLoaded,
    subtitleFormat,
    subtitleMode,
  ])

  async function loadSavedSettings() {
    try {
      const settings = isTauriRuntime()
        ? await invoke<AppSettings>('load_settings')
        : readBrowserSettings()

      setDownloadDir(settings.downloadDir || '')
      setCookieMode(isCookieMode(settings.cookieMode) ? settings.cookieMode : 'none')
      setManualCookiePath(settings.manualCookiePath || '')
      setCookieProfiles(normalizeCookieProfiles(settings))
      setSubtitleMode(isSubtitleMode(settings.subtitleMode) ? settings.subtitleMode : 'off')
      setSubtitleFormat(isSubtitleFormat(settings.subtitleFormat) ? settings.subtitleFormat : 'srt')
      setEmbedSubtitles(Boolean(settings.embedSubtitles))
      setDanmakuFormat(isDanmakuFormat(settings.danmakuFormat) ? settings.danmakuFormat : 'none')
      setDownloadPreset(
        isDownloadPreset(settings.downloadPreset)
          ? settings.downloadPreset
          : 'compatibleMp4',
      )
    } catch (err) {
      setError(String(err))
    } finally {
      setSettingsLoaded(true)
    }
  }

  async function refreshTools() {
    setCheckingTools(true)
    setError('')

    try {
      if (!isTauriRuntime()) {
        setTools(defaultTools)
        return
      }

      const versions = await invoke<ToolVersions>('get_tool_versions')
      setTools(versions)
    } catch (err) {
      setError(String(err))
    } finally {
      setCheckingTools(false)
    }
  }

  async function refreshChromeIntegration() {
    if (!isTauriRuntime()) return
    try {
      const status = await invoke<ChromeIntegrationStatus>('get_chrome_integration_status')
      setChromeIntegration(status)
    } catch (err) {
      setChromeIntegrationMessage(String(err))
    }
  }

  async function changeChromeIntegration(action: 'install' | 'remove' | 'test') {
    if (!isTauriRuntime() || chromeIntegrationBusy) return
    setChromeIntegrationBusy(true)
    setChromeIntegrationMessage('')
    try {
      if (action === 'test') {
        const message = await invoke<string>('test_chrome_integration')
        setChromeIntegrationMessage(message)
      } else {
        const command =
          action === 'install' ? 'install_chrome_integration' : 'remove_chrome_integration'
        const status = await invoke<ChromeIntegrationStatus>(command)
        setChromeIntegration(status)
        setChromeIntegrationMessage(status.message)
      }
    } catch (err) {
      setChromeIntegrationMessage(String(err))
    } finally {
      setChromeIntegrationBusy(false)
    }
  }

  async function chooseDownloadDir() {
    if (!isTauriRuntime()) {
      setError('Folder picker is available in the Tauri desktop app.')
      return
    }

    const selected = await open({
      directory: true,
      multiple: false,
      title: 'Choose download folder',
    })

    if (typeof selected === 'string') {
      setDownloadDir(selected)
    }
  }

  async function chooseCookieFile(platformKey?: PlatformKey) {
    if (!isTauriRuntime()) {
      setError('Cookie import is available in the Tauri desktop app.')
      return
    }

    const selected = await open({
      directory: false,
      multiple: false,
      title: 'Choose cookies.txt',
      filters: [{ name: 'Cookie text file', extensions: ['txt'] }],
    })

    if (typeof selected === 'string') {
      if (platformKey) {
        setCookieBusyPlatform(platformKey)
        setCookieMessage('')
        try {
          const status = await invoke<CookieFileStatus>('import_cookie_file', {
            request: { platform: platformKey, path: selected },
          })
          setCookieStatuses((current) => ({ ...current, [platformKey]: status }))
          updateCookieProfile(platformKey, {
            mode: 'manual',
            manualCookiePath: status.path,
          })
          setCookieMessage(`${platformConfigs[platformKey].label}: ${status.message}`)
        } catch (err) {
          setCookieMessage(String(err))
        } finally {
          setCookieBusyPlatform('')
        }
      } else {
        setManualCookiePath(selected)
        setCookieMode('manual')
      }
    }
  }

  async function exportChromeCookies(platformKey: PlatformKey) {
    setCookieBusyPlatform(platformKey)
    setCookieMessage('')
    try {
      const status = await invoke<CookieFileStatus>('export_browser_cookies', {
        request: { platform: platformKey, path: null },
      })
      setCookieStatuses((current) => ({ ...current, [platformKey]: status }))
      updateCookieProfile(platformKey, {
        mode: 'manual',
        manualCookiePath: status.path,
      })
      setCookieMessage(`${platformConfigs[platformKey].label}: ${status.message}`)
    } catch (err) {
      setCookieMessage(String(err))
    } finally {
      setCookieBusyPlatform('')
    }
  }

  async function validateCookieProfile(platformKey: PlatformKey) {
    const profile = getCookieProfile(cookieProfiles, platformKey)
    if (!profile.manualCookiePath) return
    setCookieBusyPlatform(platformKey)
    setCookieMessage('')
    try {
      const status = await invoke<CookieFileStatus>('validate_cookie_file', {
        request: { platform: platformKey, path: profile.manualCookiePath },
      })
      setCookieStatuses((current) => ({ ...current, [platformKey]: status }))
      setCookieMessage(`${platformConfigs[platformKey].label}: ${status.message}`)
    } catch (err) {
      setCookieMessage(String(err))
    } finally {
      setCookieBusyPlatform('')
    }
  }

  async function deleteCookieProfile(platformKey: PlatformKey) {
    const profile = getCookieProfile(cookieProfiles, platformKey)
    if (!profile.manualCookiePath) return
    if (!window.confirm(`Delete the managed ${platformConfigs[platformKey].label} cookie file?`)) {
      return
    }
    setCookieBusyPlatform(platformKey)
    setCookieMessage('')
    try {
      await invoke('delete_cookie_file', {
        request: { platform: platformKey, path: profile.manualCookiePath },
      })
      setCookieStatuses((current) => {
        const next = { ...current }
        delete next[platformKey]
        return next
      })
      updateCookieProfile(platformKey, { mode: 'none', manualCookiePath: '' })
      setCookieMessage(`${platformConfigs[platformKey].label} cookie file deleted.`)
    } catch (err) {
      setCookieMessage(String(err))
    } finally {
      setCookieBusyPlatform('')
    }
  }

  function updateCookieProfile(platformKey: PlatformKey, profile: Partial<CookieProfile>) {
    setCookieProfiles((current) => {
      const existing = getCookieProfile(current, platformKey)
      return {
        ...current,
        [platformKey]: {
          ...existing,
          ...profile,
        },
      }
    })
  }

  function buildCookieRequestGroups(targetUrls: string[], targetTitles: string[] = []) {
    const platformKeys = Array.from(
      new Set(targetUrls.map(getPlatformForUrl).filter(Boolean) as PlatformKey[]),
    )

    const missing = platformKeys.filter((key) => !cookieProfileReady(getCookieProfile(cookieProfiles, key)))
    if (missing.length > 0) {
      return {
        ok: false as const,
        message: `${missing.map((key) => platformConfigs[key].label).join(', ')} needs a cookie/session profile before preview or download.`,
      }
    }

    const groups = new Map<string, CookieRequestGroup>()
    targetUrls.forEach((url, index) => {
      const platformKey = getPlatformForUrl(url)
      const profile = platformKey
        ? getCookieProfile(cookieProfiles, platformKey)
        : {
            mode: cookieMode,
            manualCookiePath: cookieMode === 'manual' ? manualCookiePath : '',
          }
      const groupKey = `${platformKey || 'other'}::${profile.mode}::${profile.manualCookiePath}`
      const existing = groups.get(groupKey)
      if (existing) {
        existing.urls.push(url)
        if (targetTitles[index]) existing.titles.push(targetTitles[index])
      } else {
        groups.set(groupKey, {
          key: groupKey,
          urls: [url],
          titles: targetTitles[index] ? [targetTitles[index]] : [],
          cookieMode: profile.mode,
          manualCookiePath: profile.mode === 'manual' ? profile.manualCookiePath : '',
        })
      }
    })

    return {
      ok: true as const,
      groups: Array.from(groups.values()),
    }
  }

  async function refreshMetadata() {
    setError('')
    setMetadata([])

    if (!isTauriRuntime()) {
      setError('Metadata preview runs inside the Tauri desktop app.')
      return
    }

    if (urls.length === 0) {
      setError('Add at least one URL.')
      return
    }

    const groupedRequest = buildCookieRequestGroups(urls)
    if (!groupedRequest.ok) {
      setError(groupedRequest.message)
      return
    }

    setCheckingMetadata(true)
    try {
      const results = await Promise.allSettled(
        groupedRequest.groups.map((group) =>
          invoke<MetadataPreview[]>('fetch_metadata', {
            request: {
              urls: group.urls,
              cookieMode: group.cookieMode,
              manualCookiePath:
                group.cookieMode === 'manual' ? group.manualCookiePath : null,
            },
          }),
        ),
      )
      const previews = results.flatMap((result) =>
        result.status === 'fulfilled' ? result.value : [],
      )
      const sourceOrder = new Map(urls.map((url, index) => [url, index]))
      previews.sort(
        (left, right) =>
          (sourceOrder.get(left.sourceUrl) ?? Number.MAX_SAFE_INTEGER) -
          (sourceOrder.get(right.sourceUrl) ?? Number.MAX_SAFE_INTEGER),
      )
      setMetadata(previews)
      setSelectedPreviewKeys(new Set(previews.map(previewKey)))
      const failures = results
        .map((result, index) =>
          result.status === 'rejected'
            ? `${groupedRequest.groups[index].urls[0]}: ${String(result.reason)}`
            : '',
        )
        .filter(Boolean)
      if (failures.length > 0) {
        setError(
          `${previews.length} item${previews.length === 1 ? '' : 's'} previewed. ${failures.length} group${failures.length === 1 ? '' : 's'} failed:\n${failures.join('\n')}`,
        )
      }
    } catch (err) {
      setError(String(err))
    } finally {
      setCheckingMetadata(false)
    }
  }

  async function startDownload() {
    setError('')

    if (startingDownload) {
      return
    }

    if (!isTauriRuntime()) {
      setError('Downloads run inside the Tauri desktop app.')
      return
    }

    const selectedItems = metadata.length > 0 ? selectedMetadata : []
    const downloadUrls = selectedItems.length > 0 ? selectedItems.map((item) => item.url) : urls
    const downloadTitles = selectedItems.map((item) => displayPartTitle(item))

    if (downloadUrls.length === 0) {
      setError('Add at least one URL.')
      return
    }

    if (metadata.length > 0 && selectedItems.length === 0) {
      setError('Select at least one preview item to download.')
      return
    }

    if (!downloadDir) {
      setError('Choose a download folder.')
      return
    }

    if (embedSubtitles && subtitleMode !== 'off' && !tools.ffmpeg.found) {
      setError('ffmpeg is required to embed subtitles into MP4.')
      return
    }

    const groupedRequest = buildCookieRequestGroups(downloadUrls, downloadTitles)
    if (!groupedRequest.ok) {
      setError(groupedRequest.message)
      return
    }

    const optimisticJobs = groupedRequest.groups.map((group, index) => ({
      id: `queued-${Date.now()}-${index}`,
      urls: group.urls,
      titles: group.titles,
      status: 'queued' as JobStatus,
      logs: [`Queued ${group.urls.length} item${group.urls.length === 1 ? '' : 's'} from the shared list.`],
    }))
    setStartingDownload(true)
    setJobs((currentJobs) => [...optimisticJobs, ...currentJobs])

    try {
      const results = await Promise.allSettled(
        groupedRequest.groups.map((group) =>
          invoke<string>('start_download', {
            request: {
              urls: group.urls,
              downloadDir,
              preset: downloadPreset,
              cookieMode: group.cookieMode,
              manualCookiePath:
                group.cookieMode === 'manual' ? group.manualCookiePath : null,
              subtitleMode,
              subtitleFormat,
              embedSubtitles,
              danmakuFormat: group.urls.some(
                (url) => getPlatformForUrl(url) === 'bilibili',
              )
                ? danmakuFormat
                : 'none',
            },
          }),
        ),
      )
      setJobs((currentJobs) =>
        currentJobs.map((job) => {
          const index = optimisticJobs.findIndex((queued) => queued.id === job.id)
          if (index < 0) return job
          const result = results[index]
          return result.status === 'fulfilled'
            ? { ...job, id: result.value, status: 'starting' }
            : {
                ...job,
                status: 'failed',
                logs: [...job.logs, String(result.reason)],
              }
        }),
      )
      const failures = results.filter((result) => result.status === 'rejected')
      if (failures.length > 0) {
        setError(
          `${failures.length} download group${failures.length === 1 ? '' : 's'} could not start. Other groups continue normally.`,
        )
      }
    } catch (err) {
      setJobs((currentJobs) =>
        currentJobs.map((job) =>
          optimisticJobs.some((queued) => queued.id === job.id)
            ? {
                ...job,
                status: 'failed',
                logs: [...job.logs, String(err)],
              }
            : job,
        ),
      )
      setError(String(err))
    } finally {
      setStartingDownload(false)
    }
  }

  async function cancelDownload(jobId: string) {
    setError('')

    try {
      await invoke('cancel_download', { jobId })
    } catch (err) {
      setError(String(err))
    }
  }

  async function openOutput(path: string) {
    setError('')
    try {
      await invoke('open_path', { path })
    } catch (err) {
      setError(String(err))
    }
  }

  async function revealOutput(path: string) {
    setError('')
    try {
      await invoke('reveal_path', { path })
    } catch (err) {
      setError(String(err))
    }
  }

  async function convertOutput(jobId: string, path: string) {
    setError('')
    setJobs((currentJobs) =>
      currentJobs.map((job) =>
        job.id === jobId
          ? { ...job, converting: true, logs: [...job.logs, 'Converting to H.264 MP4...'] }
          : job,
      ),
    )

    try {
      const report = await invoke<MediaReport>('convert_to_h264', { path })
      setJobs((currentJobs) =>
        currentJobs.map((job) =>
          job.id === jobId
            ? {
                ...job,
                converting: false,
                outputPath: report.path,
                mediaReport: report,
                logs: [...job.logs, `Converted file: ${report.path}`],
              }
            : job,
        ),
      )
    } catch (err) {
      setJobs((currentJobs) =>
        currentJobs.map((job) =>
          job.id === jobId
            ? { ...job, converting: false, logs: [...job.logs, String(err)] }
            : job,
        ),
      )
      setError(String(err))
    }
  }

  async function saveCover(item: MetadataPreview) {
    setError('')

    if (!isTauriRuntime()) {
      setError('Cover download runs inside the Tauri desktop app.')
      return
    }

    if (!downloadDir) {
      setError('Choose a download folder before saving the cover.')
      return
    }

    if (!item.thumbnail) {
      setError('This video does not expose a cover image.')
      return
    }

    setSavingCoverUrl(item.url)
    try {
      const result = await invoke<CoverResult>('download_cover', {
        request: {
          thumbnailUrl: item.thumbnail,
          title: item.title || 'cover',
          downloadDir,
        },
      })
      setCoverPaths((current) => ({ ...current, [item.url]: result.path }))
      setCoverViewer({
        src: convertFileSrc(result.path),
        title: item.title || 'Cover image',
        path: result.path,
      })
    } catch (err) {
      setError(String(err))
    } finally {
      setSavingCoverUrl('')
    }
  }

  function viewCover(item: MetadataPreview) {
    const savedPath = coverPaths[item.url]
    if (savedPath) {
      setCoverViewer({
        src: convertFileSrc(savedPath),
        title: item.title || 'Cover image',
        path: savedPath,
      })
      return
    }

    if (item.thumbnail) {
      setCoverViewer({
        src: item.thumbnail,
        title: item.title || 'Cover image',
      })
    } else {
      setError('This video does not expose a cover image.')
    }
  }

  function togglePreview(item: MetadataPreview) {
    const key = previewKey(item)
    setSelectedPreviewKeys((current) => {
      const next = new Set(current)
      if (next.has(key)) {
        next.delete(key)
      } else {
        next.add(key)
      }
      return next
    })
  }

  function selectAllPreviews() {
    setSelectedPreviewKeys(new Set(metadata.map(previewKey)))
  }

  function clearPreviewSelection() {
    setSelectedPreviewKeys(new Set())
  }

  return (
    <main className="app-shell">
      <section className="workspace">
        <header className="topbar">
          <div>
            <p className="eyebrow">yt-dlp desktop client</p>
            <h1>BiliBili Downloader</h1>
          </div>
          <button className="icon-button" type="button" onClick={refreshTools}>
            {checkingTools ? <Loader2 className="spin" /> : <RefreshCw />}
            <span>Check tools</span>
          </button>
        </header>

        <section className="tool-strip" aria-label="Tool versions">
          <ToolBadge label="yt-dlp" tool={tools.ytDlp} />
          <ToolBadge label="ffmpeg" tool={tools.ffmpeg} />
          <ToolBadge label="ffprobe" tool={tools.ffprobe} />
        </section>

        <section className="chrome-integration-panel" aria-label="Chrome integration">
          <div className="chrome-integration-heading">
            <div className={`integration-icon ${chromeIntegration?.state || 'unknown'}`}>
              <Cable />
            </div>
            <div>
              <strong>Chrome Integration Bridge</strong>
              <span>
                {chromeIntegration?.message ||
                  'Register the desktop bridge, then load the extension separately in Chrome.'}
              </span>
            </div>
            <small className={chromeIntegration?.state || 'unknown'}>
              {chromeIntegration?.state === 'installed'
                ? 'Installed'
                : chromeIntegration?.state === 'invalid'
                  ? 'Invalid'
                  : 'Not installed'}
            </small>
          </div>
          <div className="chrome-integration-actions">
            <button
              className="secondary-button"
              type="button"
              onClick={() => changeChromeIntegration('install')}
              disabled={chromeIntegrationBusy}
            >
              <Cable />
              <span>Register bridge</span>
            </button>
            <button
              className="secondary-button"
              type="button"
              onClick={() => changeChromeIntegration('test')}
              disabled={chromeIntegrationBusy || chromeIntegration?.state !== 'installed'}
            >
              {chromeIntegrationBusy ? <Loader2 className="spin" /> : <RefreshCw />}
              <span>Test desktop bridge</span>
            </button>
            <button
              className="secondary-button"
              type="button"
              onClick={() => changeChromeIntegration('remove')}
              disabled={chromeIntegrationBusy || chromeIntegration?.state === 'notInstalled'}
            >
              <Unplug />
              <span>Remove bridge</span>
            </button>
            {chromeIntegrationMessage && <span>{chromeIntegrationMessage}</span>}
          </div>
          <div className="chrome-extension-steps">
            <strong>Chrome extension is a separate step</strong>
            <span>
              Open <code>chrome://extensions</code>, enable Developer mode, choose Load unpacked,
              then select the <code>dist-extension</code> folder.
            </span>
          </div>
        </section>

        <section className="download-panel">
          <div className="field-block">
            <label htmlFor="urls">URLs</label>
            <textarea
              id="urls"
              value={urlsText}
              onChange={(event) => setUrlsText(event.target.value)}
              placeholder="Paste one or more video, playlist, or audio URLs..."
              spellCheck={false}
            />
          </div>

          <div className="preview-actions">
            <button className="icon-button" type="button" onClick={refreshMetadata}>
              {checkingMetadata ? <Loader2 className="spin" /> : <RefreshCw />}
              <span>Preview metadata</span>
            </button>
            <span>{metadata.length > 0 ? `${metadata.length} preview ready` : 'Run preview before downloading protected links.'}</span>
          </div>

          {hasBilibiliChannelUrl && (
            <div className="notice notice-soft">
              <ShieldAlert />
              <span>
                BiliBili channel pages are previewed in limited mode to avoid endless loading. You can still download the channel, but preview only loads the first batch of items.
              </span>
            </div>
          )}

          {metadata.length > 0 && (
            <section className="metadata-list" aria-label="Metadata preview">
              <div className="selection-bar">
                <span>
                  {selectedMetadata.length}/{metadata.length} selected
                  {metadata.some((item) => item.playlistCount && item.playlistCount > 1)
                    ? ' from playlist'
                    : ''}
                </span>
                <div>
                  <button className="secondary-button" type="button" onClick={selectAllPreviews}>
                    Select all
                  </button>
                  <button className="secondary-button" type="button" onClick={clearPreviewSelection}>
                    Select none
                  </button>
                </div>
              </div>
              {metadata.map((item) => (
                <MetadataCard
                  key={item.url}
                  coverPath={coverPaths[item.url]}
                  isSavingCover={savingCoverUrl === item.url}
                  isSelected={selectedPreviewKeys.has(previewKey(item))}
                  item={item}
                  onSaveCover={saveCover}
                  onToggle={togglePreview}
                  onViewCover={viewCover}
                />
              ))}
            </section>
          )}

          <div className="control-grid">
            <div className="field-block">
              <label>Download folder</label>
              <button
                className="path-button"
                type="button"
                onClick={chooseDownloadDir}
              >
                <FolderOpen />
                <span>{downloadDir || 'Choose folder'}</span>
              </button>
            </div>

            <div className="field-block">
              <label>Format preset</label>
              <div className="segmented format-presets">
                <SegmentButton
                  active={downloadPreset === 'compatibleMp4'}
                  onClick={() => setDownloadPreset('compatibleMp4')}
                >
                  MP4
                </SegmentButton>
                <SegmentButton
                  active={downloadPreset === 'bestQuality'}
                  onClick={() => setDownloadPreset('bestQuality')}
                >
                  Best
                </SegmentButton>
                <SegmentButton
                  active={downloadPreset === 'audioOnly'}
                  onClick={() => setDownloadPreset('audioOnly')}
                >
                  Audio
                </SegmentButton>
                <SegmentButton
                  active={downloadPreset === 'videoOnly'}
                  onClick={() => setDownloadPreset('videoOnly')}
                >
                  Video
                </SegmentButton>
                <SegmentButton
                  active={downloadPreset === 'originalCodec'}
                  onClick={() => setDownloadPreset('originalCodec')}
                >
                  Original
                </SegmentButton>
              </div>
            </div>
          </div>

          {hasBilibiliUrl && (
            <section className="subtitle-panel" aria-label="Subtitles and danmaku">
              <div className="subtitle-panel-heading">
                <div>
                  <strong>Subtitles & Danmaku</strong>
                  <span>Download subtitle sidecars, BiliBili danmaku XML, or convert danmaku XML to ASS after download.</span>
                </div>
              </div>

              <div className="subtitle-grid">
                <div className="field-block">
                  <label>Subtitles</label>
                  <div className="segmented subtitle-modes">
                    <SegmentButton active={subtitleMode === 'off'} onClick={() => setSubtitleMode('off')}>
                      Off
                    </SegmentButton>
                    <SegmentButton active={subtitleMode === 'subtitles'} onClick={() => setSubtitleMode('subtitles')}>
                      Subs
                    </SegmentButton>
                    <SegmentButton active={subtitleMode === 'auto'} onClick={() => setSubtitleMode('auto')}>
                      Auto
                    </SegmentButton>
                    <SegmentButton active={subtitleMode === 'both'} onClick={() => setSubtitleMode('both')}>
                      Both
                    </SegmentButton>
                  </div>
                </div>

                <div className="field-block">
                  <label>Subtitle format</label>
                  <div className="segmented">
                    <SegmentButton active={subtitleFormat === 'srt'} onClick={() => setSubtitleFormat('srt')}>
                      SRT
                    </SegmentButton>
                    <SegmentButton active={subtitleFormat === 'vtt'} onClick={() => setSubtitleFormat('vtt')}>
                      VTT
                    </SegmentButton>
                  </div>
                </div>

                <div className="field-block">
                  <label>Danmaku</label>
                  <div className="segmented">
                    <SegmentButton active={danmakuFormat === 'none'} onClick={() => setDanmakuFormat('none')}>
                      Off
                    </SegmentButton>
                    <SegmentButton active={danmakuFormat === 'xml'} onClick={() => setDanmakuFormat('xml')}>
                      XML
                    </SegmentButton>
                    <SegmentButton active={danmakuFormat === 'ass'} onClick={() => setDanmakuFormat('ass')}>
                      ASS
                    </SegmentButton>
                  </div>
                </div>

                <label className="check-row">
                  <input
                    type="checkbox"
                    checked={embedSubtitles}
                    disabled={subtitleMode === 'off'}
                    onChange={(event) => setEmbedSubtitles(event.target.checked)}
                  />
                  <span>Embed subtitles into MP4 when possible</span>
                </label>
              </div>
            </section>
          )}

          <section className="platform-cookie-panel" aria-label="Platform cookie profiles">
              <div className="cookie-panel-heading">
                <div>
                  <strong>Platform cookie profiles</strong>
                  <span>
                    {requiredPlatformKeys.length > 0
                      ? 'Detected protected platforms from the pasted URLs.'
                      : 'Choose a platform session before preview/download.'}
                  </span>
                </div>
                {missingCookiePlatforms.length > 0 && (
                  <small>
                    Missing: {missingCookiePlatforms.map((key) => platformConfigs[key].label).join(', ')}
                  </small>
                )}
              </div>

              <div className="platform-cookie-grid">
                {(Object.keys(platformConfigs) as PlatformKey[]).map((platformKey) => (
                  <CookieProfileCard
                    key={platformKey}
                    platformKey={platformKey}
                    profile={getCookieProfile(cookieProfiles, platformKey)}
                    onChange={(profile) => updateCookieProfile(platformKey, profile)}
                    onImport={() => chooseCookieFile(platformKey)}
                    onExport={() => exportChromeCookies(platformKey)}
                    onValidate={() => validateCookieProfile(platformKey)}
                    onDelete={() => deleteCookieProfile(platformKey)}
                    status={cookieStatuses[platformKey]}
                    busy={cookieBusyPlatform === platformKey}
                  />
                ))}
              </div>
              {cookieMessage && <div className="cookie-manager-message">{cookieMessage}</div>}
            </section>

          <div className="notice">
            <ShieldAlert />
            <span>
              {requiredPlatformKeys.length > 0
                ? `${requiredPlatformKeys.map((key) => platformConfigs[key].label).join(', ')} detected. The app will use the matching platform cookie profile for preview and download.`
                : 'Cookie files are sensitive. This app passes them only to local yt-dlp and does not upload or sync them.'}
            </span>
          </div>

          {error && <div className="error-line">{error}</div>}

          <div className="action-row">
            <div className="url-count">
              {metadata.length > 0
                ? `${selectedMetadata.length} item${selectedMetadata.length === 1 ? '' : 's'} selected`
                : `${urls.length} URL${urls.length === 1 ? '' : 's'} ready`}
            </div>
            <button className="primary-button" type="button" onClick={startDownload} disabled={startingDownload}>
              <Download />
              <span>{startingDownload ? 'Starting...' : 'Start download'}</span>
            </button>
          </div>
        </section>
      </section>

      <aside className="jobs-pane">
        <div className="pane-heading">
          <Terminal />
          <h2>Queue</h2>
        </div>

        {jobs.length === 0 ? (
          <div className="empty-state">No downloads yet.</div>
        ) : (
          <div className="job-list">
            {jobs.map((job) => (
              <JobItem
                key={job.id}
                job={job}
                onCancel={cancelDownload}
                onOpen={openOutput}
                onReveal={revealOutput}
                onConvert={convertOutput}
              />
            ))}
          </div>
        )}
      </aside>

      {coverViewer && (
        <CoverModal viewer={coverViewer} onClose={() => setCoverViewer(null)} />
      )}
    </main>
  )
}

function ToolBadge({ label, tool }: { label: string; tool: ToolStatus }) {
  return (
    <div className={tool.found ? 'tool-badge ok' : 'tool-badge missing'}>
      <CheckCircle2 />
      <div>
        <strong>{label}</strong>
        <span>{tool.version || tool.error || 'Not found'}</span>
      </div>
    </div>
  )
}

function SegmentButton({
  active,
  children,
  onClick,
}: {
  active: boolean
  children: string
  onClick: () => void
}) {
  return (
    <button
      className={active ? 'segment active' : 'segment'}
      type="button"
      onClick={onClick}
    >
      {children}
    </button>
  )
}

function CookieProfileCard({
  busy,
  platformKey,
  profile,
  status,
  onChange,
  onDelete,
  onExport,
  onImport,
  onValidate,
}: {
  busy: boolean
  platformKey: PlatformKey
  profile: CookieProfile
  status?: CookieFileStatus
  onChange: (profile: Partial<CookieProfile>) => void
  onDelete: () => void
  onExport: () => void
  onImport: () => void
  onValidate: () => void
}) {
  const config = platformConfigs[platformKey]
  const ready = cookieProfileReady(profile)

  return (
    <article className={ready ? 'cookie-profile-card ready' : 'cookie-profile-card missing'}>
      <div className="cookie-profile-top">
        <div>
          <strong>{config.label}</strong>
          <span>{config.hosts.join(', ')}</span>
        </div>
        <small>{ready ? 'Ready' : 'Needs session'}</small>
      </div>

      <div className="segmented">
        <SegmentButton active={profile.mode === 'none'} onClick={() => onChange({ mode: 'none' })}>
          None
        </SegmentButton>
        <SegmentButton active={profile.mode === 'chrome'} onClick={() => onChange({ mode: 'chrome' })}>
          Chrome
        </SegmentButton>
        <SegmentButton active={profile.mode === 'manual'} onClick={() => onChange({ mode: 'manual' })}>
          Manual
        </SegmentButton>
      </div>

      <div className="cookie-file-row">
        <button className="path-button cookie-file" type="button" onClick={onImport} disabled={busy}>
          {busy ? <Loader2 className="spin" /> : <Upload />}
          <span>{profile.manualCookiePath || `Import ${config.label} cookies.txt`}</span>
        </button>
        <button className="secondary-button" type="button" onClick={onExport} disabled={busy}>
          <Download />
          <span>Export from Chrome</span>
        </button>
      </div>

      {profile.manualCookiePath && (
        <div className="cookie-manager-actions">
          <button className="secondary-button" type="button" onClick={onValidate} disabled={busy}>
            <CheckCircle2 />
            <span>Validate</span>
          </button>
          <button className="secondary-button danger" type="button" onClick={onDelete} disabled={busy}>
            <Trash2 />
            <span>Delete</span>
          </button>
        </div>
      )}

      {status && (
        <div className={status.valid ? 'cookie-status valid' : 'cookie-status invalid'}>
          <strong>{status.valid ? 'Valid cookie file' : 'Invalid cookie file'}</strong>
          <span>
            {status.cookieCount} cookies · {formatBytes(status.fileSize)}
            {status.modifiedAt ? ` · ${new Date(status.modifiedAt * 1000).toLocaleString()}` : ''}
          </span>
        </div>
      )}
    </article>
  )
}

function MetadataCard({
  coverPath,
  isSavingCover,
  isSelected,
  item,
  onSaveCover,
  onToggle,
  onViewCover,
}: {
  coverPath?: string
  isSavingCover: boolean
  isSelected: boolean
  item: MetadataPreview
  onSaveCover: (item: MetadataPreview) => void
  onToggle: (item: MetadataPreview) => void
  onViewCover: (item: MetadataPreview) => void
}) {
  return (
    <article className={isSelected ? 'metadata-card selected' : 'metadata-card'}>
      <label className="part-check">
        <input
          type="checkbox"
          checked={isSelected}
          onChange={() => onToggle(item)}
        />
      </label>
      <Thumbnail src={item.thumbnail} />
      <div>
        <div className="metadata-heading">
          <strong>{displayPartTitle(item)}</strong>
          <span>{item.platform}</span>
        </div>
        {item.playlistTitle && (
          <div className="playlist-row">
            {item.playlistTitle}
            {item.playlistIndex && item.playlistCount
              ? ` - Part ${item.playlistIndex}/${item.playlistCount}`
              : ''}
          </div>
        )}
        <div className="metadata-grid">
          <span>{item.duration ? formatDuration(item.duration) : 'Duration unknown'}</span>
          <span>{item.uploader || 'Uploader unknown'}</span>
          <span>{formatResolution(item)}</span>
          <span>{item.formatCount} formats</span>
        </div>
        <div className="codec-row">
          <span>V: {formatCodecs(item.videoCodecs)}</span>
          <span>A: {formatCodecs(item.audioCodecs)}</span>
        </div>
        <div className="recommend-row">Recommended: {item.recommendedPreset}</div>
        <div className="cover-actions">
          <button
            className="secondary-button"
            type="button"
            onClick={() => onViewCover(item)}
            disabled={!item.thumbnail && !coverPath}
          >
            <FileText />
            <span>View cover</span>
          </button>
          <button
            className="secondary-button"
            type="button"
            onClick={() => onSaveCover(item)}
            disabled={!item.thumbnail || isSavingCover}
          >
            {isSavingCover ? <Loader2 className="spin" /> : <Download />}
            <span>{coverPath ? 'Save again' : 'Save cover'}</span>
          </button>
          {coverPath && <span className="cover-saved">Saved</span>}
        </div>
        {item.warning && <small>{item.warning}</small>}
      </div>
    </article>
  )
}

function CoverModal({
  viewer,
  onClose,
}: {
  viewer: CoverViewer
  onClose: () => void
}) {
  useEffect(() => {
    function onKeyDown(event: KeyboardEvent) {
      if (event.key === 'Escape') onClose()
    }

    window.addEventListener('keydown', onKeyDown)
    return () => window.removeEventListener('keydown', onKeyDown)
  }, [onClose])

  return (
    <div className="cover-modal" onClick={onClose}>
      <div className="cover-dialog" onClick={(event) => event.stopPropagation()}>
        <div className="cover-dialog-bar">
          <div>
            <strong>{viewer.title}</strong>
            {viewer.path && <span>{viewer.path}</span>}
          </div>
          <button className="modal-close" type="button" onClick={onClose} aria-label="Close cover preview">
            <X />
          </button>
        </div>
        <div className="cover-image-stage">
          <img src={viewer.src} alt="" />
        </div>
      </div>
    </div>
  )
}

function Thumbnail({ src }: { src?: string }) {
  const [failed, setFailed] = useState(false)

  if (!src || failed) {
    return (
      <div className="thumb-empty">
        <span>No preview</span>
      </div>
    )
  }

  return <img src={src} alt="" referrerPolicy="no-referrer" onError={() => setFailed(true)} />
}

function JobItem({
  job,
  onCancel,
  onOpen,
  onReveal,
  onConvert,
}: {
  job: DownloadJob
  onCancel: (jobId: string) => void
  onOpen: (path: string) => void
  onReveal: (path: string) => void
  onConvert: (jobId: string, path: string) => void
}) {
  const active = ['queued', 'starting', 'running', 'warning'].includes(job.status)
  const canConvert =
    job.outputPath &&
    job.mediaReport &&
    !job.mediaReport.quicktimeCompatible &&
    job.status === 'completed'

  const label = job.titles?.[0] || job.urls[0]
  const extraCount = Math.max((job.titles?.length || job.urls.length) - 1, 0)

  return (
    <article className={`job-item ${job.status}`}>
      <div className="job-main">
        <div>
          <strong>{label}</strong>
          {extraCount > 0 && <span>+ {extraCount} more</span>}
        </div>
        <small>{job.status}</small>
      </div>

      <JobBatchSummary job={job} />

      <div className="progress-track">
        <div
          className="progress-fill"
          style={{ width: `${Math.min(job.percent ?? 0, 100)}%` }}
        />
      </div>

      <div className="job-meta">
        <span>{job.percent ? `${job.percent.toFixed(1)}%` : 'Waiting'}</span>
        <span>{job.speed || 'Speed pending'}</span>
        <span>{job.eta ? `ETA ${job.eta}` : 'ETA pending'}</span>
      </div>

      <div className="job-log-panel">
        <div className="job-log-header">
          <span>Activity log</span>
          <small>{job.logs.length} lines</small>
        </div>
        <pre>{job.logs.slice(-8).join('\n')}</pre>
      </div>

      {job.mediaReport && (
        <div className={job.mediaReport.quicktimeCompatible ? 'media-report ok' : 'media-report warn'}>
          <strong>
            {job.mediaReport.videoCodec || 'no video'} / {job.mediaReport.audioCodec || 'no audio'}
          </strong>
          <span>
            {job.mediaReport.width && job.mediaReport.height
              ? `${job.mediaReport.width}x${job.mediaReport.height}`
              : 'Resolution unknown'}{' '}
            {job.mediaReport.fileSize ? `- ${formatBytes(job.mediaReport.fileSize)}` : ''}
          </span>
          {job.mediaReport.warning && <small>{job.mediaReport.warning}</small>}
        </div>
      )}

      {job.outputPath && (
        <div className="job-actions">
          <button className="secondary-button" type="button" onClick={() => onOpen(job.outputPath!)}>
            <FileText />
            <span>Open file</span>
          </button>
          <button className="secondary-button" type="button" onClick={() => onReveal(job.outputPath!)}>
            <FolderOpen />
            <span>Open folder</span>
          </button>
          {canConvert && (
            <button
              className="secondary-button"
              type="button"
              onClick={() => onConvert(job.id, job.outputPath!)}
              disabled={job.converting}
            >
              {job.converting ? <Loader2 className="spin" /> : <RefreshCw />}
              <span>Convert H.264</span>
            </button>
          )}
        </div>
      )}

      {active && !job.id.startsWith('queued-') && (
        <button className="cancel-button" type="button" onClick={() => onCancel(job.id)}>
          <Square />
          <span>Cancel</span>
        </button>
      )}
    </article>
  )
}

function JobBatchSummary({ job }: { job: DownloadJob }) {
  const summary = summarizeJobActivity(job.logs)
  const batchCount = summary.totalItems && summary.currentItem
    ? `${summary.currentItem}/${summary.totalItems} clips`
    : job.urls.length > 1
      ? `${job.urls.length} URLs queued`
      : 'Single video'
  const progress =
    summary.totalItems && summary.currentItem
      ? Math.min((summary.currentItem / summary.totalItems) * 100, 100)
      : undefined
  const sourceLabel =
    summary.playlistTitle ||
    summary.activeTitle ||
    summary.lastMeaningfulLine ||
    'Waiting for activity...'

  return (
    <div className="job-batch-summary">
      <div className="job-batch-top">
        <strong>{batchCount}</strong>
        <span>{sourceLabel}</span>
      </div>
      {progress !== undefined && (
        <div className="job-batch-track" aria-label="Batch progress">
          <div className="job-batch-fill" style={{ width: `${progress}%` }} />
        </div>
      )}
    </div>
  )
}

function formatCodecs(codecs: string[]) {
  return codecs.length > 0 ? codecs.slice(0, 4).join(', ') : 'unknown'
}

type JobActivitySummary = {
  totalItems?: number
  currentItem?: number
  playlistTitle?: string
  activeTitle?: string
  lastMeaningfulLine?: string
}

function summarizeJobActivity(logs: string[]): JobActivitySummary {
  const summary: JobActivitySummary = {}

  for (const rawLine of logs) {
    const line = rawLine.trim()
    if (!line) continue

    const playlistMatch = line.match(/^\[download\] Downloading playlist: (.+)$/)
    if (playlistMatch) {
      summary.playlistTitle = playlistMatch[1]
    }

    const itemMatch = line.match(/^\[download\] Downloading item (\d+) of (\d+)$/)
    if (itemMatch) {
      summary.currentItem = Number(itemMatch[1])
      summary.totalItems = Number(itemMatch[2])
    }

    const destinationMatch = line.match(/^\[download\] Destination: (.+)$/)
    if (destinationMatch) {
      summary.activeTitle = destinationMatch[1]
    }

    const youtubeMatch = line.match(/^\[youtube\] ([^:]+): Downloading webpage$/)
    if (youtubeMatch && !summary.activeTitle) {
      summary.activeTitle = youtubeMatch[1]
    }

    if (!line.startsWith('[debug]')) {
      summary.lastMeaningfulLine = line
    }
  }

  return summary
}

function collectUrlsFromText(text: string) {
  const matches = text.match(
    /(?:https?:\/\/|www\.)[^\s<>"'`]+|(?:[\w-]+\.)+[a-z]{2,}(?::\d+)?(?:\/[^\s<>"'`]+)?/gi,
  ) || []

  const urls: string[] = []
  const seen = new Set<string>()

  for (const match of matches) {
    const normalized = normalizeUrlCandidate(match)
    if (!normalized || seen.has(normalized)) {
      continue
    }

    seen.add(normalized)
    urls.push(normalized)
  }

  return urls
}

function mergeUrlText(current: string, incoming: string[]) {
  const currentUrls = collectUrlsFromText(current)
  const merged = [...currentUrls]
  const seen = new Set(currentUrls)

  for (const value of incoming) {
    const normalized = normalizeUrlCandidate(value)
    if (normalized && !seen.has(normalized)) {
      seen.add(normalized)
      merged.push(normalized)
    }
  }

  return merged.join('\n')
}

function normalizeUrlCandidate(value: string) {
  const trimmed = value
    .trim()
    .replace(/^[<('"\u005b\s]+/, '')
    .replace(/[>)"'\u005d\s,.;!?，。！？；、]+$/, '')

  if (!trimmed) {
    return ''
  }

  if (/^[a-z][a-z\d+\-.]*:\/\//i.test(trimmed)) {
    return normalizeYoutubeWatchUrl(trimmed)
  }

  if (trimmed.startsWith('//')) {
    return normalizeYoutubeWatchUrl(`https:${trimmed}`)
  }

  if (looksLikeBareUrl(trimmed)) {
    return normalizeYoutubeWatchUrl(`https://${trimmed}`)
  }

  return trimmed
}

function looksLikeBareUrl(value: string) {
  return /^((?:[\w-]+\.)+[a-z]{2,})(?::\d+)?(?:[/?#]|$)/i.test(value)
}

function normalizeYoutubeWatchUrl(value: string) {
  try {
    const parsed = new URL(value)
    const host = parsed.hostname.toLowerCase()
    const isYoutubeWatch =
      ['youtube.com', 'www.youtube.com', 'm.youtube.com', 'music.youtube.com'].includes(host) &&
      parsed.pathname === '/watch'

    if (!isYoutubeWatch || !parsed.searchParams.has('v')) {
      return value
    }

    for (const key of ['list', 'index', 'start_radio', 'pp']) {
      parsed.searchParams.delete(key)
    }

    return parsed.toString()
  } catch {
    return value
  }
}

function previewKey(item: MetadataPreview) {
  return `${item.sourceUrl}::${item.playlistIndex ?? 0}::${item.url}`
}

function displayPartTitle(item: MetadataPreview) {
  const title = item.title || item.url
  if (!item.playlistIndex || !item.playlistCount || item.playlistCount <= 1) {
    return title
  }

  return `P${item.playlistIndex}. ${title}`
}

function readBrowserSettings(): AppSettings {
  const text = localStorage.getItem('bilibili-downloader-settings')
  if (!text) return defaultSettings

  try {
    return { ...defaultSettings, ...JSON.parse(text) }
  } catch {
    return defaultSettings
  }
}

function normalizeCookieProfiles(settings: AppSettings): Record<string, CookieProfile> {
  const profiles: Record<string, CookieProfile> = {}

  Object.entries(settings.cookieProfiles || {}).forEach(([key, profile]) => {
    profiles[key] = {
      mode: isCookieMode(profile.mode) ? profile.mode : 'none',
      manualCookiePath: profile.manualCookiePath || '',
    }
  })

  if (
    !profiles.bilibili &&
    isCookieMode(settings.cookieMode) &&
    settings.cookieMode !== 'none'
  ) {
    profiles.bilibili = {
      mode: settings.cookieMode,
      manualCookiePath: settings.manualCookiePath || '',
    }
  }

  return profiles
}

function getCookieProfile(
  profiles: Record<string, CookieProfile>,
  platformKey: PlatformKey,
) {
  return profiles[platformKey] || defaultCookieProfile
}

function cookieProfileReady(profile: CookieProfile) {
  return profile.mode === 'chrome' || (profile.mode === 'manual' && Boolean(profile.manualCookiePath))
}

function isCookieMode(value: string): value is CookieMode {
  return ['none', 'chrome', 'manual'].includes(value)
}

function isDownloadPreset(value: string): value is DownloadPreset {
  return ['compatibleMp4', 'bestQuality', 'audioOnly', 'videoOnly', 'originalCodec'].includes(value)
}

function isSubtitleMode(value: string): value is SubtitleMode {
  return ['off', 'subtitles', 'auto', 'both'].includes(value)
}

function isSubtitleFormat(value: string): value is SubtitleFormat {
  return ['srt', 'vtt'].includes(value)
}

function isDanmakuFormat(value: string): value is DanmakuFormat {
  return ['none', 'xml', 'ass'].includes(value)
}

function formatResolution(item: MetadataPreview) {
  if (item.bestWidth && item.bestHeight) return `${item.bestWidth}x${item.bestHeight}`
  if (item.bestHeight) return `${item.bestHeight}p`
  return 'Resolution unknown'
}

function formatDuration(seconds: number) {
  const total = Math.round(seconds)
  const hours = Math.floor(total / 3600)
  const minutes = Math.floor((total % 3600) / 60)
  const secs = total % 60
  if (hours > 0) {
    return `${hours}:${String(minutes).padStart(2, '0')}:${String(secs).padStart(2, '0')}`
  }
  return `${minutes}:${String(secs).padStart(2, '0')}`
}

function formatBytes(bytes: number) {
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  if (bytes < 1024 * 1024 * 1024) return `${(bytes / 1024 / 1024).toFixed(1)} MB`
  return `${(bytes / 1024 / 1024 / 1024).toFixed(1)} GB`
}

export default App
