CREATE INDEX IF NOT EXISTS idx_scans_student_id ON scans(student_id);
CREATE INDEX IF NOT EXISTS idx_scans_timestamp ON scans(timestamp);
CREATE INDEX IF NOT EXISTS idx_scans_student_timestamp ON scans(student_id, timestamp DESC);
