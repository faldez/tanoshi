use crate::app::component::manga::{Manga, Props as MangaProps};
use yew::html::{ChildrenRenderer, Component, ComponentLink};
use yew::prelude::*;
use yew::virtual_dom::{VChild, VComp, VNode};

use super::{WeakComponentLink};

#[derive(Clone, PartialEq)]
pub enum Variants {
    MangaItem(<Manga as Component>::Properties),
}

impl From<MangaProps> for Variants {
    fn from(props: MangaProps) -> Self {
        Variants::MangaItem(props)
    }
}

#[derive(Clone, PartialEq)]
pub struct MangaVariant {
    props: Variants,
}

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    pub children: ChildrenRenderer<MangaVariant>,
    pub weak_link: WeakComponentLink<MangaList>,
    #[prop_or_default]
    pub style: String,
}
pub struct MangaList {
    props: Props,
}

pub enum Msg {}

impl Component for MangaList {
    type Message = Msg;
    type Properties = Props;
    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        *props.weak_link.borrow_mut() = Some(link);
        MangaList { props }
    }
    fn update(&mut self, _: <Self as yew::html::Component>::Message) -> bool {
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
            <div class="w-full mx-auto grid grid-cols-3 md:grid-cols-4 xl:grid-cols-6 xl:grid-cols-8 gap-2 pt-12" style={&self.props.style}>
                {self.view_manga()}
            </div>
        }
    }
}

impl MangaList {
    fn view_manga(&self) -> Html {
        html! {{
            for self.props.children.iter().map(|m| m)
        }}
    }
}

impl<CHILD> From<VChild<CHILD>> for MangaVariant
where
    CHILD: Component,
    CHILD::Properties: Into<Variants>,
{
    fn from(vchild: VChild<CHILD>) -> Self {
        MangaVariant {
            props: vchild.props.into(),
        }
    }
}

impl Into<VNode> for MangaVariant {
    fn into(self) -> VNode {
        match self.props {
            Variants::MangaItem(props) => {
                VComp::new::<Manga>(props, NodeRef::default(), None).into()
            }
        }
    }
}
