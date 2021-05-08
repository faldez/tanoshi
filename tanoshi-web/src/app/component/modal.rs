use yew::html::Children;
use yew::prelude::*;

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    #[prop_or_default]
    pub children: Children,
    pub show: bool,
    pub onclose: Callback<()>,
    pub oncancel: Callback<()>,
}

pub enum Msg {
    Done,
    Cancel,
}

pub struct Modal {
    link: ComponentLink<Self>,
    props: Props,
    node_ref: NodeRef,
    classes: Vec<&'static str>,
}

impl Component for Modal {
    type Message = Msg;

    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let classes = vec!["hidden"];
        Modal {
            link,
            props,
            node_ref: NodeRef::default(),
            classes,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Done => {
                self.props.onclose.emit(());
            }
            Msg::Cancel => {
                self.props.oncancel.emit(());
            }
        }
        true
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

    fn view(&self) -> Html {
        html! {
            <div id="modals" ref={self.node_ref.clone()} class={self.classes()}>
                <div class="absolute w-full shadow p-2 flex justify-between">
                    <button class="flex rounded text-accent dark:text-accent-lighter py-1 px-2 justify-center focus:outline-none" onclick=self.link.callback(|_| Msg::Cancel)>{"Cancel"}</button>
                    <button class="flex rounded bg-accent text-white py-1 px-2 shadow justify-center focus:outline-none" onclick=self.link.callback(|_| Msg::Done)>{"Search"}</button>
                </div>
                <div class="w-full max-w-full flex flex-col mx-auto mt-12">
                    {html! {<>{ self.props.children.clone() }</>}}
                </div>
            </div>
        }
    }
}

impl Modal {
    fn classes(&self) -> Vec<&str> {
        let mut classes = self.classes.clone();
        if self.props.show {
            classes.push("slideInUp");
        } else {
            classes.push("slideOutDown");
        }
        classes
    }
}