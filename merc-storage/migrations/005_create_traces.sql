-- Create traces table
CREATE TABLE traces (
    id UUID PRIMARY KEY NOT NULL,
    parent_id UUID REFERENCES traces(id) ON DELETE SET NULL,
    request_id TEXT,
    status TEXT NOT NULL,
    status_message TEXT,
    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    ended_at TIMESTAMPTZ
);

-- Indexes
CREATE INDEX idx_traces_parent_id ON traces(parent_id);
CREATE INDEX idx_traces_request_id ON traces(request_id);
CREATE INDEX idx_traces_status ON traces(status);
CREATE INDEX idx_traces_started_at ON traces(started_at);
