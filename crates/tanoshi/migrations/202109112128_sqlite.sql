ALTER TABLE user_history ADD COLUMN is_complete BOOLEAN DEFAULT false;

--- set chapter is complete if last page is the last page of the chapter
UPDATE user_history
SET is_complete = (user_history.last_page IS (SELECT COUNT(1) - 1 FROM page WHERE page.chapter_id = user_history.chapter_id));

--- set chapter is complete if last page is the second last page of the chapter
UPDATE user_history
SET is_complete = (user_history.last_page IS (SELECT COUNT(1) - 2 FROM page WHERE page.chapter_id = user_history.chapter_id));