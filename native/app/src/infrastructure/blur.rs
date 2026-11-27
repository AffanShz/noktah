//! Kaca tembus pandang tanpa API ajaib.
//!
//! Acrylic DWM sudah dicoba dan gagal pada jendela ber-alpha per piksel
//! (hasilnya kotak hitam). Jalur yang dipakai sekarang: ambil isi layar di
//! belakang notch, perkecil 1/8, inpaint jejak pill agar tidak ada bayangan
//! gelap, blur dengan kotak blur, lalu atur saturasi warna.

use eframe::egui;

/// Menangkap area layar mentah (tanpa blur), diperkecil 1/8 dan di-inpaint jejak pill.
#[cfg(windows)]
pub fn capture_raw(x: i32, y: i32, w: i32, h: i32) -> Option<(usize, usize, Vec<u8>)> {
    use windows::Win32::Graphics::Gdi::{
        CreateCompatibleDC, CreateDIBSection, DeleteDC, DeleteObject, GetDC, ReleaseDC,
        SelectObject, SetStretchBltMode, StretchBlt, BITMAPINFO, BITMAPINFOHEADER, BI_RGB,
        DIB_RGB_COLORS, HALFTONE, SRCCOPY,
    };

    if w <= 0 || h <= 0 || w > 4096 || h > 2160 {
        return None;
    }

    const SKALA: i32 = 8;
    let dw = (w / SKALA).max(8);
    let dh = (h / SKALA).max(8);

    unsafe {
        let layar = GetDC(None);
        if layar.is_invalid() {
            return None;
        }

        let mem = CreateCompatibleDC(layar);
        if mem.is_invalid() {
            ReleaseDC(None, layar);
            return None;
        }

        let mut info = BITMAPINFO::default();
        info.bmiHeader = BITMAPINFOHEADER {
            biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: dw,
            // Tinggi negatif = baris dari atas ke bawah, sesuai urutan piksel
            // yang diharapkan egui.
            biHeight: -dh,
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB.0,
            ..Default::default()
        };

        let mut bita: *mut core::ffi::c_void = std::ptr::null_mut();
        let Ok(dib) = CreateDIBSection(mem, &info, DIB_RGB_COLORS, &mut bita, None, 0) else {
            let _ = DeleteDC(mem);
            ReleaseDC(None, layar);
            return None;
        };

        if bita.is_null() {
            let _ = DeleteObject(dib);
            let _ = DeleteDC(mem);
            ReleaseDC(None, layar);
            return None;
        }

        let lama = SelectObject(mem, dib);
        SetStretchBltMode(mem, HALFTONE);
        let ok = StretchBlt(mem, 0, 0, dw, dh, layar, x, y, w, h, SRCCOPY).as_bool();

        let mut rgba = Vec::with_capacity((dw * dh * 4) as usize);
        if ok {
            let sumber = std::slice::from_raw_parts(bita as *const u8, (dw * dh * 4) as usize);
            rgba.reserve(sumber.len());
            for px in sumber.chunks_exact(4) {
                // GDI memberi BGRA.
                rgba.push(px[2]);
                rgba.push(px[1]);
                rgba.push(px[0]);
                rgba.push(255);
            }
        }

        SelectObject(mem, lama);
        let _ = DeleteObject(dib);
        let _ = DeleteDC(mem);
        ReleaseDC(None, layar);

        if !ok {
            return None;
        }

        inpaint_pill_strip(&mut rgba, dw as usize, dh as usize);
        Some((dw as usize, dh as usize, rgba))
    }
}

#[cfg(not(windows))]
pub fn capture_raw(_x: i32, _y: i32, _w: i32, _h: i32) -> Option<(usize, usize, Vec<u8>)> {
    None
}

/// Memproses raw RGBA: menjalankan multi-pass box blur dan penyesuaian saturasi warna.
pub fn process_raw(
    w: usize,
    h: usize,
    raw: &[u8],
    passes: u8,
    saturation: f32,
) -> egui::ColorImage {
    let mut rgba = raw.to_vec();
    box_blur(
        &mut rgba,
        w,
        h,
        passes.clamp(1, 5) as usize,
    );
    apply_saturation(&mut rgba, saturation);
    egui::ColorImage::from_rgba_unmultiplied([w, h], &rgba)
}

/// Menangkap area layar dan mengembalikannya sudah dalam keadaan kabur dan berpenyesuaian saturasi.
#[allow(dead_code)]
pub fn capture(
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    passes: u8,
    saturation: f32,
) -> Option<egui::ColorImage> {
    let (dw, dh, raw) = capture_raw(x, y, w, h)?;
    Some(process_raw(dw, dh, &raw, passes, saturation))
}

