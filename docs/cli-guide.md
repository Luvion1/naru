# Panduan Penggunaan CLI Naru

Dokumen ini berisi referensi lengkap untuk perintah-perintah yang tersedia di Naru CLI.

## 🛠️ Inisialisasi

### `naru init`
Membuat direktori `.naru/` dan file dasar (`config.json`, `schema.json`) di direktori saat ini.

## 📝 Manajemen Konfigurasi

### `naru set <KEY>=<VALUE> [--env <ENV>] [--secret]`
Menyimpan nilai konfigurasi.
- `--env`: Menentukan environment (default: `development`).
- `--secret`: Menandai nilai sebagai rahasia (akan dienkripsi).

### `naru get <KEY> [--env <ENV>]`
Mengambil dan menampilkan nilai konfigurasi. Nilai terenkripsi akan didekripsi secara otomatis jika kunci tersedia.

### `naru list [--env <ENV>]`
Menampilkan semua konfigurasi dalam satu environment beserta tipe datanya.

## 📥 Impor & Ekspor

### `naru import <FILE_PATH> [--env <ENV>]`
Mengimpor data dari file eksternal. Mendukung `.env`, `.yaml`, `.yml`, dan `.json`.
*Catatan: Naru akan secara otomatis memvalidasi data terhadap skema saat impor.*

### `naru export <FILE_PATH> --format <FORMAT> [--env <ENV>]`
Mengekspor data ke file eksternal. Format yang didukung: `env`, `yaml`.

## 📐 Manajemen Skema

### `naru schema add [KEY] [--type <TYPE>] [--description <DESC>] [--secret]`
Menambahkan field ke skema. Jika `KEY` tidak diberikan, akan menjalankan wizard interaktif.

### `naru schema edit [KEY]`
Mengubah properti field yang sudah ada melalui prompt interaktif.

### `naru schema remove [KEY]`
Menghapus field dari skema.

### `naru schema view`
Menampilkan aturan skema saat ini.

## 🔒 Keamanan & Utilitas

### `naru validate`
Memeriksa apakah data di `config.json` masih mematuhi aturan di `schema.json`, termasuk pengecekan status enkripsi.

### `naru audit [--count <N>]`
Menampilkan `N` baris terakhir dari log audit.

### `naru backup create <FILE_PATH>` / `naru backup restore <FILE_PATH>`
Membuat atau memulihkan cadangan lengkap (data + skema).

### `naru convert <FROM_FILE> <TO_FILE> --from-format <F> --to-format <T>`
Mengonversi file konfigurasi antar format tanpa menyimpannya ke sistem Naru.

---
**Penting**: Sebagian besar operasi yang melibatkan data rahasia memerlukan variabel environment `NARU_ENCRYPTION_KEY` (32 karakter).
