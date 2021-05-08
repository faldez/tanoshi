use tanoshi_lib::manga::{SortByParam, SortOrderParam};
use super::Modal;
use yew::prelude::*;

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    pub show: bool,
    pub onsortbychange: Callback<SortByParam>,
    pub onsortorderchange: Callback<SortOrderParam>,
    pub onclose: Callback<()>,
    pub oncancel: Callback<()>,
    #[prop_or_default]
    pub sort_by: SortByParam,
    #[prop_or_default]
    pub sort_order: SortOrderParam,
}

pub struct Filter {
    link: ComponentLink<Self>,
    props: Props,
}

pub enum Msg {
    SortClick(SortByParam),
}

impl Component for Filter {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Filter {
            link,
            props,
        }
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        if self.props != props {
            self.props = props;
            true
        } else {
            false
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::SortClick(sort_by) => {
                if self.props.sort_by == sort_by {
                    match self.props.sort_order {
                        SortOrderParam::Asc => self.props.sort_order = SortOrderParam::Desc,
                        SortOrderParam::Desc => self.props.sort_order = SortOrderParam::Asc,
                    }
                    self.props.onsortorderchange.emit(self.props.sort_order.clone());
                } else {
                    self.props.sort_by = sort_by;
                    self.props.onsortbychange.emit(self.props.sort_by.clone());
                }
            }
        }
        true
    }

    fn view(&self) -> Html {
        html! {
            <Modal
                show={self.props.show}
                onclose={&self.props.onclose}
                oncancel={&self.props.oncancel}>
                <div class="w-full shadow py-1 px-4 dark:text-gray-300 text-gray-700">{"Sort By"}</div>
                <button class="inline-flex justify-start p-2 text-gray-700 dark:text-gray-300 focus:outline-none" onclick=self.link.callback(|_| Msg::SortClick(SortByParam::LastUpdated))>
                    {
                        match self.props.sort_by {
                            SortByParam::LastUpdated  => self.asc_or_desc(),
                            _ => self.empty_svg(),
                        }
                    }
                    <span class="text-gray-700 dark:text-gray-300">{"Last Updated"}</span>
                </button>
                <button class="inline-flex justify-start p-2 text-gray-700 dark:text-gray-300 focus:outline-none" onclick=self.link.callback(|_| Msg::SortClick(SortByParam::Title))>
                    {
                        match self.props.sort_by {
                            SortByParam::Title  => self.asc_or_desc(),
                            _ => self.empty_svg(),
                        }
                    }
                    <span class="dark:text-gray-300 text-gray-700">{"Title"}</span>
                </button>
                <button class="inline-flex justify-start p-2 text-gray-700 dark:text-gray-300 focus:outline-none" onclick=self.link.callback(|_| Msg::SortClick(SortByParam::Comment))>
                    {
                        match self.props.sort_by {
                            SortByParam::Comment  => self.asc_or_desc(),
                            _ => self.empty_svg(),
                        }
                    }
                    <span class="dark:text-gray-300 text-gray-700 dark:text-gray-300">{"Comment"}</span>
                </button>
                <button class="inline-flex justify-start p-2 text-gray-700 dark:text-gray-300 focus:outline-none" onclick=self.link.callback(|_| Msg::SortClick(SortByParam::Views))>
                    {
                        match self.props.sort_by {
                            SortByParam::Views  => self.asc_or_desc(),
                            _ => self.empty_svg(),
                        }
                    }
                    <span class="text-gray-700 dark:text-gray-300">{"Views"}</span>
                </button>
            </Modal>
        }
    }
}

impl Filter {
    fn asc_or_desc(&self) -> Html {
        match self.props.sort_order {
            SortOrderParam::Asc => html! {
                <svg fill="currentColor" viewBox="0 0 24 24" width="24" height="24" stroke="currentColor" class="mx-2 self-center flex-none"><path fill-rule="evenodd" d="M3.293 9.707a1 1 0 010-1.414l6-6a1 1 0 011.414 0l6 6a1 1 0 01-1.414 1.414L11 5.414V17a1 1 0 11-2 0V5.414L4.707 9.707a1 1 0 01-1.414 0z" clip-rule="evenodd"></path></svg>
            },
            SortOrderParam::Desc => html! {
                <svg fill="currentColor" viewBox="0 0 24 24" width="24" height="24" stroke="currentColor" class="mx-2 self-center flex-none"><path fill-rule="evenodd" d="M16.707 10.293a1 1 0 010 1.414l-6 6a1 1 0 01-1.414 0l-6-6a1 1 0 111.414-1.414L9 14.586V3a1 1 0 012 0v11.586l4.293-4.293a1 1 0 011.414 0z" clip-rule="evenodd"></path></svg>
            },
        }
    }

    fn empty_svg(&self) -> Html {
        html! {
            <svg viewBox="0 0 24 24" width="24" height="24" xmlns="http://www.w3.org/2000/svg" version="1.1" class="mx-2 self-center flex-none">
                <rect width="24" height="24" fill-opacity="0.0" stroke-opacity="0.0"/>
            </svg>
        }
    }
}
