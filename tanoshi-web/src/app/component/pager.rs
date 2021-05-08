use yew::prelude::*;
use crate::app::component::model::{SettingParams, ReadingDirection, PageRendering};
use crate::app::reader::{Reader, Msg as ReaderMsg};
use crate::app::component::{WeakComponentLink, Page};

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    pub current_page: usize,
    pub pages: Vec<String>,
    pub settings: SettingParams,
    pub reader_link: WeakComponentLink<Reader>,
    #[prop_or_default]
    pub on_page_change: Callback<usize>,
    #[prop_or_default]
    pub on_next_chapter: Callback<()>,
    #[prop_or_default]
    pub on_prev_chapter: Callback<()>,
}

pub struct Pager {
    props: Props,
    link: ComponentLink<Self>,
    page_refs: Vec<NodeRef>,
}

pub enum Msg {
    PageForward,
    PagePrevious,
    Noop,
}

impl Component for Pager {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Pager{
            props,
            link,
            page_refs: vec![],
        }
    }

    fn update(&mut self, msg: Self::Message) -> bool {
        match msg {
            Msg::PageForward => {
                if self.props.settings.page_rendering == PageRendering::LongStrip {
                    self.next_page_or_chapter();
                } else {
                    if self.props.settings.reading_direction == ReadingDirection::LeftToRight {
                        self.next_page_or_chapter();
                    } else {
                        self.prev_page_or_chapter();
                    }
                }
                self.props.on_page_change.emit(self.props.current_page);
            }
            Msg::PagePrevious => {
                if self.props.settings.page_rendering == PageRendering::LongStrip {
                    self.prev_page_or_chapter();
                } else {
                    if self.props.settings.reading_direction == ReadingDirection::LeftToRight {
                        self.prev_page_or_chapter();
                    } else {
                        self.next_page_or_chapter();
                    }
                }
                self.props.on_page_change.emit(self.props.current_page);
            }
            Msg::Noop => { return false; }
        }
        true
    }

    fn change(&mut self, props: Self::Properties) -> bool {
        if self.props != props {
            self.props = props;
            self.page_refs.resize(self.props.pages.len(), NodeRef::default());
            true
        } else {
            false
        }
    }

    fn view(&self) -> Html {
        let class = vec![
            "flex",
            "justify-center",
            "overflow-auto",
            "h-screen",
            match self.props.settings.reading_direction {
                ReadingDirection::LeftToRight => "",
                ReadingDirection::RightToLeft => "flex-row-reverse",
            },
        ];
        let should_page_hide = self.props.pages.iter().enumerate().map(|(i, _)| self.should_hide(i)).collect::<Vec<bool>>();
        let reader_link = &self.props.reader_link.borrow().clone().unwrap();
        html! {
            <div class="h-screen m-0 outline-none" id="manga-reader" tabindex="0" onkeydown=self.link.callback(|e: KeyboardEvent|
                match e.key().as_str() {
                    "ArrowRight" => Msg::PageForward,
                    "ArrowLeft"  => Msg::PagePrevious,
                    _ => Msg::Noop,
                }
            )>
                <button class="manga-navigate-left outline-none fixed focus:outline-none" onmouseup=self.link.callback(|_| Msg::PagePrevious)/>
                <button class="manga-navigate-center outline-none fixed focus:outline-none" onmouseup=reader_link.callback(|_| ReaderMsg::ToggleBar)/>
                <button class="manga-navigate-right outline-none fixed focus:outline-none" onmouseup=self.link.callback(|_| Msg::PageForward)/>
                <div class={class}>
                    {
                        for self.props.pages
                            .clone()
                            .into_iter()
                            .enumerate()
                            .map(|(i, page)| {
                                html! {
                                    <Page
                                        id={i}
                                        key={i}
                                        hidden={should_page_hide.get(i).unwrap_or(&false)}
                                        page_ref=self.page_refs[i].clone()
                                        page_rendering={&self.props.settings.page_rendering}
                                        reading_direction={&self.props.settings.reading_direction}
                                        src={self.page_or_empty(i, &page)}
                                    />
                                }
                            })
                    }
                </div>
            </div>
        }
    }
}

impl Pager {
    fn should_hide(&self, i: usize) -> bool {
        match self.props.settings.page_rendering {
            PageRendering::SinglePage => {
                i != self.props.current_page
            }
            PageRendering::DoublePage => {
                (i != self.props.current_page) && (i != self.props.current_page + 1)
            }
            _ => {
                false
            }
        }
    }

    fn page_or_empty(&self, i: usize, page: &String) -> String {
        let (before, after) = match self.props.settings.page_rendering {
            PageRendering::DoublePage => (2, 4),
            _ => (1, 2),
        };

        if i >= self.props.current_page.checked_sub(before).unwrap_or(0) && i < self.props.current_page + after
        {
            page.to_string()
        } else {
            "".to_string()
        }
    }

    fn next_page_or_chapter(&mut self) {
        let mut num = 1;
        if self.props.settings.page_rendering == PageRendering::DoublePage {
            num = 2;
        }

        let mut current_page = self.props.current_page + num;
        current_page = match self.props.pages.get(current_page) {
            Some(_) => current_page,
            None => 0,
        };

        if current_page == 0 {
            self.props.on_next_chapter.emit(());
        } else {
            self.props.current_page = current_page;
        }
    }

    fn prev_page_or_chapter(&mut self) {
        let mut num: usize = 1;
        if self.props.settings.page_rendering == PageRendering::DoublePage {
            num = 2;
        }

        let is_prev = match self.props.current_page.checked_sub(num) {
            Some(page) => {
                self.props.current_page = page;
                false
            }
            None => true,
        };

        if is_prev {
            self.props.on_prev_chapter.emit(());
        }
    }
}