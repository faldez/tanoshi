ALTER TABLE chapter ADD COLUMN downloaded_path TEXT;

UPDATE chapter SET downloaded_path = (
	SELECT downloaded_path FROM (
		SELECT 
			SUBSTRING(local_url, 0, INSTR(local_url, '.cbz') + 4) AS downloaded_path,
			(COUNT(remote_url) > 0) & (COUNT(remote_url) = COUNT(local_url)) AS downloaded
		FROM page 
		WHERE chapter_id = chapter.id
		GROUP BY chapter_id
		HAVING downloaded = true
	)
);

DROP TABLE page;