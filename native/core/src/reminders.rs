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

/// Apakah tugas memiliki pengingat terjadwal yang masih aktif (belum selesai,
/// punya tenggat waktu, dan belum pernah berbunyi).
pub fn has_pending_reminder(task: &Todo) -> bool {
    !task.done && task.notified_at.is_none() && task.due_at.is_some()
}

/// Jumlah tugas dengan pengingat terjadwal yang belum selesai dan belum berbunyi.
pub fn pending_reminder_count(todos: &[Todo]) -> usize {
    todos.iter().filter(|t| has_pending_reminder(t)).count()
}

/// Tugas terdekat yang masih menunggu pengingat.
pub fn next_due(todos: &[Todo], now_ms: i64) -> Option<DueReminder> {
    let mut best: Option<DueReminder> = None;

    for task in todos {
        if !has_pending_reminder(task) {
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
        .filter(|t| has_pending_reminder(t))
        .filter(|t| t.due_at.is_some_and(|due| due <= now_ms))
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

/// Memindai token `@HH:MM` dari string input tugas dan mengekstrak epoch ms untuk pengingat.
///
/// Jika ditemukan token yang valid dan teks tugas yang tersisa tidak kosong, mengembalikan
/// `(teks_bersih, Some(due_ms))`. Jika tidak ada atau hanya berisi token `@HH:MM`, mengembalikan
/// `(raw.trim().to_string(), None)`.
pub fn extract_inline_due(raw: &str, now_ms: i64) -> (String, Option<i64>) {
    let tokens: Vec<&str> = raw.split_whitespace().collect();
    if let Some(pos) = tokens.iter().position(|tok| {
        if tok.starts_with('@') && tok.len() >= 4 {
            time_to_due_at(&tok[1..], now_ms).is_some()
        } else {
            false
        }
    }) {
        let due_ms = time_to_due_at(&tokens[pos][1..], now_ms);
        let mut remaining_tokens = tokens.clone();
        remaining_tokens.remove(pos);
        let cleaned = remaining_tokens.join(" ");
        let trimmed = cleaned.trim().to_string();
        if !trimmed.is_empty() {
            return (trimmed, due_ms);
        }
    }
    (raw.trim().to_string(), None)
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

    #[test]
    fn test_extract_inline_due() {
        let now = 1_700_000_000_000i64;

        // Normal case with @HH:MM
        let (task, due) = extract_inline_due("Standup @09:30", now);
        assert_eq!(task, "Standup");
        assert!(due.is_some());
        assert!(due.unwrap() > now);

        // Preceding token
        let (task, due) = extract_inline_due("@14:00 Meeting with team", now);
        assert_eq!(task, "Meeting with team");
        assert!(due.is_some());

        // Middle token
        let (task, due) = extract_inline_due("Check code @16:45 before release", now);
        assert_eq!(task, "Check code before release");
        assert!(due.is_some());

        // @HH:MM alone without remaining text stays plain text
        let (task, due) = extract_inline_due("@09:30", now);
        assert_eq!(task, "@09:30");
        assert!(due.is_none());

        // Invalid time format or email ignored
        let (task, due) = extract_inline_due("contact email@bad:99 now", now);
        assert_eq!(task, "contact email@bad:99 now");
        assert!(due.is_none());

        let (task, due) = extract_inline_due("Task @25:00 invalid", now);
        assert_eq!(task, "Task @25:00 invalid");
        assert!(due.is_none());
    }

    #[test]
    fn pending_reminder_count_mengecualikan_selesai_dan_sudah_berbunyi() {
        let todos = vec![
            tugas("aktif-pending", Some(10_000), false, None),
            tugas("aktif-terlewat-belum-bunyi", Some(500), false, None),
            tugas("aktif-sudah-berbunyi", Some(10_000), false, Some(1_000)),
            tugas("selesai-punya-due", Some(10_000), true, None),
            tugas("selesai-sudah-berbunyi", Some(10_000), true, Some(1_000)),
            tugas("aktif-tanpa-due", None, false, None),
        ];

        assert_eq!(pending_reminder_count(&todos), 2);
        assert!(has_pending_reminder(&todos[0]));
        assert!(has_pending_reminder(&todos[1]));
        assert!(!has_pending_reminder(&todos[2]));
        assert!(!has_pending_reminder(&todos[3]));
        assert!(!has_pending_reminder(&todos[4]));
        assert!(!has_pending_reminder(&todos[5]));
    }
}
