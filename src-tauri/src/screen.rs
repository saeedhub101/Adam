use base64::{engine::general_purpose::STANDARD, Engine as _};
use image::{ImageBuffer, Rgba};

#[derive(Debug, Clone, serde::Serialize)]
pub struct CaptureResult {
    pub width: u32,
    pub height: u32,
    pub png_base64: String,
}

#[cfg(target_os = "windows")]
pub fn capture_desktop() -> Result<CaptureResult, String> {
    use windows::Win32::Foundation::HWND;
    use windows::Win32::Graphics::Gdi::*;
    use windows::Win32::UI::WindowsAndMessaging::{GetDC, GetDesktopWindow, GetSystemMetrics, ReleaseDC, SM_CXVIRTUALSCREEN, SM_CYVIRTUALSCREEN, SM_XVIRTUALSCREEN, SM_YVIRTUALSCREEN};

    unsafe {
        let hwnd: HWND = GetDesktopWindow();
        let hdc = GetDC(hwnd);
        if hdc.0 == 0 { return Err("Unable to acquire desktop DC".into()); }

        let left = GetSystemMetrics(SM_XVIRTUALSCREEN);
        let top = GetSystemMetrics(SM_YVIRTUALSCREEN);
        let width = GetSystemMetrics(SM_CXVIRTUALSCREEN).max(1) as u32;
        let height = GetSystemMetrics(SM_CYVIRTUALSCREEN).max(1) as u32;

        let mem = CreateCompatibleDC(hdc);
        if mem.0 == 0 {
            let _ = ReleaseDC(hwnd, hdc);
            return Err("Unable to create capture DC".into());
        }
        let bitmap = CreateCompatibleBitmap(hdc, width as i32, height as i32);
        if bitmap.0 == 0 {
            let _ = DeleteDC(mem);
            let _ = ReleaseDC(hwnd, hdc);
            return Err("Unable to create capture bitmap".into());
        }
        let old = SelectObject(mem, bitmap);
        let copied = BitBlt(mem, 0, 0, width as i32, height as i32, hdc, left, top, SRCCOPY | CAPTUREBLT).is_ok();
        if !copied {
            let _ = SelectObject(mem, old);
            let _ = DeleteObject(bitmap);
            let _ = DeleteDC(mem);
            let _ = ReleaseDC(hwnd, hdc);
            return Err("Screen capture failed".into());
        }

        let mut info = BITMAPINFO::default();
        info.bmiHeader.biSize = std::mem::size_of::<BITMAPINFOHEADER>() as u32;
        info.bmiHeader.biWidth = width as i32;
        info.bmiHeader.biHeight = -(height as i32);
        info.bmiHeader.biPlanes = 1;
        info.bmiHeader.biBitCount = 32;
        info.bmiHeader.biCompression = BI_RGB.0;

        let mut pixels = vec![0u8; width as usize * height as usize * 4];
        let scanlines = GetDIBits(
            mem,
            bitmap,
            0,
            height,
            Some(pixels.as_mut_ptr() as *mut _),
            &mut info,
            DIB_RGB_COLORS,
        );
        let _ = SelectObject(mem, old);
        let _ = DeleteObject(bitmap);
        let _ = DeleteDC(mem);
        let _ = ReleaseDC(hwnd, hdc);

        if scanlines == 0 { return Err("Unable to read captured pixels".into()); }
        let rgba = ImageBuffer::<Rgba<u8>, _>::from_raw(width, height, pixels)
            .ok_or_else(|| "Invalid capture buffer".to_string())?;
        let mut png = Vec::new();
        let mut encoder = image::codecs::png::PngEncoder::new(&mut png);
        image::ImageEncoder::write_image(&mut encoder, rgba.as_raw(), width, height, image::ExtendedColorType::Rgba8)
            .map_err(|e| e.to_string())?;
        Ok(CaptureResult { width, height, png_base64: STANDARD.encode(png) })
    }
}

#[cfg(not(target_os = "windows"))]
pub fn capture_desktop() -> Result<CaptureResult, String> {
    Err("Desktop capture is only supported on Windows".into())
}
