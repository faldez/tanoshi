use chrono::NaiveDateTime;
use dominator::{html, link, Dom};
use futures_signals::signal::{self, Mutable, SignalExt};
use serde::{Deserialize, Serialize};

use crate::common::route::Route;
use crate::utils::proxied_image_url;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Cover {
    pub id: i64,
    pub source_id: i64,
    pub path: String,
    pub title: String,
    pub cover_url: String,
    pub is_favorite: Mutable<bool>,
    pub last_read_at: Option<NaiveDateTime>,
    pub unread_chapter_count: i64,
}

impl Cover {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: i64,
        source_id: i64,
        path: String,
        title: String,
        cover_url: String,
        is_favorite: bool,
        last_read_at: Option<NaiveDateTime>,
        unread_chapter_count: i64,
    ) -> Self {
        let cover_url = proxied_image_url(&cover_url);
        Self {
            id,
            source_id,
            path,
            title,
            cover_url,
            is_favorite: Mutable::new(is_favorite),
            last_read_at,
            unread_chapter_count,
        }
    }

    #[allow(dead_code)]
    pub fn set_favorite(&self, favorite: bool) {
        self.is_favorite.set(favorite);
    }

    pub fn link(&self) -> String {
        if self.id != 0 {
            Route::Manga(self.id).url()
        } else if self.source_id != 0 && !self.path.is_empty() {
            Route::MangaBySourcePath(self.source_id, self.path.clone()).url()
        } else {
            Route::NotFound.url()
        }
    }

    pub fn render(&self) -> Dom {
        link!(self.link(), {
            .class("manga-cover")
            .class("animate__animated")
            .class("animate__faster")
            .class("animate__fadeIn")
            .class_signal("favorite", self.is_favorite.signal())
            .children(&mut [
                html!("img", {
                    .attr("src", &self.cover_url)
                    .attr("loading", "lazy")
                }),
                html!("div", {
                    .class("unread-badge")
                    .visible_signal(signal::always(self.unread_chapter_count).map(|count| count > 0))
                    .children(&mut [
                        html!("span", {
                            .text(format!("{}", self.unread_chapter_count).as_str())
                        })
                    ])
                }),
                html!("div", {
                    .class("title")
                    .children(&mut [
                        html!("span", {
                            .text(&self.title)
                        })
                    ])
                })
            ])
        })
    }
}
