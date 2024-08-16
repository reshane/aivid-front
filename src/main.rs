use yew::prelude::*;
use log::info;
use wasm_bindgen::JsValue;
use gloo_net::http::{Request, Response};
use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Deserialize)]
struct EventResp {
    event_id: String
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
struct PredictionResponse {
    data: Vec<Prediction>,
    event: String
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
struct Prediction {
    label: String,
    confidences: Vec<Confidence>
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
struct Confidence {
    label: String,
    confidence: f64
}

/*
impl Display for Prediction {
    fn fmt(&self, f: &mut Formatter<'static>) -> Result<(), std::fmt::Error> {
        
    }
}

impl Display for Confidence {
}
*/

#[function_component(App)]
fn app() -> Html {
    html! {
        <h1>{ "Hello World" }</h1>
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    let object = JsValue::from("world");
    info!("Hello {}", object.as_string().unwrap());

    wasm_bindgen_futures::spawn_local(async move {
        let fetched_data: Response = Request::get("https://raw.githubusercontent.com/reshane/aivid-front/main/src/main.rs")
            .send()
            .await
            .unwrap();
        info!("{}", fetched_data.text().await.unwrap());

        let event_id: EventResp = Request::post("https://reshane-aivid.hf.space/call/predict")
            .header("Content-Type", "application/json")
            .body("{\"data\": [{\"path\":\"https://raw.githubusercontent.com/gradio-app/gradio/main/test/test_files/bus.png\"}]}")
            .send()
            .await
            .unwrap()
            .json::<EventResp>()
            .await
            .unwrap();
        info!("{:?}", event_id.event_id);

        let prediction: String = Request::get(format!("https://reshane-aivid.hf.space/call/predict/{}", event_id.event_id).as_str())
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap();
        info!("{:?}", prediction);
    });


    yew::Renderer::<App>::new().render();
}