/// Menggantikan area jejak pill di bagian tengah atas dengan sampel piksel dari
/// baris di bawahnya sebelum proses blur, mencegah timbulnya artefak noda gelap.
pub(crate) fn inpaint_pill_strip(rgba: &mut [u8], w: usize, h: usize) {
    if w == 0 || h == 0 || rgba.len() < w * h * 4 {
        return;
    }
    let cut_h = (h / 4).max(1);
    let sample_y = cut_h.min(h.saturating_sub(1));

    let span = ((w as f32) * 0.55).round() as usize;
    let start_x = (w.saturating_sub(span)) / 2;
    let end_x = (start_x + span).min(w);

    for y in 0..cut_h {
        for x in start_x..end_x {
            let target = (y * w + x) * 4;
            let sample = (sample_y * w + x) * 4;
            rgba[target] = rgba[sample];
            rgba[target + 1] = rgba[sample + 1];
            rgba[target + 2] = rgba[sample + 2];
            rgba[target + 3] = rgba[sample + 3];
        }
    }
}

/// Menyesuaikan saturasi warna RGB terhadap luminans standar BT.709.
pub(crate) fn apply_saturation(rgba: &mut [u8], saturation: f32) {
    if (saturation - 1.0).abs() < 0.01 || !saturation.is_finite() {
        return;
    }
    for px in rgba.chunks_exact_mut(4) {
        let r = px[0] as f32;
        let g = px[1] as f32;
        let b = px[2] as f32;
        let lum = 0.2126 * r + 0.7152 * g + 0.0722 * b;
        px[0] = (lum + saturation * (r - lum)).round().clamp(0.0, 255.0) as u8;
        px[1] = (lum + saturation * (g - lum)).round().clamp(0.0, 255.0) as u8;
        px[2] = (lum + saturation * (b - lum)).round().clamp(0.0, 255.0) as u8;
    }
}

/// Kotak blur multi-lintasan (setara blur Gaussian halus).
pub(crate) fn box_blur(data: &mut [u8], w: usize, h: usize, lintasan: usize) {
    if w == 0 || h == 0 {
        return;
    }
    let mut sementara = data.to_vec();

    for _ in 0..lintasan {
        // Horizontal
        for y in 0..h {
            for x in 0..w {
                let mut total = [0u32; 3];
                let mut n = 0u32;
                for dx in -1i32..=1 {
                    let sx = x as i32 + dx;
                    if sx < 0 || sx >= w as i32 {
                        continue;
                    }
                    let i = (y * w + sx as usize) * 4;
                    total[0] += data[i] as u32;
                    total[1] += data[i + 1] as u32;
                    total[2] += data[i + 2] as u32;
                    n += 1;
                }
                let o = (y * w + x) * 4;
                for c in 0..3 {
                    sementara[o + c] = (total[c] / n.max(1)) as u8;
                }
            }
        }
        data.copy_from_slice(&sementara);

        // Vertikal
        for y in 0..h {
            for x in 0..w {
                let mut total = [0u32; 3];
                let mut n = 0u32;
                for dy in -1i32..=1 {
                    let sy = y as i32 + dy;
                    if sy < 0 || sy >= h as i32 {
                        continue;
                    }
                    let i = (sy as usize * w + x) * 4;
                    total[0] += data[i] as u32;
                    total[1] += data[i + 1] as u32;
                    total[2] += data[i + 2] as u32;
                    n += 1;
                }
                let o = (y * w + x) * 4;
                for c in 0..3 {
                    sementara[o + c] = (total[c] / n.max(1)) as u8;
                }
            }
        }
        data.copy_from_slice(&sementara);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inpaint_pill_strip() {
        let w = 10;
        let h = 8;
        let mut data = vec![0u8; w * h * 4];
        let cut_h = (h / 4).max(1);
        for x in 0..w {
            let idx = (cut_h * w + x) * 4;
            data[idx] = 100;
            data[idx + 1] = 150;
            data[idx + 2] = 200;
            data[idx + 3] = 255;
        }

        inpaint_pill_strip(&mut data, w, h);

        let target_idx = (0 * w + 4) * 4;
        assert_eq!(data[target_idx], 100);
        assert_eq!(data[target_idx + 1], 150);
        assert_eq!(data[target_idx + 2], 200);

        let outer_idx = (0 * w + 0) * 4;
        assert_eq!(data[outer_idx], 0);
    }

    #[test]
    fn test_box_blur_smooths_data() {
        let w = 4;
        let h = 4;
        let mut data = vec![0u8; w * h * 4];
        let center = (1 * w + 1) * 4;
        data[center] = 255;
        data[center + 1] = 255;
        data[center + 2] = 255;
        data[center + 3] = 255;

        box_blur(&mut data, w, h, 1);

        assert!(data[center] < 255);
        assert!(data[center] > 0);
        let adj = (1 * w + 2) * 4;
        assert!(data[adj] > 0);
    }

    #[test]
    fn test_apply_saturation() {
        let mut data = vec![255, 0, 0, 255];
        apply_saturation(&mut data, 0.0);
        assert_eq!(data[0], data[1]);
        assert_eq!(data[1], data[2]);
        assert!((data[0] as i32 - 54).abs() <= 1);

        let mut data2 = vec![120, 80, 40, 255];
        apply_saturation(&mut data2, 1.0);
        assert_eq!(data2, vec![120, 80, 40, 255]);
    }
}
