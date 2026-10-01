-- 0003_message_history.sql
-- Message history table for edit/delete audit trail.

-- ── Custom Types ──────────────────────────────────────────────────────
DO $$ BEGIN
    CREATE TYPE message_history_type AS ENUM ('edit', 'delete');
EXCEPTION WHEN duplicate_object THEN NULL;
END $$;

-- ── Message History Table ──────────────────────────────────────────────
CREATE TABLE message_history (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v7(),
    message_id UUID NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    history_type message_history_type NOT NULL,
    previous_content TEXT,
    new_content TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_message_history_message_id ON message_history(message_id);
CREATE INDEX idx_message_history_user_id ON message_history(user_id);
CREATE INDEX idx_message_history_created_at ON message_history(created_at);

-- ── RLS Policies ───────────────────────────────────────────────────────
ALTER TABLE message_history ENABLE ROW LEVEL SECURITY;

CREATE POLICY message_history_org_isolation ON message_history
    USING (
        message_id IN (
            SELECT id FROM messages WHERE org_id = current_setting('app.current_org_id')::UUID
        )
    );