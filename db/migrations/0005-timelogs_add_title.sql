
ALTER TABLE timelogs 
  ADD COLUMN title TEXT NOT NULL DEFAULT '<no title>',
  ADD CONSTRAINT title_length CHECK (LENGTH(title) BETWEEN 1 AND 150)
;
