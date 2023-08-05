use futures::future::{select, Either};
use gloo_timers::future::TimeoutFuture;
use graphql_client::GraphQLQuery;
use std::{collections::HashMap, error::Error};
use wasm_bindgen::prelude::*;

type NaiveDateTime = String;

use crate::{
    common::{Cover, Input},
    utils::{graphql_host, graphql_ws_host, local_storage},
};

use tanoshi_schema::*;

async fn post_graphql<Q>(var: Q::Variables) -> Result<Q::ResponseData, Box<dyn std::error::Error>>
where
    Q: GraphQLQuery,
{
    let url = graphql_host();

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
        _ => Err("no data".into()),
    }
}

pub type InputList = Vec<Input>;

pub async fn fetch_manga_from_source(
    source_id: i64,
    page: i64,
    query: Option<String>,
    filters: Option<InputList>,
) -> Result<browse_source::ResponseData, Box<dyn Error>> {
    let var = browse_source::Variables {
        source_id: Some(source_id),
        page: Some(page),
        query,
        filters,
    };
    let data: browse_source::ResponseData = post_graphql::<BrowseSource>(var).await?;
    Ok(data)
}

pub async fn get_latest_manga(
    source_id: i64,
    page: i64,
) -> Result<get_latest_manga::ResponseData, Box<dyn Error>> {
    let var = get_latest_manga::Variables { source_id, page };
    let data: get_latest_manga::ResponseData = post_graphql::<GetLatestManga>(var).await?;

    Ok(data)
}

pub async fn get_popular_manga(
    source_id: i64,
    page: i64,
) -> Result<get_popular_manga::ResponseData, Box<dyn Error>> {
    let var = get_popular_manga::Variables { source_id, page };
    let data: get_popular_manga::ResponseData = post_graphql::<GetPopularManga>(var).await?;
    Ok(data)
}

pub async fn fetch_source_filters(
    source_id: i64,
) -> Result<fetch_source_filters::ResponseData, Box<dyn Error>> {
    let var = fetch_source_filters::Variables { source_id };
    let data: fetch_source_filters::ResponseData = post_graphql::<FetchSourceFilters>(var).await?;
    Ok(data)
}

pub async fn fetch_manga_from_favorite(
    category_id: Option<i64>,
) -> Result<Vec<Cover>, Box<dyn Error>> {
    let var = browse_favorites::Variables { category_id };
    let data = post_graphql::<BrowseFavorites>(var).await?;

    Ok(data
        .library
        .iter()
        .map(|item| {
            Cover::new(
                item.id,
                0,
                item.path.clone(),
                item.title.clone(),
                item.cover_url.clone(),
                false,
                item.last_read_at.as_ref().and_then(|read_at| {
                    chrono::NaiveDateTime::parse_from_str(read_at, "%Y-%m-%dT%H:%M:%S%.f").ok()
                }),
                item.unread_chapter_count,
            )
        })
        .collect())
}

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

pub async fn fetch_chapter(
    chapter_id: i64,
) -> Result<fetch_chapter::FetchChapterChapter, Box<dyn Error>> {
    let var = fetch_chapter::Variables {
        chapter_id: Some(chapter_id),
    };
    let data = post_graphql::<FetchChapter>(var).await?;

    Ok(data.chapter)
}

pub async fn add_to_library(manga_id: i64, category_ids: Vec<i64>) -> Result<(), Box<dyn Error>> {
    let var = add_to_library::Variables {
        manga_id: Some(manga_id),
        category_ids: Some(category_ids.iter().map(|id| Some(*id)).collect()),
    };
    let _ = post_graphql::<AddToLibrary>(var).await?;

    Ok(())
}

pub async fn delete_from_library(manga_id: i64) -> Result<(), Box<dyn Error>> {
    let var = delete_from_library::Variables {
        manga_id: Some(manga_id),
    };
    let _ = post_graphql::<DeleteFromLibrary>(var).await?;

    Ok(())
}

pub async fn fetch_category_detail(
    id: i64,
) -> Result<fetch_category_detail::FetchCategoryDetailGetCategory, Box<dyn Error>> {
    let var = fetch_category_detail::Variables { id: Some(id) };
    let data = post_graphql::<FetchCategoryDetail>(var).await?;

    Ok(data.get_category)
}

