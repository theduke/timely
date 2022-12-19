
CREATE TABLE user_tags (
  id BIGSERIAL NOT NULL PRIMARY KEY,
  user_id BIGINT NOT NULL REFERENCES users (id) ON UPDATE RESTRICT ON DELETE RESTRICT,
  name TEXT NOT NULL,
  description TEXT,
  color TEXT,
  created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now(),
  updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now(),
  CONSTRAINT name_length CHECK (LENGTH(name) BETWEEN 1 and 100),
  CONSTRAINT description_length CHECK (description IS NULL OR LENGTH(description) BETWEEN 0 and 5000),
  CONSTRAINT color_length CHECK (color is NULL OR LENGTH(color) BETWEEN 1 and 30),
  CONSTRAINT unique_name_per_user UNIQUE (user_id, name)
);
