//! Visualizer yang benar-benar mengikuti suara.
//!
//! Membaca output audio sistem lewat WASAPI loopback, lalu menghitung energi
//! empat pita dengan filter satu kutub. Tidak ada FFT — untuk empat balok,
//! pemisahan kasar sudah cukup dan jauh lebih murah.
//!
//! Kalau perangkat audio tidak bisa dibuka (tidak ada output, atau loopback
//! tidak didukung), UI kembali memakai animasi sinus.
//!
//! Thread ini hanya hidup saat equalizer benar-benar terlihat. Sebelumnya ia
//! membaca audio sistem 35 kali per detik terus-menerus — terukur ~35% satu
//! core saat aplikasi diam, dan itu langsung memangsa anggaran frame morph.
//! Sekarang UI menyalakannya hanya ketika baris minimized sedang tampil.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Receiver};
use std::sync::Arc;
use std::time::Duration;

/// Energi empat pita, masing-masing 0.0–1.0.
pub type Bands = [f32; 4];

/// Pegangan ke thread visualizer.
pub struct AudioHandle {
    pub rx: Receiver<Bands>,
    /// true = UI sedang menampilkan equalizer. Thread tidur panjang saat false.
    pub active: Arc<AtomicBool>,
}

impl AudioHandle {
    /// Dipanggil setiap frame dari UI.
    pub fn set_active(&self, on: bool) {
        self.active.store(on, Ordering::Relaxed);
    }
}

pub fn spawn(ctx: egui::Context) -> AudioHandle {
    let (tx, rx) = channel::<Bands>();
    let active = Arc::new(AtomicBool::new(false));
    let flag = active.clone();

    let _ = std::thread::Builder::new()
        .name("noktah-audio".into())
        .stack_size(256 * 1024)
        .spawn(move || {
            #[cfg(windows)]
            run(ctx, tx, flag);

            #[cfg(not(windows))]
            let _ = (ctx, tx, flag);
        });

    AudioHandle { rx, active }
}

/// Filter satu kutub: cukup untuk memisahkan pita tanpa FFT.
#[cfg(windows)]
struct OnePole {
    y: f32,
    a: f32,
}

#[cfg(windows)]
impl OnePole {
    fn new(cutoff: f32, sample_rate: f32) -> Self {
        let a = 1.0 - (-2.0 * std::f32::consts::PI * cutoff / sample_rate).exp();
        Self { y: 0.0, a }
    }

    fn step(&mut self, x: f32) -> f32 {
        self.y += self.a * (x - self.y);
        self.y
    }
}