pub async fn fetch_categories(
) -> Result<Vec<fetch_categories::FetchCategoriesGetCategories>, Box<dyn Error>> {
    let var = fetch_categories::Variables {};
    let data = post_graphql::<FetchCategories>(var).await?;

    Ok(data.get_categories)
}

pub async fn create_category(name: &str) -> Result<(), Box<dyn Error>> {
    let var = create_category::Variables {
        name: Some(name.to_string()),
    };
    let _ = post_graphql::<CreateCategory>(var).await?;

    Ok(())
}

pub async fn update_category(id: i64, name: &str) -> Result<(), Box<dyn Error>> {
    let var = update_category::Variables {
        id: Some(id),
        name: Some(name.to_string()),
    };
    let _ = post_graphql::<UpdateCategory>(var).await?;

    Ok(())
}

pub async fn delete_category(id: i64) -> Result<(), Box<dyn Error>> {
    let var = delete_category::Variables { id: Some(id) };
    let _ = post_graphql::<DeleteCategory>(var).await?;

    Ok(())
}

pub async fn update_page_read_at(
    chapter_id: i64,
    page: i64,
    is_complete: bool,
) -> Result<(), Box<dyn Error>> {
    let var = update_page_read_at::Variables {
        chapter_id: Some(chapter_id),
        page: Some(page),
        is_complete: Some(is_complete),
    };
    let _ = post_graphql::<UpdatePageReadAt>(var).await?;

    Ok(())
}

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

pub async fn subscribe_recent_updates() -> Result<(), Box<dyn Error>> {
    use futures::StreamExt;
    use graphql_ws_client::{graphql::StreamingOperation, GraphQLClientClientBuilder};
    use serde::Serialize;
    use web_sys::{Notification, NotificationOptions};

    #[derive(Serialize)]
    struct Payload {
        token: String,
    }

    let (ws, wsio) =
        ws_stream_wasm::WsMeta::connect(graphql_ws_host(), Some(vec!["graphql-transport-ws"]))
            .await?;
    let (sink, stream) = graphql_ws_client::wasm_websocket_combined_split(ws, wsio).await;

    let token = local_storage()
        .get("token")
        .unwrap_throw()
        .unwrap_or_else(|| "".to_string());
    let mut client = GraphQLClientClientBuilder::new()
        .payload(Payload { token })
        .build(stream, sink, async_executors::AsyncStd)
        .await?;

    let op: StreamingOperation<SubscribeChapterUpdates> =
        StreamingOperation::new(subscribe_chapter_updates::Variables {});
    let mut stream = client.streaming_operation(op).await?;

    let mut updates = HashMap::<String, Vec<String>>::new();
    loop {
        match select(stream.next(), TimeoutFuture::new(1_000)).await {
            Either::Left((val, timeout)) => {
                drop(timeout);
                if let Some(Ok(item)) = val {
                    if let Some(data) = item.data.map(|data| data.recent_updates_subscription) {
                        updates
                            .entry(data.manga_title.clone())
                            .and_modify(|chapters| chapters.push(data.chapter_title.clone()))
                            .or_insert(vec![data.chapter_title.clone()]);
                    }
                } else {
                    break;
                }
            }
            Either::Right((_, _)) => {
                for (manga_title, chapters) in updates.iter() {
                    let mut opts = NotificationOptions::new();
                    if chapters.len() > 1 {
                        opts.body(&format!("{} chapter updates", chapters.len()));
                    } else if chapters.len() == 1 {
                        opts.body(&chapters[0]);
                    } else {
                        opts.body("no chapter updates");
                    }

                    let _ = Notification::new_with_options(&manga_title, &opts).unwrap_throw();
                }
                updates.clear();
            }
        }
    }

    debug!("subscribe_recent_updates");
    Ok(())
}

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

pub async fn fetch_sources(
) -> Result<std::vec::Vec<fetch_sources::FetchSourcesInstalledSources>, Box<dyn Error>> {
    let var = fetch_sources::Variables {};
    let data = post_graphql::<FetchSources>(var).await?;
    Ok(data.installed_sources)
}

