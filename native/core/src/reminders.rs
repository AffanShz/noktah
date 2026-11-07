//! Penjadwalan pengingat tugas.
//!
//! Aturannya ditulis di satu tempat: pengingat hanya untuk tugas yang belum
//! selesai, punya waktu, dan belum pernah berbunyi.

use crate::model::Todo;

#[derive(Clone, Debug, PartialEq)]
pub struct DueReminder {
    pub task: Todo,
    /// Milidetik sampai waktunya tiba; negatif berarti sudah lewat.
    pub in_ms: i64,
}

/// Tugas terdekat yang masih menunggu pengingat.
pub fn next_due(todos: &[Todo], now_ms: i64) -> Option<DueReminder> {
    let mut best: Option<DueReminder> = None;

    for task in todos {
        if task.done || task.notified_at.is_some() {
            continue;
        }
        let Some(due) = task.due_at else { continue };
        let in_ms = due - now_ms;
        if best.as_ref().map_or(true, |b| in_ms < b.in_ms) {
            best = Some(DueReminder {
                task: task.clone(),
                in_ms,
            });
        }
    }

    best
}

/// Pengingat yang waktunya sudah lewat tapi belum berbunyi — misalnya karena
/// aplikasi baru dinyalakan.
pub fn missed_reminders(todos: &[Todo], now_ms: i64) -> Vec<Todo> {
    let mut out: Vec<Todo> = todos
        .iter()
        .filter(|t| !t.done && t.notified_at.is_none())
        .filter(|t| t.due_at.map_or(false, |due| due <= now_ms))
        .cloned()
        .collect();
    out.sort_by_key(|t| t.due_at.unwrap_or(i64::MAX));
    out
}

pub fn is_overdue(task: &Todo, now_ms: i64) -> bool {
    !task.done && task.due_at.map_or(false, |due| due < now_ms)
}

/// `HH:MM` → epoch ms untuk hari ini; kalau sudah lewat, pakai besok.
pub fn time_to_due_at(value: &str, now_ms: i64) -> Option<i64> {
    use chrono::{Local, NaiveTime, TimeZone, Timelike};

    let value = value.trim();
    let (jam_raw, menit_raw) = value.split_once(':')?;
    let jam: u32 = jam_raw.parse().ok()?;
    let menit: u32 = menit_raw.parse().ok()?;
    if jam > 23 || menit > 59 {
        return None;
    }

    let now = Local.timestamp_millis_opt(now_ms).single()?;
    let target = NaiveTime::from_hms_opt(jam, menit, 0)?;
    let hari_ini = now.date_naive().and_time(target);
    let mut due = Local.from_local_datetime(&hari_ini).single()?.timestamp_millis();

    if due <= now_ms {
        let besok = now.date_naive().succ_opt()?.and_time(target);
        due = Local.from_local_datetime(&besok).single()?.timestamp_millis();
    }

    let _ = now.hour();
    Some(due)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tugas(id: &str, due: Option<i64>, done: bool, notified: Option<i64>) -> Todo {
        let mut t = Todo::new(id, "x", 0);
        t.due_at = due;
        t.done = done;
        t.notified_at = notified;
        t
    }

    #[test]
    fn memilih_tugas_terdekat() {
        let todos = vec![
            tugas("a", Some(5_000), false, None),
            tugas("b", Some(2_000), false, None),
            tugas("c", Some(9_000), false, None),
        ];
        let next = next_due(&todos, 0).unwrap();
        assert_eq!(next.task.id, "b");
        assert_eq!(next.in_ms, 2_000);
    }

    #[test]
    fn mengabaikan_selesai_tanpa_waktu_dan_sudah_diingatkan() {
        let todos = vec![
            tugas("selesai", Some(1_000), true, None),
            tugas("tanpa-waktu", None, false, None),
            tugas("sudah", Some(1_000), false, Some(500)),
            tugas("layak", Some(3_000), false, None),
        ];
        assert_eq!(next_due(&todos, 0).unwrap().task.id, "layak");
    }

    #[test]
    fn tanpa_tugas_layak_hasilnya_kosong() {
        let todos = vec![tugas("a", None, false, None)];
        assert!(next_due(&todos, 0).is_none());
    }

    #[test]
    fn in_ms_negatif_untuk_yang_terlewat() {
        let todos = vec![tugas("a", Some(1_000), false, None)];
        assert_eq!(next_due(&todos, 5_000).unwrap().in_ms, -4_000);
    }

    #[test]
    fn pengingat_terlewat_diurutkan() {
        let todos = vec![
            tugas("b", Some(3_000), false, None),
            tugas("a", Some(1_000), false, None),
            tugas("c", Some(9_000), false, None),
        ];
        let missed = missed_reminders(&todos, 5_000);
        assert_eq!(missed.len(), 2);
        assert_eq!(missed[0].id, "a");
        assert_eq!(missed[1].id, "b");
    }

    #[test]
    fn terlewat_mengabaikan_yang_selesai_dan_sudah_diingatkan() {
        let todos = vec![
            tugas("selesai", Some(1_000), true, None),
            tugas("sudah", Some(1_000), false, Some(1_000)),
        ];
        assert!(missed_reminders(&todos, 5_000).is_empty());
    }

    #[test]
    fn overdue_hanya_untuk_yang_belum_selesai() {
        assert!(is_overdue(&tugas("a", Some(1_000), false, None), 5_000));
        assert!(!is_overdue(&tugas("a", Some(1_000), true, None), 5_000));
        assert!(!is_overdue(&tugas("a", Some(9_000), false, None), 5_000));
        assert!(!is_overdue(&tugas("a", None, false, None), 5_000));
    }

    #[test]
    fn waktu_tidak_valid_ditolak() {
        let now = 1_700_000_000_000i64;
        assert!(time_to_due_at("25:00", now).is_none());
        assert!(time_to_due_at("12:60", now).is_none());
        assert!(time_to_due_at("abc", now).is_none());
        assert!(time_to_due_at("", now).is_none());
    }

    #[test]
    fn waktu_valid_menghasilkan_masa_depan() {
        let now = 1_700_000_000_000i64;
        let due = time_to_due_at("07:30", now).unwrap();
        assert!(due > now);
        // paling jauh 24 jam ke depan
        assert!(due - now <= 86_400_000);
    }
}
