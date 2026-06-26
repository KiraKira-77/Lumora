use std::{
    hash::{Hash, Hasher},
    path::{Path, PathBuf},
    sync::{Mutex, OnceLock},
};

use tauri::{window::Color, Emitter, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

#[derive(serde::Serialize)]
struct AppInfo {
    name: String,
    version: String,
}

#[derive(serde::Serialize)]
struct DesktopCategoryCount {
    category: String,
    label: String,
    count: usize,
}

#[derive(serde::Serialize)]
struct DesktopFile {
    name: String,
    path: String,
    category: String,
    label: String,
}

#[derive(serde::Serialize)]
struct DesktopScanResult {
    desktop_path: String,
    files: Vec<DesktopFile>,
    counts: Vec<DesktopCategoryCount>,
}

#[derive(serde::Serialize)]
struct DockTarget {
    label: String,
    #[serde(rename = "type")]
    item_type: String,
    target: String,
    #[serde(rename = "iconPath")]
    icon_path: Option<String>,
    #[serde(rename = "originalDesktopPath")]
    original_desktop_path: Option<String>,
}

struct DockHidePlan {
    original_path: PathBuf,
    hidden_dir: PathBuf,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct LaunchIdentity {
    executable_path: Option<PathBuf>,
    executable_name: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct RunningProcess {
    executable_path: Option<PathBuf>,
    executable_name: String,
}

#[derive(Clone, Debug, serde::Serialize)]
struct DockItemStatus {
    target: String,
    #[serde(rename = "isRunning")]
    is_running: bool,
    #[serde(rename = "needsAttention")]
    needs_attention: bool,
    #[serde(rename = "attentionSequence")]
    attention_sequence: u64,
}

#[derive(Clone, Debug)]
struct DockAttentionEntry {
    identity: LaunchIdentity,
    sequence: u64,
}

#[derive(Debug, Default)]
struct DockAttentionState {
    entries: Vec<DockAttentionEntry>,
    next_sequence: u64,
}

static DOCK_ATTENTION_STATE: OnceLock<Mutex<DockAttentionState>> = OnceLock::new();

#[cfg(target_os = "windows")]
#[derive(Clone, Debug)]
struct DockActivationEntry {
    identity: LaunchIdentity,
    hwnd: isize,
}

#[cfg(target_os = "windows")]
static DOCK_ACTIVATION_STATE: OnceLock<Mutex<Vec<DockActivationEntry>>> = OnceLock::new();

#[cfg(target_os = "windows")]
static LUMORA_APP_HANDLE: OnceLock<tauri::AppHandle> = OnceLock::new();

#[cfg(target_os = "windows")]
static SHELL_HOOK_MESSAGE: OnceLock<u32> = OnceLock::new();

#[derive(Clone, serde::Deserialize, serde::Serialize)]
struct MovedDesktopFile {
    name: String,
    category: String,
    label: String,
    original_path: String,
    current_path: String,
}

#[derive(serde::Serialize)]
struct SkippedDesktopFile {
    path: String,
    reason: String,
}

#[derive(serde::Serialize)]
struct DesktopOrganizeResult {
    target_root: String,
    moved_files: Vec<MovedDesktopFile>,
    skipped_files: Vec<SkippedDesktopFile>,
}

#[derive(serde::Serialize)]
struct DesktopUndoResult {
    restored_files: Vec<MovedDesktopFile>,
    skipped_files: Vec<SkippedDesktopFile>,
}

#[derive(serde::Serialize)]
struct FileSearchItem {
    name: String,
    path: String,
    category: String,
    label: String,
}

#[derive(serde::Serialize)]
struct FileSearchResult {
    query: String,
    roots: Vec<String>,
    total_matches: usize,
    files: Vec<FileSearchItem>,
}

#[tauri::command]
fn app_info() -> AppInfo {
    AppInfo {
        name: "Lumora".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    }
}

#[tauri::command]
fn open_target(target: String) -> Result<String, String> {
    if target.starts_with("lumora://") {
        return Ok(format!("Handled internal action: {target}"));
    }

    let expanded = expand_user_path(&target);

    #[cfg(target_os = "windows")]
    let status = std::process::Command::new("cmd")
        .arg("/C")
        .arg("start")
        .arg("")
        .arg(&expanded)
        .status();

    #[cfg(target_os = "macos")]
    let status = std::process::Command::new("open").arg(&expanded).status();

    #[cfg(all(unix, not(target_os = "macos")))]
    let status = std::process::Command::new("xdg-open").arg(&expanded).status();

    match status {
        Ok(_) => Ok(format!("Opened {expanded}")),
        Err(error) => Err(format!("Failed to open {expanded}: {error}")),
    }
}

#[tauri::command]
fn activate_or_open_target(target: String) -> Result<String, String> {
    if target.starts_with("lumora://") {
        return Ok(format!("Handled internal action: {target}"));
    }

    let cleared_attention = clear_dock_attention_for_target(&target);
    #[cfg(target_os = "windows")]
    if cleared_attention {
        emit_dock_attention_changed();
    }

    #[cfg(target_os = "windows")]
    {
        if let Some(identity) = launch_identity_for_target(&target) {
            let processes = running_processes();
            if is_tray_managed_launch_identity(&identity) {
                if try_activate_tray_icon_for_identity(&identity) {
                    return Ok(format!("Activated {} tray icon", identity.executable_name));
                }
            }
            if should_suppress_open_after_activation_miss(&identity, &processes) {
                return Ok(format!("{} is running without a safe activation window", identity.executable_name));
            }
            if let Some(hwnd) = find_window_for_launch_identity(&identity) {
                activate_window(hwnd);
                remember_window_for_launch_identity(identity.clone(), hwnd);
                return Ok(format!("Activated {}", identity.executable_name));
            }
            if let Some(hwnd) = remembered_window_for_launch_identity(&identity) {
                activate_window(hwnd);
                return Ok(format!("Activated {}", identity.executable_name));
            }
        }
    }

    open_target(target)
}

#[tauri::command]
fn dock_item_statuses(targets: Vec<String>) -> Vec<DockItemStatus> {
    dock_item_statuses_for_targets(targets)
}

#[tauri::command]
fn clear_dock_item_attention(target: String) -> Result<(), String> {
    if clear_dock_attention_for_target(&target) {
        #[cfg(target_os = "windows")]
        emit_dock_attention_changed();
    }
    Ok(())
}

#[tauri::command]
fn describe_targets(app: tauri::AppHandle, paths: Vec<String>) -> Vec<DockTarget> {
    let icon_cache_dir = app.path().app_cache_dir().ok().map(|path| path.join("icons"));
    let desktop_roots = desktop_roots();

    paths
        .iter()
        .map(|path| {
            let pb = resolve_dropped_path(path, &desktop_roots);
            let mut target = describe_target_with_icon(&pb, icon_cache_dir.as_deref());
            target.original_desktop_path = None;
            target
        })
        .collect()
}

#[tauri::command]
fn scan_desktop() -> Result<DesktopScanResult, String> {
    let desktop = desktop_path()?;
    let files = scan_desktop_files(&desktop)?;
    let counts = build_counts(&files);

    Ok(DesktopScanResult {
        desktop_path: desktop,
        files,
        counts,
    })
}

#[tauri::command]
fn search_files(query: String, roots: Option<Vec<String>>, limit: Option<usize>) -> FileSearchResult {
    let search_roots = roots
        .filter(|roots| !roots.is_empty())
        .map(|roots| roots.into_iter().map(|root| PathBuf::from(expand_user_path(&root))).collect())
        .unwrap_or_else(default_search_roots);

    search_files_in_roots(&query, &search_roots, limit.unwrap_or(30).clamp(1, 80), 8000)
}

#[tauri::command]
fn organize_desktop(paths: Option<Vec<String>>) -> Result<DesktopOrganizeResult, String> {
    let desktop = PathBuf::from(desktop_path()?);
    let desktop_canonical = desktop.canonicalize().map_err(|error| error.to_string())?;
    let target_root = desktop.join("Lumora整理");
    let candidate_paths: Vec<PathBuf> = match paths {
        Some(paths) if !paths.is_empty() => paths.into_iter().map(|path| PathBuf::from(expand_user_path(&path))).collect(),
        _ => scan_desktop_files(desktop.to_string_lossy().as_ref())?
            .into_iter()
            .map(|file| PathBuf::from(file.path))
            .collect(),
    };

    let mut moved_files = Vec::new();
    let mut skipped_files = Vec::new();

    for source in candidate_paths {
        if !source.is_file() {
            skipped_files.push(SkippedDesktopFile {
                path: source.to_string_lossy().to_string(),
                reason: "不是可整理的文件".to_string(),
            });
            continue;
        }

        let source_canonical = match source.canonicalize() {
            Ok(path) => path,
            Err(error) => {
                skipped_files.push(SkippedDesktopFile {
                    path: source.to_string_lossy().to_string(),
                    reason: error.to_string(),
                });
                continue;
            }
        };

        if !source_canonical.starts_with(&desktop_canonical) || source_canonical.starts_with(&target_root) {
            skipped_files.push(SkippedDesktopFile {
                path: source.to_string_lossy().to_string(),
                reason: "只整理桌面根目录下的文件".to_string(),
            });
            continue;
        }

        let Some(name) = source.file_name().and_then(|value| value.to_str()).map(|value| value.to_string()) else {
            skipped_files.push(SkippedDesktopFile {
                path: source.to_string_lossy().to_string(),
                reason: "文件名不可读".to_string(),
            });
            continue;
        };

        let (category, label) = classify_file(&name);
        let category_dir = target_root.join(label);
        if let Err(error) = std::fs::create_dir_all(&category_dir) {
            skipped_files.push(SkippedDesktopFile {
                path: source.to_string_lossy().to_string(),
                reason: error.to_string(),
            });
            continue;
        }

        let destination = unique_destination(&category_dir, &name);
        match std::fs::rename(&source, &destination) {
            Ok(_) => moved_files.push(MovedDesktopFile {
                name,
                category: category.to_string(),
                label: label.to_string(),
                original_path: source.to_string_lossy().to_string(),
                current_path: destination.to_string_lossy().to_string(),
            }),
            Err(error) => skipped_files.push(SkippedDesktopFile {
                path: source.to_string_lossy().to_string(),
                reason: error.to_string(),
            }),
        }
    }

    Ok(DesktopOrganizeResult {
        target_root: target_root.to_string_lossy().to_string(),
        moved_files,
        skipped_files,
    })
}

#[tauri::command]
fn undo_desktop_organize(moved_files: Vec<MovedDesktopFile>) -> DesktopUndoResult {
    let mut restored_files = Vec::new();
    let mut skipped_files = Vec::new();

    for moved_file in moved_files {
        let current_path = PathBuf::from(&moved_file.current_path);
        let original_path = PathBuf::from(&moved_file.original_path);

        if !current_path.is_file() {
            skipped_files.push(SkippedDesktopFile {
                path: moved_file.current_path.clone(),
                reason: "整理后的文件不存在".to_string(),
            });
            continue;
        }

        let Some(parent) = original_path.parent() else {
            skipped_files.push(SkippedDesktopFile {
                path: moved_file.original_path.clone(),
                reason: "原始目录不可读".to_string(),
            });
            continue;
        };

        if let Err(error) = std::fs::create_dir_all(parent) {
            skipped_files.push(SkippedDesktopFile {
                path: moved_file.original_path.clone(),
                reason: error.to_string(),
            });
            continue;
        }

        let destination = if original_path.exists() {
            unique_destination(parent, original_path.file_name().and_then(|value| value.to_str()).unwrap_or(&moved_file.name))
        } else {
            original_path
        };

        match std::fs::rename(&current_path, &destination) {
            Ok(_) => restored_files.push(MovedDesktopFile {
                current_path: destination.to_string_lossy().to_string(),
                ..moved_file
            }),
            Err(error) => skipped_files.push(SkippedDesktopFile {
                path: current_path.to_string_lossy().to_string(),
                reason: error.to_string(),
            }),
        }
    }

    DesktopUndoResult {
        restored_files,
        skipped_files,
    }
}

fn scan_desktop_files(desktop: &str) -> Result<Vec<DesktopFile>, String> {
    let mut files = Vec::new();

    for entry in std::fs::read_dir(&desktop).map_err(|error| error.to_string())? {
        let entry = entry.map_err(|error| error.to_string())?;
        let path = entry.path();

        if !path.is_file() {
            continue;
        }

        let name = entry.file_name().to_string_lossy().to_string();
        let (category, label) = classify_file(&name);
        files.push(DesktopFile {
            name,
            path: path.to_string_lossy().to_string(),
            category: category.to_string(),
            label: label.to_string(),
        });
    }

    Ok(files)
}

fn search_files_in_roots(query: &str, roots: &[PathBuf], limit: usize, max_entries: usize) -> FileSearchResult {
    let clean_query = query.trim();
    let normalized_query = clean_query.to_lowercase();
    let root_labels = roots
        .iter()
        .map(|root| root.to_string_lossy().to_string())
        .collect::<Vec<_>>();

    if normalized_query.is_empty() {
        return FileSearchResult {
            query: clean_query.to_string(),
            roots: root_labels,
            total_matches: 0,
            files: Vec::new(),
        };
    }

    let mut files = Vec::new();
    let mut total_matches = 0;
    let mut visited_entries = 0;

    for root in roots {
        search_root(
            root,
            &normalized_query,
            limit,
            max_entries,
            &mut visited_entries,
            &mut total_matches,
            &mut files,
        );

        if visited_entries >= max_entries {
            break;
        }
    }

    FileSearchResult {
        query: clean_query.to_string(),
        roots: root_labels,
        total_matches,
        files,
    }
}

fn search_root(
    root: &Path,
    normalized_query: &str,
    limit: usize,
    max_entries: usize,
    visited_entries: &mut usize,
    total_matches: &mut usize,
    files: &mut Vec<FileSearchItem>,
) {
    if !root.exists() || *visited_entries >= max_entries {
        return;
    }

    let mut stack = vec![(root.to_path_buf(), 0usize)];

    while let Some((current, depth)) = stack.pop() {
        if *visited_entries >= max_entries || depth > 5 {
            break;
        }

        let Ok(entries) = std::fs::read_dir(&current) else {
            continue;
        };

        let mut entries = entries.filter_map(Result::ok).collect::<Vec<_>>();
        entries.sort_by_key(|entry| entry.file_name().to_string_lossy().to_lowercase());

        for entry in entries {
            if *visited_entries >= max_entries {
                break;
            }

            *visited_entries += 1;
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();

            if should_skip_search_entry(&name) {
                continue;
            }

            if path.is_dir() {
                stack.push((path, depth + 1));
                continue;
            }

            if !path.is_file() || !name.to_lowercase().contains(normalized_query) {
                continue;
            }

            *total_matches += 1;
            if files.len() >= limit {
                continue;
            }

            let (category, label) = classify_file(&name);
            files.push(FileSearchItem {
                name,
                path: path.to_string_lossy().to_string(),
                category: category.to_string(),
                label: label.to_string(),
            });
        }
    }
}

fn should_skip_search_entry(name: &str) -> bool {
    matches!(
        name.to_lowercase().as_str(),
        ".git" | "node_modules" | "target" | "dist" | "lumora整理"
    )
}

fn default_search_roots() -> Vec<PathBuf> {
    match std::env::var("USERPROFILE") {
        Ok(profile) => {
            let profile = PathBuf::from(profile);
            ["Desktop", "Downloads", "Documents"]
                .iter()
                .map(|folder| profile.join(folder))
                .filter(|path| path.exists())
                .collect()
        }
        Err(_) => Vec::new(),
    }
}

#[tauri::command]
fn hide_desktop_file(app: tauri::AppHandle, path_str: String) -> Result<DockTarget, String> {
    let desktop_root = desktop_path().ok().map(PathBuf::from);
    let desktop_roots = desktop_roots();
    let pb = resolve_dropped_path(&path_str, &desktop_roots);

    let icon_cache_dir = app.path().app_cache_dir().ok().map(|path| path.join("icons"));
    let mut target = describe_target_with_icon(&pb, icon_cache_dir.as_deref());

    if let Some(hidden_root) = desktop_root.as_deref() {
        if let Some(plan) = dock_hide_plan(&pb, &desktop_roots, hidden_root) {
            let destination = move_dock_source_to_hidden(&pb, &plan)?;
            target.original_desktop_path = Some(plan.original_path.to_string_lossy().to_string());
            target.target = destination.to_string_lossy().to_string();
        }
    }

    Ok(target)
}

#[tauri::command]
fn restore_desktop_file(current_path: String, original_path: String) -> Result<(), String> {
    let current_pb = PathBuf::from(expand_user_path(&current_path));
    let original_pb = PathBuf::from(expand_user_path(&original_path));

    if !current_pb.exists() {
        return Err(format!("Dock 隐藏项目不存在: {}", current_pb.to_string_lossy()));
    }

    let parent = original_pb
        .parent()
        .ok_or_else(|| format!("原始桌面路径不可读: {}", original_pb.to_string_lossy()))?;
    std::fs::create_dir_all(parent)
        .map_err(|error| format!("无法创建原始桌面目录 {}: {error}", parent.to_string_lossy()))?;

    let destination = if original_pb.exists() {
        let file_name = original_pb
            .file_name()
            .ok_or_else(|| format!("原始桌面文件名不可读: {}", original_pb.to_string_lossy()))?;
        unique_destination(parent, &file_name.to_string_lossy())
    } else {
        original_pb
    };

    std::fs::rename(&current_pb, &destination).map_err(|error| {
        format!(
            "无法还原 Dock 项目到桌面: {} -> {}: {error}",
            current_pb.to_string_lossy(),
            destination.to_string_lossy()
        )
    })?;

    Ok(())
}

fn dock_item_statuses_for_targets(targets: Vec<String>) -> Vec<DockItemStatus> {
    #[cfg(target_os = "windows")]
    let processes = running_processes();

    targets
        .into_iter()
        .map(|target| {
            let mut is_running = false;
            let mut attention_sequence = 0;

            #[cfg(target_os = "windows")]
            {
                if let Some(identity) = launch_identity_for_target(&target) {
                    is_running = processes
                        .iter()
                        .any(|process| process_matches_launch_identity(process, &identity));
                    attention_sequence = dock_attention_sequence_for_identity(&identity).unwrap_or(0);
                }
            }

            #[cfg(not(target_os = "windows"))]
            {
                let _ = &target;
            }

            DockItemStatus {
                target,
                is_running,
                needs_attention: attention_sequence > 0,
                attention_sequence,
            }
        })
        .collect()
}

fn dock_attention_state() -> &'static Mutex<DockAttentionState> {
    DOCK_ATTENTION_STATE.get_or_init(|| Mutex::new(DockAttentionState::default()))
}

fn mark_dock_attention(identity: LaunchIdentity) -> u64 {
    let mut state = dock_attention_state()
        .lock()
        .expect("dock attention state lock poisoned");
    state.next_sequence = state.next_sequence.saturating_add(1);
    let sequence = state.next_sequence;

    if let Some(entry) = state
        .entries
        .iter_mut()
        .find(|entry| launch_identities_match(&entry.identity, &identity))
    {
        entry.identity = identity;
        entry.sequence = sequence;
        return sequence;
    }

    state.entries.push(DockAttentionEntry { identity, sequence });
    sequence
}

fn clear_dock_attention(identity: &LaunchIdentity) -> bool {
    let mut state = dock_attention_state()
        .lock()
        .expect("dock attention state lock poisoned");
    let before = state.entries.len();
    state
        .entries
        .retain(|entry| !launch_identities_match(&entry.identity, identity));
    before != state.entries.len()
}

fn clear_dock_attention_for_target(target: &str) -> bool {
    #[cfg(target_os = "windows")]
    {
        return launch_identity_for_target(target)
            .as_ref()
            .is_some_and(clear_dock_attention);
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = target;
        false
    }
}

fn dock_attention_sequence_for_identity(identity: &LaunchIdentity) -> Option<u64> {
    let state = dock_attention_state()
        .lock()
        .expect("dock attention state lock poisoned");
    state
        .entries
        .iter()
        .find(|entry| launch_identities_match(&entry.identity, identity))
        .map(|entry| entry.sequence)
}

fn launch_identities_match(a: &LaunchIdentity, b: &LaunchIdentity) -> bool {
    if let (Some(a_path), Some(b_path)) = (&a.executable_path, &b.executable_path) {
        if path_eq_ignore_ascii_case(a_path, b_path) {
            return true;
        }
    }

    a.executable_name.eq_ignore_ascii_case(&b.executable_name)
}

fn extension_is(path: &Path, expected: &str) -> bool {
    path.extension()
        .and_then(|value| value.to_str())
        .is_some_and(|value| value.eq_ignore_ascii_case(expected))
}

fn lowercase_file_name(path: &Path) -> Option<String> {
    path.file_name()
        .and_then(|value| value.to_str())
        .map(|value| value.to_lowercase())
}

fn launch_identity_from_executable_path(path: &Path) -> Option<LaunchIdentity> {
    if !extension_is(path, "exe") {
        return None;
    }

    let mut canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    let canonical_str = canonical.to_string_lossy();
    if canonical_str.starts_with(r"\\?\") {
        canonical = std::path::PathBuf::from(canonical_str.trim_start_matches(r"\\?\"));
    }

    Some(LaunchIdentity {
        executable_path: Some(canonical),
        executable_name: lowercase_file_name(path)?,
    })
}

#[cfg(target_os = "windows")]
fn launch_identity_for_target(target: &str) -> Option<LaunchIdentity> {
    let path = PathBuf::from(expand_user_path(target));

    if extension_is(&path, "lnk") {
        return resolve_lnk_target_path(&path).and_then(|target_path| launch_identity_from_executable_path(&target_path));
    }

    launch_identity_from_executable_path(&path)
}

#[cfg(target_os = "windows")]
fn resolve_lnk_target_path(path: &Path) -> Option<PathBuf> {
    resolve_lnk_target_path_with_shell(path).or_else(|| resolve_lnk_target_path_from_link_info(path))
}

#[cfg(target_os = "windows")]
fn resolve_lnk_target_path_with_shell(path: &Path) -> Option<PathBuf> {
    use std::os::windows::ffi::OsStrExt;
    use windows::core::{Interface, PCWSTR};
    use windows::Win32::System::Com::{CoCreateInstance, CoInitialize, IPersistFile, CLSCTX_INPROC_SERVER, STGM};
    use windows::Win32::UI::Shell::{IShellLinkW, ShellLink};

    unsafe {
        let _ = CoInitialize(Some(std::ptr::null_mut()));
        let link: IShellLinkW = CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER).ok()?;
        let persist: IPersistFile = link.cast().ok()?;

        let path_u16: Vec<u16> = path.as_os_str().encode_wide().chain(std::iter::once(0)).collect();
        persist.Load(PCWSTR(path_u16.as_ptr()), STGM(0)).ok()?;

        let mut target = [0u16; 32768];
        let _ = link.GetPath(&mut target, std::ptr::null_mut(), 0);
        let target = string_from_wide_null_terminated(&target)?;

        if target.trim().is_empty() {
            return None;
        }

        Some(PathBuf::from(target))
    }
}

#[cfg(target_os = "windows")]
fn resolve_lnk_target_path_from_link_info(path: &Path) -> Option<PathBuf> {
    let lnk = parselnk::Lnk::try_from(path).ok()?;
    let link_info = lnk.link_info;
    let candidates = [
        link_info.local_base_path_unicode,
        link_info.local_base_path,
        link_info.common_path_suffix_unicode,
        link_info.common_path_suffix,
    ];

    candidates
        .into_iter()
        .flatten()
        .map(|value| value.trim_matches(char::from(0)).trim().to_string())
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .find(|candidate| candidate.extension().is_some())
}

#[cfg(target_os = "windows")]
fn string_from_wide_null_terminated(value: &[u16]) -> Option<String> {
    let len = value.iter().position(|&c| c == 0).unwrap_or(value.len());
    if len == 0 {
        return None;
    }

    Some(String::from_utf16_lossy(&value[..len]))
}

#[cfg(target_os = "windows")]
fn running_processes() -> Vec<RunningProcess> {
    use windows_sys::Win32::Foundation::{CloseHandle, INVALID_HANDLE_VALUE};
    use windows_sys::Win32::System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS,
    };

    let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) };
    if snapshot == INVALID_HANDLE_VALUE {
        return Vec::new();
    }

    let mut processes = Vec::new();
    let mut entry: PROCESSENTRY32W = unsafe { std::mem::zeroed() };
    entry.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;

    let mut has_entry = unsafe { Process32FirstW(snapshot, &mut entry) } != 0;
    while has_entry {
        if let Some(name) = string_from_wide_null_terminated(&entry.szExeFile) {
            processes.push(RunningProcess {
                executable_path: process_path_for_pid(entry.th32ProcessID),
                executable_name: name.to_lowercase(),
            });
        }
        has_entry = unsafe { Process32NextW(snapshot, &mut entry) } != 0;
    }

    unsafe {
        CloseHandle(snapshot);
    }

    processes
}

