use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    path::{Path, PathBuf},
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
fn describe_targets(app: tauri::AppHandle, paths: Vec<String>) -> Vec<DockTarget> {
    let icon_cache_dir = app.path().app_cache_dir().ok().map(|path| path.join("icons"));
    let desktop_root = desktop_path().ok().map(PathBuf::from);

    paths
        .iter()
        .map(|path| {
            let mut pb = PathBuf::from(expand_user_path(path));
            if pb.is_relative() {
                if let Some(desktop) = &desktop_root {
                    let candidate = desktop.join(&pb);
                    if candidate.exists() {
                        pb = candidate;
                    }
                }
            }
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
    let mut pb = PathBuf::from(expand_user_path(&path_str));
    let desktop_root = desktop_path().ok().map(PathBuf::from);

    if pb.is_relative() {
        if let Some(desktop) = &desktop_root {
            let candidate = desktop.join(&pb);
            if candidate.exists() {
                pb = candidate;
            }
        }
    }

    let icon_cache_dir = app.path().app_cache_dir().ok().map(|path| path.join("icons"));
    let mut target = describe_target_with_icon(&pb, icon_cache_dir.as_deref());

    if let Some(desktop) = &desktop_root {
        if let Ok(canon_pb) = pb.canonicalize() {
            if let Ok(canon_desktop) = desktop.canonicalize() {
                let target_root_dir = desktop.join("Lumora整理");
                let hidden_dir = desktop.join(".lumora_dock_hidden");
                
                if canon_pb.starts_with(&canon_desktop) && !canon_pb.starts_with(&target_root_dir) && !canon_pb.starts_with(&hidden_dir) {
                    if std::fs::create_dir_all(&hidden_dir).is_ok() {
                        let file_name = pb.file_name().unwrap_or_default();
                        let destination = unique_destination(&hidden_dir, &file_name.to_string_lossy());
                        if std::fs::rename(&pb, &destination).is_ok() {
                            #[cfg(target_os = "windows")]
                            {
                                use std::os::windows::ffi::OsStrExt;
                                let wide_path: Vec<u16> = hidden_dir.as_os_str().encode_wide().chain(std::iter::once(0)).collect();
                                unsafe {
                                    windows_sys::Win32::Storage::FileSystem::SetFileAttributesW(
                                        wide_path.as_ptr(),
                                        windows_sys::Win32::Storage::FileSystem::FILE_ATTRIBUTE_HIDDEN | windows_sys::Win32::Storage::FileSystem::FILE_ATTRIBUTE_SYSTEM
                                    );
                                }
                            }
                            target.original_desktop_path = Some(pb.to_string_lossy().to_string());
                            target.target = destination.to_string_lossy().to_string();
                        }
                    }
                }
            }
        }
    }

    Ok(target)
}

#[tauri::command]
fn restore_desktop_file(current_path: String, original_path: String) -> Result<(), String> {
    let current_pb = PathBuf::from(expand_user_path(&current_path));
    let original_pb = PathBuf::from(expand_user_path(&original_path));

    if current_pb.exists() {
        if let Some(parent) = original_pb.parent() {
            if std::fs::create_dir_all(parent).is_ok() {
                let dest = unique_destination(parent, &original_pb.file_name().unwrap_or_default().to_string_lossy());
                let _ = std::fs::rename(&current_pb, &dest);
            }
        }
    }

    Ok(())
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
        
        let mut path_u16: Vec<u16> = path.as_os_str().encode_wide().chain(std::iter::once(0)).collect();
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
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            app_info,
            open_target,
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
}
