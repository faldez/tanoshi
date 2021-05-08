use super::component::model::{
    BackgroundColor, Claims, PageRendering, ReadingDirection, SettingParams, User,
};
use super::component::TopBar;

use serde::Deserialize;
use yew::format::{Json, Nothing, Text};
use yew::services::fetch::{FetchService, FetchTask, Request, Response};
use yew::{html, ChangeData, Component, ComponentLink, Html, InputData, Properties, ShouldRender};
use yew_router::components::RouterAnchor;
use yew_router::Switch;

use wasm_bindgen::prelude::*;
use web_sys::window;

use crate::app::browse::BrowseRoute;
use crate::app::AppRoute;

#[derive(Switch, Debug, Clone, PartialEq)]
pub enum SettingRoute {
    #[to = "/account"]
    Account,
    #[to = "/admin"]
    Admin,
    #[to = "/reading"]
    Reading,
    #[to = "/misc"]
    Misc,
    #[to = "/!"]
    Home,
}

pub struct UserRow {
    pub user: User,
    pub is_edit: bool,
    pub is_new: bool,
}

#[derive(Deserialize)]
pub struct UserListResponse {
    users: Vec<User>,
}

#[derive(Clone, Properties)]
pub struct Props {
    pub setting_page: SettingRoute,
}

pub struct Settings {
    fetch_task: Option<FetchTask>,
    link: ComponentLink<Self>,
    settings: SettingParams,
    token: String,
    is_admin: bool,
    users: Vec<UserRow>,
    me_username: String,
    me_role: String,
    me_password: Option<String>,
    me_confirm_password: Option<String>,
    change_password: bool,
    closure: Closure<dyn FnMut(JsValue)>,
    setting_page: SettingRoute,
    backend_version: String,
}

pub enum Msg {
    SetReadingDirection(ReadingDirection),
    SetBackgroundColor(BackgroundColor),
    SetPageRendering(PageRendering),
    Authorized(Claims),
    UserListReady(Vec<User>),
    NewUser,
    EditUser(usize),
    UsernameChange(usize, String),
    RoleChange(usize, ChangeData),
    ChangePassword,
    PasswordChange(InputData),
    ConfirmPasswordChange(InputData),
    SubmitPassword,
    PasswordChangedReady,
    SaveUser(usize),
    SaveUserSuccess(usize),
    ClearCache,
    DarkMode(InputData),
    VersionFetched(String),
    Noop,
}

impl Component for Settings {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let settings = SettingParams::parse_from_local_storage();
        let token = super::api::get_token().unwrap_or("".to_string());

        let closure = Closure::wrap(Box::new(move |value| {
            log::debug!("cache {:?}", value);
        }) as Box<dyn FnMut(JsValue)>);

