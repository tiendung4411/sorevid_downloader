use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    fs,
    io::{self, BufRead, BufReader as StdBufReader, Read, Write},
    path::{Path, PathBuf},
    process::Stdio,
    sync::{Arc, Mutex as StdMutex},
    thread,
    time::Duration as StdDuration,
};
use interprocess::local_socket::{
    prelude::*, GenericNamespaced, ListenerOptions,
};
use tauri::{AppHandle, Emitter, Manager, State};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::{Child, Command},
    sync::Mutex,
    time::{timeout, Duration},
};
use uuid::Uuid;

#[derive(Clone, Default)]
struct DownloadState {
    jobs: Arc<Mutex<HashMap<String, Arc<Mutex<Child>>>>>,
    pids: Arc<Mutex<HashMap<String, u32>>>,
    canceled: Arc<Mutex<HashSet<String>>>,
}

const NATIVE_HOST_NAME: &str = "com.sorevid.downloader";
const EXTENSION_ID: &str = "iplhkneijdhbagijdoldmdmmjbfdkifc";
const EXTENSION_ORIGIN: &str = "chrome-extension://iplhkneijdhbagijdoldmdmmjbfdkifc/";
const IPC_NAME: &str = "sorevid-downloader-native-v1";

#[derive(Clone, Default)]
struct ExtensionState {
    pending_urls: Arc<StdMutex<Vec<String>>>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct NativeRequest {
    version: u8,
    id: String,
    action: String,
    #[serde(default)]
    urls: Vec<String>,
    source: Option<NativeSource>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct NativeSource {
    page_url: Option<String>,
    title: Option<String>,
    platform: Option<String>,
    trigger: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct NativeResponse {
    version: u8,
    id: String,
    ok: bool,
    code: String,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    accepted_urls: Option<Vec<String>>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ChromeIntegrationStatus {
    state: String,
    message: String,
    manifest_path: Option<String>,
    extension_id: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ToolVersions {
    yt_dlp: ToolStatus,
    ffmpeg: ToolStatus,
    ffprobe: ToolStatus,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ToolStatus {
    found: bool,
    path: Option<String>,
    version: Option<String>,
    error: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DownloadRequest {
    urls: Vec<String>,
    download_dir: String,
    preset: DownloadPreset,
    cookie_mode: CookieMode,
    manual_cookie_path: Option<String>,
    subtitle_mode: SubtitleMode,
    subtitle_format: SubtitleFormat,
    embed_subtitles: bool,
    danmaku_format: DanmakuFormat,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct AppSettings {
    download_dir: String,
    cookie_mode: String,
    manual_cookie_path: String,
    download_preset: String,
    #[serde(default = "default_subtitle_mode")]
    subtitle_mode: String,
    #[serde(default = "default_subtitle_format")]
    subtitle_format: String,
    #[serde(default)]
    embed_subtitles: bool,
    #[serde(default = "default_danmaku_format")]
    danmaku_format: String,
    #[serde(default)]
    cookie_profiles: HashMap<String, CookieProfileSettings>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct CookieProfileSettings {
    mode: String,
    manual_cookie_path: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
enum DownloadPreset {
    CompatibleMp4,
    BestQuality,
    AudioOnly,
    VideoOnly,
    OriginalCodec,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
enum CookieMode {
    None,
    Chrome,
    Manual,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
enum SubtitleMode {
    Off,
    Subtitles,
    Auto,
    Both,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
enum SubtitleFormat {
    Srt,
    Vtt,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
enum DanmakuFormat {
    None,
    Xml,
    Ass,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct DownloadEvent {
    job_id: String,
    status: String,
    percent: Option<f32>,
    speed: Option<String>,
    eta: Option<String>,
    line: Option<String>,
    output_path: Option<String>,
    media_report: Option<MediaReport>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct MetadataRequest {
    urls: Vec<String>,
    cookie_mode: CookieMode,
    manual_cookie_path: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CoverRequest {
    thumbnail_url: String,
    title: Option<String>,
    download_dir: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CoverResult {
    path: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CookieFileRequest {
    platform: String,
    path: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CookieFileStatus {
    path: String,
    valid: bool,
    cookie_count: usize,
    file_size: u64,
    modified_at: Option<u64>,
    message: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct MetadataPreview {
    url: String,
    source_url: String,
    title: Option<String>,
    thumbnail: Option<String>,
    duration: Option<f64>,
    uploader: Option<String>,
    platform: String,
    webpage_url: Option<String>,
    playlist_title: Option<String>,
    playlist_index: Option<u64>,
    playlist_count: Option<u64>,
    format_count: usize,
    best_width: Option<u64>,
    best_height: Option<u64>,
    recommended_preset: String,
    video_codecs: Vec<String>,
    audio_codecs: Vec<String>,
    requires_session: bool,
    warning: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct MediaReport {
    path: String,
    file_size: Option<u64>,
    container: Option<String>,
    duration: Option<f64>,
    video_codec: Option<String>,
    video_tag: Option<String>,
    audio_codec: Option<String>,
    audio_tag: Option<String>,
    width: Option<u64>,
    height: Option<u64>,
    quicktime_compatible: bool,
    warning: Option<String>,
}

struct BinaryResolver {
    app: AppHandle,
}

impl BinaryResolver {
    fn new(app: AppHandle) -> Self {
        Self { app }
    }

    fn yt_dlp(&self) -> Result<PathBuf, String> {
        self.resolve("yt-dlp")
    }

    fn ffmpeg(&self) -> Result<PathBuf, String> {
        self.resolve("ffmpeg")
    }

    fn ffprobe(&self) -> Result<PathBuf, String> {
        self.resolve("ffprobe")
    }

    fn resolve(&self, name: &str) -> Result<PathBuf, String> {
        let platform_name = if cfg!(windows) {
            format!("{name}.exe")
        } else {
            name.to_string()
        };

        for root in self.resource_roots() {
            let candidate = root.join(os_bucket()).join(&platform_name);
            if candidate.is_file() {
                return Ok(candidate);
            }
        }

        which::which(&platform_name)
            .or_else(|_| which::which(name))
            .map_err(|_| {
                format!(
                    "Could not find {name}. Install it on PATH or run the sidecar fetch script."
                )
            })
    }

    fn resource_roots(&self) -> Vec<PathBuf> {
        let mut roots = Vec::new();

        if let Ok(path) = self.app.path().resource_dir() {
            roots.push(path.join("resources").join("bin"));
            roots.push(path.join("bin"));
        }

        if let Ok(cwd) = std::env::current_dir() {
            roots.push(cwd.join("src-tauri").join("resources").join("bin"));
            roots.push(cwd.join("resources").join("bin"));
        }

        roots
    }
}

fn os_bucket() -> &'static str {
    if cfg!(target_os = "macos") {
        "macos"
    } else if cfg!(windows) {
        "windows"
    } else {
        "linux"
    }
}

pub fn should_run_native_messaging() -> bool {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    args.iter().any(|arg| arg == "--native-messaging")
        || args
            .first()
            .map(|arg| arg.starts_with("chrome-extension://"))
            .unwrap_or(false)
}

pub fn run_native_messaging() -> Result<(), String> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let origin = args
        .iter()
        .find(|arg| arg.starts_with("chrome-extension://"))
        .cloned()
        .unwrap_or_default();
    let stdin = io::stdin();
    let mut input = stdin.lock();
    let stdout = io::stdout();
    let mut output = stdout.lock();

    loop {
        let Some(payload) = read_native_frame(&mut input)? else {
            break;
        };

        let request = serde_json::from_slice::<NativeRequest>(&payload);
        let response = match request {
            Ok(request) if origin == EXTENSION_ORIGIN => {
                handle_native_host_request(request)
            }
            Ok(request) => response_error(
                request.id,
                "origin_denied",
                "This extension is not allowed to use Sorevid.",
            ),
            Err(error) => response_error(
                String::new(),
                "invalid_request",
                &format!("Invalid native request: {error}"),
            ),
        };

        write_native_frame(&mut output, &response)?;
    }

    Ok(())
}

fn read_native_frame(reader: &mut impl Read) -> Result<Option<Vec<u8>>, String> {
    let mut length_bytes = [0u8; 4];
    match reader.read_exact(&mut length_bytes) {
        Ok(()) => {}
        Err(error) if error.kind() == io::ErrorKind::UnexpectedEof => return Ok(None),
        Err(error) => return Err(format!("Failed to read native message length: {error}")),
    }

    let length = u32::from_le_bytes(length_bytes) as usize;
    if length == 0 || length > 64 * 1024 * 1024 {
        return Err("Native message length is invalid.".to_string());
    }
    let mut payload = vec![0u8; length];
    reader
        .read_exact(&mut payload)
        .map_err(|error| format!("Failed to read native message: {error}"))?;
    Ok(Some(payload))
}

fn write_native_frame(writer: &mut impl Write, response: &NativeResponse) -> Result<(), String> {
    let payload = serde_json::to_vec(response)
        .map_err(|error| format!("Failed to serialize native response: {error}"))?;
    if payload.len() > 1024 * 1024 {
        return Err("Native response exceeded Chrome's 1 MB limit.".to_string());
    }
    writer
        .write_all(&(payload.len() as u32).to_le_bytes())
        .and_then(|_| writer.write_all(&payload))
        .and_then(|_| writer.flush())
        .map_err(|error| format!("Failed to write native response: {error}"))
}

fn handle_native_host_request(request: NativeRequest) -> NativeResponse {
    if let Err(message) = validate_native_request(&request) {
        return response_error(request.id, "invalid_request", &message);
    }

    match send_ipc_request(&request) {
        Ok(response) => response,
        Err(_) => {
            if let Err(error) = start_gui_app() {
                return response_error(request.id, "app_unavailable", &error);
            }

            let started = std::time::Instant::now();
            while started.elapsed() < StdDuration::from_secs(8) {
                thread::sleep(StdDuration::from_millis(200));
                if let Ok(response) = send_ipc_request(&request) {
                    return response;
                }
            }

            response_error(
                request.id,
                "app_unavailable",
                "Sorevid did not become ready within 8 seconds.",
            )
        }
    }
}

fn validate_native_request(request: &NativeRequest) -> Result<(), String> {
    if request.version != 1 || request.id.trim().is_empty() {
        return Err("Unsupported protocol version or missing request ID.".to_string());
    }
    if !matches!(request.action.as_str(), "ping" | "import_urls") {
        return Err("Unsupported native action.".to_string());
    }
    if request.action == "import_urls" && request.urls.is_empty() {
        return Err("At least one URL is required.".to_string());
    }
    if let Some(source) = &request.source {
        if !matches!(
            source.trigger.as_str(),
            "popup" | "context-menu" | "player-button" | "desktop-test"
        ) {
            return Err("Unsupported request trigger.".to_string());
        }
    }
    Ok(())
}

fn send_ipc_request(request: &NativeRequest) -> Result<NativeResponse, String> {
    let name = IPC_NAME
        .to_ns_name::<GenericNamespaced>()
        .map_err(|error| format!("Invalid Sorevid IPC name: {error}"))?;
    let mut stream = LocalSocketStream::connect(name)
        .map_err(|error| format!("Could not connect to Sorevid: {error}"))?;
    let mut payload = serde_json::to_vec(request)
        .map_err(|error| format!("Failed to serialize IPC request: {error}"))?;
    payload.push(b'\n');
    stream
        .write_all(&payload)
        .map_err(|error| format!("Failed to send IPC request: {error}"))?;
    stream
        .flush()
        .map_err(|error| format!("Failed to flush IPC request: {error}"))?;

    let mut response = String::new();
    StdBufReader::new(stream)
        .read_line(&mut response)
        .map_err(|error| format!("Failed to read IPC response: {error}"))?;
    serde_json::from_str(response.trim())
        .map_err(|error| format!("Invalid IPC response: {error}"))
}

fn start_gui_app() -> Result<(), String> {
    let executable = std::env::current_exe()
        .map_err(|error| format!("Could not locate Sorevid executable: {error}"))?;
    let mut command = std::process::Command::new(executable);
    command.arg("--from-native-host");
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000);
    }
    command
        .spawn()
        .map(|_| ())
        .map_err(|error| format!("Could not start Sorevid: {error}"))
}

fn start_ipc_server(app: AppHandle, extension_state: ExtensionState) {
    thread::spawn(move || {
        let Ok(name) = IPC_NAME.to_ns_name::<GenericNamespaced>() else {
            return;
        };
        let Ok(listener) = ListenerOptions::new().name(name).create_sync() else {
            return;
        };

        for connection in listener.incoming().flatten() {
            let app = app.clone();
            let extension_state = extension_state.clone();
            thread::spawn(move || {
                handle_ipc_connection(connection, &app, &extension_state);
            });
        }
    });
}

fn handle_ipc_connection(
    connection: LocalSocketStream,
    app: &AppHandle,
    extension_state: &ExtensionState,
) {
    let mut reader = StdBufReader::new(connection);
    let mut line = String::new();
    let response = match reader.read_line(&mut line) {
        Ok(0) => response_error(
            String::new(),
            "invalid_request",
            "The IPC request was empty.",
        ),
        Ok(_) => match serde_json::from_str::<NativeRequest>(line.trim()) {
            Ok(request) => process_gui_request(app, extension_state, request),
            Err(error) => response_error(
                String::new(),
                "invalid_request",
                &format!("Invalid IPC request: {error}"),
            ),
        },
        Err(error) => response_error(
            String::new(),
            "invalid_request",
            &format!("Could not read IPC request: {error}"),
        ),
    };

    if let Ok(mut payload) = serde_json::to_vec(&response) {
        payload.push(b'\n');
        let _ = reader.get_mut().write_all(&payload);
        let _ = reader.get_mut().flush();
    }
}

fn process_gui_request(
    app: &AppHandle,
    extension_state: &ExtensionState,
    request: NativeRequest,
) -> NativeResponse {
    if let Err(message) = validate_native_request(&request) {
        return response_error(request.id, "invalid_request", &message);
    }

    focus_main_window(app);
    if request.action == "ping" {
        return response_ok(request.id, "Sorevid is connected.", None);
    }

    let mut accepted_urls = Vec::new();
    let mut seen = HashSet::new();
    for url in &request.urls {
        if let Some(normalized) = normalize_url_candidate(url) {
            if is_http_url(&normalized) && seen.insert(normalized.clone()) {
                accepted_urls.push(normalized);
            }
        }
    }

    if accepted_urls.is_empty() {
        return response_error(
            request.id,
            "unsupported_url",
            "No supported HTTP or HTTPS URL was found.",
        );
    }

    if let Ok(mut pending) = extension_state.pending_urls.lock() {
        for url in &accepted_urls {
            if !pending.contains(url) {
                pending.push(url.clone());
            }
        }
    }
    let _ = app.emit("extension-import", accepted_urls.clone());

    response_ok(
        request.id,
        "URL sent to Sorevid.",
        Some(accepted_urls),
    )
}

fn focus_main_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

fn is_http_url(value: &str) -> bool {
    value.starts_with("https://") || value.starts_with("http://")
}

fn response_ok(id: String, message: &str, accepted_urls: Option<Vec<String>>) -> NativeResponse {
    NativeResponse {
        version: 1,
        id,
        ok: true,
        code: "ok".to_string(),
        message: message.to_string(),
        accepted_urls,
    }
}

fn response_error(id: String, code: &str, message: &str) -> NativeResponse {
    NativeResponse {
        version: 1,
        id,
        ok: false,
        code: code.to_string(),
        message: message.to_string(),
        accepted_urls: None,
    }
}

#[tauri::command]
async fn get_tool_versions(app: AppHandle) -> ToolVersions {
    let resolver = BinaryResolver::new(app);

    ToolVersions {
        yt_dlp: probe_tool(resolver.yt_dlp(), &["--version"]).await,
        ffmpeg: probe_tool(resolver.ffmpeg(), &["-version"]).await,
        ffprobe: probe_tool(resolver.ffprobe(), &["-version"]).await,
    }
}

#[tauri::command]
fn load_settings(app: AppHandle) -> AppSettings {
    let path = settings_path(&app);
    let Some(path) = path else {
        return default_settings();
    };

    fs::read_to_string(path)
        .ok()
        .and_then(|text| serde_json::from_str::<AppSettings>(&text).ok())
        .unwrap_or_else(default_settings)
}

#[tauri::command]
fn save_settings(app: AppHandle, settings: AppSettings) -> Result<(), String> {
    let path =
        settings_path(&app).ok_or_else(|| "Could not resolve app config directory.".to_string())?;
    let parent = path
        .parent()
        .ok_or_else(|| "Could not resolve app config directory.".to_string())?;
    fs::create_dir_all(parent)
        .map_err(|error| format!("Failed to create settings directory: {error}"))?;
    let text = serde_json::to_string_pretty(&settings)
        .map_err(|error| format!("Failed to serialize settings: {error}"))?;
    fs::write(path, text).map_err(|error| format!("Failed to save settings: {error}"))
}

#[tauri::command]
fn drain_extension_imports(state: State<'_, ExtensionState>) -> Vec<String> {
    state
        .pending_urls
        .lock()
        .map(|mut urls| std::mem::take(&mut *urls))
        .unwrap_or_default()
}

#[tauri::command]
fn get_chrome_integration_status(app: AppHandle) -> ChromeIntegrationStatus {
    chrome_integration_status(&app)
}

#[tauri::command]
fn install_chrome_integration(app: AppHandle) -> Result<ChromeIntegrationStatus, String> {
    #[cfg(windows)]
    {
        use winreg::{enums::HKEY_CURRENT_USER, RegKey};

        let manifest_path = native_host_manifest_path(&app)?;
        let parent = manifest_path
            .parent()
            .ok_or_else(|| "Could not resolve Chrome integration folder.".to_string())?;
        fs::create_dir_all(parent)
            .map_err(|error| format!("Failed to create Chrome integration folder: {error}"))?;
        let executable = std::env::current_exe()
            .map_err(|error| format!("Could not locate Sorevid executable: {error}"))?;
        let manifest = serde_json::json!({
            "name": NATIVE_HOST_NAME,
            "description": "Sorevid Downloader Chrome integration",
            "path": executable,
            "type": "stdio",
            "allowed_origins": [EXTENSION_ORIGIN]
        });
        fs::write(
            &manifest_path,
            serde_json::to_vec_pretty(&manifest)
                .map_err(|error| format!("Failed to serialize native host manifest: {error}"))?,
        )
        .map_err(|error| format!("Failed to write native host manifest: {error}"))?;

        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let key_path = format!(
            r"Software\Google\Chrome\NativeMessagingHosts\{NATIVE_HOST_NAME}"
        );
        let (key, _) = hkcu
            .create_subkey(&key_path)
            .map_err(|error| format!("Failed to register Chrome integration: {error}"))?;
        key.set_value("", &manifest_path.display().to_string())
            .map_err(|error| format!("Failed to register native host path: {error}"))?;

        return Ok(chrome_integration_status(&app));
    }

    #[cfg(not(windows))]
    Err("Chrome integration installation is currently supported on Windows only.".to_string())
}

#[tauri::command]
fn remove_chrome_integration(app: AppHandle) -> Result<ChromeIntegrationStatus, String> {
    #[cfg(windows)]
    {
        use winreg::{enums::HKEY_CURRENT_USER, RegKey};

        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let key_path = format!(
            r"Software\Google\Chrome\NativeMessagingHosts\{NATIVE_HOST_NAME}"
        );
        match hkcu.delete_subkey_all(&key_path) {
            Ok(()) => {}
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => {
                return Err(format!("Failed to remove Chrome integration: {error}"));
            }
        }
        if let Ok(path) = native_host_manifest_path(&app) {
            match fs::remove_file(path) {
                Ok(()) => {}
                Err(error) if error.kind() == io::ErrorKind::NotFound => {}
                Err(error) => {
                    return Err(format!("Failed to remove native host manifest: {error}"));
                }
            }
        }
        return Ok(chrome_integration_status(&app));
    }

    #[cfg(not(windows))]
    Err("Chrome integration removal is currently supported on Windows only.".to_string())
}

#[tauri::command]
fn test_chrome_integration(app: AppHandle) -> Result<String, String> {
    let status = chrome_integration_status(&app);
    if status.state != "installed" {
        return Err(status.message);
    }
    let request = NativeRequest {
        version: 1,
        id: Uuid::new_v4().to_string(),
        action: "ping".to_string(),
        urls: Vec::new(),
        source: Some(NativeSource {
            page_url: None,
            title: Some("Desktop integration test".to_string()),
            platform: None,
            trigger: "desktop-test".to_string(),
        }),
    };
    let response = send_ipc_request(&request)?;
    if response.ok {
        Ok("Desktop bridge is ready. Load the extension separately in chrome://extensions.".to_string())
    } else {
        Err(response.message)
    }
}

fn native_host_manifest_path(app: &AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_config_dir()
        .map(|dir| {
            dir.join("chrome-integration")
                .join(format!("{NATIVE_HOST_NAME}.json"))
        })
        .map_err(|error| format!("Could not resolve app config directory: {error}"))
}

fn chrome_integration_status(app: &AppHandle) -> ChromeIntegrationStatus {
    #[cfg(windows)]
    {
        use winreg::{enums::HKEY_CURRENT_USER, RegKey};

        let expected_path = match native_host_manifest_path(app) {
            Ok(path) => path,
            Err(error) => {
                return ChromeIntegrationStatus {
                    state: "invalid".to_string(),
                    message: error,
                    manifest_path: None,
                    extension_id: EXTENSION_ID.to_string(),
                };
            }
        };
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let key_path = format!(
            r"Software\Google\Chrome\NativeMessagingHosts\{NATIVE_HOST_NAME}"
        );
        let registered_path = hkcu
            .open_subkey(&key_path)
            .and_then(|key| key.get_value::<String, _>(""));

        return match registered_path {
            Err(error) if error.kind() == io::ErrorKind::NotFound => ChromeIntegrationStatus {
                state: "notInstalled".to_string(),
                message: "Desktop bridge is not registered with Chrome.".to_string(),
                manifest_path: Some(expected_path.display().to_string()),
                extension_id: EXTENSION_ID.to_string(),
            },
            Err(error) => ChromeIntegrationStatus {
                state: "invalid".to_string(),
                message: format!("Could not read Chrome integration registry key: {error}"),
                manifest_path: Some(expected_path.display().to_string()),
                extension_id: EXTENSION_ID.to_string(),
            },
            Ok(path) => {
                let registered = PathBuf::from(path);
                let valid_manifest = registered == expected_path
                    && registered.is_file()
                    && fs::read_to_string(&registered)
                        .ok()
                        .and_then(|text| serde_json::from_str::<serde_json::Value>(&text).ok())
                        .map(|value| {
                            value.get("name").and_then(|value| value.as_str())
                                == Some(NATIVE_HOST_NAME)
                                && value
                                    .get("allowed_origins")
                                    .and_then(|value| value.as_array())
                                    .map(|origins| {
                                        origins.iter().any(|origin| {
                                            origin.as_str() == Some(EXTENSION_ORIGIN)
                                        })
                                    })
                                    .unwrap_or(false)
                        })
                        .unwrap_or(false);

                if valid_manifest {
                    ChromeIntegrationStatus {
                        state: "installed".to_string(),
                        message: "Desktop bridge is registered. The extension must still be loaded in Chrome.".to_string(),
                        manifest_path: Some(registered.display().to_string()),
                        extension_id: EXTENSION_ID.to_string(),
                    }
                } else {
                    ChromeIntegrationStatus {
                        state: "invalid".to_string(),
                        message: "Desktop bridge registration exists but its native host manifest is invalid.".to_string(),
                        manifest_path: Some(registered.display().to_string()),
                        extension_id: EXTENSION_ID.to_string(),
                    }
                }
            }
        };
    }

    #[cfg(not(windows))]
    ChromeIntegrationStatus {
        state: "invalid".to_string(),
        message: "Chrome integration is currently supported on Windows only.".to_string(),
        manifest_path: None,
        extension_id: EXTENSION_ID.to_string(),
    }
}

async fn probe_tool(path: Result<PathBuf, String>, args: &[&str]) -> ToolStatus {
    let Ok(path) = path else {
        return ToolStatus {
            found: false,
            path: None,
            version: None,
            error: path.err(),
        };
    };

    match Command::new(&path).args(args).output().await {
        Ok(output) => {
            let text = if output.stdout.is_empty() {
                String::from_utf8_lossy(&output.stderr).to_string()
            } else {
                String::from_utf8_lossy(&output.stdout).to_string()
            };
            let version = text.lines().next().map(|line| line.trim().to_string());

            ToolStatus {
                found: output.status.success(),
                path: Some(path.display().to_string()),
                version,
                error: if output.status.success() {
                    None
                } else {
                    Some(text)
                },
            }
        }
        Err(error) => ToolStatus {
            found: false,
            path: Some(path.display().to_string()),
            version: None,
            error: Some(error.to_string()),
        },
    }
}

#[tauri::command]
async fn fetch_metadata(
    app: AppHandle,
    request: MetadataRequest,
) -> Result<Vec<MetadataPreview>, String> {
    let request = normalize_metadata_request(request);
    validate_metadata_request(&request)?;

    let resolver = BinaryResolver::new(app);
    let yt_dlp = resolver.yt_dlp()?;
    let mut previews = Vec::new();

    for url in &request.urls {
        let mut args = vec![
            "--dump-single-json".to_string(),
            "--skip-download".to_string(),
            "--no-warnings".to_string(),
        ];
        if is_bilibili_channel_url(url) {
            args.extend(["--playlist-end".to_string(), "24".to_string()]);
        }
        args.extend(build_cookie_args(
            &request.cookie_mode,
            request.manual_cookie_path.as_deref(),
        ));
        args.push(url.clone());

        let output = timeout(
            Duration::from_secs(if is_bilibili_channel_url(url) { 45 } else { 45 }),
            Command::new(&yt_dlp).args(&args).output(),
        )
        .await
        .map_err(|_| {
            if is_bilibili_channel_url(url) {
                "Channel preview timed out. BiliBili space pages are previewed in a limited mode to avoid endless loading. Try a direct video link or download the channel directly.".to_string()
            } else {
                "Metadata preview timed out.".to_string()
            }
        })?
        .map_err(|error| format!("Failed to run metadata preflight: {error}"))?;

        if !output.status.success() {
            let text = stderr_or_stdout(&output);
            return Err(friendly_error(&text));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let value = parse_metadata_json(&stdout)?;
        previews.extend(metadata_previews_from_value(url, &value));
    }

    Ok(previews)
}

#[tauri::command]
async fn download_cover(request: CoverRequest) -> Result<CoverResult, String> {
    if request.thumbnail_url.trim().is_empty() {
        return Err("No cover image is available for this video.".to_string());
    }

    let download_dir = PathBuf::from(&request.download_dir);
    if !download_dir.is_dir() {
        return Err("Choose a valid download folder before saving the cover.".to_string());
    }

    let response = reqwest::Client::new()
        .get(&request.thumbnail_url)
        .header(
            reqwest::header::USER_AGENT,
            "Mozilla/5.0 BiliBili Downloader",
        )
        .send()
        .await
        .map_err(|error| format!("Failed to download cover image: {error}"))?;

    if !response.status().is_success() {
        return Err(format!(
            "Failed to download cover image: HTTP {}",
            response.status()
        ));
    }

    let content_type = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .map(ToString::to_string);
    let bytes = response
        .bytes()
        .await
        .map_err(|error| format!("Failed to read cover image: {error}"))?;
    if bytes.is_empty() {
        return Err("Cover image response was empty.".to_string());
    }

    let extension = image_extension(content_type.as_deref(), &request.thumbnail_url);
    let title = request.title.as_deref().unwrap_or("cover");
    let filename = unique_filename(&download_dir, &sanitize_filename(title), &extension);
    fs::write(&filename, bytes).map_err(|error| format!("Failed to save cover image: {error}"))?;

    Ok(CoverResult {
        path: filename.display().to_string(),
    })
}

#[tauri::command]
async fn export_browser_cookies(
    app: AppHandle,
    request: CookieFileRequest,
) -> Result<CookieFileStatus, String> {
    let platform = validate_cookie_platform(&request.platform)?;
    let resolver = BinaryResolver::new(app.clone());
    let yt_dlp = resolver.yt_dlp()?;
    let output_path = managed_cookie_path(&app, platform)?;
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("Failed to create cookie storage folder: {error}"))?;
    }

    let temporary_path = output_path.with_extension("cookies.txt.tmp");
    let _ = fs::remove_file(&temporary_path);
    let output = Command::new(&yt_dlp)
        .args([
            "--cookies-from-browser",
            "chrome",
            "--cookies",
            temporary_path
                .to_str()
                .ok_or_else(|| "Cookie output path is not valid UTF-8.".to_string())?,
            "--skip-download",
            "--no-warnings",
            platform_cookie_probe_url(platform),
        ])
        .output()
        .await
        .map_err(|error| format!("Failed to export cookies from Chrome: {error}"))?;

    if let Err(validation_error) = validate_cookie_file_path(&temporary_path) {
        let _ = fs::remove_file(&temporary_path);
        return Err(format!(
            "Chrome cookie export failed. Close Chrome and try again if its cookie database is locked.\n{}\n{}",
            friendly_error(&stderr_or_stdout(&output)),
            validation_error
        ));
    }
    if output_path.exists() {
        fs::remove_file(&output_path)
            .map_err(|error| format!("Failed to replace previous cookie file: {error}"))?;
    }
    fs::rename(&temporary_path, &output_path)
        .map_err(|error| format!("Failed to save exported cookie file: {error}"))?;
    cookie_file_status(&output_path)
}

#[tauri::command]
fn import_cookie_file(
    app: AppHandle,
    request: CookieFileRequest,
) -> Result<CookieFileStatus, String> {
    let platform = validate_cookie_platform(&request.platform)?;
    let source = request
        .path
        .as_deref()
        .map(PathBuf::from)
        .ok_or_else(|| "Choose a cookies.txt file.".to_string())?;
    validate_cookie_file_path(&source)?;
    let output_path = managed_cookie_path(&app, platform)?;
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("Failed to create cookie storage folder: {error}"))?;
    }
    if source != output_path {
        fs::copy(&source, &output_path)
            .map_err(|error| format!("Failed to import cookie file: {error}"))?;
    }
    cookie_file_status(&output_path)
}

#[tauri::command]
fn validate_cookie_file(request: CookieFileRequest) -> Result<CookieFileStatus, String> {
    let path = request
        .path
        .as_deref()
        .map(PathBuf::from)
        .ok_or_else(|| "No cookie file is selected.".to_string())?;
    cookie_file_status(&path)
}

#[tauri::command]
fn delete_cookie_file(
    app: AppHandle,
    request: CookieFileRequest,
) -> Result<(), String> {
    let platform = validate_cookie_platform(&request.platform)?;
    let managed_path = managed_cookie_path(&app, platform)?;
    let requested_path = request.path.as_deref().map(PathBuf::from);
    if requested_path.as_ref() != Some(&managed_path) {
        return Err(
            "Sorevid only deletes cookie files stored in its managed cookie folder.".to_string(),
        );
    }
    match fs::remove_file(&managed_path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(format!("Failed to delete cookie file: {error}")),
    }
}

fn validate_cookie_platform(platform: &str) -> Result<&str, String> {
    match platform {
        "bilibili" | "douyin" => Ok(platform),
        _ => Err("Unsupported cookie platform.".to_string()),
    }
}

fn platform_cookie_probe_url(platform: &str) -> &'static str {
    match platform {
        "bilibili" => "https://www.bilibili.com/",
        "douyin" => "https://www.douyin.com/",
        _ => "https://example.com/",
    }
}

fn managed_cookie_path(app: &AppHandle, platform: &str) -> Result<PathBuf, String> {
    app.path()
        .app_config_dir()
        .map(|dir| dir.join("cookies").join(format!("{platform}.cookies.txt")))
        .map_err(|error| format!("Could not resolve app config directory: {error}"))
}

fn validate_cookie_file_path(path: &Path) -> Result<(), String> {
    let status = cookie_file_status(path)?;
    if status.valid {
        Ok(())
    } else {
        Err(status.message)
    }
}

fn cookie_file_status(path: &Path) -> Result<CookieFileStatus, String> {
    if !path.is_file() {
        return Err("Cookie file does not exist.".to_string());
    }
    let metadata =
        fs::metadata(path).map_err(|error| format!("Failed to inspect cookie file: {error}"))?;
    let text =
        fs::read_to_string(path).map_err(|error| format!("Failed to read cookie file: {error}"))?;
    let has_header = text.lines().take(3).any(|line| {
        line.trim() == "# Netscape HTTP Cookie File"
            || line.contains("Netscape HTTP Cookie File")
    });
    let cookie_count = text
        .lines()
        .filter(|line| {
            let trimmed = line.trim();
            let record = trimmed.strip_prefix("#HttpOnly_").unwrap_or(trimmed);
            !trimmed.is_empty()
                && (!trimmed.starts_with('#') || trimmed.starts_with("#HttpOnly_"))
                && record.split('\t').count() >= 7
        })
        .count();
    let valid = has_header && cookie_count > 0;
    let modified_at = metadata
        .modified()
        .ok()
        .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|duration| duration.as_secs());
    Ok(CookieFileStatus {
        path: path.display().to_string(),
        valid,
        cookie_count,
        file_size: metadata.len(),
        modified_at,
        message: if valid {
            format!("Valid Netscape cookie file with {cookie_count} cookies.")
        } else if !has_header {
            "Invalid cookie file: missing Netscape HTTP Cookie File header.".to_string()
        } else {
            "Cookie file contains no usable cookie records.".to_string()
        },
    })
}

#[tauri::command]
async fn start_download(
    app: AppHandle,
    state: State<'_, DownloadState>,
    request: DownloadRequest,
) -> Result<String, String> {
    let request = normalize_download_request(request);
    validate_request(&request)?;

    let job_id = Uuid::new_v4().to_string();
    let resolver = BinaryResolver::new(app.clone());
    let yt_dlp = resolver.yt_dlp()?;
    let ffprobe = resolver.ffprobe().ok();
    let ffmpeg_path = resolver.ffmpeg().ok();
    let ffmpeg_location = ffmpeg_path
        .as_ref()
        .and_then(|path| path.parent().map(|parent| parent.display().to_string()));
    if request.embed_subtitles && ffmpeg_location.is_none() {
        return Err("ffmpeg is required to embed subtitles into MP4.".to_string());
    }
    let args = build_yt_dlp_args(&request, ffmpeg_location.as_deref());
    let output_paths = Arc::new(Mutex::new(Vec::<PathBuf>::new()));
    let danmaku_format = request.danmaku_format.clone();

    emit_event(
        &app,
        DownloadEvent {
            job_id: job_id.clone(),
            status: "starting".to_string(),
            percent: None,
            speed: None,
            eta: None,
            line: Some(format!("yt-dlp {}", args.join(" "))),
            output_path: None,
            media_report: None,
        },
    );

    let mut command = Command::new(yt_dlp);
    command
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = command
        .spawn()
        .map_err(|error| format!("Failed to start yt-dlp: {error}"))?;
    let pid = child.id();

    let stdout = child.stdout.take();
    let stderr = child.stderr.take();
    let child = Arc::new(Mutex::new(child));

    state
        .jobs
        .lock()
        .await
        .insert(job_id.clone(), child.clone());
    if let Some(pid) = pid {
        state.pids.lock().await.insert(job_id.clone(), pid);
    }

    if let Some(stdout) = stdout {
        spawn_line_reader(
            app.clone(),
            job_id.clone(),
            stdout,
            false,
            Some(output_paths.clone()),
        );
    }

    if let Some(stderr) = stderr {
        spawn_line_reader(app.clone(), job_id.clone(), stderr, true, None);
    }

    let state_handle = state.inner().clone();
    let app_handle = app.clone();
    let wait_job_id = job_id.clone();

    tauri::async_runtime::spawn(async move {
        let status = child.lock().await.wait().await;
        state_handle.jobs.lock().await.remove(&wait_job_id);
        state_handle.pids.lock().await.remove(&wait_job_id);

        let was_canceled = state_handle.canceled.lock().await.remove(&wait_job_id);
        if was_canceled {
            return;
        }

        match status {
            Ok(exit) if exit.success() => {
                let paths = output_paths.lock().await.clone();
                if paths.is_empty() {
                    emit_event(
                        &app_handle,
                        DownloadEvent {
                            job_id: wait_job_id.clone(),
                            status: "completed".to_string(),
                            percent: Some(100.0),
                            speed: None,
                            eta: None,
                            line: Some("Download completed.".to_string()),
                            output_path: None,
                            media_report: None,
                        },
                    );
                } else {
                    for path in paths {
                        let danmaku_messages = if matches!(danmaku_format, DanmakuFormat::Ass) {
                            convert_danmaku_sidecars(&path)
                                .unwrap_or_else(|error| vec![format!("Danmaku ASS conversion failed: {error}")])
                        } else {
                            Vec::new()
                        };
                        for message in danmaku_messages {
                            emit_event(
                                &app_handle,
                                DownloadEvent {
                                    job_id: wait_job_id.clone(),
                                    status: "running".to_string(),
                                    percent: None,
                                    speed: None,
                                    eta: None,
                                    line: Some(message),
                                    output_path: None,
                                    media_report: None,
                                },
                            );
                        }

                        let report = match ffprobe.as_ref() {
                            Some(ffprobe_path) => probe_media_with(ffprobe_path, &path).await.ok(),
                            None => None,
                        };
                        emit_event(
                            &app_handle,
                            DownloadEvent {
                                job_id: wait_job_id.clone(),
                                status: "completed".to_string(),
                                percent: Some(100.0),
                                speed: None,
                                eta: None,
                                line: Some(match &report {
                                    Some(report) => media_report_message(report),
                                    None => "Download completed.".to_string(),
                                }),
                                output_path: Some(path.display().to_string()),
                                media_report: report,
                            },
                        );
                    }
                }
            }
            Ok(exit) => emit_event(
                &app_handle,
                DownloadEvent {
                    job_id: wait_job_id,
                    status: "failed".to_string(),
                    percent: None,
                    speed: None,
                    eta: None,
                    line: Some(format!(
                        "yt-dlp exited with code {}. If Chrome cookies failed, open Chrome, sign in to the site, then try again or import cookies.txt.",
                        exit.code().map_or("unknown".to_string(), |code| code.to_string())
                    )),
                    output_path: None,
                    media_report: None,
                },
            ),
            Err(error) => emit_event(
                &app_handle,
                DownloadEvent {
                    job_id: wait_job_id,
                    status: "failed".to_string(),
                    percent: None,
                    speed: None,
                    eta: None,
                    line: Some(format!("Failed while waiting for yt-dlp: {error}")),
                    output_path: None,
                    media_report: None,
                },
            ),
        }
    });

    Ok(job_id)
}

#[tauri::command]
async fn cancel_download(
    app: AppHandle,
    state: State<'_, DownloadState>,
    job_id: String,
) -> Result<(), String> {
    let pid = state.pids.lock().await.remove(&job_id);
    state.jobs.lock().await.remove(&job_id);
    state.canceled.lock().await.insert(job_id.clone());

    let Some(pid) = pid else {
        return Err("Download job was not found or already finished.".to_string());
    };

    kill_process(pid)
        .await
        .map_err(|error| format!("Failed to cancel download: {error}"))?;

    emit_event(
        &app,
        DownloadEvent {
            job_id,
            status: "canceled".to_string(),
            percent: None,
            speed: None,
            eta: None,
            line: Some("Download canceled.".to_string()),
            output_path: None,
            media_report: None,
        },
    );

    Ok(())
}

async fn kill_process(pid: u32) -> Result<(), String> {
    let pid = pid.to_string();
    let status = if cfg!(windows) {
        Command::new("taskkill")
            .args(["/PID", &pid, "/T", "/F"])
            .status()
            .await
    } else {
        Command::new("kill").args(["-TERM", &pid]).status().await
    }
    .map_err(|error| format!("Failed to start process killer: {error}"))?;

    if status.success() {
        Ok(())
    } else {
        Err("The operating system could not cancel that download.".to_string())
    }
}

#[tauri::command]
async fn probe_media(app: AppHandle, path: String) -> Result<MediaReport, String> {
    let ffprobe = BinaryResolver::new(app).ffprobe()?;
    probe_media_with(&ffprobe, Path::new(&path)).await
}

#[tauri::command]
async fn convert_to_h264(app: AppHandle, path: String) -> Result<MediaReport, String> {
    let resolver = BinaryResolver::new(app);
    let ffmpeg = resolver.ffmpeg()?;
    let ffprobe = resolver.ffprobe()?;
    let input = PathBuf::from(&path);
    if !input.is_file() {
        return Err("The selected media file does not exist.".to_string());
    }

    let output = h264_output_path(&input);
    let status = Command::new(&ffmpeg)
        .args([
            "-y",
            "-hide_banner",
            "-i",
            &path,
            "-c:v",
            if cfg!(target_os = "macos") {
                "h264_videotoolbox"
            } else {
                "libx264"
            },
            "-b:v",
            "3500k",
            "-tag:v",
            "avc1",
            "-c:a",
            "aac",
            "-b:a",
            "192k",
            "-movflags",
            "+faststart",
            output
                .to_str()
                .ok_or_else(|| "Output path is not valid UTF-8.".to_string())?,
        ])
        .status()
        .await
        .map_err(|error| format!("Failed to start ffmpeg: {error}"))?;

    if !status.success() {
        return Err(format!(
            "ffmpeg conversion failed with code {}.",
            status
                .code()
                .map_or("unknown".to_string(), |code| code.to_string())
        ));
    }

    probe_media_with(&ffprobe, &output).await
}

#[tauri::command]
fn open_path(path: String) -> Result<(), String> {
    let path = PathBuf::from(path);
    if !path.exists() {
        return Err("Path does not exist.".to_string());
    }

    let status = if cfg!(target_os = "macos") {
        std::process::Command::new("open").arg(&path).status()
    } else if cfg!(windows) {
        std::process::Command::new("explorer").arg(&path).status()
    } else {
        std::process::Command::new("xdg-open").arg(&path).status()
    }
    .map_err(|error| format!("Failed to open path: {error}"))?;

    if status.success() {
        Ok(())
    } else {
        Err("The operating system could not open that path.".to_string())
    }
}

#[tauri::command]
fn reveal_path(path: String) -> Result<(), String> {
    let path = PathBuf::from(path);
    let folder = if path.is_dir() {
        path
    } else {
        path.parent()
            .ok_or_else(|| "Could not find parent folder.".to_string())?
            .to_path_buf()
    };

    open_path(folder.display().to_string())
}

fn validate_request(request: &DownloadRequest) -> Result<(), String> {
    if request.urls.is_empty() {
        return Err("Add at least one URL.".to_string());
    }

    if request.download_dir.trim().is_empty() {
        return Err("Choose a download folder.".to_string());
    }

    let download_dir = PathBuf::from(&request.download_dir);
    if !download_dir.is_dir() {
        return Err("The selected download folder does not exist.".to_string());
    }

    if matches!(request.cookie_mode, CookieMode::None) {
        if let Some(host) = session_required_host(&request.urls) {
            return Err(format!(
                "{host} often rejects anonymous downloader requests. Choose Chrome cookies or import cookies.txt before starting."
            ));
        }
    }

    if matches!(request.cookie_mode, CookieMode::Manual) {
        let path = request
            .manual_cookie_path
            .as_ref()
            .ok_or_else(|| "Choose a cookies.txt file.".to_string())?;

        if !PathBuf::from(path).is_file() {
            return Err("The selected cookies.txt file does not exist.".to_string());
        }
    }

    Ok(())
}

fn validate_metadata_request(request: &MetadataRequest) -> Result<(), String> {
    if request.urls.is_empty() {
        return Err("Add at least one URL.".to_string());
    }

    if matches!(request.cookie_mode, CookieMode::None) {
        if let Some(host) = session_required_host(&request.urls) {
            return Err(format!(
                "{host} usually needs a session. Choose Chrome cookies or import cookies.txt before preview/download."
            ));
        }
    }

    if matches!(request.cookie_mode, CookieMode::Manual) {
        let path = request
            .manual_cookie_path
            .as_ref()
            .ok_or_else(|| "Choose a cookies.txt file.".to_string())?;

        if !PathBuf::from(path).is_file() {
            return Err("The selected cookies.txt file does not exist.".to_string());
        }
    }

    Ok(())
}

fn session_required_host(urls: &[String]) -> Option<&'static str> {
    const HOSTS: [&str; 6] = [
        "bilibili.com",
        "b23.tv",
        "space.bilibili.com",
        "douyin.com",
        "iesdouyin.com",
        "amemv.com",
    ];

    urls.iter().find_map(|url| {
        let normalized = url.trim().to_ascii_lowercase();
        HOSTS.iter().copied().find(|host| {
            normalized.contains(&format!("://{host}"))
                || normalized.contains(&format!(".{host}"))
                || normalized.contains(&format!("/{host}"))
        })
    })
}

fn normalize_download_request(mut request: DownloadRequest) -> DownloadRequest {
    request.urls = normalize_url_list(request.urls);
    request
}

fn normalize_metadata_request(mut request: MetadataRequest) -> MetadataRequest {
    request.urls = normalize_url_list(request.urls);
    request
}

fn normalize_url_list(urls: Vec<String>) -> Vec<String> {
    urls.into_iter()
        .filter_map(|url| normalize_url_candidate(&url))
        .collect()
}

fn normalize_url_candidate(value: &str) -> Option<String> {
    let trimmed = value
        .trim()
        .trim_start_matches(&['<', '(', '\'', '"', '[', '{', ' '][..])
        .trim_end_matches(
            &[
                '>', ')', '\'', '"', ']', '}', ',', '.', ';', '!', '?', '，', '。', '！', '？',
                '；', '、',
            ][..],
        )
        .trim();

    if trimmed.is_empty() {
        return None;
    }

    if trimmed.contains("://") {
        return Some(normalize_youtube_watch_url(trimmed));
    }

    if trimmed.starts_with("//") {
        return Some(normalize_youtube_watch_url(&format!("https:{trimmed}")));
    }

    if looks_like_bare_url(trimmed) {
        return Some(normalize_youtube_watch_url(&format!("https://{trimmed}")));
    }

    Some(trimmed.to_string())
}

fn looks_like_bare_url(value: &str) -> bool {
    regex::Regex::new(r"^((?:[\w-]+\.)+[a-z]{2,})(?::\d+)?(?:[/?#]|$)")
        .map(|re| re.is_match(value))
        .unwrap_or(false)
}

fn normalize_youtube_watch_url(value: &str) -> String {
    let Some((scheme, rest)) = value.split_once("://") else {
        return value.to_string();
    };
    let Some((host, path_and_query)) = rest.split_once('/') else {
        return value.to_string();
    };

    let host_lower = host.to_ascii_lowercase();
    if !matches!(
        host_lower.as_str(),
        "youtube.com" | "www.youtube.com" | "m.youtube.com" | "music.youtube.com"
    ) || !path_and_query.starts_with("watch?")
    {
        return value.to_string();
    }

    let query = &path_and_query["watch?".len()..];
    let mut kept_params = Vec::new();
    let mut has_video_id = false;
    for param in query.split('&') {
        let key = param.split_once('=').map(|(key, _)| key).unwrap_or(param);
        if key == "v" {
            has_video_id = true;
        }
        if !matches!(key, "list" | "index" | "start_radio" | "pp") {
            kept_params.push(param);
        }
    }

    if !has_video_id {
        return value.to_string();
    }

    format!("{scheme}://{host}/watch?{}", kept_params.join("&"))
}

fn build_cookie_args(cookie_mode: &CookieMode, manual_cookie_path: Option<&str>) -> Vec<String> {
    match cookie_mode {
        CookieMode::None => Vec::new(),
        CookieMode::Chrome => vec!["--cookies-from-browser".to_string(), "chrome".to_string()],
        CookieMode::Manual => manual_cookie_path
            .map(|path| vec!["--cookies".to_string(), path.to_string()])
            .unwrap_or_default(),
    }
}

fn build_yt_dlp_args(request: &DownloadRequest, ffmpeg_location: Option<&str>) -> Vec<String> {
    let mut args = vec![
        "--newline".to_string(),
        "--no-color".to_string(),
        "--progress-template".to_string(),
        "download:%(progress._percent_str)s|%(progress._speed_str)s|%(progress._eta_str)s"
            .to_string(),
        "--merge-output-format".to_string(),
        "mp4".to_string(),
        "-P".to_string(),
        request.download_dir.clone(),
        "-o".to_string(),
        "%(title)s [%(id)s].%(ext)s".to_string(),
    ];

    if let Some(location) = ffmpeg_location {
        args.extend(["--ffmpeg-location".to_string(), location.to_string()]);
    }

    match request.preset {
        DownloadPreset::CompatibleMp4 => {
            args.extend([
                "-f".to_string(),
                "bv*[vcodec^=avc1]+ba[acodec^=mp4a]/bv*[vcodec^=avc1]+ba/b[ext=mp4]/bv*+ba/b"
                    .to_string(),
            ]);
        }
        DownloadPreset::BestQuality => {
            args.extend(["-f".to_string(), "bv*+ba/b".to_string()]);
        }
        DownloadPreset::AudioOnly => {
            args.extend([
                "-x".to_string(),
                "--audio-format".to_string(),
                "mp3".to_string(),
            ]);
        }
        DownloadPreset::VideoOnly => {
            args.extend([
                "-f".to_string(),
                "bv*[vcodec^=avc1]/bv*[ext=mp4]/bv*".to_string(),
            ]);
        }
        DownloadPreset::OriginalCodec => {
            args.extend(["-f".to_string(), "bv*+ba/b".to_string()]);
            args.retain(|arg| arg != "--merge-output-format" && arg != "mp4");
        }
    }

    let wants_regular_subtitles = matches!(
        request.subtitle_mode,
        SubtitleMode::Subtitles | SubtitleMode::Both
    );
    let wants_auto_subtitles = matches!(
        request.subtitle_mode,
        SubtitleMode::Auto | SubtitleMode::Both
    );
    let wants_danmaku = !matches!(request.danmaku_format, DanmakuFormat::None);
    if wants_regular_subtitles || wants_danmaku {
        args.push("--write-subs".to_string());
    }

    if wants_auto_subtitles {
        args.push("--write-auto-subs".to_string());
    }

    if wants_regular_subtitles || wants_auto_subtitles || wants_danmaku {
        args.extend(["--sub-langs".to_string(), "all".to_string()]);
        args.extend([
            "--sub-format".to_string(),
            subtitle_format_arg(&request.subtitle_format).to_string(),
        ]);
    }

    if request.embed_subtitles && !matches!(request.subtitle_mode, SubtitleMode::Off) {
        args.push("--embed-subs".to_string());
    }

    args.extend(build_cookie_args(
        &request.cookie_mode,
        request.manual_cookie_path.as_deref(),
    ));

    args.extend([
        "--print".to_string(),
        "after_move:downloaded:%(filepath)s".to_string(),
    ]);
    args.extend(request.urls.iter().cloned());
    args
}

fn subtitle_format_arg(format: &SubtitleFormat) -> &'static str {
    match format {
        SubtitleFormat::Srt => "srt/best",
        SubtitleFormat::Vtt => "vtt/best",
    }
}

fn spawn_line_reader<R>(
    app: AppHandle,
    job_id: String,
    stream: R,
    is_error: bool,
    output_paths: Option<Arc<Mutex<Vec<PathBuf>>>>,
) where
    R: tokio::io::AsyncRead + Unpin + Send + 'static,
{
    tauri::async_runtime::spawn(async move {
        let mut lines = BufReader::new(stream).lines();

        while let Ok(Some(line)) = lines.next_line().await {
            if let Some(path) = line.strip_prefix("downloaded:") {
                let path = PathBuf::from(path.trim());
                if let Some(paths) = &output_paths {
                    paths.lock().await.push(path.clone());
                }

                emit_event(
                    &app,
                    DownloadEvent {
                        job_id: job_id.clone(),
                        status: "running".to_string(),
                        percent: None,
                        speed: None,
                        eta: None,
                        line: Some(format!("Saved file: {}", path.display())),
                        output_path: Some(path.display().to_string()),
                        media_report: None,
                    },
                );
                continue;
            }

            let event = parse_progress_line(&job_id, &line).unwrap_or_else(|| DownloadEvent {
                job_id: job_id.clone(),
                status: if is_error { "warning" } else { "running" }.to_string(),
                percent: None,
                speed: None,
                eta: None,
                line: Some(if is_error {
                    friendly_error(&line)
                } else {
                    line.clone()
                }),
                output_path: None,
                media_report: None,
            });

            emit_event(&app, event);
        }
    });
}

fn parse_progress_line(job_id: &str, line: &str) -> Option<DownloadEvent> {
    let payload = line.strip_prefix("download:")?;
    let mut parts = payload.split('|');
    let percent = parts
        .next()
        .and_then(|value| value.trim().trim_end_matches('%').parse::<f32>().ok());
    let speed = parts
        .next()
        .map(str::trim)
        .filter(|value| !value.is_empty() && *value != "N/A")
        .map(ToString::to_string);
    let eta = parts
        .next()
        .map(str::trim)
        .filter(|value| !value.is_empty() && *value != "N/A")
        .map(ToString::to_string);

    Some(DownloadEvent {
        job_id: job_id.to_string(),
        status: "running".to_string(),
        percent,
        speed,
        eta,
        line: Some(line.to_string()),
        output_path: None,
        media_report: None,
    })
}

fn emit_event(app: &AppHandle, event: DownloadEvent) {
    let _ = app.emit("download-event", event);
}

fn stderr_or_stdout(output: &std::process::Output) -> String {
    if output.stderr.is_empty() {
        String::from_utf8_lossy(&output.stdout).to_string()
    } else {
        String::from_utf8_lossy(&output.stderr).to_string()
    }
}

fn friendly_error(text: &str) -> String {
    let lower = text.to_ascii_lowercase();
    if lower.contains("http error 412") || lower.contains("precondition failed") {
        "HTTP 412 Precondition Failed. This site rejected the anonymous request. Use Chrome cookies or import cookies.txt, then retry.".to_string()
    } else if lower.contains("http error 403") || lower.contains("forbidden") {
        "HTTP 403 Forbidden. The site refused access. Try Chrome cookies/cookies.txt, check login status, or use a proxy if the region is blocked.".to_string()
    } else if lower.contains("captcha") {
        "The site is asking for captcha/verification. Open the site in Chrome, complete verification, then retry with Chrome cookies.".to_string()
    } else if lower.contains("login") || lower.contains("sign in") {
        "Login appears to be required. Use Chrome cookies from a logged-in browser or import cookies.txt.".to_string()
    } else if lower.contains("requested format is not available") {
        "The selected format is not available for this URL. Try Best quality or Keep original codec.".to_string()
    } else if lower.contains("ffmpeg") && lower.contains("not found") {
        "ffmpeg was not found, so merge/convert may fail. Reinstall or refetch the sidecar binaries.".to_string()
    } else {
        text.to_string()
    }
}

fn parse_metadata_json(stdout: &str) -> Result<serde_json::Value, String> {
    let trimmed = stdout.trim();
    if trimmed.is_empty() {
        return Err("yt-dlp did not return metadata JSON.".to_string());
    }

    if let Ok(value) = serde_json::from_str(trimmed) {
        return Ok(value);
    }

    let json_line = stdout
        .lines()
        .rev()
        .find(|line| line.trim_start().starts_with('{'))
        .ok_or_else(|| "yt-dlp did not return metadata JSON.".to_string())?;

    serde_json::from_str(json_line)
        .map_err(|error| format!("Failed to parse metadata JSON: {error}"))
}

fn metadata_previews_from_value(
    source_url: &str,
    value: &serde_json::Value,
) -> Vec<MetadataPreview> {
    let entries = value
        .get("entries")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default();

    if entries.is_empty() {
        return vec![metadata_from_value(source_url, value, None, None, None)];
    }

    let playlist_title = json_string(value, "title")
        .or_else(|| json_string(value, "playlist_title"))
        .or_else(|| json_string(value, "series"));
    let playlist_count = Some(entries.iter().filter(|entry| !entry.is_null()).count() as u64);

    entries
        .iter()
        .enumerate()
        .filter(|(_, entry)| !entry.is_null())
        .map(|(index, entry)| {
            metadata_from_value(
                source_url,
                entry,
                playlist_title.clone(),
                Some((index + 1) as u64),
                playlist_count,
            )
        })
        .collect()
}

fn metadata_from_value(
    source_url: &str,
    value: &serde_json::Value,
    playlist_title: Option<String>,
    playlist_index: Option<u64>,
    playlist_count: Option<u64>,
) -> MetadataPreview {
    let formats = value
        .get("formats")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default();
    let video_codecs = unique_codecs(&formats, "vcodec");
    let audio_codecs = unique_codecs(&formats, "acodec");
    let (best_width, best_height) = best_resolution(&formats);
    let platform = value
        .get("extractor_key")
        .or_else(|| value.get("extractor"))
        .and_then(serde_json::Value::as_str)
        .map(ToString::to_string)
        .unwrap_or_else(|| detect_platform(source_url).to_string());
    let item_url = metadata_download_url(source_url, value);
    let requires_session = get_session_required_host(source_url).is_some();

    MetadataPreview {
        url: item_url,
        source_url: source_url.to_string(),
        title: json_string(value, "title"),
        thumbnail: json_string(value, "thumbnail")
            .or_else(|| {
                value
                    .get("thumbnails")
                    .and_then(serde_json::Value::as_array)
                    .and_then(|items| items.first())
                    .and_then(|thumb| thumb.get("url"))
                    .and_then(serde_json::Value::as_str)
                    .map(ToString::to_string)
            })
            .and_then(|url| normalize_thumbnail_url(&url)),
        duration: value.get("duration").and_then(serde_json::Value::as_f64),
        uploader: json_string(value, "uploader").or_else(|| json_string(value, "channel")),
        platform,
        webpage_url: json_string(value, "webpage_url"),
        playlist_title,
        playlist_index,
        playlist_count,
        format_count: formats.len(),
        best_width,
        best_height,
        recommended_preset: recommended_preset(&video_codecs, &audio_codecs),
        video_codecs,
        audio_codecs,
        requires_session,
        warning: if requires_session {
            Some("This platform often needs a logged-in session.".to_string())
        } else {
            None
        },
    }
}

fn metadata_download_url(source_url: &str, value: &serde_json::Value) -> String {
    for key in ["webpage_url", "original_url", "url"] {
        if let Some(candidate) = json_string(value, key) {
            if candidate.starts_with("http://") || candidate.starts_with("https://") {
                return candidate;
            }
        }
    }

    source_url.to_string()
}

fn normalize_thumbnail_url(url: &str) -> Option<String> {
    let trimmed = url.trim();
    if trimmed.is_empty() {
        None
    } else if trimmed.starts_with("//") {
        Some(format!("https:{trimmed}"))
    } else if let Some(rest) = trimmed.strip_prefix("http://") {
        Some(format!("https://{rest}"))
    } else {
        Some(trimmed.to_string())
    }
}

fn best_resolution(formats: &[serde_json::Value]) -> (Option<u64>, Option<u64>) {
    formats
        .iter()
        .filter_map(|format| {
            let width = format.get("width").and_then(serde_json::Value::as_u64);
            let height = format.get("height").and_then(serde_json::Value::as_u64);
            height.map(|height| (width, height))
        })
        .max_by_key(|(_, height)| *height)
        .map(|(width, height)| (width, Some(height)))
        .unwrap_or((None, None))
}

fn recommended_preset(video_codecs: &[String], audio_codecs: &[String]) -> String {
    let has_h264 = video_codecs
        .iter()
        .any(|codec| codec.starts_with("avc1") || codec == "h264");
    let has_aac = audio_codecs
        .iter()
        .any(|codec| codec.starts_with("mp4a") || codec == "aac");
    if has_h264 && has_aac {
        "MP4".to_string()
    } else if has_h264 {
        "MP4, with audio remux".to_string()
    } else {
        "Best, then convert H.264 if needed".to_string()
    }
}

fn unique_codecs(formats: &[serde_json::Value], key: &str) -> Vec<String> {
    let mut codecs = Vec::new();
    for format in formats {
        let Some(codec) = format.get(key).and_then(serde_json::Value::as_str) else {
            continue;
        };
        if codec != "none" && !codecs.iter().any(|known| known == codec) {
            codecs.push(codec.to_string());
        }
    }
    codecs.truncate(8);
    codecs
}

fn json_string(value: &serde_json::Value, key: &str) -> Option<String> {
    value
        .get(key)
        .and_then(serde_json::Value::as_str)
        .map(ToString::to_string)
}

fn detect_platform(url: &str) -> &'static str {
    let lower = url.to_ascii_lowercase();
    if lower.contains("bilibili.com")
        || lower.contains("b23.tv")
        || lower.contains("space.bilibili.com")
    {
        "BiliBili"
    } else if lower.contains("douyin.com")
        || lower.contains("iesdouyin.com")
        || lower.contains("amemv.com")
    {
        "Douyin"
    } else if lower.contains("youtube.com") || lower.contains("youtu.be") {
        "YouTube"
    } else {
        "Generic"
    }
}

fn is_bilibili_channel_url(url: &str) -> bool {
    let lower = url.to_ascii_lowercase();
    lower.contains("space.bilibili.com")
        || lower.contains("bilibili.com/space/")
        || lower.contains("space.bilibili.com/")
}

fn default_settings() -> AppSettings {
    AppSettings {
        download_dir: String::new(),
        cookie_mode: "none".to_string(),
        manual_cookie_path: String::new(),
        download_preset: "compatibleMp4".to_string(),
        subtitle_mode: default_subtitle_mode(),
        subtitle_format: default_subtitle_format(),
        embed_subtitles: false,
        danmaku_format: default_danmaku_format(),
        cookie_profiles: HashMap::new(),
    }
}

fn default_subtitle_mode() -> String {
    "off".to_string()
}

fn default_subtitle_format() -> String {
    "srt".to_string()
}

fn default_danmaku_format() -> String {
    "none".to_string()
}

fn settings_path(app: &AppHandle) -> Option<PathBuf> {
    app.path()
        .app_config_dir()
        .ok()
        .map(|dir| dir.join("settings.json"))
}

fn get_session_required_host(url: &str) -> Option<&'static str> {
    session_required_host(&[url.to_string()])
}

async fn probe_media_with(ffprobe: &Path, path: &Path) -> Result<MediaReport, String> {
    if !path.is_file() {
        return Err("The selected media file does not exist.".to_string());
    }

    let output = Command::new(ffprobe)
        .args([
            "-v",
            "quiet",
            "-print_format",
            "json",
            "-show_format",
            "-show_streams",
            path.to_str()
                .ok_or_else(|| "Media path is not valid UTF-8.".to_string())?,
        ])
        .output()
        .await
        .map_err(|error| format!("Failed to run ffprobe: {error}"))?;

    if !output.status.success() {
        return Err(friendly_error(&stderr_or_stdout(&output)));
    }

    let value: serde_json::Value = serde_json::from_slice(&output.stdout)
        .map_err(|error| format!("Failed to parse ffprobe JSON: {error}"))?;
    Ok(media_report_from_value(path, &value))
}

fn media_report_from_value(path: &Path, value: &serde_json::Value) -> MediaReport {
    let streams = value
        .get("streams")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default();
    let video = streams.iter().find(|stream| {
        stream.get("codec_type").and_then(serde_json::Value::as_str) == Some("video")
    });
    let audio = streams.iter().find(|stream| {
        stream.get("codec_type").and_then(serde_json::Value::as_str) == Some("audio")
    });

    let video_codec = video.and_then(|stream| json_string(stream, "codec_name"));
    let audio_codec = audio.and_then(|stream| json_string(stream, "codec_name"));
    let video_tag = video.and_then(|stream| json_string(stream, "codec_tag_string"));
    let audio_tag = audio.and_then(|stream| json_string(stream, "codec_tag_string"));
    let quicktime_compatible = video_codec
        .as_deref()
        .map(|codec| matches!(codec, "h264" | "mpeg4"))
        .unwrap_or(true)
        && audio_codec
            .as_deref()
            .map(|codec| matches!(codec, "aac" | "mp3" | "alac"))
            .unwrap_or(true);

    let warning = if video_codec.as_deref() == Some("av1") {
        Some("Video codec is AV1. QuickTime/Finder may play audio only. Convert to H.264 for compatibility.".to_string())
    } else if video_codec.as_deref() == Some("hevc") {
        Some("Video codec is HEVC. Some players may need conversion to H.264.".to_string())
    } else if !quicktime_compatible {
        Some("This file may not be fully compatible with QuickTime. Convert to H.264 if playback fails.".to_string())
    } else {
        None
    };

    MediaReport {
        path: path.display().to_string(),
        file_size: fs::metadata(path).ok().map(|meta| meta.len()),
        container: value
            .get("format")
            .and_then(|format| json_string(format, "format_name")),
        duration: value
            .get("format")
            .and_then(|format| format.get("duration"))
            .and_then(serde_json::Value::as_str)
            .and_then(|duration| duration.parse::<f64>().ok()),
        video_codec,
        video_tag,
        audio_codec,
        audio_tag,
        width: video
            .and_then(|stream| stream.get("width"))
            .and_then(serde_json::Value::as_u64),
        height: video
            .and_then(|stream| stream.get("height"))
            .and_then(serde_json::Value::as_u64),
        quicktime_compatible,
        warning,
    }
}

fn media_report_message(report: &MediaReport) -> String {
    let video = report.video_codec.as_deref().unwrap_or("no video");
    let audio = report.audio_codec.as_deref().unwrap_or("no audio");
    match &report.warning {
        Some(warning) => format!("Download completed. Video: {video}, audio: {audio}. {warning}"),
        None => format!("Download completed. Video: {video}, audio: {audio}."),
    }
}

fn h264_output_path(input: &Path) -> PathBuf {
    let parent = input.parent().unwrap_or_else(|| Path::new("."));
    let stem = input
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("converted");
    let mut candidate = parent.join(format!("{stem} H264.mp4"));
    let mut index = 2;
    while candidate.exists() {
        candidate = parent.join(format!("{stem} H264 {index}.mp4"));
        index += 1;
    }
    candidate
}

fn convert_danmaku_sidecars(video_path: &Path) -> Result<Vec<String>, String> {
    let parent = video_path
        .parent()
        .ok_or_else(|| "Could not find downloaded file folder.".to_string())?;
    let stem = video_path
        .file_stem()
        .and_then(|value| value.to_str())
        .ok_or_else(|| "Downloaded file name is not valid UTF-8.".to_string())?;
    let candidates = fs::read_dir(parent)
        .map_err(|error| format!("Failed to scan downloaded folder: {error}"))?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.extension()
                .and_then(|value| value.to_str())
                .map(|extension| extension.eq_ignore_ascii_case("xml"))
                .unwrap_or(false)
        })
        .filter(|path| {
            path.file_name()
                .and_then(|value| value.to_str())
                .map(|name| name.starts_with(stem))
                .unwrap_or(false)
        })
        .collect::<Vec<_>>();

    let mut messages = Vec::new();
    for xml_path in candidates {
        let ass_path = xml_path.with_extension("ass");
        convert_danmaku_xml_to_ass(&xml_path, &ass_path)?;
        messages.push(format!("Converted danmaku ASS: {}", ass_path.display()));
    }
    Ok(messages)
}

fn convert_danmaku_xml_to_ass(xml_path: &Path, ass_path: &Path) -> Result<(), String> {
    let xml = fs::read_to_string(xml_path)
        .map_err(|error| format!("Failed to read danmaku XML: {error}"))?;
    let re = regex::Regex::new(r#"<d\s+p="([^"]+)">([\s\S]*?)</d>"#)
        .map_err(|error| format!("Failed to prepare danmaku parser: {error}"))?;
    let mut events = Vec::new();

    for (index, captures) in re.captures_iter(&xml).enumerate() {
        let Some(p) = captures.get(1).map(|value| value.as_str()) else {
            continue;
        };
        let Some(text) = captures.get(2).map(|value| value.as_str()) else {
            continue;
        };
        let parts = p.split(',').collect::<Vec<_>>();
        let start = parts
            .first()
            .and_then(|value| value.parse::<f64>().ok())
            .unwrap_or(0.0);
        let mode = parts
            .get(1)
            .and_then(|value| value.parse::<u32>().ok())
            .unwrap_or(1);
        let size = parts
            .get(2)
            .and_then(|value| value.parse::<u32>().ok())
            .unwrap_or(25)
            .clamp(18, 48);
        let color = parts
            .get(3)
            .and_then(|value| value.parse::<u32>().ok())
            .unwrap_or(16_777_215);
        let event = danmaku_ass_event(index, start, mode, size, color, text);
        events.push(event);
    }

    if events.is_empty() {
        return Err(format!(
            "No danmaku comments found in {}",
            xml_path.display()
        ));
    }

    let mut ass = String::from(
        "[Script Info]\nScriptType: v4.00+\nPlayResX: 1280\nPlayResY: 720\nWrapStyle: 2\nScaledBorderAndShadow: yes\n\n[V4+ Styles]\nFormat: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding\nStyle: Default,Arial,32,&H00FFFFFF,&H00FFFFFF,&H80000000,&H80000000,0,0,0,0,100,100,0,0,1,1.5,0,7,20,20,20,1\n\n[Events]\nFormat: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\n",
    );
    ass.push_str(&events.join("\n"));
    ass.push('\n');
    fs::write(ass_path, ass).map_err(|error| format!("Failed to write danmaku ASS: {error}"))
}

fn danmaku_ass_event(
    index: usize,
    start: f64,
    mode: u32,
    size: u32,
    color: u32,
    text: &str,
) -> String {
    let end = start + if mode == 4 || mode == 5 { 4.0 } else { 8.0 };
    let row = index % 14;
    let y = 32 + (row * 46);
    let color = ass_color(color);
    let text = ass_escape(&xml_unescape(text));
    let override_block = match mode {
        4 => format!(
            "{{\\an2\\pos(640,{})\\fs{}\\c{}}}",
            680 - (row * 38),
            size,
            color
        ),
        5 => format!("{{\\an8\\pos(640,{})\\fs{}\\c{}}}", y, size, color),
        _ => format!("{{\\move(1280,{y},-560,{y})\\fs{}\\c{}}}", size, color),
    };

    format!(
        "Dialogue: 0,{},{},Default,,0,0,0,,{}{}",
        ass_time(start),
        ass_time(end),
        override_block,
        text
    )
}

fn ass_time(seconds: f64) -> String {
    let centiseconds = (seconds.max(0.0) * 100.0).round() as u64;
    let cs = centiseconds % 100;
    let total_seconds = centiseconds / 100;
    let s = total_seconds % 60;
    let total_minutes = total_seconds / 60;
    let m = total_minutes % 60;
    let h = total_minutes / 60;
    format!("{h}:{m:02}:{s:02}.{cs:02}")
}

fn ass_color(rgb: u32) -> String {
    let r = (rgb >> 16) & 0xff;
    let g = (rgb >> 8) & 0xff;
    let b = rgb & 0xff;
    format!("&H00{b:02X}{g:02X}{r:02X}&")
}

fn ass_escape(text: &str) -> String {
    text.replace('\\', "\\\\")
        .replace('{', "\\{")
        .replace('}', "\\}")
        .replace('\n', "\\N")
}

fn xml_unescape(text: &str) -> String {
    text.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&amp;", "&")
}

fn image_extension(content_type: Option<&str>, url: &str) -> String {
    if let Some(content_type) = content_type {
        if content_type.contains("png") {
            return "png".to_string();
        }
        if content_type.contains("webp") {
            return "webp".to_string();
        }
        if content_type.contains("gif") {
            return "gif".to_string();
        }
        if content_type.contains("jpeg") || content_type.contains("jpg") {
            return "jpg".to_string();
        }
    }

    let clean_url = url.split('?').next().unwrap_or(url).to_ascii_lowercase();
    for extension in ["jpg", "jpeg", "png", "webp", "gif"] {
        if clean_url.ends_with(&format!(".{extension}")) {
            return if extension == "jpeg" {
                "jpg".to_string()
            } else {
                extension.to_string()
            };
        }
    }

    "jpg".to_string()
}

fn sanitize_filename(value: &str) -> String {
    let mut name = value
        .chars()
        .map(|ch| match ch {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            ch if ch.is_control() => '_',
            ch => ch,
        })
        .collect::<String>()
        .trim()
        .trim_matches('.')
        .to_string();

    if name.is_empty() {
        name = "cover".to_string();
    }
    name.chars().take(120).collect()
}

fn unique_filename(dir: &Path, stem: &str, extension: &str) -> PathBuf {
    let mut candidate = dir.join(format!("{stem} cover.{extension}"));
    let mut index = 2;
    while candidate.exists() {
        candidate = dir.join(format!("{stem} cover {index}.{extension}"));
        index += 1;
    }
    candidate
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _, _| {
            focus_main_window(app);
        }))
        .plugin(tauri_plugin_dialog::init())
        .manage(DownloadState::default())
        .manage(ExtensionState::default())
        .setup(|app| {
            let extension_state = app.state::<ExtensionState>().inner().clone();
            start_ipc_server(app.handle().clone(), extension_state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_tool_versions,
            load_settings,
            save_settings,
            drain_extension_imports,
            get_chrome_integration_status,
            install_chrome_integration,
            remove_chrome_integration,
            test_chrome_integration,
            fetch_metadata,
            download_cover,
            export_browser_cookies,
            import_cookie_file,
            validate_cookie_file,
            delete_cookie_file,
            start_download,
            cancel_download,
            probe_media,
            convert_to_h264,
            open_path,
            reveal_path
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn request(cookie_mode: CookieMode, manual_cookie_path: Option<String>) -> DownloadRequest {
        DownloadRequest {
            urls: vec!["https://example.com/watch?v=a&b=c".to_string()],
            download_dir: "/tmp/download folder".to_string(),
            preset: DownloadPreset::CompatibleMp4,
            cookie_mode,
            manual_cookie_path,
            subtitle_mode: SubtitleMode::Off,
            subtitle_format: SubtitleFormat::Srt,
            embed_subtitles: false,
            danmaku_format: DanmakuFormat::None,
        }
    }

    #[test]
    fn native_message_framing_round_trips() {
        let response = response_ok(
            "request-1".to_string(),
            "ready",
            Some(vec!["https://example.com/video".to_string()]),
        );
        let mut bytes = Vec::new();
        write_native_frame(&mut bytes, &response).expect("frame should serialize");
        let payload = read_native_frame(&mut Cursor::new(bytes))
            .expect("frame should read")
            .expect("frame should exist");
        let decoded: NativeResponse =
            serde_json::from_slice(&payload).expect("response should decode");

        assert!(decoded.ok);
        assert_eq!(decoded.id, "request-1");
        assert_eq!(
            decoded.accepted_urls,
            Some(vec!["https://example.com/video".to_string()])
        );
    }

    #[test]
    fn native_request_rejects_unknown_version_and_action() {
        let invalid_version = NativeRequest {
            version: 2,
            id: "one".to_string(),
            action: "ping".to_string(),
            urls: Vec::new(),
            source: None,
        };
        let invalid_action = NativeRequest {
            version: 1,
            id: "two".to_string(),
            action: "launch_missiles".to_string(),
            urls: Vec::new(),
            source: None,
        };

        assert!(validate_native_request(&invalid_version).is_err());
        assert!(validate_native_request(&invalid_action).is_err());
    }

    #[test]
    fn netscape_cookie_files_are_validated() {
        let path = std::env::temp_dir().join(format!("sorevid-cookie-test-{}.txt", Uuid::new_v4()));
        fs::write(
            &path,
            "# Netscape HTTP Cookie File\n.example.com\tTRUE\t/\tTRUE\t2147483647\tsession\tsecret\n#HttpOnly_.example.com\tTRUE\t/\tTRUE\t2147483647\tauth\tsecret\n",
        )
        .expect("cookie fixture should write");

        let status = cookie_file_status(&path).expect("cookie fixture should validate");
        let _ = fs::remove_file(path);

        assert!(status.valid);
        assert_eq!(status.cookie_count, 2);
    }

    #[test]
    fn imported_urls_are_normalized_and_deduplicated() {
        let values = [
            "bilibili.com/video/BV1oVXaBEEF6/",
            "https://bilibili.com/video/BV1oVXaBEEF6/",
            "not a URL",
        ];
        let mut seen = HashSet::new();
        let normalized = values
            .iter()
            .filter_map(|value| normalize_url_candidate(value))
            .filter(|value| is_http_url(value))
            .filter(|value| seen.insert(value.clone()))
            .collect::<Vec<_>>();

        assert_eq!(
            normalized,
            vec!["https://bilibili.com/video/BV1oVXaBEEF6/".to_string()]
        );
    }

    #[test]
    fn chrome_cookie_mode_adds_cookies_from_browser_args() {
        let args = build_yt_dlp_args(&request(CookieMode::Chrome, None), None);

        assert!(args
            .windows(2)
            .any(|pair| pair == ["--cookies-from-browser", "chrome"]));
        assert!(!args.iter().any(|arg| arg == "--cookies"));
    }

    #[test]
    fn manual_cookie_mode_adds_cookie_file_args() {
        let args = build_yt_dlp_args(
            &request(
                CookieMode::Manual,
                Some("/tmp/cookies file.txt".to_string()),
            ),
            None,
        );

        assert!(args
            .windows(2)
            .any(|pair| pair == ["--cookies", "/tmp/cookies file.txt"]));
        assert!(!args.iter().any(|arg| arg == "--cookies-from-browser"));
    }

    #[test]
    fn url_and_download_path_remain_single_arguments() {
        let args = build_yt_dlp_args(&request(CookieMode::None, None), None);

        assert!(args.iter().any(|arg| arg == "/tmp/download folder"));
        assert!(args
            .iter()
            .any(|arg| arg == "https://example.com/watch?v=a&b=c"));
    }

    #[test]
    fn bare_video_links_are_normalized_with_https() {
        assert_eq!(
            normalize_url_candidate("bilibili.com/video/BV1oVXaBEEF6/"),
            Some("https://bilibili.com/video/BV1oVXaBEEF6/".to_string())
        );
        assert_eq!(
            normalize_url_candidate("v.douyin.com/U6b4BxLLFQ8/"),
            Some("https://v.douyin.com/U6b4BxLLFQ8/".to_string())
        );
        assert_eq!(
            normalize_url_candidate(
                "youtube.com/watch?v=C_cx4B1IaC4&list=RDLVQ9eobn_pY&index=2&pp=8AUB"
            ),
            Some("https://youtube.com/watch?v=C_cx4B1IaC4".to_string())
        );
        assert_eq!(
            normalize_url_candidate("youtube.com/playlist?list=RDLVQ9eobn_pY"),
            Some("https://youtube.com/playlist?list=RDLVQ9eobn_pY".to_string())
        );
    }

    #[test]
    fn best_video_prefers_quicktime_compatible_mp4() {
        let args = build_yt_dlp_args(&request(CookieMode::None, None), Some("/tmp/ffmpeg-bin"));

        assert!(args
            .windows(2)
            .any(|pair| pair == ["--merge-output-format", "mp4"]));
        assert!(args
            .windows(2)
            .any(|pair| pair == ["--ffmpeg-location", "/tmp/ffmpeg-bin"]));
        assert!(args
            .windows(2)
            .any(|pair| pair[0] == "-f" && pair[1].contains("vcodec^=avc1")));
    }

    #[test]
    fn bilibili_without_cookies_is_blocked_before_download() {
        let mut request = request(CookieMode::None, None);
        request.urls = vec!["https://www.bilibili.com/video/BV1oVXaBEEF6/".to_string()];
        request.download_dir = std::env::temp_dir().display().to_string();

        let error = validate_request(&request).expect_err("request should require cookies");
        assert!(error.contains("bilibili.com"));
    }

    #[test]
    fn bilibili_with_chrome_cookies_can_start() {
        let mut request = request(CookieMode::Chrome, None);
        request.urls = vec!["https://www.bilibili.com/video/BV1oVXaBEEF6/".to_string()];
        request.download_dir = std::env::temp_dir().display().to_string();

        assert!(validate_request(&request).is_ok());
    }

    #[test]
    fn bilibili_channel_urls_are_detected() {
        assert!(is_bilibili_channel_url(
            "https://space.bilibili.com/6087825?spm_id_from=333.1007.tianma.8-1-27.click"
        ));
    }

    #[test]
    fn subtitle_options_add_yt_dlp_subtitle_args() {
        let mut request = request(CookieMode::None, None);
        request.subtitle_mode = SubtitleMode::Both;
        request.subtitle_format = SubtitleFormat::Vtt;
        request.embed_subtitles = true;
        let args = build_yt_dlp_args(&request, Some("/tmp/ffmpeg-bin"));

        assert!(args.iter().any(|arg| arg == "--write-subs"));
        assert!(args.iter().any(|arg| arg == "--write-auto-subs"));
        assert!(args
            .windows(2)
            .any(|pair| pair == ["--sub-format", "vtt/best"]));
        assert!(args.iter().any(|arg| arg == "--embed-subs"));
    }

    #[test]
    fn danmaku_ass_requests_subtitle_sidecars() {
        let mut request = request(CookieMode::None, None);
        request.danmaku_format = DanmakuFormat::Ass;
        let args = build_yt_dlp_args(&request, None);

        assert!(args.iter().any(|arg| arg == "--write-subs"));
        assert!(args.windows(2).any(|pair| pair == ["--sub-langs", "all"]));
    }
}