#[cfg(target_os = "windows")]
fn process_path_for_pid(pid: u32) -> Option<PathBuf> {
    use windows_sys::Win32::Foundation::CloseHandle;
    use windows_sys::Win32::System::Threading::{OpenProcess, QueryFullProcessImageNameW, PROCESS_QUERY_LIMITED_INFORMATION};

    let handle = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid) };
    if handle.is_null() {
        return None;
    }

    let mut buffer = vec![0u16; 32768];
    let mut size = buffer.len() as u32;
    let ok = unsafe { QueryFullProcessImageNameW(handle, 0, buffer.as_mut_ptr(), &mut size) };

    unsafe {
        CloseHandle(handle);
    }

    if ok == 0 || size == 0 {
        return None;
    }

    Some(PathBuf::from(String::from_utf16_lossy(&buffer[..size as usize])))
}

#[cfg(target_os = "windows")]
fn process_matches_launch_identity(process: &RunningProcess, identity: &LaunchIdentity) -> bool {
    if let (Some(process_path), Some(identity_path)) = (&process.executable_path, &identity.executable_path) {
        if path_eq_ignore_ascii_case(process_path, identity_path) {
            return true;
        }
    }

    process.executable_name.eq_ignore_ascii_case(&identity.executable_name)
}

fn should_suppress_open_after_activation_miss(identity: &LaunchIdentity, processes: &[RunningProcess]) -> bool {
    is_tray_managed_launch_identity(identity)
        && processes
            .iter()
            .any(|process| process_matches_launch_identity(process, identity))
}

