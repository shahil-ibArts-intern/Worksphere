-- 0002_dm_tables.sql
-- Direct message tables for 1:1 and group DMs.

-- ── Custom Types ──────────────────────────────────────────────────────
DO $$ BEGIN
    CREATE TYPE dm_conversation_type AS ENUM ('direct', 'group');
EXCEPTION WHEN duplicate_object THEN NULL;
END $$;

-- ── DM Conversations Table ─────────────────────────────────────────────
CREATE TABLE dm_conversations (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v7(),
    org_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    name VARCHAR(255),
    is_group BOOLEAN NOT NULL DEFAULT FALSE,
    created_by UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE INDEX idx_dm_conversations_org_id ON dm_conversations(org_id) WHERE deleted_at IS NULL;
CREATE INDEX idx_dm_conversations_created_by ON dm_conversations(created_by);

-- ── DM Participants Table ──────────────────────────────────────────────
CREATE TABLE dm_participants (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v7(),
    conversation_id UUID NOT NULL REFERENCES dm_conversations(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    org_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    joined_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_read_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(conversation_id, user_id)
);

CREATE INDEX idx_dm_participants_conversation_id ON dm_participants(conversation_id);
CREATE INDEX idx_dm_participants_user_id ON dm_participants(user_id);
CREATE INDEX idx_dm_participants_org_id ON dm_participants(org_id);

-- ── RLS Policies ───────────────────────────────────────────────────────
ALTER TABLE dm_conversations ENABLE ROW LEVEL SECURITY;
ALTER TABLE dm_participants ENABLE ROW LEVEL SECURITY;

CREATE POLICY dm_conversations_org_isolation ON dm_conversations
    USING (org_id = current_setting('app.current_org_id')::UUID);

CREATE POLICY dm_participants_org_isolation ON dm_participants
    USING (org_id = current_setting('app.current_org_id')::UUID);

-- ── updated_at Trigger ─────────────────────────────────────────────────
CREATE TRIGGER update_dm_conversations_updated_at BEFORE UPDATE ON dm_conversations
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_dm_participants_updated_at BEFORE UPDATE ON dm_participants
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();