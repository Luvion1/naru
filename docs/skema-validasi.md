# Skema & Validasi Naru

Naru memastikan aplikasi Anda tidak rusak karena kesalahan tipe data konfigurasi melalui sistem skema yang kuat.

## 📋 Tipe Data yang Didukung
1.  **String**: Teks biasa.
2.  **Integer**: Angka bilangan bulat (64-bit).
3.  **Boolean**: Nilai `true` atau `false`.

## 📏 Aturan Validasi
Setiap field dalam skema dapat memiliki aturan tambahan:
-   **String**: `min_length` dan `max_length`.
-   **Integer**: `min_value` dan `max_value`.

## 🛠️ Cara Membuat Skema

### Melalui CLI (Interaktif)
Kami merekomendasikan penggunaan wizard interaktif untuk menghindari kesalahan penulisan JSON:
```bash
naru schema add
```
Wizard akan menuntun Anda memasukkan nama kunci, tipe data, deskripsi, status rahasia, dan aturan validasi.

### Melalui File JSON
Anda juga bisa mengedit `.naru/schema.json` secara langsung:
```json
{
  "key": "app_port",
  "type": "integer",
  "description": "Port utama aplikasi",
  "validation": {
    "min_value": 1024,
    "max_value": 65535
  },
  "is_secret": false
}
```

## 🔄 Validasi Saat Impor
Saat Anda menjalankan `naru import`, Naru tidak hanya sekadar menyalin data. Naru memuat skema dan memeriksa setiap baris dalam file impor. Jika ada satu nilai saja yang tidak valid (misalnya memasukkan teks ke field `integer`), maka seluruh proses impor akan dibatalkan untuk menjaga integritas data.