        Settings {
            fetch_task: None,
            link,
            settings,
            token,
            is_admin: false,
            users: vec![],
            me_username: "".to_string(),
            me_role: "".to_string(),
            me_confirm_password: None,
            me_password: None,
            change_password: false,
            closure,
            setting_page: props.setting_page,
            backend_version: "".to_string(),
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::SetReadingDirection(value) => {
                self.settings.reading_direction = value;
                self.settings.save();
            }
            Msg::SetBackgroundColor(value) => {
                self.settings.background_color = value;
                self.settings.save();
            }
            Msg::SetPageRendering(value) => {
                self.settings.page_rendering = value;
                self.settings.save();
            }
            Msg::Authorized(claim) => {
                self.is_admin = claim.role == "ADMIN".to_string();
                self.me_username = claim.sub;
                self.me_role = claim.role;
                if self.is_admin {
                    self.fetch_users();
                } else {
                    self.fetch_version();
                }
            }
            Msg::UserListReady(users) => {
                self.users = users
                    .iter()
                    .map(|user| UserRow {
                        user: user.clone(),
                        is_edit: false,
                        is_new: false,
                    })
                    .collect();
                self.fetch_version();
            }
            Msg::NewUser => self.users.push(UserRow {
                user: User {
                    username: "New user".to_string(),
                    password: None,
                    role: "READER".to_string(),
                    telegram_chat_id: None,
                },
                is_edit: true,
                is_new: true,
            }),
            Msg::EditUser(i) => {
                self.users[i].is_edit = true;
            }
            Msg::UsernameChange(i, username) => {
                self.users[i].user.username = username;
            }
            Msg::RoleChange(i, e) => match e {
                ChangeData::Select(el) => {
                    self.users[i].user.role = el.value().clone();
                }
                _ => {}
            },
            Msg::ChangePassword => {
                self.change_password = !self.change_password;
            }
            Msg::PasswordChange(e) => {
                self.me_password = Some(e.value);
            }
            Msg::ConfirmPasswordChange(e) => {
                self.me_confirm_password = Some(e.value);
            }
            Msg::SubmitPassword => {
                self.change_password();
            }
            Msg::PasswordChangedReady => {
                self.me_password = None;
                self.me_confirm_password = None;
                self.change_password = false;
            }
            Msg::SaveUser(i) => {
                if self.users[i].is_new {
                    self.register_user(i);
                } else {
                    self.modify_user_role(i);
                }
            }
            Msg::SaveUserSuccess(i) => {
                self.users[i].is_edit = false;
                self.users[i].is_new = false;
            }
            Msg::ClearCache => {
                let _ = window()
                    .expect("should get window")
                    .caches()
                    .expect("should get caches")
                    .delete("tanoshi")
                    .then(&self.closure)
                    .catch(&self.closure);

                let _ = window()
                    .expect("should get window")
                    .location()
                    .reload()
                    .expect("should reload");
            }
            Msg::DarkMode(data) => {
                if data.value == "false" {
                    let _ = window()
                        .unwrap()
                        .document()
                        .unwrap()
                        .document_element()
                        .unwrap()
                        .class_list()
                        .add_1("dark");
                    self.settings.dark_mode = true;
                } else {
                    let _ = window()
                        .unwrap()
                        .document()
                        .unwrap()
                        .document_element()
                        .unwrap()
                        .class_list()
                        .remove_1("dark");
                    self.settings.dark_mode = false;
                }
                self.settings.save();
            }
            Msg::VersionFetched(version) => {
                self.backend_version = version;
            }
            Msg::Noop => {
                return false;
            }
        }
        true
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        if self.setting_page != props.setting_page {
            self.setting_page = props.setting_page;
            true
        } else {
            false
        }
    }

    fn view(&self) -> Html {
        html! {
            <div class="pb-20 pt-12" style="margin-top:env(safe-area-inset-top)">
                <TopBar>
                    <span class="w-full text-center">{"Settings"}</span>
                </TopBar>
                {
                    match &self.setting_page {
                        SettingRoute::Account => self.account_setting(),
                        SettingRoute::Admin => self.admin_settings(),
                        SettingRoute::Reading => self.reading_settings(),
                        SettingRoute::Misc => self.misc_settings(),
                        SettingRoute::Home => html!{
                            <div class="flex flex-col m-2 rounded bg-white dark:bg-gray-900 divide-y divide-gray-200 dark:divide-gray-800 border-t border-b border-gray-200 dark:border-gray-800">
                                <RouterAnchor<BrowseRoute>
                                    classes="flex inline-flex justify-center p-2 content-center hover:bg-gray-200 dark:hover:bg-gray-800"
                                    route=BrowseRoute::Settings(SettingRoute::Account)>
                                    <div class="w-full xl:w-1/2 flex justify-between text-gray-900 dark:text-gray-300">
                                        <span>{"Account"}</span>
                                        <svg viewBox="0 0 20 20" fill="currentColor" class="chevron-right w-6 h-6"><path fill-rule="evenodd" d="M7.293 14.707a1 1 0 010-1.414L10.586 10 7.293 6.707a1 1 0 011.414-1.414l4 4a1 1 0 010 1.414l-4 4a1 1 0 01-1.414 0z" clip-rule="evenodd"></path></svg>
                                    </div>
                                </RouterAnchor<BrowseRoute>>
                                <RouterAnchor<BrowseRoute>
                                    classes="flex inline-flex justify-center p-2 content-center hover:bg-gray-200 dark:hover:bg-gray-700"
                                    route=BrowseRoute::Settings(SettingRoute::Admin)>
                                    <div class="w-full xl:w-1/2 flex justify-between text-gray-900 dark:text-gray-300">
                                        <span>{"Admin"}</span>
                                        <svg viewBox="0 0 20 20" fill="currentColor" class="chevron-right w-6 h-6"><path fill-rule="evenodd" d="M7.293 14.707a1 1 0 010-1.414L10.586 10 7.293 6.707a1 1 0 011.414-1.414l4 4a1 1 0 010 1.414l-4 4a1 1 0 01-1.414 0z" clip-rule="evenodd"></path></svg>
                                    </div>
                                </RouterAnchor<BrowseRoute>>
                                <RouterAnchor<BrowseRoute>
                                    classes="flex inline-flex justify-center p-2 content-center hover:bg-gray-200 dark:hover:bg-gray-700"
                                    route=BrowseRoute::Settings(SettingRoute::Reading)>
                                    <div class="w-full xl:w-1/2 flex justify-between text-gray-900 dark:text-gray-300">
                                        <span>{"Reading"}</span>
                                        <svg viewBox="0 0 20 20" fill="currentColor" class="chevron-right w-6 h-6"><path fill-rule="evenodd" d="M7.293 14.707a1 1 0 010-1.414L10.586 10 7.293 6.707a1 1 0 011.414-1.414l4 4a1 1 0 010 1.414l-4 4a1 1 0 01-1.414 0z" clip-rule="evenodd"></path></svg>
                                    </div>
                                </RouterAnchor<BrowseRoute>>
                                <RouterAnchor<BrowseRoute>
                                    classes="flex inline-flex justify-center p-2 content-center hover:bg-gray-200 dark:hover:bg-gray-700"
                                    route=BrowseRoute::Settings(SettingRoute::Misc)>
                                    <div class="w-full xl:w-1/2 flex justify-between text-gray-900 dark:text-gray-300">
                                        <span>{"Misc"}</span>
                                        <svg viewBox="0 0 20 20" fill="currentColor" class="chevron-right w-6 h-6"><path fill-rule="evenodd" d="M7.293 14.707a1 1 0 010-1.414L10.586 10 7.293 6.707a1 1 0 011.414-1.414l4 4a1 1 0 010 1.414l-4 4a1 1 0 01-1.414 0z" clip-rule="evenodd"></path></svg>
                                    </div>
                                </RouterAnchor<BrowseRoute>>
                            </div>
                        }
                    }
                }
            </div>
        }
    }

    fn rendered(&mut self, first_render: bool) {
        if first_render {
            self.validate_token();
        }
    }
}

