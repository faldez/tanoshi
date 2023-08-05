CREATE TABLE download_queue (
    id INTEGER PRIMARY KEY,
    source_name INTEGER NOT NULL,
    manga_title TEXT NOT NULL,
    chapter_title TEXT NOT NULL,
    rank INTEGER NOT NULL,
    url TEXT NOT NULL UNIQUE,
    date_added TIMESTAMP NOT NULL
);