fn is_tray_managed_launch_identity(identity: &LaunchIdentity) -> bool {
    matches!(identity.executable_name.as_str(), "weixin.exe" | "wechat.exe")
}

fn tray_button_text_matches_launch_identity(text: &str, identity: &LaunchIdentity) -> bool {
    let text = text.trim();
    if text.is_empty() {
        return false;
    }

    match identity.executable_name.as_str() {
        "weixin.exe" | "wechat.exe" => text.contains("微信") && !text.contains("企业微信"),
        _ => false,
    }
}

#[cfg(target_os = "windows")]
#[derive(Clone)]
struct TrayToolbarButton {
    toolbar: windows_sys::Win32::Foundation::HWND,
    rect: windows_sys::Win32::Foundation::RECT,
    text: String,
}

#[cfg(target_os = "windows")]
fn try_activate_tray_icon_for_identity(identity: &LaunchIdentity) -> bool {
    let Some(button) = find_tray_icon_button_for_identity(identity) else {
        return false;
    };

    click_tray_toolbar_button(&button)
}

#[cfg(target_os = "windows")]
fn find_tray_icon_button_for_identity(identity: &LaunchIdentity) -> Option<TrayToolbarButton> {
    tray_toolbar_buttons()
        .into_iter()
        .find(|button| tray_button_text_matches_launch_identity(&button.text, identity))
}

#[cfg(target_os = "windows")]
fn tray_toolbar_buttons() -> Vec<TrayToolbarButton> {
    use windows_sys::Win32::Foundation::{HWND, LPARAM};
    use windows_sys::Win32::UI::WindowsAndMessaging::EnumWindows;

    unsafe extern "system" fn enum_window(hwnd: HWND, lparam: LPARAM) -> i32 {
        let windows = &mut *(lparam as *mut Vec<HWND>);
        let class_name = window_class_name(hwnd);
        if class_name == "Shell_TrayWnd" || class_name == "NotifyIconOverflowWindow" {
            windows.push(hwnd);
        }
        1
    }

    let mut windows = Vec::new();
    unsafe {
        EnumWindows(Some(enum_window), &mut windows as *mut Vec<HWND> as LPARAM);
    }

    windows
        .into_iter()
        .flat_map(tray_toolbar_buttons_for_window)
        .collect()
}

#[cfg(target_os = "windows")]
fn tray_toolbar_buttons_for_window(window: windows_sys::Win32::Foundation::HWND) -> Vec<TrayToolbarButton> {
    use windows_sys::Win32::Foundation::{HWND, LPARAM};
    use windows_sys::Win32::UI::WindowsAndMessaging::EnumChildWindows;

    unsafe extern "system" fn enum_child(hwnd: HWND, lparam: LPARAM) -> i32 {
        let toolbars = &mut *(lparam as *mut Vec<HWND>);
        if window_class_name(hwnd) == "ToolbarWindow32" {
            toolbars.push(hwnd);
        }
        1
    }

    let mut toolbars = Vec::new();
    unsafe {
        EnumChildWindows(window, Some(enum_child), &mut toolbars as *mut Vec<HWND> as LPARAM);
    }

    toolbars
        .into_iter()
        .flat_map(read_tray_toolbar_buttons)
        .collect()
}

#[cfg(target_os = "windows")]
fn read_tray_toolbar_buttons(toolbar: windows_sys::Win32::Foundation::HWND) -> Vec<TrayToolbarButton> {
    use windows_sys::Win32::Foundation::RECT;
    use windows_sys::Win32::System::Diagnostics::Debug::{ReadProcessMemory, WriteProcessMemory};
    use windows_sys::Win32::System::Memory::{VirtualAllocEx, VirtualFreeEx, MEM_COMMIT, MEM_RELEASE, MEM_RESERVE, PAGE_READWRITE};
    use windows_sys::Win32::System::Threading::{
        OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_VM_OPERATION, PROCESS_VM_READ, PROCESS_VM_WRITE,
    };
    use windows_sys::Win32::UI::WindowsAndMessaging::{GetWindowThreadProcessId, SendMessageW};

    const TB_BUTTONCOUNT: u32 = 0x0418;
    const TB_GETBUTTON: u32 = 0x0417;
    const TB_GETBUTTONTEXTW: u32 = 0x044B;
    const TB_GETITEMRECT: u32 = 0x041D;
    const REMOTE_BYTES: usize = 4096;
    const BUTTON_OFFSET: usize = 0;
    const TEXT_OFFSET: usize = 512;
    const RECT_OFFSET: usize = 2048;
    const TEXT_BYTES: usize = 1024;

    let count = unsafe { SendMessageW(toolbar, TB_BUTTONCOUNT, 0, 0) as i32 };
    if count <= 0 {
        return Vec::new();
    }

    let mut pid = 0u32;
    unsafe {
        GetWindowThreadProcessId(toolbar, &mut pid);
    }
    if pid == 0 {
        return Vec::new();
    }

    let process = unsafe {
        OpenProcess(
            PROCESS_VM_OPERATION
                | PROCESS_VM_READ
                | PROCESS_VM_WRITE
                | PROCESS_QUERY_INFORMATION
                | PROCESS_QUERY_LIMITED_INFORMATION,
            0,
            pid,
        )
    };
    if process.is_null() {
        return Vec::new();
    }

    let remote = unsafe {
        VirtualAllocEx(
            process,
            std::ptr::null(),
            REMOTE_BYTES,
            MEM_COMMIT | MEM_RESERVE,
            PAGE_READWRITE,
        )
    };
    if remote.is_null() {
        unsafe {
            windows_sys::Win32::Foundation::CloseHandle(process);
        }
        return Vec::new();
    }

    let remote_button = unsafe { (remote as *mut u8).add(BUTTON_OFFSET) } as *mut _;
    let remote_text = unsafe { (remote as *mut u8).add(TEXT_OFFSET) } as *mut _;
    let remote_rect = unsafe { (remote as *mut u8).add(RECT_OFFSET) } as *mut _;
    let mut buttons = Vec::new();

    for index in 0..count {
        unsafe {
            SendMessageW(toolbar, TB_GETBUTTON, index as usize, remote_button as isize);
        }

        let mut button_bytes = [0u8; 64];
        let mut bytes_read = 0usize;
        let read_button = unsafe {
            ReadProcessMemory(
                process,
                remote_button,
                button_bytes.as_mut_ptr() as *mut _,
                button_bytes.len(),
                &mut bytes_read,
            )
        } != 0;
        if !read_button || bytes_read < 8 {
            continue;
        }

        let id_command = i32::from_le_bytes([
            button_bytes[4],
            button_bytes[5],
            button_bytes[6],
            button_bytes[7],
        ]);

        let zero_text = [0u8; TEXT_BYTES];
        let mut bytes_written = 0usize;
        unsafe {
            WriteProcessMemory(
                process,
                remote_text,
                zero_text.as_ptr() as *const _,
                zero_text.len(),
                &mut bytes_written,
            );
            SendMessageW(toolbar, TB_GETBUTTONTEXTW, id_command as usize, remote_text as isize);
        }

        let mut text_bytes = [0u8; TEXT_BYTES];
        let read_text = unsafe {
            ReadProcessMemory(
                process,
                remote_text,
                text_bytes.as_mut_ptr() as *mut _,
                text_bytes.len(),
                &mut bytes_read,
            )
        } != 0;
        if !read_text {
            continue;
        }
        let text = string_from_remote_wide_bytes(&text_bytes);

        let zero_rect = [0u8; std::mem::size_of::<RECT>()];
        unsafe {
            WriteProcessMemory(
                process,
                remote_rect,
                zero_rect.as_ptr() as *const _,
                zero_rect.len(),
                &mut bytes_written,
            );
            SendMessageW(toolbar, TB_GETITEMRECT, index as usize, remote_rect as isize);
        }

        let mut rect = RECT {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        };
        let read_rect = unsafe {
            ReadProcessMemory(
                process,
                remote_rect,
                &mut rect as *mut RECT as *mut _,
                std::mem::size_of::<RECT>(),
                &mut bytes_read,
            )
        } != 0;
        if !read_rect || text.trim().is_empty() {
            continue;
        }

        buttons.push(TrayToolbarButton {
            toolbar,
            rect,
            text,
        });
    }

    unsafe {
        VirtualFreeEx(process, remote, 0, MEM_RELEASE);
        windows_sys::Win32::Foundation::CloseHandle(process);
    }

    buttons
}

