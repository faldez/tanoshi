use yew::prelude::*;

#[derive(Clone, PartialEq)]
pub enum Type {
    Error,
    #[allow(dead_code)]
    Info,
}

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    pub visible: bool,
    pub toast_type: Type,
    pub message: String,
}

pub struct Toast {
    props: Props,
    link: ComponentLink<Self>,
}

pub enum Msg {
    Close,
}

impl Component for Toast {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Toast { props, link }
    }

    fn update(&mut self, msg: Self::Message) -> bool {
        match msg {
            Msg::Close => self.props.visible = false,
        }
        true
    }

    fn change(&mut self, props: Self::Properties) -> bool {
        if self.props != props {
            self.props = props;
            true
        } else {
            false
        }
    }

    fn view(&self) -> Html {
        let mut classes = vec![
            "animated faseter border px-4 py-3 rounded fixed inset-x-0 bottom-0 z-50 mx-48 mb-24",
        ];
        match self.props.toast_type {
            Type::Error => {
                classes.push("bg-red-100");
                classes.push("border-red-400");
                classes.push("text-red-700");
            }
            Type::Info => {
                classes.push("bg-blue-100");
                classes.push("border-blue-400");
                classes.push("text-blue-700");
            }
        }
        if !self.props.visible {
            classes.push("hidden");
            classes.push("slideInDown");
        } else {
            classes.push("slideInUp");
        }

        return html! {
            <div class={classes} role="alert">
                <strong class="font-bold">{"Error!"}</strong>
                <span class="block sm:inline ml-2">{&self.props.message}</span>
                <button class="absolute top-0 bottom-0 right-0 px-4 py-3 focus:outline-none" onclick={self.link.callback(|_| Msg::Close)} >
                    <svg class="fill-current h-6 w-6 text-red-500" role="button" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20"><title>{"Close"}</title><path d="M14.348 14.849a1.2 1.2 0 0 1-1.697 0L10 11.819l-2.651 3.029a1.2 1.2 0 1 1-1.697-1.697l2.758-3.15-2.759-3.152a1.2 1.2 0 1 1 1.697-1.697L10 8.183l2.651-3.031a1.2 1.2 0 1 1 1.697 1.697l-2.758 3.152 2.758 3.15a1.2 1.2 0 0 1 0 1.698z"/></svg>
                </button>
            </div>
        };
    }
}

impl Toast {}