pub async fn fetch_all_sources() -> Result<fetch_all_sources::ResponseData, Box<dyn Error>> {
    let var = fetch_all_sources::Variables {};
    let data = post_graphql::<FetchAllSources>(var).await?;
    Ok(data)
}

pub async fn fetch_source(
    source_id: i64,
) -> Result<fetch_source_detail::FetchSourceDetailSource, Box<dyn Error>> {
    let var = fetch_source_detail::Variables {
        source_id: Some(source_id),
    };
    let data = post_graphql::<FetchSourceDetail>(var).await?;
    Ok(data.source)
}

pub async fn set_preferences(source_id: i64, preferences: InputList) -> Result<(), Box<dyn Error>> {
    let var = set_preferences::Variables {
        source_id,
        preferences,
    };
    let _ = post_graphql::<SetPreferences>(var).await?;
    Ok(())
}

pub async fn install_source(source_id: i64) -> Result<i64, Box<dyn Error>> {
    let var = install_source::Variables {
        source_id: Some(source_id),
    };
    let data = post_graphql::<InstallSource>(var).await?;
    Ok(data.install_source)
}

pub async fn update_source(source_id: i64) -> Result<i64, Box<dyn Error>> {
    let var = update_source::Variables {
        source_id: Some(source_id),
    };
    let data = post_graphql::<UpdateSource>(var).await?;
    Ok(data.update_source)
}

pub async fn uninstall_source(source_id: i64) -> Result<i64, Box<dyn Error>> {
    let var = uninstall_source::Variables {
        source_id: Some(source_id),
    };
    let data = post_graphql::<UninstallSource>(var).await?;
    Ok(data.uninstall_source)
}

pub async fn user_login(username: String, password: String) -> Result<String, Box<dyn Error>> {
    let var = user_login::Variables {
        login: user_login::LoginInput { username, password },
    };
    let data = post_graphql::<UserLogin>(var).await?;
    Ok(data.login)
}

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

pub async fn fetch_me() -> Result<fetch_me::FetchMeMe, Box<dyn Error>> {
    let var = fetch_me::Variables {};
    let data = post_graphql::<FetchMe>(var).await?;
    Ok(data.me)
}

pub async fn user_register(
    username: String,
    password: String,
    is_admin: bool,
) -> Result<(), Box<dyn Error>> {
    let var = user_register::Variables {
        login: user_register::LoginInput { username, password },
        is_admin,
    };
    let _ = post_graphql::<UserRegister>(var).await?;
    Ok(())
}

pub async fn delete_user(user_id: i64) -> Result<(), Box<dyn Error>> {
    let var = delete_user::Variables { user_id };
    let _ = post_graphql::<DeleteUser>(var).await?;
    Ok(())
}

pub async fn change_password(
    old_password: String,
    new_password: String,
) -> Result<(), Box<dyn Error>> {
    let var = change_user_password::Variables {
        input: change_user_password::ChangePasswordInput {
            old_password,
            new_password,
        },
    };
    let _ = post_graphql::<ChangeUserPassword>(var).await?;
    Ok(())
}

pub async fn update_profile(
    telegram_chat_id: Option<i64>,
    pushover_user_key: Option<String>,
    gotify_token: Option<String>,
) -> Result<(), Box<dyn Error>> {
    let var = update_profile::Variables {
        input: update_profile::ProfileInput {
            telegram_chat_id,
            pushover_user_key,
            gotify_token,
        },
    };
    let _ = post_graphql::<UpdateProfile>(var).await?;
    Ok(())
}

pub async fn fetch_server_status(
) -> Result<fetch_server_status::FetchServerStatusServerStatus, Box<dyn Error>> {
    let var = fetch_server_status::Variables {};
    let data = post_graphql::<FetchServerStatus>(var).await?;
    Ok(data.server_status)
}

pub async fn test_telegram(chat_id: i64) -> Result<(), Box<dyn Error>> {
    let var = test_telegram::Variables {
        chat_id: Some(chat_id),
    };
    let _ = post_graphql::<TestTelegram>(var).await?;
    Ok(())
}

