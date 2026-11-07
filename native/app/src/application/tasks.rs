//! Use case daftar tugas.
//!
//! Semua aturan di sini bisa diuji tanpa membuka jendela: menambah,
//! mencentang, menghapus, dan meringkas. Presentasi hanya memanggil method
//! ini dan menggambar hasilnya.

use super::ports::{files, Store};
use noktah_core::model::Todo;
use noktah_core::reminders;

/// Daftar tugas beserta aturan mainnya.
///
/// Menyimpan sendiri daftarnya dan tahu cara memuat/menyimpannya lewat
/// [`Store`]. Presentasi tidak menyentuh `Vec<Todo>` secara langsung untuk
/// operasi yang punya aturan — supaya tidak ada dua tempat yang bisa
/// melanggar aturan yang sama.
pub struct TaskList<S: Store> {
    items: Vec<Todo>,
    store: S,
}

impl<S: Store> TaskList<S> {
    pub fn load(store: S) -> Self {
        let items: Vec<Todo> = store.load(files::TASKS);
        Self { items, store }
    }

    pub fn items(&self) -> &[Todo] {
        &self.items
    }

    pub fn open_count(&self) -> usize {
        self.items.iter().filter(|t| !t.done).count()
    }

    pub fn done_count(&self) -> usize {
        self.items.iter().filter(|t| t.done).count()
    }

    /// Tambah tugas di urutan teratas.
    ///
    /// Mengembalikan `false` kalau teksnya kosong — pemanggil yang memutuskan
    /// cara menampilkannya (banner, getar, diam), bukan use case ini.
    pub fn add(&mut self, text: &str, due_hint: &str, now_ms: i64) -> bool {
        let text = text.trim();
        if text.is_empty() {
            return false;
        }

        let mut task = Todo::new(format!("t_{now_ms:x}"), text, now_ms);
        let hint = due_hint.trim();
        if !hint.is_empty() {
            task.due_at = reminders::time_to_due_at(hint, now_ms);
        }

        self.items.insert(0, task);
        self.persist();
        true
    }

    pub fn toggle(&mut self, id: &str) {
        if let Some(t) = self.items.iter_mut().find(|t| t.id == id) {
            t.done = !t.done;
            self.persist();
        }
    }

    pub fn remove(&mut self, id: &str) {
        let sebelum = self.items.len();
        self.items.retain(|t| t.id != id);
        if self.items.len() != sebelum {
            self.persist();
        }
    }

    /// Buang semua yang sudah selesai. Mengembalikan berapa yang terbuang.
    pub fn clear_done(&mut self) -> usize {
        let sebelum = self.items.len();
        self.items.retain(|t| !t.done);
        let jumlah = sebelum - self.items.len();
        if jumlah > 0 {
            self.persist();
        }
        jumlah
    }

    /// Tandai tugas bertenggat yang belum diberi tahu, dan kembalikan teks
    /// notifikasinya. Dipanggil dari loop utama tiap beberapa detik.
    pub fn take_due_notifications(&mut self, now_ms: i64) -> Vec<String> {
        let mut pesan = Vec::new();
        let mut berubah = false;

        for t in self.items.iter_mut() {
            if t.done || t.notified_at.is_some() {
                continue;
            }
            if let Some(due) = t.due_at {
                if due <= now_ms {
                    t.notified_at = Some(now_ms);
                    pesan.push(t.text.clone());
                    berubah = true;
                }
            }
        }

        if berubah {
            self.persist();
        }
        pesan
    }

    /// Tugas berikutnya yang paling dekat tenggatnya, untuk ditampilkan di pill.
    pub fn next_due(&self, now_ms: i64) -> Option<&Todo> {
        self.items
            .iter()
            .filter(|t| !t.done)
            .min_by_key(|t| t.due_at.unwrap_or(i64::MAX))
            .filter(|t| t.due_at.is_some_and(|d| d >= now_ms) || t.due_at.is_none())
    }

    /// Pindahkan daftar dari penyimpanan lain (mis. setelah impor setelan).
    pub fn replace(&mut self, items: Vec<Todo>) {
        self.items = items;
        self.persist();
    }

    fn persist(&self) {
        self.store.save(files::TASKS, &self.items);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::fake::FakeStore;

    fn daftar() -> TaskList<FakeStore> {
        TaskList::load(FakeStore::default())
    }

    #[test]
    fn tambah_menolak_teks_kosong() {
        let mut t = daftar();
        assert!(!t.add("   ", "", 1));
        assert!(t.items().is_empty());
    }

    #[test]
    fn tambah_menaruh_di_urutan_teratas() {
        let mut t = daftar();
        t.add("pertama", "", 1);
        t.add("kedua", "", 2);
        assert_eq!(t.items()[0].text, "kedua");
    }

    #[test]
    fn centang_bolak_balik_mengubah_hitungan() {
        let mut t = daftar();
        t.add("a", "", 1);
        let id = t.items()[0].id.clone();
        assert_eq!(t.open_count(), 1);

        t.toggle(&id);
        assert_eq!(t.open_count(), 0);
        assert_eq!(t.done_count(), 1);

        t.toggle(&id);
        assert_eq!(t.open_count(), 1);
    }

    #[test]
    fn tambah_dengan_jam_mengisi_tenggat() {
        let mut t = daftar();
        let now = 1_700_000_000_000;
        t.add("minum obat", "14:30", now);
        assert!(t.items()[0].due_at.is_some());
    }

    #[test]
    fn tenggat_lewat_hanya_dinotifikasi_sekali() {
        let mut t = daftar();
        let now = 1_700_000_000_000;
        t.add("a", "00:01", now);
        // beri tenggat yang pasti sudah lewat
        t.items[0].due_at = Some(now - 60_000);

        assert_eq!(t.take_due_notifications(now).len(), 1);
        assert_eq!(t.take_due_notifications(now).len(), 0);
    }

    #[test]
    fn hapus_mengembalikan_false_kalau_id_tidak_ada() {
        let mut t = daftar();
        t.add("a", "", 1);
        let id = t.items()[0].id.clone();
        t.remove(&id);
        assert!(t.items().is_empty());
        // memanggil lagi tidak boleh panik
        t.remove(&id);
    }

    #[test]
    fn bersihkan_selesai_hanya_membuang_yang_selesai() {
        let mut t = daftar();
        t.add("a", "", 1);
        t.add("b", "", 2);
        let id = t.items()[0].id.clone();
        t.toggle(&id);

        assert_eq!(t.clear_done(), 1);
        assert_eq!(t.items().len(), 1);
        assert!(!t.items()[0].done);
    }

    #[test]
    fn tersimpan_dan_dimuat_ulang() {
        let store = FakeStore::default();
        {
            let mut t = TaskList::load(store.clone());
            t.add("bertahan", "", 1);
        }
        let lagi = TaskList::load(store);
        assert_eq!(lagi.items().len(), 1);
        assert_eq!(lagi.items()[0].text, "bertahan");
    }
}
