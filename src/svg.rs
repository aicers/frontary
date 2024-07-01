use anyhow::{anyhow, Result};
use gloo_utils::{document, window};
use reqwasm::http::Request;
use scraper::{Html as SHtml, Selector};
use strum_macros::Display;
use web_sys::{Element, Node};
use yew::{html, virtual_dom::AttrValue, Component, Context, Html, Properties};

pub struct Model {
    svg: Option<String>,
    error_msg: Option<FetchErrorMessage>,
}

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    pub path: AttrValue,
    #[prop_or(None)]
    pub width: Option<u32>,
    #[prop_or(None)]
    pub height: Option<u32>,
    #[prop_or(None)]
    pub class: Option<AttrValue>,
}

pub enum Message {
    SvgFetched(String),
    InvalidSvg,
    HttpNoSucess,
    ResponseError,
    QueryError,
}

#[derive(Display)]
pub enum FetchErrorMessage {
    #[strum(serialize = "Invalid SVG format")]
    InvalidSvg,
    #[strum(serialize = "HTTP Failure")]
    HttpNoSucess,
    #[strum(serialize = "Response Error")]
    ResponseError,
    #[strum(serialize = "Query Error")]
    QueryError,
}

impl Component for Model {
    type Message = Message;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let s = Self {
            svg: None,
            error_msg: None,
        };
        Self::fetch(ctx);
        s
    }

    fn update(&mut self, _: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Message::SvgFetched(svg) => {
                self.svg = Some(svg);
                self.error_msg = None;
            }
            Message::InvalidSvg => {
                self.error_msg = Some(FetchErrorMessage::InvalidSvg);
            }
            Message::HttpNoSucess => {
                self.error_msg = Some(FetchErrorMessage::HttpNoSucess);
            }
            Message::ResponseError => {
                self.error_msg = Some(FetchErrorMessage::ResponseError);
            }
            Message::QueryError => {
                self.error_msg = Some(FetchErrorMessage::QueryError);
            }
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        if let Some(err_msg) = self.error_msg.as_ref() {
            html! {
                if cfg!(feature = "test") {
                    <div>{ err_msg.to_string() }</div>
                }
            }
        } else if let Some(svg) = self.svg.as_ref() {
            match Self::element(ctx, svg) {
                Ok(elem) => {
                    let node: Node = elem.into();
                    Html::VRef(node)
                }
                Err(e) => {
                    if cfg!(feature = "test") {
                        html! {
                            <div>{ e.to_string() }</div>
                        }
                    } else {
                        html! {}
                    }
                }
            }
        } else {
            html! {}
        }
    }
}

impl Model {
    fn fetch(ctx: &Context<Self>) {
        if let Ok(req) = request(ctx.props().path.as_str()) {
            ctx.link().send_future(async {
                if let Ok(res) = req.send().await {
                    if res.ok() {
                        let svg = res.text().await.unwrap_or_default();
                        let svg = svg.trim();
                        if svg.to_lowercase().starts_with("<svg") {
                            Message::SvgFetched(svg.to_string())
                        } else {
                            Message::InvalidSvg
                        }
                    } else {
                        Message::HttpNoSucess
                    }
                } else {
                    Message::ResponseError
                }
            });
        } else {
            ctx.link().send_message(Message::QueryError);
        };
    }

    fn element(ctx: &Context<Self>, svg: &str) -> Result<Element> {
        let div: Element = document().create_element("div").map_err(|e| {
            anyhow!(
                "failed to create a div element: {}",
                e.as_string().unwrap_or_default()
            )
        })?;
        if let Some(class) = ctx.props().class.as_ref() {
            div.set_class_name(class);
        }

        let doc = SHtml::parse_document(svg);
        let width = select(ctx.props().width, &doc, "width");
        let height = select(ctx.props().height, &doc, "height");

        if let Some(width) = width {
            div.set_attribute("width", &width).map_err(|e| {
                anyhow!(
                    "failed to set the width attribute: {}",
                    e.as_string().unwrap_or_default()
                )
            })?;
        }
        if let Some(height) = height {
            div.set_attribute("height", &height).map_err(|e| {
                anyhow!(
                    "failed to set the height attribute: {}",
                    e.as_string().unwrap_or_default()
                )
            })?;
        }
        div.set_inner_html(svg);

        Ok(div)
    }
}

fn request(path: &str) -> Result<Request> {
    let mut uri = window().location().origin().map_err(|e| {
        anyhow!(
            "failed to get the origin: {}",
            e.as_string().unwrap_or_default()
        )
    })?;
    uri.push_str(path);
    let request = Request::get(&uri);
    Ok(request)
}

fn select(user_value: Option<u32>, doc: &SHtml, key: &str) -> Option<String> {
    user_value.map(|x| x.to_string()).or_else(|| {
        let select = format!("[{key}]");
        let select = Selector::parse(&select);
        select.ok().and_then(move |select| {
            doc.select(&select)
                .next()
                .and_then(|element| element.value().attr(key).map(ToString::to_string))
        })
    })
}
