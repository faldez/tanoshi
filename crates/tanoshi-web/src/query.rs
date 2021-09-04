use graphql_client::GraphQLQuery;
use std::error::Error;
use wasm_bindgen::prelude::*;

type NaiveDateTime = String;

use crate::{
    common::Cover,
    utils::{local_storage, window},
};

async fn post_graphql<Q>(var: Q::Variables) -> Result<Q::ResponseData, Box<dyn std::error::Error>>
where
    Q: GraphQLQuery,
{
    let url = [
        window()
            .document()
            .unwrap_throw()
            .location()
            .unwrap_throw()
            .origin()
            .unwrap(),
        "/graphql".to_string(),
    ]
    .join("");
    let token = local_storage()
        .get("token")
        .unwrap_throw()
        .unwrap_or_else(|| "".to_string());
    let request_body = Q::build_query(var);
    let client = reqwest::Client::new();
    let mut req = client.post(url);
    if !token.is_empty() {
        req = req.header("Authorization", format!("Bearer {}", token));
    }
    let res = req.json(&request_body).send().await?;
    let response_body: graphql_client::Response<Q::ResponseData> = res.json().await?;
    match (response_body.data, response_body.errors) {
        (Some(data), _) => Ok(data) as Result<_, _>,
        (_, Some(errors)) => {
            return Err(errors
                .iter()
                .map(|e| format!("{}", e))
                .collect::<Vec<String>>()
                .join(", ")
                .into());
        }
        _ => {
            return Err("no data".into());
        }
    }
}

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/browse_source.graphql",
    response_derives = "Debug, Clone"
)]
pub struct BrowseSource;

pub async fn fetch_manga_from_source(
    source_id: i64,
    page: i64,
    keyword: Option<String>,
    sort_by: browse_source::SortByParam,
    sort_order: browse_source::SortOrderParam,
) -> Result<Vec<Cover>, Box<dyn Error>> {
    let var = browse_source::Variables {
        source_id: Some(source_id),
        keyword,
        page: Some(page),
        sort_by: Some(sort_by),
        sort_order: Some(sort_order),
    };
    let data: browse_source::ResponseData = post_graphql::<BrowseSource>(var).await?;

    let covers = data
        .browse_source
        .iter()
        .map(|item| {
            Cover::new(
                item.id,
                item.source_id,
                item.path.clone(),
                item.title.clone(),
                item.cover_url.clone(),
                item.is_favorite,
            )
        })
        .collect();
    Ok(covers)
}

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/browse_favorites.graphql",
    response_derives = "Debug"
)]
pub struct BrowseFavorites;

pub async fn fetch_manga_from_favorite(refresh: bool) -> Result<Vec<Cover>, Box<dyn Error>> {
    let var = browse_favorites::Variables {
        refresh: Some(refresh),
    };
    let data = post_graphql::<BrowseFavorites>(var).await?;

    Ok(data
        .library
        .iter()
        .map(|item| {
            Cover::new(
                item.id,
                item.source_id,
                item.path.clone(),
                item.title.clone(),
                item.cover_url.clone(),
                false,
            )
        })
        .collect())
}

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/fetch_manga_by_source_path.graphql",
    response_derives = "Debug"
)]
pub struct FetchMangaBySourcePath;

pub async fn fetch_manga_by_source_path(
    source_id: i64,
    path: String,
) -> Result<fetch_manga_by_source_path::FetchMangaBySourcePathMangaBySourcePath, Box<dyn Error>> {
    let var = fetch_manga_by_source_path::Variables {
        source_id: Some(source_id),
        path: Some(path),
    };
    let data = post_graphql::<FetchMangaBySourcePath>(var).await?;

    Ok(data.manga_by_source_path)
}

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/fetch_manga_detail.graphql",
    response_derives = "Debug"
)]
pub struct FetchMangaDetail;

pub async fn fetch_manga_detail(
    id: i64,
    refresh: bool,
) -> Result<fetch_manga_detail::FetchMangaDetailManga, Box<dyn Error>> {
    let var = fetch_manga_detail::Variables {
        id: Some(id),
        refresh: Some(refresh),
    };
    let data = post_graphql::<FetchMangaDetail>(var).await?;

    Ok(data.manga)
}

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/fetch_chapter.graphql",
    response_derives = "Debug"
)]
pub struct FetchChapter;

