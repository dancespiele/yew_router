//! Router and Route components

use crate::route_info::RouteInfo;
use crate::route_agent::{RouteAgent, RouteRequest};
use yew::Bridged;
use yew::{
    html,
    virtual_dom::VNode,
    Bridge, Component, ComponentLink, Html, Properties, Renderable, ShouldRender,
};
use crate::YewRouterState;
use log::{warn, trace};
use yew_router_path_matcher::{PathMatcher};
use yew::html::ChildrenWithProps;

/// A nested component used inside of [Router](struct.Router.html) that can determine if a
/// sub-component can be rendered.
pub struct Route<T: for<'de> YewRouterState<'de>> {
    props: RouteProps<T>
}

/// Properties for Route.
#[derive(Properties)]
pub struct RouteProps<T: for<'de> YewRouterState<'de>> {
    #[props(required)]
    pub path: PathMatcher<Router<T>>,
}

impl <T: for<'de> YewRouterState<'de>> Component for Route<T> {
    type Message = ();
    type Properties = RouteProps<T>;

    fn create(props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        Route {
            props
        }
    }

    fn update(&mut self, _msg: Self::Message) -> bool {
        false
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props = props;
        true
    }
}

/// Rendering control flow component.
///
/// Based on the current url and its child [Routes](struct.Route.html), it will choose one route and
/// render its associated component.
pub struct Router<T: for<'de> YewRouterState<'de>> {
    route: RouteInfo<T>,
    props: Props<T>,
    router_agent: Box<dyn Bridge<RouteAgent<T>>>,
}

/// Message for Router.
pub enum Msg<T> {
    UpdateRoute(RouteInfo<T>),
}

/// Properties for Router.
#[derive(Properties)]
pub struct Props<T: for<'de> YewRouterState<'de>> {
    #[props(required)]
    children: ChildrenWithProps<Route<T>, Router<T>>
}

impl <T> Component for Router<T>
    where T: for<'de> YewRouterState<'de>
{
    type Message = Msg<T>;
    type Properties = Props<T>;

    fn create(props: Self::Properties, mut link: ComponentLink<Self>) -> Self {
        let callback = link.send_back(Msg::UpdateRoute);
        let mut router_agent = RouteAgent::bridge(callback);

        router_agent.send(RouteRequest::GetCurrentRoute);
        Router {
            route: Default::default(), // This must be updated by immediately requesting a route update from the service bridge.
            props,
            router_agent,
        }
    }

//    fn mounted(&mut self) -> ShouldRender {
//        self.router_agent.send(RouterRequest::GetCurrentRoute);
//        false
//    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::UpdateRoute(route) => {
                let did_change = self.route != route;
                self.route = route;
                did_change
            }
        }
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props = props;
        true
    }
}

impl <T: for<'de> YewRouterState<'de>> Renderable<Router<T>> for Router<T>
{
    fn view(&self) -> VNode<Self> {

        trace!("Routing one of {} routes for  {:?}", self.props.children.iter().count(), &self.route);
        self.props.children.iter()
            .filter_map(|route_possibility| -> Option<Html<Self>> {
                route_possibility.props.path
                    .match_path(&self.route)
                    .map(|(_rest, hm)| {
                        (route_possibility.props.path.render_fn)(&hm)
                    })
                    .ok()
                    .flatten_stable()
            })
            .next() // Take the first path that succeeds.
            .map(|x| -> Html<Self> {
                trace!("Route matched.");
                x
            })
            .unwrap_or_else(|| {
                warn!("Routing failed. No default case was provided.");
                html! { <></>}
            })
    }
}


trait Flatten<T> {
    /// Because flatten is a nightly feature. I'm making a new variant of the function here for stable use.
    /// The naming is changed to avoid this getting clobbered when object_flattening 60258 is stabilized.
    fn flatten_stable(self) -> Option<T>;
}

impl<T> Flatten<T> for Option<Option<T>> {
    fn flatten_stable(self) -> Option<T> {
        match self {
            None => None,
            Some(v) => v,
        }
    }
}