impl Settings {
    fn validate_token(&mut self) {
        let req = Request::get("/api/validate")
            .header("Authorization", self.token.clone())
            .body(Nothing)
            .expect("failed to build request");

        if let Ok(task) = FetchService::fetch(
            req,
            self.link
                .callback(|response: Response<Json<Result<Claims, anyhow::Error>>>| {
                    if let (meta, Json(Ok(res))) = response.into_parts() {
                        if meta.status.is_success() {
                            return Msg::Authorized(res);
                        }
                    }
                    Msg::Noop
                }),
        ) {
            self.fetch_task = Some(FetchTask::from(task));
        }
    }

    fn fetch_users(&mut self) {
        let req = Request::get("/api/user")
            .header("Authorization", self.token.clone())
            .body(Nothing)
            .expect("failed to build request");

        if let Ok(task) = FetchService::fetch(
            req,
            self.link.callback(
                |response: Response<Json<Result<UserListResponse, anyhow::Error>>>| {
                    if let (meta, Json(Ok(res))) = response.into_parts() {
                        if meta.status.is_success() {
                            return Msg::UserListReady(res.users);
                        }
                    }
                    Msg::Noop
                },
            ),
        ) {
            self.fetch_task = Some(FetchTask::from(task));
        }
    }

    fn modify_user_role(&mut self, i: usize) {
        let req = Request::put("/api/user/role")
            .header("Authorization", self.token.clone())
            .header("Content-Type", "application/json")
            .body(Json(&self.users[i].user))
            .expect("failed to build request");

        if let Ok(task) = FetchService::fetch(
            req,
            self.link.callback(move |response: Response<Text>| {
                if let (meta, Ok(_res)) = response.into_parts() {
                    if meta.status.is_success() {
                        return Msg::SaveUserSuccess(i);
                    }
                }
                Msg::Noop
            }),
        ) {
            self.fetch_task = Some(FetchTask::from(task));
        }
    }

