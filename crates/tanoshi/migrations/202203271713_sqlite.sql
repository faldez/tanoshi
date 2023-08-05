CREATE TABLE tracker_manga (
    id INTEGER PRIMARY KEY,
    user_id INTEGER NOT NULL,
    manga_id INTEGER NOT NULL,
    tracker VARCHAR(256) NOT NULL,
    tracker_manga_id VARCHAR(256) NOT NULL,
    UNIQUE(user_id, manga_id, tracker),
    FOREIGN KEY (user_id) REFERENCES user(id) ON DELETE CASCADE,
    FOREIGN KEY (manga_id) REFERENCES manga(id) ON DELETE CASCADE
);