pub async fn fetch_chapter(
    chapter_id: i64,
) -> Result<fetch_chapter::FetchChapterChapter, Box<dyn Error>> {
    let var = fetch_chapter::Variables {
        chapter_id: Some(chapter_id),
    };
    let data = post_graphql::<FetchChapter>(var).await?;

    Ok(data.chapter)
}

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/add_to_library.graphql",
    response_derives = "Debug"
)]
pub struct AddToLibrary;

pub async fn add_to_library(manga_id: i64) -> Result<(), Box<dyn Error>> {
    let var = add_to_library::Variables {
        manga_id: Some(manga_id),
    };
    let _ = post_graphql::<AddToLibrary>(var).await?;

    Ok(())
}

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/delete_from_library.graphql",
    response_derives = "Debug"
)]
pub struct DeleteFromLibrary;

pub async fn delete_from_library(manga_id: i64) -> Result<(), Box<dyn Error>> {
    let var = delete_from_library::Variables {
        manga_id: Some(manga_id),
    };
    let _ = post_graphql::<DeleteFromLibrary>(var).await?;

    Ok(())
}

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/update_page_read_at.graphql",
    response_derives = "Debug"
)]
pub struct UpdatePageReadAt;

pub async fn update_page_read_at(chapter_id: i64, page: i64) -> Result<(), Box<dyn Error>> {
    let var = update_page_read_at::Variables {
        chapter_id: Some(chapter_id),
        page: Some(page),
    };
    let _ = post_graphql::<UpdatePageReadAt>(var).await?;

    Ok(())
}

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/fetch_recent_updates.graphql",
    response_derives = "Debug"
)]
pub struct FetchRecentUpdates;

pub async fn fetch_recent_updates(
    cursor: Option<String>,
) -> Result<fetch_recent_updates::FetchRecentUpdatesRecentUpdates, Box<dyn Error>> {
    let var = fetch_recent_updates::Variables {
        first: Some(20),
        cursor,
    };
    let data = post_graphql::<FetchRecentUpdates>(var).await?;
    Ok(data.recent_updates)
}

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/fetch_histories.graphql",
    response_derives = "Debug"
)]
pub struct FetchHistories;

pub async fn fetch_histories(
    cursor: Option<String>,
) -> Result<fetch_histories::FetchHistoriesRecentChapters, Box<dyn Error>> {
    let var = fetch_histories::Variables {
        first: Some(20),
        cursor,
    };
    let data = post_graphql::<FetchHistories>(var).await?;
    Ok(data.recent_chapters)
}

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/fetch_sources.graphql",
    response_derives = "Debug"
)]
pub struct FetchSources;

pub async fn fetch_sources(
) -> Result<std::vec::Vec<fetch_sources::FetchSourcesInstalledSources>, Box<dyn Error>> {
    let var = fetch_sources::Variables {};
    let data = post_graphql::<FetchSources>(var).await?;
    Ok(data.installed_sources)
}

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/fetch_all_sources.graphql",
    response_derives = "Debug"
)]
pub struct FetchAllSources;

pub async fn fetch_all_sources() -> Result<fetch_all_sources::ResponseData, Box<dyn Error>> {
    let var = fetch_all_sources::Variables {};
    let data = post_graphql::<FetchAllSources>(var).await?;
    Ok(data)
}

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/fetch_source.graphql",
    response_derives = "Debug"
)]
pub struct FetchSourceDetail;

#[allow(dead_code)]
pub async fn fetch_source(
    source_id: i64,
) -> Result<fetch_source_detail::FetchSourceDetailSource, Box<dyn Error>> {
    let var = fetch_source_detail::Variables {
        source_id: Some(source_id),
    };
    let data = post_graphql::<FetchSourceDetail>(var).await?;
    Ok(data.source)
}

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/install_source.graphql",
    response_derives = "Debug"
)]
pub struct InstallSource;

pub async fn install_source(source_id: i64) -> Result<i64, Box<dyn Error>> {
    let var = install_source::Variables {
        source_id: Some(source_id),
    };
    let data = post_graphql::<InstallSource>(var).await?;
    Ok(data.install_source)
}

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/update_source.graphql",
    response_derives = "Debug"
)]
pub struct UpdateSource;

pub async fn update_source(source_id: i64) -> Result<i64, Box<dyn Error>> {
    let var = update_source::Variables {
        source_id: Some(source_id),
    };
    let data = post_graphql::<UpdateSource>(var).await?;
    Ok(data.update_source)
}

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/uninstall_source.graphql",
    response_derives = "Debug"
)]
pub struct UninstallSource;