    fn change_password(&mut self) {
        if self.me_password != self.me_confirm_password {
            return;
        }

        let req = Request::put("/api/user/password")
            .header("Authorization", self.token.clone())
            .header("Content-Type", "text/plain")
            .body(Ok(self.me_password.clone().unwrap()))
            .expect("failed to build request");

        if let Ok(task) = FetchService::fetch(
            req,
            self.link.callback(move |response: Response<Text>| {
                if let (meta, Ok(_res)) = response.into_parts() {
                    if meta.status.is_success() {
                        return Msg::PasswordChangedReady;
                    }
                }
                Msg::Noop
            }),
        ) {
            self.fetch_task = Some(FetchTask::from(task));
        }
    }

    fn register_user(&mut self, i: usize) {
        let req = Request::post("/api/register")
            .header("Authorization", self.token.clone())
            .header("Content-Type", "application/json")
            .body(Json(&self.users[i].user))
            .expect("failed to build request");

        if let Ok(task) = FetchService::fetch(
            req,
            self.link.callback(move |response: Response<Text>| {
                if let (meta, Ok(_res)) = response.into_parts() {
                    if meta.status.is_success() {
                        return Msg::SaveUserSuccess(i);
                    }
                }
                Msg::Noop
            }),
        ) {
            self.fetch_task = Some(FetchTask::from(task));
        }
    }

    fn fetch_version(&mut self) {
        let req = Request::get("/api/version")
            .body(Nothing)
            .expect("failed to build request");

        if let Ok(task) = FetchService::fetch(
            req,
            self.link.callback(|response: Response<Text>| {
                let (meta, version) = response.into_parts();
                if meta.status.is_success() {
                    Msg::VersionFetched(version.unwrap_or("".to_string()))
                } else {
                    Msg::Noop
                }
            }),
        ) {
            self.fetch_task = Some(FetchTask::from(task));
        }
    }

    fn setting_card(&self, label: &str, child: Html) -> Html {
        html! {
            <div>
                <div class="w-full xl:w-1/2 flex flex-wrap justify-between p-2 mx-auto">
                    <span class="my-auto text-gray-800 dark:text-gray-200">{label}</span>
                    {child}
                </div>
            </div>
        }
    }

