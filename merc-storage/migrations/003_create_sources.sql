-- Create sources table
CREATE TABLE sources (
    id UUID PRIMARY KEY NOT NULL,
    scope_id UUID NOT NULL,
    external_id TEXT NOT NULL,
    type TEXT NOT NULL,
    uri TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes
CREATE INDEX idx_sources_scope_id ON sources(scope_id);
CREATE INDEX idx_sources_external_id ON sources(external_id);
CREATE INDEX idx_sources_type ON sources(type);
CREATE INDEX idx_sources_created_at ON sources(created_at);
