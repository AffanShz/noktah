//! Jembatan ke SMTC (Global System Media Transport Controls).
//!
//! Berjalan di thread sendiri karena panggilan WinRT bersifat blocking.
//! Hasilnya dikirim lewat channel; UI tidak pernah menunggu.

use noktah_core::model::MediaState;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::Duration;

#[cfg(windows)]
use windows::Media::Control::GlobalSystemMediaTransportControlsSessionManager;

pub enum Cmd {
    PlayPause,
    Next,
    Prev,
    Seek(i64),
}

pub struct Update {
    pub state: MediaState,
    pub art: Option<Vec<u8>>,
    /// Penanda lagu, supaya artwork hanya diambil sekali per lagu.
    pub track_id: String,
}

pub struct MusicHandle {
    pub rx: Receiver<Update>,
    pub tx: Sender<Cmd>,
}

pub fn spawn(ctx: egui::Context) -> MusicHandle {
    let (tx_update, rx_update) = channel();
    let (tx_cmd, rx_cmd) = channel();

    let _ = std::thread::Builder::new()
        .name("noktah-smtc".into())
        .stack_size(256 * 1024)
        .spawn(move || {
            #[cfg(windows)]
            run(ctx, tx_update, rx_cmd);

            #[cfg(not(windows))]
            {
                let _ = (ctx, tx_update, rx_cmd);
            }
        });

    MusicHandle {
        rx: rx_update,
        tx: tx_cmd,
    }
}

#[cfg(windows)]
fn run(ctx: egui::Context, tx: Sender<Update>, rx: Receiver<Cmd>) {
    use windows::Media::Control::GlobalSystemMediaTransportControlsSessionManager as Manager;
    use windows::Win32::System::Com::{CoInitializeEx, COINIT_MULTITHREADED};

    unsafe {
        let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
    }

    let mut last_track = String::new();
    let mut last_art_for = String::new();
    // Membuat manager SMTC setiap 250 ms itu mahal: tiap panggilan melakukan
    // aktivasi WinRT penuh. Dibuat sekali, dipakai ulang.
    let mut manager = unsafe { Manager::RequestAsync().ok().and_then(|op| op.get().ok()) };
    let mut gagal = 0u32;

    loop {
        std::thread::sleep(Duration::from_millis(500));

        // Perintah dari UI dikerjakan lebih dulu supaya terasa responsif.
        while let Ok(cmd) = rx.try_recv() {
            // Perintah dan pembacaan memakai manager yang sama supaya objek
            // WinRT di bawahnya konsisten. Manager dibuat ulang otomatis saat
            // gagal 20x (lihat bawah).
            unsafe { apply_cmd(&manager, cmd) };
            ctx.request_repaint();
        }

        let state_opt = unsafe { read_state(manager.as_ref()) };
        if state_opt.is_none() {
            gagal += 1;
            if gagal >= 20 {
                gagal = 0;
                manager = unsafe { Manager::RequestAsync().ok().and_then(|op| op.get().ok()) };
            }
        } else {
            gagal = 0;
        }

        let Some(state) = state_opt else {
            let kosong = MediaState::default();
            let _ = tx.send(Update {
                state: kosong,
                art: None,
                track_id: String::new(),
            });
            last_track.clear();
            ctx.request_repaint();
            continue;
        };

        let track_id = format!("{}|{}", state.title, state.artist);
        let art = if state.active && track_id != last_art_for {
            last_art_for = track_id.clone();
            unsafe { read_art(manager.as_ref()) }
        } else {
            None
        };

        let berubah = track_id != last_track;
        last_track = track_id.clone();

        let _ = tx.send(Update {
            state,
            art,
            track_id,
        });

        // Selalu minta repaint: posisi lagu bergerak terus.
        if berubah || true {
            ctx.request_repaint();
        }
    }
}

