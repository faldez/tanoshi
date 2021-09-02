CREATE TABLE page (
    id INTEGER PRIMARY KEY,
    source_id INTEGER NOT NULL,
    manga_id INTEGER NOT NULL,
    chapter_id INTEGER NOT NULL,
    rank INTEGER NOT NULL,
    remote_url TEXT NOT NULL,
    local_url TEXT,
    UNIQUE (chapter_id, rank)
);

INSERT INTO page (source_id, manga_id, chapter_id, rank, remote_url)
SELECT 
    source_id, 
    manga_id, 
    chapter.id AS chapter_id, 
    json_each.key AS rank, 
    json_each.value AS remote_url 
FROM chapter, json_each(pages);

ALTER TABLE chapter DROP COLUMN pages;