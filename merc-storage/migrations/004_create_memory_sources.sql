-- Create memory_sources junction table
CREATE TABLE memory_sources (
    memory_id UUID NOT NULL REFERENCES memories(id) ON DELETE CASCADE,
    source_id UUID NOT NULL REFERENCES sources(id) ON DELETE CASCADE,
    confidence REAL NOT NULL CHECK (confidence >= 0 AND confidence <= 1),
    text TEXT,
    hash TEXT NOT NULL,
    start_offset INTEGER NOT NULL,
    end_offset INTEGER NOT NULL,
    PRIMARY KEY (memory_id, source_id)
);

-- Indexes
CREATE INDEX idx_memory_sources_memory_id ON memory_sources(memory_id);
CREATE INDEX idx_memory_sources_source_id ON memory_sources(source_id);