pub async fn test_pushover(user_key: &str) -> Result<(), Box<dyn Error>> {
    let var = test_pushover::Variables {
        user_key: Some(user_key.to_string()),
    };
    let _ = post_graphql::<TestPushover>(var).await?;
    Ok(())
}

pub async fn test_gotify(token: &str) -> Result<(), Box<dyn Error>> {
    let var = test_gotify::Variables {
        token: Some(token.to_string()),
    };
    let _ = post_graphql::<TestGotify>(var).await?;
    Ok(())
}

pub async fn test_desktop_notification() -> Result<(), Box<dyn Error>> {
    let var = test_desktop_notification::Variables {};
    let _ = post_graphql::<TestDesktopNotification>(var).await?;
    Ok(())
}

pub async fn mark_chapter_as_read(chapter_ids: &[i64]) -> Result<(), Box<dyn Error>> {
    let var = mark_chapter_as_read::Variables {
        chapter_ids: Some(chapter_ids.iter().map(|id| Some(*id)).collect()),
    };

    let _ = post_graphql::<MarkChapterAsRead>(var).await?;
    Ok(())
}

pub async fn mark_chapter_as_unread(chapter_ids: &[i64]) -> Result<(), Box<dyn Error>> {
    let var = mark_chapter_as_unread::Variables {
        chapter_ids: Some(chapter_ids.iter().map(|id| Some(*id)).collect()),
    };

    let _ = post_graphql::<MarkChapterAsUnread>(var).await?;
    Ok(())
}

pub async fn download_chapters(chapter_ids: &[i64]) -> Result<(), Box<dyn Error>> {
    let var = download_chapters::Variables {
        ids: Some(chapter_ids.iter().map(|id| Some(*id)).collect()),
    };

    let _ = post_graphql::<DownloadChapters>(var).await?;
    Ok(())
}

pub async fn remove_downloaded_chapters(chapter_ids: &[i64]) -> Result<(), Box<dyn Error>> {
    let var = remove_downloaded_chapters::Variables {
        ids: Some(chapter_ids.iter().map(|id| Some(*id)).collect()),
    };

    let _ = post_graphql::<RemoveDownloadedChapters>(var).await?;
    Ok(())
}

pub async fn fetch_download_queue(
) -> Result<Vec<fetch_download_queue::FetchDownloadQueueDownloadQueue>, Box<dyn Error>> {
    let var = fetch_download_queue::Variables {};

    Ok(post_graphql::<FetchDownloadQueue>(var)
        .await?
        .download_queue)
}

pub async fn fetch_downloaded_chapters(
    cursor: Option<String>,
) -> Result<fetch_downloaded_chapters::FetchDownloadedChaptersGetDownloadedChapters, Box<dyn Error>>
{
    let var = fetch_downloaded_chapters::Variables {
        first: Some(20),
        cursor,
    };
    let data = post_graphql::<FetchDownloadedChapters>(var).await?;
    Ok(data.get_downloaded_chapters)
}

pub async fn update_chapter_priority(chapter_id: i64, priority: i64) -> Result<(), Box<dyn Error>> {
    let var = update_chapter_priority::Variables {
        id: Some(chapter_id),
        priority: Some(priority),
    };
    let _ = post_graphql::<UpdateChapterPriority>(var).await?;
    Ok(())
}

pub async fn remove_chapter_from_queue(ids: &[i64]) -> Result<(), Box<dyn Error>> {
    let var = remove_chapter_from_queue::Variables {
        ids: Some(ids.iter().map(|id| Some(*id)).collect()),
    };
    let _ = post_graphql::<RemoveChapterFromQueue>(var).await?;
    Ok(())
}

pub async fn pause_download() -> Result<bool, Box<dyn Error>> {
    let var = pause_download::Variables {};
    let data = post_graphql::<PauseDownload>(var).await?;
    Ok(data.pause_download)
}

pub async fn resume_download() -> Result<bool, Box<dyn Error>> {
    let var = resume_download::Variables {};
    let data = post_graphql::<ResumeDownload>(var).await?;
    Ok(data.resume_download)
}

