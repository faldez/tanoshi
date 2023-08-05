ALTER TABLE user_library RENAME TO user_library_old;

CREATE TABLE user_library (
    id INTEGER PRIMARY KEY,
    user_id INTEGER NOT NULL,
    manga_id INTEGER NOT NULL,
    UNIQUE(user_id, manga_id),
    FOREIGN KEY (user_id) REFERENCES user(id) ON DELETE CASCADE,
    FOREIGN KEY (manga_id) REFERENCES manga(id) ON DELETE CASCADE
);

INSERT INTO user_library(user_id, manga_id) SELECT user_id, manga_id FROM user_library_old;

DROP TABLE user_library_old;

CREATE TABLE user_category (
    id INTEGER PRIMARY KEY,
    user_id INTEGER NOT NULL,
    name VARCHAR(255) NOT NULL UNIQUE,
    UNIQUE(user_id, name),
    FOREIGN KEY (user_id) REFERENCES user(id) ON DELETE CASCADE
);

CREATE TABLE library_category (
    library_id INTEGER NOT NULL,
    category_id INTEGER NOT NULL,
    PRIMARY KEY(library_id, category_id),
    FOREIGN KEY (library_id) REFERENCES user_library(id) ON DELETE CASCADE,
    FOREIGN KEY (category_id) REFERENCES user_category(id) ON DELETE CASCADE
);