pub async fn uninstall_source(source_id: i64) -> Result<i64, Box<dyn Error>> {
    let var = uninstall_source::Variables {
        source_id: Some(source_id),
    };
    let data = post_graphql::<UninstallSource>(var).await?;
    Ok(data.uninstall_source)
}

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/login.graphql",
    response_derives = "Debug"
)]
pub struct UserLogin;

pub async fn user_login(username: String, password: String) -> Result<String, Box<dyn Error>> {
    let var = user_login::Variables {
        username: Some(username),
        password: Some(password),
    };
    let data = post_graphql::<UserLogin>(var).await?;
    Ok(data.login)
}

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/fetch_users.graphql",
    response_derives = "Debug"
)]
pub struct FetchUserList;

pub async fn fetch_users() -> Result<
    (
        fetch_user_list::FetchUserListMe,
        Vec<fetch_user_list::FetchUserListUsers>,
    ),
    Box<dyn Error>,
> {
    let var = fetch_user_list::Variables {};
    let data = post_graphql::<FetchUserList>(var).await?;
    Ok((data.me, data.users))
}

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/fetch_me.graphql",
    response_derives = "Debug"
)]
pub struct FetchMe;

pub async fn fetch_me() -> Result<fetch_me::FetchMeMe, Box<dyn Error>> {
    let var = fetch_me::Variables {};
    let data = post_graphql::<FetchMe>(var).await?;
    Ok(data.me)
}

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/register.graphql",
    response_derives = "Debug"
)]
pub struct UserRegister;

pub async fn user_register(
    username: String,
    password: String,
    is_admin: bool,
) -> Result<(), Box<dyn Error>> {
    let var = user_register::Variables {
        username: Some(username),
        password: Some(password),
        is_admin: Some(is_admin),
    };
    let _ = post_graphql::<UserRegister>(var).await?;
    Ok(())
}

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/change_password.graphql",
    response_derives = "Debug"
)]
pub struct ChangeUserPassword;

pub async fn change_password(
    old_password: String,
    new_password: String,
) -> Result<(), Box<dyn Error>> {
    let var = change_user_password::Variables {
        old_password: Some(old_password),
        new_password: Some(new_password),
    };
    let _ = post_graphql::<ChangeUserPassword>(var).await?;
    Ok(())
}

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/update_profile.graphql",
    response_derives = "Debug"
)]
pub struct UpdateProfile;

pub async fn update_profile(telegram_chat_id: Option<i64>) -> Result<(), Box<dyn Error>> {
    let var = update_profile::Variables { telegram_chat_id };
    let _ = post_graphql::<UpdateProfile>(var).await?;
    Ok(())
}

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/fetch_server_status.graphql",
    response_derives = "Debug"
)]
pub struct FetchServerStatus;

pub async fn server_status(
) -> Result<fetch_server_status::FetchServerStatusServerStatus, Box<dyn Error>> {
    let var = fetch_server_status::Variables {};
    let data = post_graphql::<FetchServerStatus>(var).await?;
    Ok(data.server_status)
}

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/test_telegram.graphql",
    response_derives = "Debug"
)]
pub struct TestTelegram;

pub async fn test_telegram(chat_id: i64) -> Result<(), Box<dyn Error>> {
    let var = test_telegram::Variables {
        chat_id: Some(chat_id),
    };
    let _ = post_graphql::<TestTelegram>(var).await?;
    Ok(())
}

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/mark_chapter_as_read.graphql",
    response_derives = "Debug"
)]
pub struct MarkChapterAsRead;

pub async fn mark_chapter_as_read(chapter_ids: &[i64]) -> Result<(), Box<dyn Error>> {
    let var = mark_chapter_as_read::Variables {
        chapter_ids: Some(chapter_ids.iter().map(|id| Some(*id)).collect()),
    };

    let _ = post_graphql::<MarkChapterAsRead>(var).await?;
    Ok(())
}

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/mark_chapter_as_unread.graphql",
    response_derives = "Debug"
)]
pub struct MarkChapterAsUnread;

pub async fn mark_chapter_as_unread(chapter_ids: &[i64]) -> Result<(), Box<dyn Error>> {
    let var = mark_chapter_as_unread::Variables {
        chapter_ids: Some(chapter_ids.iter().map(|id| Some(*id)).collect()),
    };

    let _ = post_graphql::<MarkChapterAsUnread>(var).await?;
    Ok(())
}
