ALTER TABLE user_history ADD COLUMN is_complete BOOLEAN DEFAULT false;

UPDATE user_history
SET is_complete = (user_history.last_page = (SELECT COUNT(1) FROM page WHERE page.chapter_id = user_history.chapter_id) OR (user_history.last_page =  (SELECT COUNT(1) FROM page WHERE page.chapter_id = user_history.chapter_id) - 1));