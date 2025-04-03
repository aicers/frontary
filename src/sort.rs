use yew::{Html, Properties, function_component, html};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Unsorted,
    Ascending,
    Descending,
}

#[derive(Clone, PartialEq, Eq, Properties)]
pub struct Props {
    #[prop_or(Status::Unsorted)]
    pub status: Status,
}

#[function_component(Model)]
pub fn model(props: &Props) -> Html {
    let class = match props.status {
        Status::Unsorted => "basic-sort-unsorted",
        Status::Ascending => "basic-sort-ascending",
        Status::Descending => "basic-sort-descending",
    };

    html! {
        <div class={class}>
        </div>
    }
}
