use super::catalogue::{chapter::ReadProgress, Manga};
use crate::db::MangaDatabase;
use async_graphql::{dataloader::Loader, Result};
use chrono::NaiveDateTime;
use itertools::Itertools;
use std::{collections::HashMap, sync::Arc};

pub struct DatabaseLoader {
    pub mangadb: MangaDatabase,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UserFavoriteId(pub i64, pub i64);

#[async_trait::async_trait]
impl Loader<UserFavoriteId> for DatabaseLoader {
    type Value = bool;

    type Error = Arc<anyhow::Error>;

    async fn load(
        &self,
        keys: &[UserFavoriteId],
    ) -> Result<HashMap<UserFavoriteId, Self::Value>, Self::Error> {
        let user_id = keys
            .iter()
            .next()
            .map(|key| key.0)
            .ok_or_else(|| anyhow::anyhow!("no user id"))?;
        let manga_ids: Vec<i64> = keys.iter().map(|key| key.1).collect();
        let res = self
            .mangadb
            .is_user_library_by_manga_ids(user_id, &manga_ids)
            .await?
            .into_iter()
            .map(|(manga_id, is_library)| (UserFavoriteId(user_id, manga_id), is_library))
            .collect();
        Ok(res)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UserFavoritePath(pub i64, pub String);

#[async_trait::async_trait]
impl Loader<UserFavoritePath> for DatabaseLoader {
    type Value = bool;

    type Error = Arc<anyhow::Error>;

    async fn load(
        &self,
        keys: &[UserFavoritePath],
    ) -> Result<HashMap<UserFavoritePath, Self::Value>, Self::Error> {
        let user_id = keys
            .iter()
            .next()
            .map(|key| key.0)
            .ok_or_else(|| anyhow::anyhow!("no user id"))?;
        let manga_paths: Vec<String> = keys.iter().map(|key| key.1.clone()).collect();
        let res = self
            .mangadb
            .is_user_library_by_manga_paths(user_id, &manga_paths)
            .await?
            .into_iter()
            .map(|(manga_path, is_library)| (UserFavoritePath(user_id, manga_path), is_library))
            .collect();
        Ok(res)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UserLastReadId(pub i64, pub i64);

#[async_trait::async_trait]
impl Loader<UserLastReadId> for DatabaseLoader {
    type Value = NaiveDateTime;

    type Error = Arc<anyhow::Error>;

    async fn load(
        &self,
        keys: &[UserLastReadId],
    ) -> Result<HashMap<UserLastReadId, Self::Value>, Self::Error> {
        let user_id = keys
            .iter()
            .next()
            .map(|key| key.0)
            .ok_or_else(|| anyhow::anyhow!("no user id"))?;
        let manga_ids: Vec<i64> = keys.iter().map(|key| key.1).collect();
        let res = self
            .mangadb
            .get_last_read_at_by_user_id_and_manga_ids(user_id, &manga_ids)
            .await?
            .into_iter()
            .map(|(manga_id, read_at)| (UserLastReadId(user_id, manga_id), read_at))
            .collect();
        Ok(res)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UserUnreadChaptersId(pub i64, pub i64);

#[async_trait::async_trait]
impl Loader<UserUnreadChaptersId> for DatabaseLoader {
    type Value = i64;

    type Error = Arc<anyhow::Error>;

    async fn load(
        &self,
        keys: &[UserUnreadChaptersId],
    ) -> Result<HashMap<UserUnreadChaptersId, Self::Value>, Self::Error> {
        let user_id = keys
            .iter()
            .next()
            .map(|key| key.0)
            .ok_or_else(|| anyhow::anyhow!("no user id"))?;
        let manga_ids: Vec<i64> = keys.iter().map(|key| key.1).collect();
        let res = self
            .mangadb
            .get_user_library_unread_chapters(user_id, &manga_ids)
            .await?
            .into_iter()
            .map(|(manga_id, count)| (UserUnreadChaptersId(user_id, manga_id), count))
            .collect();
        Ok(res)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UserHistoryId(pub i64, pub i64);

#[async_trait::async_trait]
impl Loader<UserHistoryId> for DatabaseLoader {
    type Value = ReadProgress;

    type Error = Arc<anyhow::Error>;

    async fn load(
        &self,
        keys: &[UserHistoryId],
    ) -> Result<HashMap<UserHistoryId, Self::Value>, Self::Error> {
        let user_id = keys
            .iter()
            .next()
            .map(|key| key.0)
            .ok_or_else(|| anyhow::anyhow!("no user id"))?;
        let chapter_ids: Vec<i64> = keys.iter().map(|key| key.1).collect();
        let res = self
            .mangadb
            .get_user_history_progress_by_chapter_ids(user_id, &chapter_ids)
            .await?
            .into_iter()
            .map(|(chapter_id, progress)| (UserHistoryId(user_id, chapter_id), progress.into()))
            .collect();
        Ok(res)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MangaId(pub i64);

#[async_trait::async_trait]
impl Loader<MangaId> for DatabaseLoader {
    type Value = Manga;

    type Error = Arc<anyhow::Error>;

    async fn load(&self, keys: &[MangaId]) -> Result<HashMap<MangaId, Self::Value>, Self::Error> {
        let keys: Vec<i64> = keys.iter().map(|key| key.0).collect();
        let res = self
            .mangadb
            .get_manga_by_ids(&keys)
            .await?
            .into_iter()
            .map(|m| (MangaId(m.id), m.into()))
            .collect();
        Ok(res)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PrevChapterId(pub i64);

#[async_trait::async_trait]
impl Loader<PrevChapterId> for DatabaseLoader {
    type Value = i64;

    type Error = Arc<anyhow::Error>;

    async fn load(
        &self,
        keys: &[PrevChapterId],
    ) -> Result<HashMap<PrevChapterId, Self::Value>, Self::Error> {
        let keys: Vec<i64> = keys.iter().map(|key| key.0).collect();
        let res = self
            .mangadb
            .get_prev_chapter_id_by_ids(&keys)
            .await?
            .into_iter()
            .map(|(chapter_id, prev_chapter_id)| (PrevChapterId(chapter_id), prev_chapter_id))
            .collect();
        Ok(res)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NextChapterId(pub i64);

#[async_trait::async_trait]
impl Loader<NextChapterId> for DatabaseLoader {
    type Value = i64;

    type Error = Arc<anyhow::Error>;

    async fn load(
        &self,
        keys: &[NextChapterId],
    ) -> Result<HashMap<NextChapterId, Self::Value>, Self::Error> {
        let keys: Vec<i64> = keys.iter().map(|key| key.0).collect();
        let res = self
            .mangadb
            .get_next_chapter_id_by_ids(&keys)
            .await?
            .into_iter()
            .map(|(chapter_id, next_chapter_id)| (NextChapterId(chapter_id), next_chapter_id))
            .collect();
        Ok(res)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UserTrackerMangaId(pub i64, pub i64);

#[async_trait::async_trait]
impl Loader<UserTrackerMangaId> for DatabaseLoader {
    type Value = Vec<(String, Option<String>)>;

    type Error = Arc<anyhow::Error>;

    async fn load(
        &self,
        keys: &[UserTrackerMangaId],
    ) -> Result<HashMap<UserTrackerMangaId, Self::Value>, Self::Error> {
        let user_id = keys
            .iter()
            .next()
            .map(|key| key.0)
            .ok_or_else(|| anyhow::anyhow!("no user id"))?;

        let manga_ids: Vec<i64> = keys.iter().map(|key| key.1).collect();
        let res = self
            .mangadb
            .get_tracker_manga_ids(user_id, &manga_ids)
            .await?
            .iter()
            .group_by(|m| UserTrackerMangaId(user_id, m.manga_id))
            .into_iter()
            .map(|(key, group)| {
                (
                    key,
                    (group
                        .map(|v| (v.tracker.clone(), v.tracker_manga_id.clone()))
                        .collect()),
                )
            })
            .collect();
        Ok(res)
    }
}
