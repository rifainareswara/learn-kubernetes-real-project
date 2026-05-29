-- =============================================================================
-- Migration: Create Tasks Table
-- =============================================================================
-- Tabel ini menyimpan task/todo milik user.
--
-- Konsep yang dipelajari:
--   - FOREIGN KEY (REFERENCES) untuk relasi antar tabel
--   - ON DELETE CASCADE: jika user dihapus, semua task-nya ikut terhapus
--   - CHECK constraint untuk membatasi nilai kolom
--   - Composite INDEX untuk query yang sering dipakai bersamaan
-- =============================================================================

CREATE TABLE IF NOT EXISTS tasks (
    id          UUID         PRIMARY KEY DEFAULT gen_random_uuid(),

    -- REFERENCES users(id) = Foreign Key ke tabel users
    -- ON DELETE CASCADE = otomatis hapus task jika user-nya dihapus
    user_id     UUID         NOT NULL REFERENCES users(id) ON DELETE CASCADE,

    title       VARCHAR(255) NOT NULL,
    description TEXT,

    -- CHECK constraint memastikan status hanya bisa salah satu dari 3 nilai ini
    status      VARCHAR(20)  NOT NULL DEFAULT 'pending'
        CHECK (status IN ('pending', 'in_progress', 'completed')),

    -- Priority juga dibatasi dengan CHECK
    priority    VARCHAR(10)  NOT NULL DEFAULT 'medium'
        CHECK (priority IN ('low', 'medium', 'high')),

    -- due_date opsional (boleh NULL)
    due_date    TIMESTAMPTZ,

    created_at  TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

-- Index untuk query yang sering dipakai
CREATE INDEX IF NOT EXISTS idx_tasks_user_id ON tasks(user_id);
CREATE INDEX IF NOT EXISTS idx_tasks_status ON tasks(status);

-- Composite index: untuk query "tampilkan semua task user X dengan status Y"
CREATE INDEX IF NOT EXISTS idx_tasks_user_status ON tasks(user_id, status);
