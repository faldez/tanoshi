use web_sys::HtmlElement;
use yew::prelude::*;

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    pub visible: bool,
    pub disabled: bool,
    pub length: usize,
    pub current_page: usize,
    #[prop_or_default]
    pub on_next_chapter: Callback<()>,
    #[prop_or_default]
    pub on_prev_chapter: Callback<()>,
    pub on_input: Callback<InputData>,
}

pub struct ReaderSeekbar {
    props: Props,
    link: ComponentLink<Self>,
    root_ref: NodeRef,
}

pub enum Msg {
    PrevChapter,
    NextChapter
}

impl Component for ReaderSeekbar {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        ReaderSeekbar {
            props,
            link,
            root_ref: NodeRef::default(),
        }
    }

    fn update(&mut self, msg: Self::Message) -> bool {
        match msg {
            Msg::PrevChapter => {
                self.props.on_prev_chapter.emit(());
                return true;
            }
            Msg::NextChapter => {
                self.props.on_next_chapter.emit(());
                return true;
            }
        }
    }

    fn change(&mut self, props: Self::Properties) -> bool {
        if self.props != props {
            self.props = props;
            if !self.props.visible {
                if let Some(bar) = self.root_ref.cast::<HtmlElement>() {
                    bar.class_list()
                        .remove_1("slideInUp")
                        .expect("failed remove class");
                    bar.class_list()
                        .add_1("slideOutDown")
                        .expect("failed add class");
                }
            } else {
                if let Some(bar) = self.root_ref.cast::<HtmlElement>() {
                    bar.class_list()
                        .remove_1("slideOutDown")
                        .expect("failed remove class");
                    bar.class_list()
                        .add_1("slideInUp")
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
                class="animated slideInUp faster block fixed inset-x-0 bottom-0 z-50 bg-white dark:bg-gray-900 opacity-75 shadow pb-safe-bottom text-black dark:text-white shadow dark:shadow-none">
                <div class="flex px-4 py-5 justify-center">
                    <button onclick=self.link.callback(|_| Msg::PrevChapter) class="focus:outline-none">
                        <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6 stroke-current" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
                        </svg>
                    </button>
                    <div class="w-full"></div>
                    <span class="mx-4">{self.props.current_page + 1}</span>
                    <span>{"/"}</span>
                    <span class="mx-4">{self.props.length}</span>
                    <div class="w-full"></div>
                    <button onclick=self.link.callback(|_| Msg::NextChapter) class="focus:outline-none">
                        <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6 stroke-current" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
                        </svg>
                    </button>
                </div>
            </div>
        }
    }
}
