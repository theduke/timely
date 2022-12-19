
CREATE TABLE timelogs_user_tags(
  user_tag_id BIGINT NOT NULL REFERENCES user_tags (id),
  timelog_id BIGINT NOT NULL REFERENCES timelogs (id)
)
