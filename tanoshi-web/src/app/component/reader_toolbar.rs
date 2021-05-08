use crate::app::browse::BrowseRoute;
use crate::app::AppRoute;
use web_sys::{HtmlElement, MouseEvent};
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    pub visible: bool,
    pub manga_id: i32,
    pub title: String,
    pub chapter: String,
    pub on_refresh: Callback<MouseEvent>,
}

pub struct ReaderToolbar {
    props: Props,
    root_ref: NodeRef,
    title_ref: NodeRef,
}

pub enum Msg {}

impl Component for ReaderToolbar {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        ReaderToolbar {
            props,
            root_ref: NodeRef::default(),
            title_ref: NodeRef::default(),
        }
    }

    fn update(&mut self, _msg: Self::Message) -> bool {
        false
    }

    fn change(&mut self, props: Self::Properties) -> bool {
        if self.props != props {
            self.props = props;

            if let Some(title) = self.title_ref.cast::<HtmlElement>() {
                let _ = title.set_inner_html(self.props.title.as_str());
            }
            if !self.props.visible {
                if let Some(bar) = self.root_ref.cast::<HtmlElement>() {
                    bar.class_list()
                        .remove_1("slideInDown")
                        .expect("failed remove class");
                    bar.class_list()
                        .add_1("slideOutUp")
                        .expect("failed add class");
                }
            } else {
                if let Some(bar) = self.root_ref.cast::<HtmlElement>() {
                    bar.class_list()
                        .remove_1("slideOutUp")
                        .expect("failed remove class");
                    bar.class_list()
                        .add_1("slideInDown")
                        .expect("failed add class");
                }
            }
            true
        } else {
            false
        }
    }

    fn view(&self) -> Html {
        html! {
            <div ref=self.root_ref.clone()
                class="flex justify-between items-center animated slideInDown faster block fixed inset-x-0 top-0 z-50 bg-white dark:bg-gray-900 z-50 content-end opacity-75 text-black dark:text-white shadow dark:shadow-none"
                style="padding-top: calc(env(safe-area-inset-top) + .5rem)">
                <RouterAnchor<AppRoute> classes="z-50 mx-2 mb-2" route=AppRoute::Browse(BrowseRoute::Detail(self.props.manga_id))>
                    <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                    </svg>
                </RouterAnchor<AppRoute>>
                <div class="flex flex-col mx-2 mb-2">
                    <span ref=self.title_ref.clone() class="text-center truncate"></span>
                    <span class="text-center text-sm">{&self.props.chapter}</span>
                </div>
                <div></div>
                // <button
                //     onclick={&self.props.on_refresh}
                //     class="z-50 mx-2 mb-2">
                //     <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                //         <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 5v.01M12 12v.01M12 19v.01M12 6a1 1 0 110-2 1 1 0 010 2zm0 7a1 1 0 110-2 1 1 0 010 2zm0 7a1 1 0 110-2 1 1 0 010 2z" />
                //     </svg>
                // </button>
            </div>
        }
    }
}