#[cfg(target_os = "windows")]
fn string_from_remote_wide_bytes(bytes: &[u8]) -> String {
    let mut values = Vec::new();
    for chunk in bytes.chunks_exact(2) {
        let value = u16::from_le_bytes([chunk[0], chunk[1]]);
        if value == 0 {
            break;
        }
        values.push(value);
    }

    String::from_utf16_lossy(&values)
}

#[cfg(target_os = "windows")]
fn click_tray_toolbar_button(button: &TrayToolbarButton) -> bool {
    use windows_sys::Win32::UI::WindowsAndMessaging::{SendMessageW, WM_LBUTTONDOWN, WM_LBUTTONUP};

    const MK_LBUTTON: usize = 0x0001;

    let x = (button.rect.left + button.rect.right) / 2;
    let y = (button.rect.top + button.rect.bottom) / 2;
    if x <= 0 && y <= 0 {
        return false;
    }

    let lparam = ((y & 0xffff) << 16) | (x & 0xffff);
    unsafe {
        SendMessageW(button.toolbar, WM_LBUTTONDOWN, MK_LBUTTON, lparam as isize);
        SendMessageW(button.toolbar, WM_LBUTTONUP, 0, lparam as isize);
    }

    true
}

fn path_eq_ignore_ascii_case(a: &Path, b: &Path) -> bool {
    a.to_string_lossy().eq_ignore_ascii_case(&b.to_string_lossy())
}

#[cfg(target_os = "windows")]
fn find_window_for_launch_identity(identity: &LaunchIdentity) -> Option<windows_sys::Win32::Foundation::HWND> {
    use windows_sys::Win32::Foundation::HWND;
    use windows_sys::Win32::UI::WindowsAndMessaging::EnumWindows;

    struct WindowSearch {
        identity: LaunchIdentity,
        hwnd: HWND,
        rank: Option<u8>,
        area: i64,
    }

    unsafe extern "system" fn enum_window(hwnd: HWND, lparam: isize) -> i32 {
        use windows_sys::Win32::UI::WindowsAndMessaging::GetWindowThreadProcessId;

        let class_name = window_class_name(hwnd);
        let title = window_title(hwnd);
        let info = window_activation_info(hwnd, &class_name, &title);
        let Some(rank) = window_activation_rank_for_info(info) else {
            return 1;
        };

        let mut pid = 0u32;
        GetWindowThreadProcessId(hwnd, &mut pid);
        if pid == 0 {
            return 1;
        }

        let search = &mut *(lparam as *mut WindowSearch);
        let process_path = process_path_for_pid(pid);
        let process = RunningProcess {
            executable_name: process_path.as_deref().and_then(lowercase_file_name).unwrap_or_default(),
            executable_path: process_path,
        };

        if process_matches_launch_identity(&process, &search.identity) {
            let area = window_activation_area(info);
            if is_better_window_candidate(rank, area, search.rank, search.area) {
                search.hwnd = hwnd;
                search.rank = Some(rank);
                search.area = area;
            }
        }

        1
    }

    let mut search = WindowSearch {
        identity: identity.clone(),
        hwnd: std::ptr::null_mut(),
        rank: None,
        area: 0,
    };

    unsafe {
        EnumWindows(Some(enum_window), &mut search as *mut WindowSearch as isize);
    }

    if search.hwnd.is_null() {
        None
    } else {
        Some(search.hwnd)
    }
}

#[cfg(all(target_os = "windows", test))]
fn window_activation_rank(is_visible: bool, has_owner: bool, class_name: &str, title: &str) -> Option<u8> {
    window_activation_rank_for_info(WindowActivationInfo {
        is_visible,
        is_iconic: false,
        has_owner,
        class_name,
        title,
        ex_style: 0,
        cloaked: false,
        left: 0,
        top: 0,
        width: 500,
        height: 500,
    })
}

#[cfg(target_os = "windows")]
#[derive(Clone, Copy)]
struct WindowActivationInfo<'a> {
    is_visible: bool,
    is_iconic: bool,
    has_owner: bool,
    class_name: &'a str,
    title: &'a str,
    ex_style: u32,
    cloaked: bool,
    left: i32,
    top: i32,
    width: i32,
    height: i32,
}

#[cfg(target_os = "windows")]
fn window_activation_rank_for_info(info: WindowActivationInfo<'_>) -> Option<u8> {
    if info.has_owner || info.cloaked || activation_ex_style_is_tool_window(info.ex_style) || activation_ex_style_is_transparent(info.ex_style) {
        return None;
    }

    if is_ignored_activation_window(info.class_name, info.title) {
        return None;
    }

    if info.is_visible {
        if !info.is_iconic && is_offscreen_icon_placeholder(info) {
            return None;
        }
        return Some(if info.is_iconic { 1 } else { 0 });
    }

    None
}

#[cfg(target_os = "windows")]
fn window_activation_area(info: WindowActivationInfo<'_>) -> i64 {
    i64::from(info.width.max(0)) * i64::from(info.height.max(0))
}

#[cfg(target_os = "windows")]
fn is_better_window_candidate(rank: u8, area: i64, current_rank: Option<u8>, current_area: i64) -> bool {
    match current_rank {
        None => true,
        Some(existing_rank) if rank < existing_rank => true,
        Some(existing_rank) if rank == existing_rank => area > current_area,
        _ => false,
    }
}

#[cfg(target_os = "windows")]
fn activation_ex_style_is_tool_window(ex_style: u32) -> bool {
    (ex_style & windows_sys::Win32::UI::WindowsAndMessaging::WS_EX_TOOLWINDOW) != 0
}

#[cfg(target_os = "windows")]
fn activation_ex_style_is_transparent(ex_style: u32) -> bool {
    (ex_style & windows_sys::Win32::UI::WindowsAndMessaging::WS_EX_TRANSPARENT) != 0
}

#[cfg(target_os = "windows")]
fn is_ignored_activation_window(class_name: &str, title: &str) -> bool {
    let class_name = class_name.to_ascii_lowercase();
    let title = title.to_ascii_lowercase();

    title.trim().is_empty()
        || title.contains("图片和视频")
        || title.contains("photos and videos")
        || class_name.contains("trayiconmessage")
        || class_name.contains("powermessagewindow")
        || class_name.contains("systemmessagewindow")
        || class_name.contains("ime")
        || class_name.contains("sogou")
        || class_name.contains("sopy_")
        || class_name.contains("qwindowtoolsavebits")
}

#[cfg(target_os = "windows")]
fn is_offscreen_icon_placeholder(info: WindowActivationInfo<'_>) -> bool {
    if !info.is_visible {
        return false;
    }

    info.left < -10_000 || info.top < -10_000 || info.width < 10 || info.height < 10
}

#[cfg(target_os = "windows")]
unsafe fn window_activation_info<'a>(
    hwnd: windows_sys::Win32::Foundation::HWND,
    class_name: &'a str,
    title: &'a str,
) -> WindowActivationInfo<'a> {
    use windows_sys::Win32::Foundation::RECT;
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        GetWindow, GetWindowLongW, GetWindowRect, IsIconic, IsWindowVisible, GWL_EXSTYLE, GW_OWNER,
    };

    let mut rect = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    let has_rect = GetWindowRect(hwnd, &mut rect) != 0;

    WindowActivationInfo {
        is_visible: IsWindowVisible(hwnd) != 0,
        is_iconic: IsIconic(hwnd) != 0,
        has_owner: !GetWindow(hwnd, GW_OWNER).is_null(),
        class_name,
        title,
        ex_style: GetWindowLongW(hwnd, GWL_EXSTYLE) as u32,
        cloaked: window_is_cloaked(hwnd),
        left: if has_rect { rect.left } else { 0 },
        top: if has_rect { rect.top } else { 0 },
        width: if has_rect { rect.right - rect.left } else { 0 },
        height: if has_rect { rect.bottom - rect.top } else { 0 },
    }
}

#[cfg(target_os = "windows")]
fn window_is_cloaked(hwnd: windows_sys::Win32::Foundation::HWND) -> bool {
    use windows_sys::Win32::Graphics::Dwm::{DwmGetWindowAttribute, DWMWA_CLOAKED};

    let mut cloaked = 0i32;
    let ok = unsafe {
        DwmGetWindowAttribute(
            hwnd,
            DWMWA_CLOAKED as u32,
            &mut cloaked as *mut i32 as *mut _,
            std::mem::size_of::<i32>() as u32,
        )
    } == 0;

    ok && cloaked != 0
}

#[cfg(target_os = "windows")]
fn dock_activation_state() -> &'static Mutex<Vec<DockActivationEntry>> {
    DOCK_ACTIVATION_STATE.get_or_init(|| Mutex::new(Vec::new()))
}

#[cfg(target_os = "windows")]
fn remember_window_for_launch_identity(identity: LaunchIdentity, hwnd: windows_sys::Win32::Foundation::HWND) {
    if hwnd.is_null() || !window_is_rememberable_activation_candidate(hwnd, &identity) {
        return;
    }

    let mut state = dock_activation_state()
        .lock()
        .expect("dock activation state lock poisoned");
    state.retain(|entry| !launch_identities_match(&entry.identity, &identity));
    state.push(DockActivationEntry {
        identity,
        hwnd: hwnd as isize,
    });
    if state.len() > 64 {
        let overflow = state.len() - 64;
        state.drain(0..overflow);
    }
}

