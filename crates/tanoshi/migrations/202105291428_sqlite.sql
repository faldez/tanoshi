CREATE TABLE manga (
    id INTEGER PRIMARY KEY,
    source_id INTEGER NOT NULL,
    title TEXT NOT NULL,
    author TEXT,
    genre TEXT,
    status TEXT,
    description TEXT,
    path TEXT NOT NULL,
    cover_url TEXT NOT NULL,
    date_added TIMESTAMP NOT NULL,
    UNIQUE (source_id, path)
);
CREATE TABLE chapter (
    id INTEGER PRIMARY KEY,
    source_id INTEGER NOT NULL,
    manga_id INTEGER NOT NULL,
    title TEXT,
    path TEXT NOT NULL,
    number FLOAT NOT NULL,
    scanlator TEXT DEFAULT '',
    uploaded TIMESTAMP NOT NULL,
    date_added TIMESTAMP NOT NULL,
    pages JSON DEFAULT '[]',
    UNIQUE (source_id, path)
);
CREATE TABLE "user" (
    id INTEGER PRIMARY KEY,
    username VARCHAR(255) UNIQUE,
    password VARCHAR(255) NOT NULL,
    is_admin BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE TABLE user_library (
    user_id INTEGER NOT NULL,
    manga_id INTEGER NOT NULL,
    PRIMARY KEY(user_id, manga_id),
    FOREIGN KEY (user_id) REFERENCES user(id) ON DELETE CASCADE ON UPDATE NO ACTION,
    FOREIGN KEY (manga_id) REFERENCES manga(id) ON DELETE CASCADE ON UPDATE NO ACTION
);
CREATE TABLE user_history (
    user_id INTEGER,
    chapter_id INTEGER,
    last_page INTEGER NOT NULL DEFAULT 1,
    read_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY(user_id, chapter_id),
    FOREIGN KEY (user_id) REFERENCES user(id) ON DELETE CASCADE ON UPDATE NO ACTION,
    FOREIGN KEY (chapter_id) REFERENCES chapter(id) ON DELETE CASCADE ON UPDATE NO ACTION
);