#[cfg(windows)]
fn run(ctx: egui::Context, tx: std::sync::mpsc::Sender<Bands>, active: Arc<AtomicBool>) {
    use windows::Win32::Media::Audio::{
        eConsole, eRender, IAudioCaptureClient, IAudioClient, IMMDeviceEnumerator, MMDeviceEnumerator,
        AUDCLNT_SHAREMODE_SHARED, AUDCLNT_STREAMFLAGS_LOOPBACK, WAVEFORMATEX,
    };
    use windows::Win32::System::Com::{
        CoCreateInstance, CoInitializeEx, CLSCTX_ALL, COINIT_MULTITHREADED,
    };

    unsafe {
        let _ = CoInitializeEx(None, COINIT_MULTITHREADED);

        let Ok(enumerator) =
            CoCreateInstance::<_, IMMDeviceEnumerator>(&MMDeviceEnumerator, None, CLSCTX_ALL)
        else {
            return;
        };
        let Ok(device) = enumerator.GetDefaultAudioEndpoint(eRender, eConsole) else {
            return;
        };
        let Ok(client) = device.Activate::<IAudioClient>(CLSCTX_ALL, None) else {
            return;
        };
        let Ok(format_ptr) = client.GetMixFormat() else {
            return;
        };

        let format: WAVEFORMATEX = *format_ptr;
        let sample_rate = format.nSamplesPerSec as f32;
        let channels = format.nChannels.max(1) as usize;
        let bits = format.wBitsPerSample as usize;

        // 200 ms buffer, tanpa periodicity (mode loopback mengabaikannya).
        if client
            .Initialize(
                AUDCLNT_SHAREMODE_SHARED,
                AUDCLNT_STREAMFLAGS_LOOPBACK,
                2_000_000,
                0,
                format_ptr,
                None,
            )
            .is_err()
        {
            return;
        }

        let Ok(capture) = client.GetService::<IAudioCaptureClient>() else {
            return;
        };
        if client.Start().is_err() {
            return;
        }

        let mut low = OnePole::new(150.0, sample_rate);
        let mut mid_low = OnePole::new(500.0, sample_rate);
        let mut mid_high = OnePole::new(2000.0, sample_rate);

        let mut halus = [0.0f32; 4];

        loop {
            // Tidak ada yang menampilkan equalizer: tidur panjang. Buffer
            // loopback menumpuk, jadi saat bangun isinya dibuang dulu —
            // lebih baik sepi sekejap daripada meledak dengan data lama.
            if !active.load(Ordering::Relaxed) {
                std::thread::sleep(Duration::from_millis(1000));
                let _ = capture.GetNextPacketSize();
                continue;
            }

            std::thread::sleep(Duration::from_millis(28));

            let mut jumlah_paket = match capture.GetNextPacketSize() {
                Ok(n) => n,
                Err(_) => break,
            };

            let mut energi = [0.0f32; 4];
            let mut sampel = 0u32;

            while jumlah_paket > 0 {
                let mut data: *mut u8 = std::ptr::null_mut();
                let mut frames = 0u32;
                let mut flags = 0u32;
                let mut pos = 0u64;
                let mut qpc = 0u64;

                if capture
                    .GetBuffer(
                        &mut data,
                        &mut frames,
                        &mut flags,
                        Some(&mut pos as *mut u64),
                        Some(&mut qpc as *mut u64),
                    )
                    .is_err()
                {
                    break;
                }

                // AUDCLNT_BUFFERFLAGS_SILENT = 0x2
                if flags & 0x2 == 0 && !data.is_null() && frames > 0 {
                    let total = frames as usize * channels;

                    for i in 0..frames as usize {
                        let mut mono = 0.0f32;
                        for c in 0..channels {
                            let idx = i * channels + c;
                            if idx >= total {
                                break;
                            }
                            mono += if bits == 32 {
                                let p = data as *const f32;
                                *p.add(idx)
                            } else {
                                let p = data as *const i16;
                                *p.add(idx) as f32 / 32768.0
                            };
                        }
                        mono /= channels as f32;

                        let b0 = low.step(mono);
                        let b1_full = mid_low.step(mono);
                        let b2_full = mid_high.step(mono);

                        let bands = [b0, b1_full - b0, b2_full - b1_full, mono - b2_full];
                        for k in 0..4 {
                            energi[k] += bands[k] * bands[k];
                        }
                        sampel += 1;
                    }
                }

                let _ = capture.ReleaseBuffer(frames);
                jumlah_paket = match capture.GetNextPacketSize() {
                    Ok(n) => n,
                    Err(_) => break,
                };
            }

            let n = sampel.max(1) as f32;
            for k in 0..4 {
                // RMS, lalu diskalakan agar terlihat; akar kuadrat membuat
                // perbedaan keras-pelan lebih terasa.
                let rms = (energi[k] / n).sqrt();
                let nilai = (rms * 9.0).clamp(0.0, 1.0);

                // Naik cepat, turun pelan — seperti meteran audio sungguhan.
                let lama = halus[k];
                halus[k] = if nilai > lama {
                    lama + (nilai - lama) * 0.65
                } else {
                    lama + (nilai - lama) * 0.18
                };
            }

            let _ = tx.send(halus);
            ctx.request_repaint();
        }

        let _ = client.Stop();
    }
}
