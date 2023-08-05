CREATE TABLE tracker_credential (
    id INTEGER PRIMARY KEY,
    user_id INTEGER NOT NULL,
    tracker VARCHAR(256) NOT NULL,
    token_type VARCHAR(256),
    access_token TEXT NOT NULL,
    refresh_token TEXT,
    expires_in TIMESTAMP,
    UNIQUE(user_id, tracker),
    FOREIGN KEY (user_id) REFERENCES user(id) ON DELETE CASCADE
);