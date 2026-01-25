# Naru - Secure Configuration Manager

Naru adalah alat baris perintah (CLI) yang dirancang untuk manajemen konfigurasi aplikasi yang aman, terstruktur, dan sadar skema. Naru memastikan integritas data konfigurasi Anda lintas environment (development, staging, production) dengan enkripsi otomatis dan sistem audit yang ketat.

## 🚀 Fitur Utama

- **Enkripsi AES-GCM**: Mengamankan data sensitif secara otomatis menggunakan standar industri.
- **Validasi Skema**: Memastikan nilai konfigurasi sesuai dengan tipe data (string, integer, boolean) dan aturan (min/max).
- **Multi-Environment**: Manajemen terpisah untuk berbagai lingkungan pengembangan.
- **Sistem Audit**: Pencatatan setiap perubahan status (Set, Import, Env, Schema) dengan masking nilai rahasia.
- **Impor/Ekspor Fleksibel**: Mendukung format `.env`, `YAML`, dan `JSON`.
- **Interaktif Wizard**: Editor skema interaktif untuk memudahkan pemeliharaan aturan data.

## 📁 Dokumentasi Lengkap

Untuk memahami Naru lebih dalam, silakan baca panduan berikut:

1.  [**Panduan Penggunaan CLI**](./docs/panduan-cli.md) - Referensi lengkap perintah `naru`.
2.  [**Arsitektur Inti**](./docs/arsitektur-inti.md) - Penjelasan struktur internal dan desain sistem.
3.  [**Model Keamanan**](./docs/model-keamanan.md) - Detail enkripsi dan perlindungan data rahasia.
4.  [**Sistem Audit**](./docs/sistem-audit.md) - Cara Naru mencatat aktivitas dan menjaga privasi log.
5.  [**Skema & Validasi**](./docs/skema-validasi.md) - Panduan membuat aturan validasi data.

## 🛠️ Instalasi Cepat

Naru dibangun menggunakan Rust. Pastikan Anda memiliki toolchain Rust terbaru:

```bash
git clone <repository-url>
cd naru
cargo build --release
```

Binary akan tersedia di `target/release/naru`.

---
© 2026 Naru Project. Dibuat untuk keamanan dan kenyamanan DevOps.
