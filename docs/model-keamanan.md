# Model Keamanan Naru

Keamanan adalah pilar utama Naru. Kami menerapkan perlindungan berlapis untuk memastikan data konfigurasi Anda tetap rahasia dan utuh.

## 🔑 Enkripsi Data

Naru menggunakan algoritma **AES-256-GCM** (Advanced Encryption Standard dengan Galois/Counter Mode). Algoritma ini dipilih karena:
1.  **Kerahasiaan**: Mengenkripsi data sehingga tidak dapat dibaca tanpa kunci.
2.  **Integritas**: Mendeteksi jika data telah dimodifikasi secara ilegal.

### Kunci Enkripsi
Kunci diambil dari variabel environment `NARU_ENCRYPTION_KEY`.
-   **Panjang**: Harus tepat 32 byte (untuk AES-256).
-   **Penyimpanan**: Naru **TIDAK PERNAH** menyimpan kunci ini di disk. Kunci harus disediakan setiap kali aplikasi dijalankan untuk operasi yang melibatkan rahasia.

## 🛡️ Sanitasi & Proteksi

### Directory Traversal
Semua path file yang diberikan melalui CLI (misalnya saat `import` atau `export`) divalidasi di `src/core/security.rs` untuk mencegah akses ke file di luar direktori yang diizinkan (seperti `/etc/passwd`).

### Sanitasi Nilai
Nilai string dibersihkan dari karakter kontrol yang berbahaya (seperti null bytes) untuk mencegah eksploitasi pada aplikasi hilir yang menggunakan konfigurasi tersebut.

### Penguncian File (File Locking)
Untuk mencegah korupsi data saat beberapa proses Naru berjalan bersamaan, Naru menggunakan penguncian file eksklusif saat menulis ke `config.json` atau `schema.json`.

## 🎭 Masking Audit
Naru secara otomatis mengenali jika sebuah field adalah rahasia berdasarkan:
1.  Flag `--secret` saat menjalankan `set`.
2.  Definisi `is_secret: true` di dalam `schema.json`.

Jika salah satu kondisi di atas terpenuhi, nilai asli **TIDAK AKAN PERNAH** muncul di `audit.log`. Sebagai gantinya, string `********` akan dicatat.
