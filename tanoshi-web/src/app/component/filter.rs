use tanoshi_lib::manga::{SortByParam, SortOrderParam};
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
    node_ref: NodeRef,
    classes: Vec<&'static str>,
}

pub enum Msg {
    SortClick(SortByParam),
    Done,
    Cancel,
}

impl Component for Filter {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let classes = vec!["hidden"];
        Filter {
            link,
            props,
            node_ref: NodeRef::default(),
            classes,
        }
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        if self.props != props {
            if self.props.show != props.show {
                if self.classes[0] == "hidden" {
                    self.classes = vec![
                        "animated",
                        "faster",
                        "fixed",
                        "h-1/2",
                        "z-25",
                        "inset-x-0",
                        "bottom-0",
                        "mx-auto",
                        "h-auto",
                        "w-full",
                        "lg:w-1/2",
                        "lg:w-1/2",
                        "rounded-t-md",
                        "bg-white",
                        "border",
                        "border-t",
                        "border-l",
                        "border-r",
                        "border-gray-300",
                        "dark:border-gray-700",
                        "safe-bottom",
                        "flex",
                        "flex-col",
                        "bg-white",
                        "dark:bg-gray-900"
                    ];
                }
            }
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
            Msg::Done => {
                self.props.onclose.emit(());
            }
            Msg::Cancel => {
                self.props.oncancel.emit(());
            }
        }
        true
    }

    fn view(&self) -> Html {
        html! {
            <div id="filter" ref={self.node_ref.clone()} class={self.classes()}>
                <div class="absolute w-full shadow p-2 flex justify-between">
                    <button class="flex rounded text-accent dark:text-accent-lighter py-1 px-2 justify-center" onclick=self.link.callback(|_| Msg::Cancel)>{"Cancel"}</button>
                    <button class="flex rounded bg-accent text-white py-1 px-2 shadow justify-center" onclick=self.link.callback(|_| Msg::Done)>{"Search"}</button>
                </div>
                <div class="w-full max-w-full flex flex-col mx-auto mt-12">
                    <div class="w-full shadow  py-1 px-4 dark:text-gray-300 text-gray-700">{"Sort By"}</div>
                    <button class="inline-flex justify-center p-2 text-gray-700 dark:text-gray-300" onclick=self.link.callback(|_| Msg::SortClick(SortByParam::LastUpdated))>
                        {
                            match self.props.sort_by {
                                SortByParam::LastUpdated  => self.asc_or_desc(),
                                _ => self.empty_svg(),
                            }
                        }
                        <span class="text-gray-700 dark:text-gray-300">{"Last Updated"}</span>
                    </button>
                    <button class="inline-flex justify-center p-2 text-gray-700 dark:text-gray-300" onclick=self.link.callback(|_| Msg::SortClick(SortByParam::Title))>
                        {
                            match self.props.sort_by {
                                SortByParam::Title  => self.asc_or_desc(),
                                _ => self.empty_svg(),
                            }
                        }
                        <span class="dark:text-gray-300 text-gray-700">{"Title"}</span>
                    </button>
                    <button class="inline-flex justify-center p-2 text-gray-700 dark:text-gray-300" onclick=self.link.callback(|_| Msg::SortClick(SortByParam::Comment))>
                        {
                            match self.props.sort_by {
                                SortByParam::Comment  => self.asc_or_desc(),
                                _ => self.empty_svg(),
                            }
                        }
                        <span class="dark:text-gray-300 text-gray-700 dark:text-gray-300">{"Comment"}</span>
                    </button>
                    <button class="inline-flex justify-center p-2 text-gray-700 dark:text-gray-300" onclick=self.link.callback(|_| Msg::SortClick(SortByParam::Views))>
                        {
                            match self.props.sort_by {
                                SortByParam::Views  => self.asc_or_desc(),
                                _ => self.empty_svg(),
                            }
                        }
                        <span class="text-gray-700 dark:text-gray-300">{"Views"}</span>
                    </button>
                </div>
            </div>
        }
    }
}

impl Filter {
    fn classes(&self) -> Vec<&str> {
        let mut classes = self.classes.clone();
        if self.props.show {
            classes.push("slideInUp");
        } else {
            classes.push("slideOutDown");
        }
        classes
    }

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