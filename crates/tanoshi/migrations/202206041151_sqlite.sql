CREATE INDEX idx_chapter_number ON chapter(number);
CREATE INDEX idx_chapter_uploaded_number ON chapter(uploaded, number);
CREATE INDEX idx_user_history_read_at ON user_history(read_at);
CREATE INDEX idx_user_category_name ON user_category(name);
CREATE INDEX idx_manga_title ON manga(title);