pub async fn download_status() -> Result<bool, Box<dyn Error>> {
    let var = download_status::Variables {};
    let data = post_graphql::<DownloadStatus>(var).await?;
    Ok(data.download_status)
}

pub async fn myanimelist_login_start(
) -> Result<myanimelist_login_start::MyanimelistLoginStartMyanimelistLoginStart, Box<dyn Error>> {
    let var = myanimelist_login_start::Variables {};
    let data = post_graphql::<MyanimelistLoginStart>(var).await?;
    Ok(data.myanimelist_login_start)
}

pub async fn myanimelist_login_end(
    code: String,
    state: String,
    csrf_state: String,
    pkce_code_verifier: String,
) -> Result<(), Box<dyn Error>> {
    let var = myanimelist_login_end::Variables {
        code,
        state,
        csrf_state,
        pkce_code_verifier,
    };
    let _ = post_graphql::<MyanimelistLoginEnd>(var).await?;
    Ok(())
}

pub async fn tracker_logout(tracker: String) -> Result<(), Box<dyn Error>> {
    let var = tracker_logout::Variables { tracker };
    let _ = post_graphql::<TrackerLogout>(var).await?;
    Ok(())
}

pub async fn anilist_login_start(
) -> Result<anilist_login_start::AnilistLoginStartAnilistLoginStart, Box<dyn Error>> {
    let var = anilist_login_start::Variables {};
    let data = post_graphql::<AnilistLoginStart>(var).await?;
    Ok(data.anilist_login_start)
}

pub async fn anilist_login_end(code: String) -> Result<(), Box<dyn Error>> {
    let var = anilist_login_end::Variables { code };
    let _ = post_graphql::<AnilistLoginEnd>(var).await?;
    Ok(())
}

pub async fn search_tracker_manga(
    tracker: String,
    title: String,
) -> Result<Vec<search_tracker_manga::SearchTrackerMangaSearchTrackerManga>, Box<dyn Error>> {
    let var = search_tracker_manga::Variables { tracker, title };
    let data = post_graphql::<SearchTrackerManga>(var).await?;

    Ok(data.search_tracker_manga)
}

pub async fn track_manga(
    manga_id: i64,
    tracker: String,
    tracker_manga_id: String,
) -> Result<(), Box<dyn Error>> {
    let var = track_manga::Variables {
        manga_id,
        tracker,
        tracker_manga_id,
    };
    let _ = post_graphql::<TrackManga>(var).await?;

    Ok(())
}

pub async fn untrack_manga(manga_id: i64, tracker: String) -> Result<(), Box<dyn Error>> {
    let var = untrack_manga::Variables { manga_id, tracker };
    let _ = post_graphql::<UntrackManga>(var).await?;

    Ok(())
}

pub async fn fetch_manga_tracker_status(
    manga_id: i64,
) -> Result<
    Vec<fetch_manga_tracker_status::FetchMangaTrackerStatusMangaTrackerStatus>,
    Box<dyn Error>,
> {
    let var = fetch_manga_tracker_status::Variables { manga_id };
    let data = post_graphql::<FetchMangaTrackerStatus>(var).await?;

    Ok(data.manga_tracker_status)
}

pub async fn update_tracker_status(
    tracker: String,
    tracker_manga_id: String,
    status: Option<String>,
    score: Option<i64>,
    num_chapters_read: Option<i64>,
    start_date: Option<String>,
    finish_date: Option<String>,
) -> Result<(), Box<dyn Error>> {
    let var = update_tracker_status::Variables {
        tracker,
        tracker_manga_id,
        status: update_tracker_status::TrackerStatusInput {
            status,
            score,
            num_chapters_read,
            start_date,
            finish_date,
        },
    };
    let _ = post_graphql::<UpdateTrackerStatus>(var).await?;

    Ok(())
}

pub async fn refresh_chapters(manga_id: Option<i64>, wait: bool) -> Result<(), Box<dyn Error>> {
    let var = refresh_chapters::Variables { manga_id, wait };
    let _ = post_graphql::<RefreshChapters>(var).await?;

    Ok(())
}
