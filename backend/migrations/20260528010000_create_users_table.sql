-- =============================================================================
-- Migration: Create Users Table
-- =============================================================================
-- Tabel ini menyimpan data user yang terdaftar di aplikasi.
--
-- Konsep yang dipelajari:
--   - UUID sebagai primary key (lebih aman dari auto-increment integer)
--   - UNIQUE constraint untuk mencegah duplikasi
--   - TIMESTAMPTZ untuk menyimpan waktu dengan timezone
--   - INDEX untuk mempercepat query pencarian
-- =============================================================================

CREATE TABLE IF NOT EXISTS users (
    -- gen_random_uuid() adalah fungsi PostgreSQL untuk generate UUID v4
    id            UUID         PRIMARY KEY DEFAULT gen_random_uuid(),

    -- UNIQUE memastikan tidak ada 2 user dengan username/email yang sama
    username      VARCHAR(50)  UNIQUE NOT NULL,
    email         VARCHAR(255) UNIQUE NOT NULL,

    -- Password TIDAK disimpan langsung, tapi dalam bentuk hash (Argon2)
    password_hash VARCHAR(255) NOT NULL,

    -- TIMESTAMPTZ = timestamp with time zone (selalu dalam UTC di PostgreSQL)
    -- DEFAULT NOW() otomatis mengisi waktu saat row dibuat
    created_at    TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at    TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

-- INDEX mempercepat query WHERE email = '...' (dipakai saat login)
CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);
CREATE INDEX IF NOT EXISTS idx_users_username ON users(username);
