//! Kaca tembus pandang tanpa API ajaib.
//!
//! Acrylic DWM sudah dicoba dan gagal pada jendela ber-alpha per piksel
//! (hasilnya kotak hitam). Jalur yang dipakai sekarang: ambil isi layar di
//! belakang notch, perkecil 1/8, blur dengan kotak blur, lalu gambar sebagai
//! tekstur latar. Perkecilan itu sendiri sudah menghilangkan detail, jadi dua
//! lintasan blur sudah cukup dan biayanya di bawah satu milidetik.

use eframe::egui;

/// Menangkap area layar dan mengembalikannya sudah dalam keadaan kabur.
#[cfg(windows)]
pub fn capture(x: i32, y: i32, w: i32, h: i32) -> Option<egui::ColorImage> {
    use windows::Win32::Graphics::Gdi::{
        CreateCompatibleDC, CreateDIBSection, DeleteDC, DeleteObject, GetDC, ReleaseDC,
        SelectObject, SetStretchBltMode, StretchBlt, BITMAPINFO, BITMAPINFOHEADER, BI_RGB,
        DIB_RGB_COLORS, HALFTONE, SRCCOPY,
    };

    if w <= 0 || h <= 0 {
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
            DeleteDC(mem);
            ReleaseDC(None, layar);
            return None;
        };

        if bita.is_null() {
            DeleteObject(dib);
            DeleteDC(mem);
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
        DeleteObject(dib);
        DeleteDC(mem);
        ReleaseDC(None, layar);

        if !ok {
            return None;
        }

        box_blur(&mut rgba, dw as usize, dh as usize, 2);

        Some(egui::ColorImage::from_rgba_unmultiplied(
            [dw as usize, dh as usize],
            &rgba,
        ))
    }
}

#[cfg(not(windows))]
pub fn capture(_x: i32, _y: i32, _w: i32, _h: i32) -> Option<egui::ColorImage> {
    None
}

/// Kotak blur dua lintasan (setara blur Gaussian kasar).
fn box_blur(data: &mut [u8], w: usize, h: usize, lintasan: usize) {
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