    fn admin_settings(&self) -> Html {
        html! {
            <>
            <div class="flex flex-col m-2 rounded bg-white dark:bg-gray-900" id="admin">
                <div class="table w-full xl:w-1/2 mx-auto">
                    <div class="table-header-group">
                        <div class="table-row">
                            <th class="table-cell w-1/3 p-2 border-b border-gray-100 dark:border-gray-800 text-left text-gray-800 dark:text-gray-200">{"Username"}</th>
                            <th class="table-cell w-1/3 p-2 border-b border-gray-100 dark:border-gray-800 text-center text-gray-800 dark:text-gray-200">{"Role"}</th>
                            <th class="table-cell w-1/3 p-2 border-b border-gray-100 dark:border-gray-800 text-right text-gray-800 dark:text-gray-200">{"Actions"}</th>
                        </div>
                    </div>
                    <div class="table-row-group">
                    {
                    for (0..self.users.len()).map(|i| html!{
                        <div class="table-row">
                            <div class="table-cell p-2 border-b border-gray-100 dark:border-gray-800 text-left text-gray-800 dark:text-gray-200">{
                                if !self.users[i].is_edit || !self.users[i].is_new {
                                   html!{self.users[i].user.username.clone()}
                                } else {
                                    html!{
                                        <input
                                            class="w-full p-1 bg-gray-100 dark:bg-gray-800 border-b border-gray-100 dark:border-gray-800 text-gray-800 dark:text-gray-200 focus:outline-none"
                                            value=self.users[i].user.username.clone()
                                            oninput=self.link.callback(move |e: InputData| Msg::UsernameChange(i, e.value))/>
                                    }
                                }
                            }</div>
                            <div class="table-cell p-2 border-b border-gray-100 dark:border-gray-800 text-center text-gray-800 dark:text-gray-200">
                            {
                                if !self.users[i].is_edit {
                                    html!{self.users[i].user.role.clone()}
                                } else {
                                    html!{
                                        <select class="rounded bg-gray-100 dark:bg-gray-700 p-1" onchange=self.link.callback(move |e: ChangeData| Msg::RoleChange(i, e))>
                                            <option class="bg-gray-100 dark:bg-gray-700" value="READER" selected={self.users[i].user.role.clone() == "READER".to_string()}>{"READER"}</option>
                                            <option class="bg-gray-100 dark:bg-gray-700" value="ADMIN" selected={self.users[i].user.role.clone() == "ADMIN".to_string()}>{"ADMIN"}</option>
                                        </select>
                                    }
                                }
                            }
                            </div>
                            <div class="table-cell p-2 border-b border-gray-200 dark:border-gray-800 text-right text-gray-800 dark:text-gray-200">
                                <button
                                    class="text-gray-800 dark:text-gray-200 hover:text-accent font-bold p-1 rounded focus:outline-none"
                                    onclick={
                                        if !self.users[i].is_edit {
                                            self.link.callback(move |_| Msg::EditUser(i))
                                        } else {
                                            self.link.callback(move |_| Msg::SaveUser(i))
                                        }
                                    }>
                                    {
                                        if !self.users[i].is_edit {
                                            html!{
                                                <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
                                                </svg>
                                            }
                                        } else {
                                            html!{
                                                <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
                                                </svg>
                                            }
                                        }
                                    }
                                </button>
                            </div>
                        </div>
                    })
                    }
                    </div>
                </div>
            </div>
            <button class="w-full xl:w-1/2 flex justify-center mt-2 mx-auto rounded border border-gray-100 dark:border-gray-900 shadow dark:shadow-none bg-white dark:bg-gray-900 hover:bg-gray-100 dark:hover:bg-gray-900 text-gray-800 dark:text-gray-200 font-semibold py-2 px-4 focus:outline-none"
                onclick=self.link.callback(|_| Msg::NewUser)>
                {"New User"}
            </button>
            </>
        }
    }