#[cfg(target_os = "windows")]
fn remembered_window_for_launch_identity(identity: &LaunchIdentity) -> Option<windows_sys::Win32::Foundation::HWND> {
    let remembered = {
        let state = dock_activation_state()
            .lock()
            .expect("dock activation state lock poisoned");
        state
            .iter()
            .rev()
            .find(|entry| launch_identities_match(&entry.identity, identity))
            .map(|entry| entry.hwnd)
    }?;

    let hwnd = remembered as windows_sys::Win32::Foundation::HWND;
    if window_is_rememberable_activation_candidate(hwnd, identity) {
        Some(hwnd)
    } else {
        None
    }
}

#[cfg(target_os = "windows")]
fn window_is_rememberable_activation_candidate(
    hwnd: windows_sys::Win32::Foundation::HWND,
    identity: &LaunchIdentity,
) -> bool {
    use windows_sys::Win32::UI::WindowsAndMessaging::IsWindow;

    if hwnd.is_null() || unsafe { IsWindow(hwnd) } == 0 {
        return false;
    }

    let Some(window_identity) = launch_identity_for_window(hwnd) else {
        return false;
    };
    if !launch_identities_match(&window_identity, identity) {
        return false;
    }

    unsafe {
        let class_name = window_class_name(hwnd);
        let title = window_title(hwnd);
        let info = window_activation_info(hwnd, &class_name, &title);

        window_activation_rank_for_info(info).is_some()
    }
}

#[cfg(target_os = "windows")]
unsafe fn window_title(hwnd: windows_sys::Win32::Foundation::HWND) -> String {
    use windows_sys::Win32::UI::WindowsAndMessaging::GetWindowTextW;

    let mut buffer = vec![0u16; 512];
    let len = GetWindowTextW(hwnd, buffer.as_mut_ptr(), buffer.len() as i32);
    if len <= 0 {
        return String::new();
    }

    String::from_utf16_lossy(&buffer[..len as usize])
}

#[cfg(target_os = "windows")]
unsafe fn window_class_name(hwnd: windows_sys::Win32::Foundation::HWND) -> String {
    use windows_sys::Win32::UI::WindowsAndMessaging::GetClassNameW;

    let mut buffer = vec![0u16; 256];
    let len = GetClassNameW(hwnd, buffer.as_mut_ptr(), buffer.len() as i32);
    if len <= 0 {
        return String::new();
    }

    String::from_utf16_lossy(&buffer[..len as usize])
}

#[cfg(target_os = "windows")]
fn activate_window(hwnd: windows_sys::Win32::Foundation::HWND) {
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        IsIconic, IsWindowVisible, SetForegroundWindow, ShowWindowAsync, GetForegroundWindow, SW_MINIMIZE,
    };

    unsafe {
        if GetForegroundWindow() == hwnd {
            ShowWindowAsync(hwnd, SW_MINIMIZE);
            return;
        }

        let show_command = activation_show_command(IsIconic(hwnd) != 0, IsWindowVisible(hwnd) != 0);
        ShowWindowAsync(hwnd, show_command);
        SetForegroundWindow(hwnd);
    }
}

#[cfg(target_os = "windows")]
fn activation_show_command(is_iconic: bool, is_visible: bool) -> i32 {
    use windows_sys::Win32::UI::WindowsAndMessaging::{SW_RESTORE, SW_SHOW};

    if is_iconic || !is_visible {
        SW_RESTORE
    } else {
        SW_SHOW
    }
}

#[cfg(target_os = "windows")]
const HSHELL_FLASH_EVENT: u32 = windows_sys::Win32::UI::WindowsAndMessaging::HSHELL_REDRAW
    | windows_sys::Win32::UI::WindowsAndMessaging::HSHELL_HIGHBIT;

#[cfg(target_os = "windows")]
const HSHELL_RUDEAPPACTIVATED_EVENT: u32 = windows_sys::Win32::UI::WindowsAndMessaging::HSHELL_WINDOWACTIVATED
    | windows_sys::Win32::UI::WindowsAndMessaging::HSHELL_HIGHBIT;

#[cfg(target_os = "windows")]
fn register_dock_shell_hook(app: &tauri::AppHandle) {
    use windows_sys::Win32::UI::{
        Shell::SetWindowSubclass,
        WindowsAndMessaging::RegisterShellHookWindow,
    };

    let _ = LUMORA_APP_HANDLE.set(app.clone());

    let Some(window) = app.get_webview_window(dock_window_label()) else {
        return;
    };

    let Ok(hwnd) = window.hwnd() else {
        return;
    };
    let hwnd = hwnd.0 as windows_sys::Win32::Foundation::HWND;

    unsafe {
        let _ = SetWindowSubclass(hwnd, Some(dock_shell_hook_proc), 1, 0);
        let registered = RegisterShellHookWindow(hwnd) != 0;
        if !registered {
            eprintln!("Lumora failed to register shell hook window");
        }
    }
}

#[cfg(not(target_os = "windows"))]
fn register_dock_shell_hook<R: tauri::Runtime>(_app: &tauri::AppHandle<R>) {}

#[cfg(target_os = "windows")]
unsafe extern "system" fn dock_shell_hook_proc(
    hwnd: windows_sys::Win32::Foundation::HWND,
    msg: u32,
    wparam: windows_sys::Win32::Foundation::WPARAM,
    lparam: windows_sys::Win32::Foundation::LPARAM,
    _uid_subclass: usize,
    _ref_data: usize,
) -> windows_sys::Win32::Foundation::LRESULT {
    use windows_sys::Win32::UI::Shell::DefSubclassProc;

    if msg == shell_hook_message() {
        handle_shell_hook(wparam as u32, lparam as windows_sys::Win32::Foundation::HWND);
    }

    DefSubclassProc(hwnd, msg, wparam, lparam)
}

#[cfg(target_os = "windows")]
fn shell_hook_message() -> u32 {
    *SHELL_HOOK_MESSAGE.get_or_init(|| {
        use std::os::windows::ffi::OsStrExt;
        use windows_sys::Win32::UI::WindowsAndMessaging::RegisterWindowMessageW;

        let message_name: Vec<u16> = std::ffi::OsStr::new("SHELLHOOK")
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();
        unsafe { RegisterWindowMessageW(message_name.as_ptr()) }
    })
}

#[cfg(target_os = "windows")]
fn handle_shell_hook(code: u32, hwnd: windows_sys::Win32::Foundation::HWND) {
    match code {
        HSHELL_FLASH_EVENT => {
            if let Some(identity) = launch_identity_for_window(hwnd) {
                mark_dock_attention(identity);
                emit_dock_attention_changed();
            }
        }
        windows_sys::Win32::UI::WindowsAndMessaging::HSHELL_WINDOWACTIVATED
        | HSHELL_RUDEAPPACTIVATED_EVENT => {
            if let Some(identity) = launch_identity_for_window(hwnd) {
                remember_window_for_launch_identity(identity.clone(), hwnd);
                if clear_dock_attention(&identity) {
                    emit_dock_attention_changed();
                }
            }
        }
        _ => {}
    }
}

#[cfg(target_os = "windows")]
fn launch_identity_for_window(hwnd: windows_sys::Win32::Foundation::HWND) -> Option<LaunchIdentity> {
    use windows_sys::Win32::UI::WindowsAndMessaging::GetWindowThreadProcessId;

    if hwnd.is_null() {
        return None;
    }

    let mut pid = 0u32;
    unsafe {
        GetWindowThreadProcessId(hwnd, &mut pid);
    }

    if pid == 0 {
        return None;
    }

    let executable_path = process_path_for_pid(pid)?;
    Some(LaunchIdentity {
        executable_name: lowercase_file_name(&executable_path)?,
        executable_path: Some(executable_path),
    })
}

#[cfg(target_os = "windows")]
fn emit_dock_attention_changed() {
    if let Some(app) = LUMORA_APP_HANDLE.get() {
        let _ = app.emit("lumora://dock-attention-changed", ());
    }
}

fn build_counts(files: &[DesktopFile]) -> Vec<DesktopCategoryCount> {
    let categories = [
        ("Images", "图片"),
        ("Docs", "文档"),
        ("Archives", "压缩包"),
        ("Installers", "安装包"),
        ("Videos", "视频"),
        ("Projects", "项目"),
        ("Inbox", "收纳箱"),
    ];

    categories
        .iter()
        .map(|(category, label)| DesktopCategoryCount {
            category: (*category).to_string(),
            label: (*label).to_string(),
            count: files.iter().filter(|file| file.category == *category).count(),
        })
        .collect()
}

fn desktop_path() -> Result<String, String> {
    let user_profile = std::env::var("USERPROFILE").map_err(|_| "USERPROFILE is not set".to_string())?;
    Ok(format!("{user_profile}\\Desktop"))
}

fn common_desktop_path() -> Option<PathBuf> {
    std::env::var("PUBLIC")
        .ok()
        .map(PathBuf::from)
        .map(|path| path.join("Desktop"))
        .filter(|path| path.exists())
}

fn desktop_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();

    if let Ok(desktop) = desktop_path() {
        roots.push(PathBuf::from(desktop));
    }

    if let Some(common_desktop) = common_desktop_path() {
        roots.push(common_desktop);
    }

    roots
}

fn resolve_dropped_path(target: &str, desktop_roots: &[PathBuf]) -> PathBuf {
    let path = PathBuf::from(expand_user_path(target));
    if !path.is_relative() {
        return path;
    }

    for desktop in desktop_roots {
        let candidate = desktop.join(&path);
        if candidate.exists() {
            return candidate;
        }
    }

    path
}

fn dock_hide_plan(source: &Path, desktop_roots: &[PathBuf], hidden_root: &Path) -> Option<DockHidePlan> {
    let original_path = source.to_path_buf();
    let source = source.canonicalize().ok()?;
    let hidden_dir = hidden_root.join(".lumora_dock_hidden");
    let canonical_hidden_dir = hidden_dir.canonicalize().unwrap_or_else(|_| hidden_dir.clone());

    for desktop in desktop_roots {
        let canonical_desktop = desktop.canonicalize().ok()?;
        let target_root_dir = desktop.join("Lumora整理");
        let canonical_target_root_dir = target_root_dir.canonicalize().unwrap_or(target_root_dir);

        if source.starts_with(&canonical_desktop)
            && !source.starts_with(&canonical_target_root_dir)
            && !source.starts_with(&canonical_hidden_dir)
        {
            return Some(DockHidePlan {
                original_path,
                hidden_dir,
            });
        }
    }

    None
}

