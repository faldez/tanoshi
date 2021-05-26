CREATE TABLE user(
    id INTEGER PRIMARY KEY,
    username VARCHAR(255) UNIQUE,
    password VARCHAR(255) NOT NULL,
    role INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE TABLE user_favorite(
    user_id INTEGER,
    manga_id INTEGER,
    PRIMARY_KEY(user_id, manga_id),
    FOREIGN KEY user_id REFERENCES user(id) ON CASCADE DELETE ON UPDATE NO ACTION,
    FOREIGN KEY manga_id REFERENCES manga(id) ON CASCADE DELETE ON UPDATE NO ACTION
)
CREATE TABLE user_history(
    user_id INTEGER,
    chapter_id INTEGER,
    last_page INTEGER NOT NULL DEFAULT 1,
    read_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY_KEY(user_id, chapter_id),
    FOREIGN KEY user_id REFERENCES user(id) ON CASCADE DELETE ON UPDATE NO ACTION,
    FOREIGN KEY manga_id REFERENCES manga(id) ON CASCADE DELETE ON UPDATE NO ACTION
);