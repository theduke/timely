
CREATE TABLE timelogs(
  id BIGSERIAL NOT NULL PRIMARY KEY,
  user_id BIGINT NOT NULL REFERENCES users (id) ON UPDATE RESTRICT ON DELETE RESTRICT,
  created_at TIMESTAMP WITH TIME ZONE NOT NULL,
  started_at TIMESTAMP WITH TIME ZONE NOT NULL,
  finished_at TIMESTAMP WITH TIME ZONE,
  description TEXT,
  CONSTRAINT finished_after_started CHECK (finished_at IS NULL OR finished_at >= started_at),
  CONSTRAINT description_length CHECK (description IS NULL or LENGTH(description) < 5000)
);