fn move_dock_source_to_hidden(source: &Path, plan: &DockHidePlan) -> Result<PathBuf, String> {
    std::fs::create_dir_all(&plan.hidden_dir)
        .map_err(|error| format!("无法创建 Dock 隐藏目录 {}: {error}", plan.hidden_dir.to_string_lossy()))?;

    let file_name = source
        .file_name()
        .ok_or_else(|| format!("桌面项目文件名不可读: {}", source.to_string_lossy()))?;
    let destination = unique_destination(&plan.hidden_dir, &file_name.to_string_lossy());

    std::fs::rename(source, &destination).map_err(|error| {
        format!(
            "无法把桌面项目移动到 Dock 隐藏目录: {} -> {}: {error}",
            source.to_string_lossy(),
            destination.to_string_lossy()
        )
    })?;

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::ffi::OsStrExt;
        let wide_path: Vec<u16> = plan.hidden_dir.as_os_str().encode_wide().chain(std::iter::once(0)).collect();
        unsafe {
            windows_sys::Win32::Storage::FileSystem::SetFileAttributesW(
                wide_path.as_ptr(),
                windows_sys::Win32::Storage::FileSystem::FILE_ATTRIBUTE_HIDDEN | windows_sys::Win32::Storage::FileSystem::FILE_ATTRIBUTE_SYSTEM
            );
        }
    }

    Ok(destination)
}

fn expand_user_path(target: &str) -> String {
    match std::env::var("USERPROFILE") {
        Ok(profile) => target.replace("%USERPROFILE%", &profile),
        Err(_) => target.to_string(),
    }
}

fn classify_file(name: &str) -> (&'static str, &'static str) {
    let extension = std::path::Path::new(name)
        .extension()
        .and_then(|value| value.to_str())
        .map(|value| value.to_lowercase());

    match extension.as_deref() {
        Some("png" | "jpg" | "jpeg" | "webp" | "gif" | "svg") => ("Images", "图片"),
        Some("pdf" | "doc" | "docx" | "xls" | "xlsx" | "ppt" | "pptx" | "txt" | "md") => {
            ("Docs", "文档")
        }
        Some("zip" | "rar" | "7z" | "tar" | "gz") => ("Archives", "压缩包"),
        Some("exe" | "msi") => ("Installers", "安装包"),
        Some("mp4" | "mov" | "mkv" | "avi") => ("Videos", "视频"),
        Some("sln" | "csproj" | "package" | "json" | "ts" | "tsx" | "rs") => ("Projects", "项目"),
        _ => ("Inbox", "收纳箱"),
    }
}

#[tauri::command]
fn update_dock_bounds(app: tauri::AppHandle, width: f64, height: f64) {
    if let Some(win) = app.get_webview_window("dock") {
        if let Ok(Some(monitor)) = win.current_monitor() {
            let scale = monitor.scale_factor();
            let win_width = width.round();
            let win_height = height.round();
            
            let _ = win.set_size(tauri::LogicalSize::new(win_width, win_height));
            
            let screen_w = (monitor.size().width as f64) / scale;
            let screen_h = (monitor.size().height as f64) / scale;
            
            let x = ((screen_w - win_width) / 2.0).round();
            let y = (screen_h - win_height - 10.0).round();
            
            let _ = win.set_position(tauri::LogicalPosition::new(x, y));
        }
    }
}

#[cfg(test)]
fn describe_target(path: &Path) -> DockTarget {
    describe_target_with_icon(path, None)
}

fn describe_target_with_icon(path: &Path, icon_cache_dir: Option<&Path>) -> DockTarget {
    let item_type = if path.is_dir() {
        "folder"
    } else {
        match path.extension().and_then(|value| value.to_str()).map(|value| value.to_lowercase()).as_deref() {
            Some("exe" | "lnk" | "bat" | "cmd" | "ps1") => "app",
            _ => "file",
        }
    };

    let label = match item_type {
        "folder" => path
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("文件夹")
            .to_string(),
        _ => path
            .file_stem()
            .and_then(|value| value.to_str())
            .or_else(|| path.file_name().and_then(|value| value.to_str()))
            .unwrap_or("文件")
            .to_string(),
    };

    DockTarget {
        label,
        item_type: item_type.to_string(),
        target: path.to_string_lossy().to_string(),
        icon_path: icon_cache_dir.and_then(|cache_dir| extract_icon_png(path, cache_dir)),
        original_desktop_path: None,
    }
}

fn resolve_lnk_hicon(path: &std::path::Path) -> Option<windows_sys::Win32::UI::WindowsAndMessaging::HICON> {
    use std::os::windows::ffi::OsStrExt;
    use windows::core::{Interface, PCWSTR};
    use windows::Win32::System::Com::{CoInitialize, CoCreateInstance, IPersistFile, CLSCTX_INPROC_SERVER, STGM};
    use windows::Win32::UI::Shell::{IShellLinkW, ShellLink};

    unsafe {
        let _ = CoInitialize(Some(std::ptr::null_mut()));
        let link: IShellLinkW = CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER).ok()?;
        let persist: IPersistFile = link.cast().ok()?;
        
        let path_u16: Vec<u16> = path.as_os_str().encode_wide().chain(std::iter::once(0)).collect();
        persist.Load(PCWSTR(path_u16.as_ptr()), STGM(0)).ok()?;
        
        let mut icon_path = [0u16; 1024];
        let mut icon_index = 0i32;
        let _ = link.GetIconLocation(&mut icon_path, &mut icon_index);
        
        let mut final_path = icon_path;
        let mut len = final_path.iter().position(|&c| c == 0).unwrap_or(final_path.len());
        
        if len == 0 {
            // Fall back to target path
            let mut target = [0u16; 1024];
            let _ = link.GetPath(&mut target, std::ptr::null_mut(), 0);
            final_path = target;
            len = final_path.iter().position(|&c| c == 0).unwrap_or(final_path.len());
        }
        
        if len == 0 {
            return None;
        }
        
        use windows::Win32::UI::Shell::ExtractIconExW;
        let mut phiconlarge = [windows::Win32::UI::WindowsAndMessaging::HICON::default(); 1];
        let mut phiconsmall = [windows::Win32::UI::WindowsAndMessaging::HICON::default(); 1];
        
        let extracted = ExtractIconExW(
            PCWSTR(final_path.as_ptr()),
            icon_index,
            Some(phiconlarge.as_mut_ptr()),
            Some(phiconsmall.as_mut_ptr()),
            1
        );
        
        if extracted > 0 && !phiconlarge[0].is_invalid() {
            if !phiconsmall[0].is_invalid() {
                let _ = windows::Win32::UI::WindowsAndMessaging::DestroyIcon(phiconsmall[0]);
            }
            // cast to windows_sys HICON
            return Some(phiconlarge[0].0 as windows_sys::Win32::UI::WindowsAndMessaging::HICON);
        }
        if extracted > 0 && !phiconsmall[0].is_invalid() {
            return Some(phiconsmall[0].0 as windows_sys::Win32::UI::WindowsAndMessaging::HICON);
        }
        None
    }
}

#[cfg(target_os = "windows")]
fn extract_icon_png(path: &std::path::Path, icon_cache_dir: &std::path::Path) -> Option<String> {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::{
        Graphics::Gdi::{
            DeleteObject, GetDC, GetDIBits, GetObjectW, ReleaseDC, BITMAP, BITMAPINFO, BITMAPINFOHEADER, BI_RGB,
            DIB_RGB_COLORS,
        },
        Storage::FileSystem::FILE_ATTRIBUTE_NORMAL,
        UI::{
            Shell::{SHGetFileInfoW, SHFILEINFOW, SHGFI_ICON, SHGFI_LARGEICON},
            WindowsAndMessaging::{DestroyIcon, GetIconInfo, ICONINFO},
        },
    };

    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    path.to_string_lossy().to_lowercase().hash(&mut hasher);
    let output = icon_cache_dir.join(format!("{:x}.png", hasher.finish()));
    if output.exists() {
        return Some(output.to_string_lossy().to_string());
    }

    let _ = std::fs::create_dir_all(icon_cache_dir);

    let mut wide_path: Vec<u16> = path.as_os_str().encode_wide().collect();
    wide_path.push(0);

    let mut icon = std::ptr::null_mut();
    
    if let Some(ext) = path.extension() {
        if ext.to_string_lossy().to_lowercase() == "lnk" {
            if let Some(hicon) = resolve_lnk_hicon(path) {
                icon = hicon;
            }
        }
    }
    
    if icon.is_null() {
        let mut shell_info: SHFILEINFOW = unsafe { std::mem::zeroed() };
        let result = unsafe {
            SHGetFileInfoW(
                wide_path.as_ptr(),
                FILE_ATTRIBUTE_NORMAL,
                &mut shell_info,
                std::mem::size_of::<SHFILEINFOW>() as u32,
                SHGFI_ICON | SHGFI_LARGEICON,
            )
        };

        if result == 0 || shell_info.hIcon.is_null() {
            return None;
        }
        icon = shell_info.hIcon;
    }
    let png_result = unsafe {
        let mut icon_info: ICONINFO = std::mem::zeroed();
        if GetIconInfo(icon, &mut icon_info) == 0 {
            let _ = DestroyIcon(icon);
            return None;
        }

        let bitmap_handle = if !icon_info.hbmColor.is_null() {
            icon_info.hbmColor
        } else {
            icon_info.hbmMask
        };

        if bitmap_handle.is_null() {
            if !icon_info.hbmColor.is_null() {
                let _ = DeleteObject(icon_info.hbmColor);
            }
            if !icon_info.hbmMask.is_null() {
                let _ = DeleteObject(icon_info.hbmMask);
            }
            let _ = DestroyIcon(icon);
            return None;
        }

        let mut bitmap: BITMAP = std::mem::zeroed();
        let object_size = GetObjectW(
            bitmap_handle,
            std::mem::size_of::<BITMAP>() as i32,
            &mut bitmap as *mut _ as *mut std::ffi::c_void,
        );

        if object_size == 0 || bitmap.bmWidth <= 0 || bitmap.bmHeight <= 0 {
            if !icon_info.hbmColor.is_null() {
                let _ = DeleteObject(icon_info.hbmColor);
            }
            if !icon_info.hbmMask.is_null() {
                let _ = DeleteObject(icon_info.hbmMask);
            }
            let _ = DestroyIcon(icon);
            return None;
        }

        let width = bitmap.bmWidth as u32;
        let height = bitmap.bmHeight as u32;
        let mut info = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: width as i32,
                biHeight: -(height as i32),
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB,
                ..std::mem::zeroed()
            },
            ..std::mem::zeroed()
        };

        let mut buffer = vec![0u8; (width * height * 4) as usize];
        let hdc = GetDC(std::ptr::null_mut());
        let dib_result = GetDIBits(
            hdc,
            bitmap_handle,
            0,
            height,
            buffer.as_mut_ptr() as *mut std::ffi::c_void,
            &mut info,
            DIB_RGB_COLORS,
        );
        let _ = ReleaseDC(std::ptr::null_mut(), hdc);

        if !icon_info.hbmColor.is_null() {
            let _ = DeleteObject(icon_info.hbmColor);
        }
        if !icon_info.hbmMask.is_null() {
            let _ = DeleteObject(icon_info.hbmMask);
        }
        let _ = DestroyIcon(icon);

        if dib_result == 0 {
            return None;
        }

        let alpha_is_empty = buffer.chunks_exact(4).all(|pixel| pixel[3] == 0);
        for pixel in buffer.chunks_exact_mut(4) {
            pixel.swap(0, 2);
            if alpha_is_empty {
                pixel[3] = 255;
            }
        }

        image::RgbaImage::from_raw(width, height, buffer)?.save(&output).ok()?;
        Some(output.to_string_lossy().to_string())
    };

    png_result
}

