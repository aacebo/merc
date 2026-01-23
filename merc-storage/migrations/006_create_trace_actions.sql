-- Create trace_actions table
CREATE TABLE trace_actions (
    trace_id UUID NOT NULL REFERENCES traces(id) ON DELETE CASCADE,
    target_id UUID NOT NULL,
    target TEXT NOT NULL,
    action TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (trace_id, target_id, action, created_at)
);

-- Indexes
CREATE INDEX idx_trace_actions_trace_id ON trace_actions(trace_id);
CREATE INDEX idx_trace_actions_target_id ON trace_actions(target_id);
CREATE INDEX idx_trace_actions_target ON trace_actions(target);
CREATE INDEX idx_trace_actions_action ON trace_actions(action);
CREATE INDEX idx_trace_actions_created_at ON trace_actions(created_at);
