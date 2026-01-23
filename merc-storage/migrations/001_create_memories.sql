-- Create memories table
CREATE TABLE memories (
    id UUID PRIMARY KEY NOT NULL,
    scope_id UUID NOT NULL,
    score REAL NOT NULL CHECK (score >= 0 AND score <= 1),
    confidence REAL NOT NULL CHECK (confidence >= 0 AND confidence <= 1),
    importance REAL NOT NULL CHECK (importance >= 0 AND importance <= 1),
    sensitivity TEXT NOT NULL,
    tags TEXT[] NOT NULL DEFAULT '{}',
    embedding REAL[],
    expires_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes
CREATE INDEX idx_memories_scope_id ON memories(scope_id);
CREATE INDEX idx_memories_importance ON memories(importance);
CREATE INDEX idx_memories_sensitivity ON memories(sensitivity);
CREATE INDEX idx_memories_expires_at ON memories(expires_at);
CREATE INDEX idx_memories_created_at ON memories(created_at);
CREATE INDEX idx_memories_updated_at ON memories(updated_at);
