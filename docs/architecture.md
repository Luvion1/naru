# Arsitektur Inti Naru

Naru dibangun dengan filosofi modularitas, keamanan, dan kejelasan kode. Dokumen ini menjelaskan struktur internal proyek.

## 🏗️ Struktur Proyek

```text
src/
├── main.rs          # Titik masuk CLI & dispatcher perintah
├── cli/             # Antarmuka Pengguna
│   ├── parser.rs    # Definisi struktur perintah (Clap)
│   ├── interactive.rs # Wizard interaktif (Dialoguer)
│   └── mod.rs
└── core/            # Logika Bisnis (Backend)
    ├── models.rs    # Struktur data inti (Serde)
    ├── persistence.rs # Penanganan baca/tulis file & penggabungan data
    ├── validation.rs # Mesin validasi skema
    ├── security.rs   # Sanitasi input & pengecekan file
    ├── crypto.rs     # Implementasi AES-GCM
    ├── audit.rs      # Sistem pencatatan aktivitas
    ├── locking.rs    # Mekanisme file locking untuk konsistensi
    └── schema.rs     # Manipulasi metadata skema
```

## 🔄 Alur Data

1.  **Input**: `parser.rs` menangkap argumen dari pengguna.
2.  **Dispatch**: `main.rs` mengarahkan ke fungsi yang sesuai di `core/`.
3.  **Load**: `persistence.rs` memuat data dari `.naru/config.json`.
4.  **Validate**: `validation.rs` memastikan input baru mematuhi skema.
5.  **Secure**: Jika field adalah rahasia, `crypto.rs` mengenkripsi nilai menggunakan kunci dari environment.
6.  **Audit**: `audit.rs` mencatatkan aksi (dengan masking) ke `audit.log`.
7.  **Persist**: `persistence.rs` menyimpan kembali data ke disk dengan penguncian file eksklusif via `locking.rs`.

## 💾 Format Penyimpanan

Naru menggunakan JSON sebagai format penyimpanan internal utama karena kemudahannya dalam serialisasi struktur data Rust yang kompleks.
-   `config.json`: Menyimpan nilai aktual per environment.
-   `schema.json`: Menyimpan metadata dan aturan validasi.
-   `audit.log`: File append-only berisi baris JSON untuk setiap aksi.
