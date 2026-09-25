use base64::{engine::general_purpose::STANDARD, Engine as _};
use image::{ImageBuffer, Rgba};

#[derive(Debug, Clone, serde::Serialize)]
pub struct CaptureResult {
    pub width: u32,
    pub height: u32,
    pub original_width: u32,
    pub original_height: u32,
    pub scale_x: f32,
    pub scale_y: f32,
    pub png_base64: String,
}

const MAX_LONG_EDGE: u32 = 1568;

#[cfg(target_os = "windows")]
fn foreground_is_excluded(excluded: &[String]) -> bool {
    use std::path::PathBuf;
    use windows::Win32::Foundation::HWND;
    use windows::Win32::System::Threading::{OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_FORMAT, PROCESS_QUERY_LIMITED_INFORMATION};
    use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId};
    let hwnd: HWND = unsafe { GetForegroundWindow() };
    if hwnd.0.is_null() { return false; }
    let mut pid = 0u32;
    unsafe { GetWindowThreadProcessId(hwnd, Some(&mut pid)); }
    if pid == 0 { return false; }
    let process = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) };
    let Ok(process) = process else { return false; };
    let mut buf = vec![0u16; 1024];
    let mut len = buf.len() as u32;
    let ok = unsafe { QueryFullProcessImageNameW(process, PROCESS_NAME_FORMAT(0), windows::core::PWSTR(buf.as_mut_ptr()), &mut len).is_ok() };
    unsafe { let _ = windows::Win32::Foundation::CloseHandle(process); }
    if !ok { return false; }
    let path = PathBuf::from(String::from_utf16_lossy(&buf[..len as usize]));
    let exe = path.file_name().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();
    let full = path.to_string_lossy().to_lowercase();
    excluded.iter().any(|raw| {
        let item = raw.trim().to_lowercase();
        !item.is_empty() && (exe == item || full == item || full.contains(&item))
    })
}

#[cfg(target_os = "windows")]
pub fn capture_desktop(excluded_apps: &[String]) -> Result<CaptureResult, String> {
    use windows::Win32::Foundation::HWND;
    use windows::Win32::Graphics::Gdi::*;
    use windows::Win32::UI::WindowsAndMessaging::{GetDesktopWindow, GetSystemMetrics, SM_CXVIRTUALSCREEN, SM_CYVIRTUALSCREEN, SM_XVIRTUALSCREEN, SM_YVIRTUALSCREEN};
    if foreground_is_excluded(excluded_apps) {
        return Err("Screen capture blocked because the foreground application is excluded by Privacy settings.".into());
    }
    unsafe {
        let hwnd: HWND = GetDesktopWindow();
        let hdc = GetDC(Some(hwnd));
        if hdc.0.is_null() { return Err("Unable to acquire desktop DC".into()); }
        let left = GetSystemMetrics(SM_XVIRTUALSCREEN);
        let top = GetSystemMetrics(SM_YVIRTUALSCREEN);
        let original_width = GetSystemMetrics(SM_CXVIRTUALSCREEN).max(1) as u32;
        let original_height = GetSystemMetrics(SM_CYVIRTUALSCREEN).max(1) as u32;
        let mem = CreateCompatibleDC(Some(hdc));
        if mem.0.is_null() { let _ = ReleaseDC(Some(hwnd), hdc); return Err("Unable to create capture DC".into()); }
        let bitmap = CreateCompatibleBitmap(hdc, original_width as i32, original_height as i32);
        if bitmap.0.is_null() { let _ = DeleteDC(mem); let _ = ReleaseDC(Some(hwnd), hdc); return Err("Unable to create capture bitmap".into()); }
        let old = SelectObject(mem, bitmap.into());
        let copied = BitBlt(mem, 0, 0, original_width as i32, original_height as i32, Some(hdc), left, top, SRCCOPY | CAPTUREBLT).is_ok();
        if !copied { let _ = SelectObject(mem, old); let _ = DeleteObject(bitmap.into()); let _ = DeleteDC(mem); let _ = ReleaseDC(Some(hwnd), hdc); return Err("Screen capture failed".into()); }
        let mut info = BITMAPINFO::default();
        info.bmiHeader.biSize = std::mem::size_of::<BITMAPINFOHEADER>() as u32;
        info.bmiHeader.biWidth = original_width as i32;
        info.bmiHeader.biHeight = -(original_height as i32);
        info.bmiHeader.biPlanes = 1;
        info.bmiHeader.biBitCount = 32;
        info.bmiHeader.biCompression = BI_RGB.0;
        let mut pixels = vec![0u8; original_width as usize * original_height as usize * 4];
        let scanlines = GetDIBits(mem, bitmap, 0, original_height, Some(pixels.as_mut_ptr() as *mut _), &mut info, DIB_RGB_COLORS);
        let _ = SelectObject(mem, old); let _ = DeleteObject(bitmap.into()); let _ = DeleteDC(mem); let _ = ReleaseDC(Some(hwnd), hdc);
        if scanlines == 0 { return Err("Unable to read captured pixels".into()); }
        for px in pixels.chunks_exact_mut(4) { px.swap(0, 2); }
        let rgba = ImageBuffer::<Rgba<u8>, _>::from_raw(original_width, original_height, pixels).ok_or_else(|| "Invalid capture buffer".to_string())?;
        let scale = (MAX_LONG_EDGE as f32 / original_width.max(original_height) as f32).min(1.0);
        let target_width = ((original_width as f32 * scale).round() as u32).max(1);
        let target_height = ((original_height as f32 * scale).round() as u32).max(1);
        let resized = if scale < 0.9999 { image::imageops::resize(&rgba, target_width, target_height, image::imageops::FilterType::Triangle) } else { rgba };
        let mut png = Vec::new();
        let encoder = image::codecs::png::PngEncoder::new(&mut png);
        image::ImageEncoder::write_image(encoder, resized.as_raw(), resized.width(), resized.height(), image::ExtendedColorType::Rgba8).map_err(|e| e.to_string())?;
        Ok(CaptureResult { width: resized.width(), height: resized.height(), original_width, original_height, scale_x: original_width as f32 / resized.width() as f32, scale_y: original_height as f32 / resized.height() as f32, png_base64: STANDARD.encode(png) })
    }
}

#[cfg(not(target_os = "windows"))]
pub fn capture_desktop(_: &[String]) -> Result<CaptureResult, String> {
    Err("Desktop capture is only supported on Windows".into())
}