    fn account_setting(&self) -> Html {
        html! {
            <>
            <div class="flex flex-col p-2 rounded bg-white dark:bg-gray-900 divide-y divide-gray-100 dark:divide-gray-800" id="account-setting">
                {self.setting_card("Username", html! {
                    <span class="text-gray-800 dark:text-gray-200">{self.me_username.clone()}</span>
                })}
                {self.setting_card("Role", html! {
                    <span class="text-gray-800 dark:text-gray-200">{self.me_role.clone()}</span>
                })}
                {
                    if self.change_password {
                        html!{
                            <>
                            {self.setting_card("New Password", html! {
                                <div class="flex">
                                <input
                                    class="w-full p-1 bg-gray-100 dark:bg-gray-800 text-gray-800 dark:text-gray-200 focus:outline-none"
                                    type="password"
                                    value=self.me_password.clone().unwrap_or("".to_string()).to_owned()
                                    oninput=self.link.callback(|e| Msg::PasswordChange(e))/>
                                </div>
                            })}
                            {self.setting_card("Confirm Password", html! {
                                <div class="flex flex-col">
                                <input
                                    class="w-full p-1 bg-gray-100 dark:bg-gray-800 text-gray-800 dark:text-gray-200 focus:outline-none"
                                    type="password"
                                    value=self.me_confirm_password.clone().unwrap_or("".to_string()).to_owned()
                                    oninput=self.link.callback(|e| Msg::ConfirmPasswordChange(e))/>
                                {
                                    if self.me_password != self.me_confirm_password {
                                        html!{<span class="text-xs text-red-500">{"Password doesn't match"}</span>}
                                    }
                                    else {
                                        html!{}
                                    }
                                }
                                </div>
                            })}
                            <button class={"w-full xl:w-1/2 bg-white dark:bg-gray-800 shadow dark:shadow-none hover:bg-gray-100 dark:hover:bg-gray-700 text-gray-800 dark:text-gray-200 mx-auto py-2 px-4 border border-gray-100 dark:border-gray-900 focus:outline-none"}
                                onclick=self.link.callback(|_| Msg::SubmitPassword)>
                                {"Submit"}
                            </button>
                            </>
                        }
                    } else {
                        html!{}
                    }
                }
            </div>
            <button class={"w-full xl:w-1/2 flex justify-center bg-white dark:bg-gray-900 hover:bg-gray-100 dark:hover:bg-gray-700 text-gray-800 dark:text-gray-200 font-semibold mx-auto py-2 px-4 rounded shadow dark:shadow-none focus:outline-none"}
                onclick=self.link.callback(|_| Msg::ChangePassword)>
                {if !self.change_password {"Change Password"} else {"Cancel"}}
            </button>
            <RouterAnchor<AppRoute> route=AppRoute::Logout classes={"w-full xl:w-1/2 flex justify-center mt-2 bg-white dark:bg-gray-900 hover:bg-gray-100 dark:hover:bg-gray-700 mx-auto py-2 px-4 rounded shadow dark:shadow-none"}>
                <span class="text-red-600 font-semibold text-center">{"Sign Out"}</span>
            </RouterAnchor<AppRoute>>
            </>
        }
    }