#[cfg(not(target_os = "windows"))]
fn extract_icon_png(_path: &Path, _icon_cache_dir: &Path) -> Option<String> {
    None
}

fn unique_destination(dir: &Path, file_name: &str) -> PathBuf {
    let candidate = dir.join(file_name);
    if !candidate.exists() {
        return candidate;
    }

    let file_path = Path::new(file_name);
    let stem = file_path.file_stem().and_then(|value| value.to_str()).unwrap_or(file_name);
    let extension = file_path.extension().and_then(|value| value.to_str());

    for index in 1.. {
        let next_name = match extension {
            Some(extension) => format!("{stem} ({index}).{extension}"),
            None => format!("{stem} ({index})"),
        };
        let next = dir.join(next_name);
        if !next.exists() {
            return next;
        }
    }

    unreachable!("infinite iterator must return a destination")
}

fn launcher_shortcut() -> &'static str {
    "Alt+Space"
}

fn dock_window_label() -> &'static str {
    "dock"
}

fn launcher_window_label() -> &'static str {
    "launcher"
}

fn configure_overlay_window<R: tauri::Runtime>(window: &tauri::WebviewWindow<R>) {
    let _ = window.set_shadow(false);
    let _ = window.set_background_color(Some(Color(0, 0, 0, 0)));
}

