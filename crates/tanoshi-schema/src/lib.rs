pub mod model;

use graphql_client::GraphQLQuery;
use model::Input;

type NaiveDateTime = String;

type InputList = Vec<Input>;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/browse_source.graphql",
    response_derives = "Debug, Clone"
)]
pub struct BrowseSource;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/get_latest_manga.graphql",
    response_derives = "Debug, Clone, PartialEq, Eq"
)]
pub struct GetLatestManga;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/get_popular_manga.graphql",
    response_derives = "Debug, Clone"
)]
pub struct GetPopularManga;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/fetch_source_filters.graphql",
    response_derives = "Debug, Clone"
)]
pub struct FetchSourceFilters;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/browse_favorites.graphql",
    response_derives = "Debug"
)]
pub struct BrowseFavorites;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/fetch_manga_by_source_path.graphql",
    response_derives = "Debug"
)]
pub struct FetchMangaBySourcePath;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/fetch_manga_detail.graphql",
    response_derives = "Debug"
)]
pub struct FetchMangaDetail;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/fetch_chapter.graphql",
    response_derives = "Debug"
)]
pub struct FetchChapter;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/add_to_library.graphql",
    response_derives = "Debug"
)]
pub struct AddToLibrary;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/delete_from_library.graphql",
    response_derives = "Debug"
)]
pub struct DeleteFromLibrary;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/fetch_category_detail.graphql",
    response_derives = "Debug"
)]
pub struct FetchCategoryDetail;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/fetch_categories.graphql",
    response_derives = "Debug"
)]
pub struct FetchCategories;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/create_category.graphql",
    response_derives = "Debug"
)]
pub struct CreateCategory;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/update_category.graphql",
    response_derives = "Debug"
)]
pub struct UpdateCategory;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/delete_category.graphql",
    response_derives = "Debug"
)]
pub struct DeleteCategory;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/update_page_read_at.graphql",
    response_derives = "Debug"
)]
pub struct UpdatePageReadAt;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/fetch_recent_updates.graphql",
    response_derives = "Debug"
)]
pub struct FetchRecentUpdates;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/subscribe_chapter_updates.graphql",
    response_derives = "Debug"
)]
pub struct SubscribeChapterUpdates;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/fetch_histories.graphql",
    response_derives = "Debug"
)]
pub struct FetchHistories;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/fetch_sources.graphql",
    response_derives = "Debug"
)]
pub struct FetchSources;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/fetch_all_sources.graphql",
    response_derives = "Debug"
)]
pub struct FetchAllSources;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/fetch_source.graphql",
    response_derives = "Debug"
)]
pub struct FetchSourceDetail;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/set_preferences.graphql",
    response_derives = "Debug"
)]
pub struct SetPreferences;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/install_source.graphql",
    response_derives = "Debug"
)]
pub struct InstallSource;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/update_source.graphql",
    response_derives = "Debug"
)]
pub struct UpdateSource;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/uninstall_source.graphql",
    response_derives = "Debug"
)]
pub struct UninstallSource;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/login.graphql",
    response_derives = "Debug"
)]
pub struct UserLogin;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/fetch_users.graphql",
    response_derives = "Debug"
)]
pub struct FetchUserList;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/fetch_me.graphql",
    response_derives = "Debug"
)]
pub struct FetchMe;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/register.graphql",
    response_derives = "Debug"
)]
pub struct UserRegister;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/delete_user.graphql",
    response_derives = "Debug"
)]
pub struct DeleteUser;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/change_password.graphql",
    response_derives = "Debug"
)]
pub struct ChangeUserPassword;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/update_profile.graphql",
    response_derives = "Debug"
)]
pub struct UpdateProfile;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/fetch_server_status.graphql",
    response_derives = "Debug"
)]
pub struct FetchServerStatus;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/test_telegram.graphql",
    response_derives = "Debug"
)]
pub struct TestTelegram;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/test_pushover.graphql",
    response_derives = "Debug"
)]
pub struct TestPushover;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/test_gotify.graphql",
    response_derives = "Debug"
)]
pub struct TestGotify;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/test_desktop_notification.graphql",
    response_derives = "Debug"
)]
pub struct TestDesktopNotification;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/mark_chapter_as_read.graphql",
    response_derives = "Debug"
)]
pub struct MarkChapterAsRead;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/mark_chapter_as_unread.graphql",
    response_derives = "Debug"
)]
pub struct MarkChapterAsUnread;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/download_chapters.graphql",
    response_derives = "Debug"
)]
pub struct DownloadChapters;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/remove_downloaded_chapters.graphql",
    response_derives = "Debug"
)]
pub struct RemoveDownloadedChapters;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/fetch_download_queue.graphql",
    response_derives = "Debug"
)]
pub struct FetchDownloadQueue;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/fetch_downloaded_chapters.graphql",
    response_derives = "Debug"
)]
pub struct FetchDownloadedChapters;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/update_chapter_priority.graphql",
    response_derives = "Debug"
)]
pub struct UpdateChapterPriority;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/remove_chapter_from_queue.graphql",
    response_derives = "Debug"
)]
pub struct RemoveChapterFromQueue;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/pause_download.graphql",
    response_derives = "Debug"
)]
pub struct PauseDownload;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/resume_download.graphql",
    response_derives = "Debug"
)]
pub struct ResumeDownload;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/download_status.graphql",
    response_derives = "Debug"
)]
pub struct DownloadStatus;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/myanimelist_login_start.graphql",
    response_derives = "Debug"
)]
pub struct MyanimelistLoginStart;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/myanimelist_login_end.graphql",
    response_derives = "Debug"
)]
pub struct MyanimelistLoginEnd;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/tracker_logout.graphql",
    response_derives = "Debug"
)]
pub struct TrackerLogout;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/anilist_login_start.graphql",
    response_derives = "Debug"
)]
pub struct AnilistLoginStart;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/anilist_login_end.graphql",
    response_derives = "Debug"
)]
pub struct AnilistLoginEnd;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/search_tracker_manga.graphql",
    response_derives = "Debug"
)]
pub struct SearchTrackerManga;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/track_manga.graphql",
    response_derives = "Debug"
)]
pub struct TrackManga;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/untrack_manga.graphql",
    response_derives = "Debug"
)]
pub struct UntrackManga;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/fetch_manga_tracker_status.graphql",
    response_derives = "Debug"
)]
pub struct FetchMangaTrackerStatus;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/update_tracker_status.graphql",
    response_derives = "Debug"
)]
pub struct UpdateTrackerStatus;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/refresh_chapters.graphql",
    response_derives = "Debug"
)]
pub struct RefreshChapters;