    fn reading_settings(&self) -> Html {
        html! {
            <div class="bg-white dark:bg-gray-900 flex flex-col rounded m-2 divide-y divide-gray-200 dark:divide-gray-800" id="reading-setting">
                {
                    self.setting_card("Direction", html! {
                        <div class="w-full flex flex-grow">
                            <button class={
                                format!("{} hover:bg-gray-400 text-gray-700 dark:text-gray-300 px-2 w-full rounded-l focus:outline-none",
                                if self.settings.reading_direction == ReadingDirection::RightToLeft { "bg-gray-400 dark:bg-gray-700" } else {"bg-gray-300 dark:bg-gray-800"})}
                                onclick=self.link.callback(|_| Msg::SetReadingDirection(ReadingDirection::RightToLeft))>
                                {"Right to Left"}
                            </button>
                            <button class={
                                format!("{} hover:bg-gray-400 text-gray-700 dark:text-gray-300 px-2 w-full rounded-r focus:outline-none",
                                if self.settings.reading_direction == ReadingDirection::LeftToRight { "bg-gray-400 dark:bg-gray-700" } else {"bg-gray-300 dark:bg-gray-800"})}
                                onclick=self.link.callback(|_| Msg::SetReadingDirection(ReadingDirection::LeftToRight))>
                                {"Left to Right"}
                            </button>
                        </div>
                    })
                }
                {
                    self.setting_card("Background", html! {
                        <div class="inline-flex">
                            <button class={
                                format!("{} hover:bg-gray-400 text-gray-700 dark:text-gray-300 px-2 w-full rounded-l focus:outline-none",
                                if self.settings.background_color == BackgroundColor::White { "bg-gray-400 dark:bg-gray-700" } else {"bg-gray-300 dark:bg-gray-800"})}
                                onclick=self.link.callback(|_| Msg::SetBackgroundColor(BackgroundColor::White))>
                                {"White"}
                            </button>
                            <button class={
                                format!("{} hover:bg-gray-400 text-gray-700 dark:text-gray-300 px-2 w-full rounded-r focus:outline-none",
                                if self.settings.background_color == BackgroundColor::Black { "bg-gray-400 dark:bg-gray-700" } else {"bg-gray-300 dark:bg-gray-800"})}
                                onclick=self.link.callback(|_| Msg::SetBackgroundColor(BackgroundColor::Black))>
                                {"Black"}
                            </button>
                        </div>
                    })
                }
                {
                    self.setting_card("Mode", html! {
                        <div class="inline-flex">
                            <button class={
                                format!("{} hover:bg-gray-400 text-gray-700 dark:text-gray-300 px-2 w-full rounded-l focus:outline-none",
                                if self.settings.page_rendering == PageRendering::SinglePage { "bg-gray-400 dark:bg-gray-700" } else {"bg-gray-300 dark:bg-gray-800"})}
                                onclick=self.link.callback(|_| Msg::SetPageRendering(PageRendering::SinglePage))>
                                {"Single"}
                            </button>
                            <button class={
                                format!("{} hover:bg-gray-400 text-gray-700 dark:text-gray-300 px-2 w-full focus:outline-none",
                                if self.settings.page_rendering == PageRendering::DoublePage { "bg-gray-400 dark:bg-gray-700" } else {"bg-gray-300 dark:bg-gray-800"})}
                                onclick=self.link.callback(|_| Msg::SetPageRendering(PageRendering::DoublePage))>
                                {"Double"}
                            </button>
                            <button class={
                                 format!("{} hover:bg-gray-400 text-gray-700 dark:text-gray-300 px-2 w-full rounded-r focus:outline-none",
                                 if self.settings.page_rendering == PageRendering::LongStrip { "bg-gray-400 dark:bg-gray-700" } else {"bg-gray-300 dark:bg-gray-800"})}
                                 onclick=self.link.callback(|_| Msg::SetPageRendering(PageRendering::LongStrip))>
                                 {"Webtoon"}
                             </button>
                        </div>
                    })
                }
            </div>
        }
    }

    fn misc_settings(&self) -> Html {
        html! {
            <div class="bg-white dark:bg-gray-900 flex flex-col border-b border-t border-gray-300 dark:border-gray-700 divide-y divide-gray-300 dark:divide-gray-700" id="misc-setting">
                {
                    self.setting_card("Dark Mode", html! {
                    <div class="relative inline-block w-10 mr-2 align-middle select-none transition duration-200 ease-in">
                        <input type="checkbox" name="toggle" id="toggle" class="toggle-checkbox absolute block w-6 h-6 rounded-full bg-white border-4 appearance-none cursor-pointer focus:outline-none"
                        value={self.settings.dark_mode} checked={self.settings.dark_mode} oninput=self.link.callback(|e| Msg::DarkMode(e))/>
                        <label for="toggle" class="toggle-label block overflow-hidden h-6 rounded-full bg-gray-300 cursor-pointer"></label>
                    </div>
                    })
                }
                {
                    self.setting_card("Web Version", html! {
                        <span class={"text-gray-800 dark:text-gray-200"}>
                            {super::VERSION}
                        </span>
                    })
                }
                {
                    self.setting_card("Backend Version", html! {
                        <span class={"text-gray-800 dark:text-gray-200"}>
                            {&self.backend_version}
                        </span>
                    })
                }
                {
                    self.setting_card("Clear Cache", html! {
                        <button class={"bg-gray-300 hover:bg-gray-400 text-gray-800 font-bold py-1 px-2 rounded-l focus:outline-none"}
                            onclick=self.link.callback(|_| Msg::ClearCache)>
                            {"Clear"}
                        </button>
                    })
                }
            </div>
        }
    }
}