fn configure_overlay_windows<R: tauri::Runtime>(app: &tauri::AppHandle<R>) {
    for label in [dock_window_label(), launcher_window_label()] {
        if let Some(window) = app.get_webview_window(label) {
            configure_overlay_window(&window);
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum ShortcutRegistrationStatus {
    Registered,
    Unavailable(String),
}

fn shortcut_registration_status(result: Result<(), String>) -> ShortcutRegistrationStatus {
    match result {
        Ok(()) => ShortcutRegistrationStatus::Registered,
        Err(error) => ShortcutRegistrationStatus::Unavailable(error),
    }
}

fn position_dock_window<R: tauri::Runtime>(app: &tauri::AppHandle<R>) {
    let Some(window) = app.get_webview_window(dock_window_label()) else {
        return;
    };

    configure_overlay_window(&window);
    let _ = window.set_always_on_top(true);

    let Ok(Some(monitor)) = window.primary_monitor() else {
        return;
    };

    let Ok(window_size) = window.outer_size() else {
        return;
    };

    let work_area = monitor.work_area();
    let x = work_area.position.x + ((work_area.size.width as i32 - window_size.width as i32) / 2).max(0);
    let y = work_area.position.y + work_area.size.height as i32 - window_size.height as i32 - 8;
    let _ = window.set_position(tauri::PhysicalPosition::new(x, y.max(work_area.position.y)));
    let _ = window.show();
}

fn show_launcher_window<R: tauri::Runtime>(app: &tauri::AppHandle<R>) -> Result<(), String> {
    let Some(window) = app.get_webview_window(launcher_window_label()) else {
        return Err("Launcher window not found".to_string());
    };

    configure_overlay_window(&window);
    let _ = window.set_always_on_top(true);
    let _ = window.center();
    window.unminimize().map_err(|error| error.to_string())?;
    window.show().map_err(|error| error.to_string())?;
    window.set_focus().map_err(|error| error.to_string())?;
    let _ = window.emit("lumora://launcher-focus", ());
    Ok(())
}

fn hide_launcher_window<R: tauri::Runtime>(app: &tauri::AppHandle<R>) -> Result<(), String> {
    let Some(window) = app.get_webview_window(launcher_window_label()) else {
        return Err("Launcher window not found".to_string());
    };

    window.hide().map_err(|error| error.to_string())
}

fn toggle_launcher_window<R: tauri::Runtime>(app: &tauri::AppHandle<R>) -> Result<bool, String> {
    let Some(window) = app.get_webview_window(launcher_window_label()) else {
        return Err("Launcher window not found".to_string());
    };

    match window.is_visible().map_err(|error| error.to_string())? {
        true => {
            window.hide().map_err(|error| error.to_string())?;
            Ok(false)
        }
        false => {
            show_launcher_window(app)?;
            Ok(true)
        }
    }
}

#[tauri::command]
fn toggle_launcher(app: tauri::AppHandle) -> Result<String, String> {
    let is_visible = toggle_launcher_window(&app)?;
    Ok(if is_visible {
        "Launcher shown".to_string()
    } else {
        "Launcher hidden".to_string()
    })
}

#[tauri::command]
fn hide_launcher(app: tauri::AppHandle) -> Result<String, String> {
    hide_launcher_window(&app)?;
    Ok("Launcher hidden".to_string())
}

fn register_launcher_shortcut<R: tauri::Runtime>(app: &tauri::AppHandle<R>) -> ShortcutRegistrationStatus {
    let result = app
        .global_shortcut()
        .on_shortcut(launcher_shortcut(), |app, _shortcut, event| {
            if event.state == ShortcutState::Pressed {
                let _ = show_launcher_window(app);
            }
        })
        .map_err(|error| error.to_string());

    let status = shortcut_registration_status(result);
    if let ShortcutRegistrationStatus::Unavailable(error) = &status {
        eprintln!("Lumora shortcut {} unavailable: {}", launcher_shortcut(), error);
    }

    status
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .setup(|app| {
            configure_overlay_windows(app.handle());
            let _ = hide_launcher_window(app.handle());
            position_dock_window(app.handle());
            let _ = register_launcher_shortcut(app.handle());
            register_dock_shell_hook(app.handle());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            app_info,
            open_target,
            activate_or_open_target,
            dock_item_statuses,
            clear_dock_item_attention,
            describe_targets,
            scan_desktop,
            search_files,
            organize_desktop,
            undo_desktop_organize,
            toggle_launcher,
            hide_launcher,
            hide_desktop_file,
            restore_desktop_file,
            update_dock_bounds
        ])
        .run(tauri::generate_context!())
        .expect("error while running Lumora");
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_test_dir(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("lumora_test_{name}_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("create temp test dir");
        dir
    }

    #[test]
    fn unique_destination_does_not_overwrite_existing_files() {
        let dir = temp_test_dir("unique_destination");
        let existing = dir.join("proposal.pdf");
        std::fs::write(&existing, b"existing").expect("write existing file");

        let next = unique_destination(&dir, "proposal.pdf");

        assert_eq!(next.file_name().and_then(|value| value.to_str()), Some("proposal (1).pdf"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn describe_target_uses_filesystem_metadata_and_extensions() {
        let dir = temp_test_dir("describe_target");
        let app_path = dir.join("Lumora.exe");
        let doc_path = dir.join("brief.pdf");
        let folder_path = dir.join("Project");
        std::fs::write(&app_path, b"app").expect("write app file");
        std::fs::write(&doc_path, b"doc").expect("write doc file");
        std::fs::create_dir_all(&folder_path).expect("create folder");

        assert_eq!(describe_target(&app_path).item_type, "app");
        assert_eq!(describe_target(&doc_path).item_type, "file");
        assert_eq!(describe_target(&folder_path).item_type, "folder");
        assert_eq!(describe_target(&doc_path).label, "brief");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn search_files_matches_names_case_insensitively_and_limits_results() {
        let dir = temp_test_dir("search_files");
        std::fs::create_dir_all(dir.join("Project")).expect("create nested dir");
        std::fs::write(dir.join("Project").join("Lumora Brief.pdf"), b"brief").expect("write brief");
        std::fs::write(dir.join("Project").join("lumora-notes.md"), b"notes").expect("write notes");
        std::fs::write(dir.join("Project").join("other.txt"), b"other").expect("write other");

        let results = search_files_in_roots("LUMORA", &[dir.clone()], 1, 10);

        assert_eq!(results.total_matches, 2);
        assert_eq!(results.files.len(), 1);
        assert_eq!(results.files[0].name, "Lumora Brief.pdf");
        assert_eq!(results.files[0].category, "Docs");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn search_files_ignores_blank_queries() {
        let dir = temp_test_dir("blank_search");
        std::fs::write(dir.join("anything.pdf"), b"doc").expect("write doc");

        let results = search_files_in_roots("   ", &[dir.clone()], 10, 10);

        assert_eq!(results.total_matches, 0);
        assert!(results.files.is_empty());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn launcher_shortcut_is_alt_space() {
        assert_eq!(launcher_shortcut(), "Alt+Space");
    }

    #[test]
    fn window_labels_match_dock_and_launcher_config() {
        assert_eq!(dock_window_label(), "dock");
        assert_eq!(launcher_window_label(), "launcher");
    }

    #[test]
    fn shortcut_registration_failure_is_non_fatal() {
        let status = shortcut_registration_status(Err("HotKey already registered".to_string()));

        assert_eq!(
            status,
            ShortcutRegistrationStatus::Unavailable("HotKey already registered".to_string())
        );
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn extract_icon_png_writes_a_cached_png_for_windows_executables() {
        let dir = temp_test_dir("icon_cache");
        let exe = std::env::current_exe().expect("current test exe path");

        let icon_path = extract_icon_png(&exe, &dir).expect("extract icon png");

        let metadata = std::fs::metadata(&icon_path).expect("icon png metadata");
        assert!(metadata.len() > 0);
        assert_eq!(Path::new(&icon_path).extension().and_then(|value| value.to_str()), Some("png"));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn resolve_dropped_path_checks_common_desktop_for_relative_shortcuts() {
        let user_desktop = temp_test_dir("user_desktop");
        let common_desktop = temp_test_dir("common_desktop");
        let common_shortcut = common_desktop.join("TencentVideo.lnk");
        std::fs::write(&common_shortcut, b"shortcut").expect("write common desktop shortcut");

        let resolved = resolve_dropped_path(
            "TencentVideo.lnk",
            &[user_desktop.clone(), common_desktop.clone()],
        );

        assert_eq!(resolved, common_shortcut);
        let _ = std::fs::remove_dir_all(&user_desktop);
        let _ = std::fs::remove_dir_all(&common_desktop);
    }

    #[test]
    fn dock_hide_plan_accepts_files_from_common_desktop() {
        let user_desktop = temp_test_dir("hide_user_desktop");
        let common_desktop = temp_test_dir("hide_common_desktop");
        let common_shortcut = common_desktop.join("TencentQQ.lnk");
        std::fs::write(&common_shortcut, b"shortcut").expect("write common desktop shortcut");

        let plan = dock_hide_plan(
            &common_shortcut,
            &[user_desktop.clone(), common_desktop.clone()],
            &user_desktop,
        )
        .expect("common desktop shortcut should be hideable");

        assert_eq!(plan.original_path, common_shortcut);
        assert_eq!(plan.hidden_dir, user_desktop.join(".lumora_dock_hidden"));

        let _ = std::fs::remove_dir_all(&user_desktop);
        let _ = std::fs::remove_dir_all(&common_desktop);
    }

    #[test]
    fn move_dock_source_to_hidden_moves_the_desktop_file() {
        let user_desktop = temp_test_dir("move_hide_user_desktop");
        let common_desktop = temp_test_dir("move_hide_common_desktop");
        let source = common_desktop.join("TencentQQ.lnk");
        std::fs::write(&source, b"shortcut").expect("write desktop shortcut");
        let plan = DockHidePlan {
            original_path: source.clone(),
            hidden_dir: user_desktop.join(".lumora_dock_hidden"),
        };

        let destination = move_dock_source_to_hidden(&source, &plan).expect("move desktop shortcut to hidden dir");

        assert!(!source.exists());
        assert!(destination.exists());
        assert_eq!(destination.parent(), Some(plan.hidden_dir.as_path()));

        let _ = std::fs::remove_dir_all(&user_desktop);
        let _ = std::fs::remove_dir_all(&common_desktop);
    }

    #[test]
    fn restore_desktop_file_moves_hidden_file_back_to_original_path() {
        let dir = temp_test_dir("restore_dock_source");
        let hidden_dir = dir.join(".lumora_dock_hidden");
        std::fs::create_dir_all(&hidden_dir).expect("create hidden dir");
        let hidden_file = hidden_dir.join("TencentQQ.lnk");
        let original_file = dir.join("TencentQQ.lnk");
        std::fs::write(&hidden_file, b"shortcut").expect("write hidden shortcut");

        restore_desktop_file(
            hidden_file.to_string_lossy().to_string(),
            original_file.to_string_lossy().to_string(),
        )
        .expect("restore hidden shortcut");

        assert!(!hidden_file.exists());
        assert!(original_file.exists());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn restore_desktop_file_errors_when_hidden_file_is_missing() {
        let dir = temp_test_dir("restore_missing_dock_source");
        let missing_hidden_file = dir.join(".lumora_dock_hidden").join("TencentQQ.lnk");
        let original_file = dir.join("TencentQQ.lnk");

        let result = restore_desktop_file(
            missing_hidden_file.to_string_lossy().to_string(),
            original_file.to_string_lossy().to_string(),
        );

        assert!(result.is_err());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn launch_identity_from_exe_target_uses_lowercase_process_name() {
        let identity = launch_identity_from_executable_path(Path::new(r"C:\Program Files\Tencent\WeChat\WeChat.exe"))
            .expect("exe target should produce a launch identity");

        assert_eq!(identity.executable_name, "wechat.exe");
        assert_eq!(
            identity.executable_path,
            Some(PathBuf::from(r"C:\Program Files\Tencent\WeChat\WeChat.exe"))
        );
    }

    #[test]
    fn running_wechat_suppresses_open_after_activation_miss() {
        let identity = launch_identity_from_executable_path(Path::new(r"D:\software\Weixin\Weixin.exe"))
            .expect("weixin identity");
        let processes = vec![RunningProcess {
            executable_path: Some(PathBuf::from(r"D:\software\Weixin\Weixin.exe")),
            executable_name: "weixin.exe".to_string(),
        }];

        assert!(should_suppress_open_after_activation_miss(&identity, &processes));
    }

    #[test]
    fn non_tray_apps_still_open_after_activation_miss() {
        let identity = launch_identity_from_executable_path(Path::new(r"C:\Tools\Notepad.exe"))
            .expect("notepad identity");
        let processes = vec![RunningProcess {
            executable_path: Some(PathBuf::from(r"C:\Tools\Notepad.exe")),
            executable_name: "notepad.exe".to_string(),
        }];

        assert!(!should_suppress_open_after_activation_miss(&identity, &processes));
    }

    #[test]
    fn wechat_tray_text_matches_personal_wechat_but_not_enterprise_wechat() {
        let identity = launch_identity_from_executable_path(Path::new(r"D:\software\Weixin\Weixin.exe"))
            .expect("weixin identity");

        assert!(tray_button_text_matches_launch_identity("微信", &identity));
        assert!(tray_button_text_matches_launch_identity("微信: 八月", &identity));
        assert!(!tray_button_text_matches_launch_identity("企业微信: 杨宽", &identity));
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn resolve_lnk_target_path_reads_link_info_when_shell_get_path_is_empty() {
        let dir = temp_test_dir("link_info_lnk");
        let shortcut = dir.join("Weixin.lnk");
        let target = r"D:\software\Weixin\Weixin.exe";
        write_link_info_only_shortcut(&shortcut, target);

        let resolved = resolve_lnk_target_path(&shortcut).expect("resolve link info target");

        assert_eq!(resolved, PathBuf::from(target));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn hidden_titled_qt_windows_are_not_guessed_as_activation_candidates() {
        assert_eq!(window_activation_rank(false, false, "Qt51514QWindowIcon", "Weixin"), None);
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn tool_windows_are_not_activation_candidates() {
        assert_eq!(
            window_activation_rank_for_info(WindowActivationInfo {
                is_visible: true,
                is_iconic: false,
                has_owner: false,
                class_name: "Qt51514QWindowToolSaveBits",
                title: "Weixin",
                ex_style: windows_sys::Win32::UI::WindowsAndMessaging::WS_EX_TOOLWINDOW,
                cloaked: false,
                left: 72,
                top: 64,
                width: 1563,
                height: 871,
            }),
            None
        );
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn wechat_media_windows_are_not_activation_candidates() {
        assert_eq!(
            window_activation_rank_for_info(WindowActivationInfo {
                is_visible: true,
                is_iconic: false,
                has_owner: false,
                class_name: "Qt51514QWindowIcon",
                title: "图片和视频",
                ex_style: 0,
                cloaked: false,
                left: 65,
                top: 23,
                width: 1578,
                height: 919,
            }),
            None
        );
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn minimized_offscreen_windows_are_activation_candidates() {
        assert_eq!(
            window_activation_rank_for_info(WindowActivationInfo {
                is_visible: true,
                is_iconic: true,
                has_owner: false,
                class_name: "Qt51514QWindowIcon",
                title: "Weixin",
                ex_style: 0,
                cloaked: false,
                left: -21333,
                top: -21333,
                width: 158,
                height: 26,
            }),
            Some(1)
        );
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn offscreen_non_minimized_placeholders_are_not_activation_candidates() {
        assert_eq!(
            window_activation_rank_for_info(WindowActivationInfo {
                is_visible: true,
                is_iconic: false,
                has_owner: false,
                class_name: "Qt51514QWindowIcon",
                title: "Weixin",
                ex_style: 0,
                cloaked: false,
                left: -21333,
                top: -21333,
                width: 158,
                height: 26,
            }),
            None
        );
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn hidden_large_wechat_windows_are_not_activation_candidates() {
        assert_eq!(
            window_activation_rank_for_info(WindowActivationInfo {
                is_visible: false,
                is_iconic: false,
                has_owner: false,
                class_name: "Qt51514QWindowIcon",
                title: "微信",
                ex_style: 0,
                cloaked: false,
                left: 448,
                top: 171,
                width: 849,
                height: 669,
            }),
            None
        );
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn hidden_message_and_input_windows_are_not_activation_candidates() {
        assert_eq!(window_activation_rank(false, false, "Chrome_SystemMessageWindow", ""), None);
        assert_eq!(
            window_activation_rank(false, false, "Qt51514WxTrayIconMessageWindowClass", "WxTrayIconMessageWindow"),
            None
        );
        assert_eq!(window_activation_rank(false, true, "Qt51514QWindowIcon", "Weixin"), None);
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn hidden_windows_restore_instead_of_plain_show() {
        use windows_sys::Win32::UI::WindowsAndMessaging::{SW_RESTORE, SW_SHOW};

        assert_eq!(activation_show_command(false, false), SW_RESTORE);
        assert_eq!(activation_show_command(false, true), SW_SHOW);
        assert_eq!(activation_show_command(true, true), SW_RESTORE);
    }

    #[test]
    fn dock_item_statuses_preserve_unresolved_targets_as_not_running() {
        let statuses = dock_item_statuses_for_targets(vec!["lumora://trash".to_string()]);

        assert_eq!(statuses.len(), 1);
        assert_eq!(statuses[0].target, "lumora://trash");
        assert!(!statuses[0].is_running);
        assert!(!statuses[0].needs_attention);
    }

    #[test]
    fn dock_item_statuses_reflect_and_clear_attention_by_launch_identity() {
        let target = r"C:\Program Files\Tencent\WeChat\WeChat.exe".to_string();
        let identity = launch_identity_from_executable_path(Path::new(&target)).expect("exe target identity");

        mark_dock_attention(identity);
        let statuses = dock_item_statuses_for_targets(vec![target.clone()]);

        assert_eq!(statuses[0].target, target);
        assert!(statuses[0].needs_attention);
        assert!(statuses[0].attention_sequence > 0);

        clear_dock_attention_for_target(&target);
        let statuses = dock_item_statuses_for_targets(vec![target]);

        assert!(!statuses[0].needs_attention);
    }

    #[cfg(target_os = "windows")]
    fn write_link_info_only_shortcut(path: &Path, target: &str) {
        fn push_u32(bytes: &mut Vec<u8>, value: u32) {
            bytes.extend_from_slice(&value.to_le_bytes());
        }

        fn push_u16(bytes: &mut Vec<u8>, value: u16) {
            bytes.extend_from_slice(&value.to_le_bytes());
        }

        let target_bytes = target.as_bytes();
        let volume_id_size = 16u32;
        let local_base_path_offset = 0x1c + volume_id_size;
        let common_path_suffix_offset = local_base_path_offset + target_bytes.len() as u32 + 1;
        let link_info_size = common_path_suffix_offset + 1;
        let mut bytes = Vec::new();

        push_u32(&mut bytes, 0x4c);
        bytes.extend_from_slice(&[
            0x01, 0x14, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0xc0, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x46,
        ]);
        push_u32(&mut bytes, 0x2);
        push_u32(&mut bytes, 0);
        push_u32(&mut bytes, 0);
        push_u32(&mut bytes, 0);
        push_u32(&mut bytes, 0);
        push_u32(&mut bytes, 0);
        push_u32(&mut bytes, 0);
        push_u32(&mut bytes, 0);
        push_u32(&mut bytes, 0);
        push_u32(&mut bytes, 0);
        push_u32(&mut bytes, 1);
        push_u16(&mut bytes, 0);
        push_u16(&mut bytes, 0);
        push_u32(&mut bytes, 0);
        push_u32(&mut bytes, 0);

        push_u32(&mut bytes, link_info_size);
        push_u32(&mut bytes, 0x1c);
        push_u32(&mut bytes, 1);
        push_u32(&mut bytes, 0x1c);
        push_u32(&mut bytes, local_base_path_offset);
        push_u32(&mut bytes, 0);
        push_u32(&mut bytes, common_path_suffix_offset);
        push_u32(&mut bytes, volume_id_size);
        push_u32(&mut bytes, 3);
        push_u32(&mut bytes, 0);
        push_u32(&mut bytes, volume_id_size);
        bytes.extend_from_slice(target_bytes);
        bytes.push(0);
        bytes.push(0);

        std::fs::write(path, bytes).expect("write link info shortcut");
    }
}
