-- Create facets table
CREATE TABLE facets (
    id UUID PRIMARY KEY NOT NULL,
    memory_id UUID NOT NULL REFERENCES memories(id) ON DELETE CASCADE,
    type TEXT NOT NULL,
    confidence REAL NOT NULL CHECK (confidence >= 0 AND confidence <= 1),
    data BYTEA NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes
CREATE INDEX idx_facets_memory_id ON facets(memory_id);
CREATE INDEX idx_facets_type ON facets(type);
CREATE INDEX idx_facets_created_at ON facets(created_at);
CREATE INDEX idx_facets_updated_at ON facets(updated_at);