#[cfg(windows)]
unsafe fn apply_cmd(
    manager: &Option<GlobalSystemMediaTransportControlsSessionManager>,
    cmd: Cmd,
) {
    let Some(manager) = manager else { return };
    let Some(session) = manager.GetCurrentSession().ok() else {
        return;
    };

    match cmd {
        Cmd::PlayPause => {
            let _ = session.TryTogglePlayPauseAsync().and_then(|op| op.get());
        }
        Cmd::Next => {
            let _ = session.TrySkipNextAsync().and_then(|op| op.get());
        }
        Cmd::Prev => {
            let _ = session.TrySkipPreviousAsync().and_then(|op| op.get());
        }
        Cmd::Seek(ms) => {
            // `TryChangePlaybackPositionAsync` adalah operasi WinRT async:
            // tanpa `.get()` hasilnya di-queue dan bisa gugur sebelum sempat
            // berjalan. Menunggu hasil di sini membuat seek benar-benar
            // dieksekusi, dan memberi waktu ekstra bagi SMTC agar posisinya
            // berubah sebelum pembacaan berikutnya.
            let _ = session
                .TryChangePlaybackPositionAsync(ms * 10_000)
                .and_then(|op| op.get());
        }
    }
}

/// Waktu sekarang dalam satuan 100 ns sejak 1601, sama seperti `DateTime`.
#[cfg(windows)]
fn unix_now_ticks() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    const EPOCH_DIFF: i64 = 116_444_736_000_000_000;
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    EPOCH_DIFF + (now.as_nanos() / 100) as i64
}

#[cfg(windows)]
unsafe fn read_state(
    manager: Option<&windows::Media::Control::GlobalSystemMediaTransportControlsSessionManager>,
) -> Option<MediaState> {
    use windows::Media::Control::GlobalSystemMediaTransportControlsSessionPlaybackStatus as Status;

    let manager = manager?;
    let session = manager.GetCurrentSession().ok()?;

    let props = session.TryGetMediaPropertiesAsync().ok()?.get().ok()?;
    let title = props.Title().map(|s| s.to_string_lossy()).unwrap_or_default();
    let artist = props
        .Artist()
        .map(|s| s.to_string_lossy())
        .unwrap_or_default();
    let album = props
        .AlbumTitle()
        .map(|s| s.to_string_lossy())
        .unwrap_or_default();

    let playing = session
        .GetPlaybackInfo()
        .and_then(|info| info.PlaybackStatus())
        .map(|s| s == Status::Playing)
        .unwrap_or(false);

    // Posisi dari SMTC adalah nilai saat `LastUpdatedTime`, bukan saat ini.
    // Tanpa ekstrapolasi, lirik selalu tertinggal sejauh umur pembacaan itu.
    let (position_ms, duration_ms) = session
        .GetTimelineProperties()
        .map(|t| {
            let pos = t.Position().map(|v| v.Duration).unwrap_or(0);
            let end = t.EndTime().map(|v| v.Duration).unwrap_or(0);
            let updated = t.LastUpdatedTime().map(|d| d.UniversalTime).unwrap_or(0);

            let elapsed = if playing && updated > 0 {
                (unix_now_ticks() - updated).max(0)
            } else {
                0
            };

            ((pos + elapsed) / 10_000, end / 10_000)
        })
        .unwrap_or((0, 0));

    let source = session
        .SourceAppUserModelId()
        .map(|s| s.to_string_lossy())
        .unwrap_or_default();

    Some(MediaState {
        active: !title.is_empty(),
        title,
        artist,
        album,
        source,
        playing,
        position_ms,
        duration_ms,
    })
}

#[cfg(windows)]
unsafe fn read_art(
    manager: Option<&windows::Media::Control::GlobalSystemMediaTransportControlsSessionManager>,
) -> Option<Vec<u8>> {
    use windows::Storage::Streams::DataReader;

    let manager = manager?;
    let session = manager.GetCurrentSession().ok()?;
    let props = session.TryGetMediaPropertiesAsync().ok()?.get().ok()?;
    let thumb = props.Thumbnail().ok()?;

    let stream = thumb.OpenReadAsync().ok()?.get().ok()?;
    let size = stream.Size().ok()? as u32;
    if size == 0 || size > 8 * 1024 * 1024 {
        return None;
    }

    let reader = DataReader::CreateDataReader(&stream).ok()?;
    reader.LoadAsync(size).ok()?.get().ok()?;

    let mut buffer = vec![0u8; size as usize];
    reader.ReadBytes(&mut buffer).ok()?;
    Some(buffer)
}
