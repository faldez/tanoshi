use yew::html::Children;
use yew::prelude::*;

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    #[prop_or_default]
    pub children: Children,
}

pub struct TopBar {
    props: Props,
}

pub enum Msg {}

impl Component for TopBar {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        TopBar { props }
    }

    fn update(&mut self, _msg: Self::Message) -> bool {
        false
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
        html! {
            <div class="w-full pl-2 xl:pl-48 pr-2 pb-2 flex justify-between block fixed inset-x-0 top-0 z-50 text-base xl:text-lg font-normal xl:font-medium text-white xl:text-black xl:dark:text-white bg-gray-900 xl:bg-white xl:dark:bg-gray-900" style="padding-top: calc(env(safe-area-inset-top) + .5rem)">
            {
                if self.props.children.is_empty() {
                    html! {}
                } else {
                    html! {<>{ self.props.children.clone() }</>}
                }
            }
            </div>
        }
    }
}
