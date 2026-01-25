# Sistem Audit Naru

Sistem audit Naru memberikan jejak aktivitas transparan untuk setiap perubahan yang terjadi pada sistem manajemen konfigurasi Anda.

## 📝 Lokasi & Format Log
Log disimpan di `.naru/audit.log`. Setiap entri adalah objek JSON satu baris, sehingga memudahkan pengolahan dengan alat bantu seperti `jq` atau sistem agregasi log eksternal.

### Contoh Entri
```json
{"timestamp":"2026-01-25T07:06:00Z","action":"SET","environment":"development","key":"db_port","old_value":"5432","new_value":"5433","user":"dev_user"}
```

## 🔍 Aksi yang Dicatat
Naru mencatat hampir seluruh operasi modifikasi:
-   `SET`: Penambahan atau perubahan nilai tunggal.
-   `IMPORT`: Impor massal data dari file.
-   `ENV_ADD` / `ENV_REMOVE`: Manipulasi environment.
-   `SCHEMA_ADD` / `SCHEMA_EDIT` / `SCHEMA_REMOVE`: Perubahan pada skema.
-   `BACKUP_RESTORE`: Pemulihan data dari cadangan.

## 👤 Identifikasi Pengguna
Naru mencoba mengidentifikasi siapa yang melakukan aksi dengan mengambil variabel environment `USER` atau `USERNAME`. Jika tidak tersedia, identitas akan dicatat sebagai `root` (dalam lingkungan terbatas) atau `null`.

## 🛡️ Kebijakan Masking
Privasi adalah prioritas. Jika sebuah field ditandai sebagai rahasia, Naru akan melakukan masking pada kolom `old_value` dan `new_value`. 

| Skenario | Pencatatan Nilai |
| :--- | :--- |
| Field Biasa | Nilai mentah (plaintext) |
| Field Rahasia | `********` |
| Password/Key | `********` |

Mekanisme ini memastikan bahwa meskipun file `audit.log` jatuh ke tangan yang salah, penyerang tidak akan mendapatkan kunci akses atau kredensial database Anda.
