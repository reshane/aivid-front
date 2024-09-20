use wasm_bindgen_futures::JsFuture;
use wasm_bindgen::prelude::*;
use yew::prelude::*;
use log::info;
use wasm_bindgen::JsValue;
use gloo_net::http::{Request, Response};
use gloo_file::{callbacks::FileReader, File};
use serde::Deserialize;
use std::ops::Deref;
use std::collections::HashMap;
use web_sys::{Event, HtmlInputElement,RequestMode, RequestInit};


#[derive(Debug, Clone, PartialEq, Deserialize)]
struct EventResp {
    event_id: String
}

#[derive(Debug, Clone, PartialEq, Deserialize, Default)]
pub struct PredictionResponse {
    data: Vec<Prediction>,
    event: String
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Prediction {
    label: String,
    confidences: Vec<Confidence>
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
struct Confidence {
    label: String,
    confidence: f64
}

pub enum Msg {
    LoadedBytes(String, Vec<u8>),
    Files(Vec<File>),
    ResultFromSpawn(String),
    Update()
}

pub struct FileDataComponent {
    files: Vec<String>,
    readers: HashMap<String, FileReader>,
    predictions: String
}


impl Component for FileDataComponent {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            files: Vec::new(),
            readers: HashMap::default(),
            predictions: String::new()
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Files(files) => {
                log::info!("Files selected: {}", files.len());
                for file in files.into_iter() {
                    let file_name = file.name();
                    let task = {
                        let file_name = file_name.clone();
                        let link = ctx.link().clone();

                        gloo_file::callbacks::read_as_bytes(&file, move |res| {
                            link.send_message(Msg::LoadedBytes(
                                    file_name,
                                    res.expect("failed to read file"),
                                    ))
                        })
                    };
                    self.readers.insert(file_name, task);
                }
                true
            }
            Msg::LoadedBytes(file_name, data) => {
                log::info!("Processing {}", file_name);

                let link = ctx.link().clone();
                log::info!("Sending update message");
                link.send_message(Msg::Update());

                let image_data = base64::encode(data);
                self.files.push(image_data);
                self.readers.remove(&file_name);
                true
            },
            Msg::ResultFromSpawn(predictions) => {
                info!("Predictions {:?}", predictions);
                self.predictions = predictions;
                true
            },
            Msg::Update() => {
                log::info!("Update!");
                Self::predict(ctx);
                true
            }
        }
    }
        fn view(&self, ctx: &Context<Self>) -> Html {
        let on_change = ctx.link().callback(move |e: Event| {
            let mut selected_files = Vec::new();
            let input: HtmlInputElement = e.target_unchecked_into();
            if let Some(files) = input.files() {
                let files = js_sys::try_iter(&files)
                    .unwrap()
                    .unwrap()
                    .map(|v| web_sys::File::from(v.unwrap()))
                    .map(File::from);
                selected_files.extend(files);
            }
            Msg::Files(selected_files)
        });
        html! {
            <div>
                <div>
                    {"Choose a image file:"}
                </div>
                <div>
                    <input type="file" accept="image/png, image/gif" onchange={on_change} multiple=false/>
                </div>
                <div>
                { for self.files.iter().map(|f| Self::view_file(f))}
                { format!("{:?}", self.predictions)  }
                </div>
            </div>
        }
    }
}




impl FileDataComponent {
    fn view_prediction(pred: Prediction) -> Html {
        html! {
            { format!("{:?}", pred) }
        }
    }
    fn predict(ctx: &Context<Self>) {
        log::info!("Predicting!");
        let link = ctx.link().clone();

        wasm_bindgen_futures::spawn_local(async move {
            let event_id: EventResp = Request::post("https://reshane-aivid.hf.space/call/predict")
                .header("Content-Type", "application/json")
                .body("{\"data\": [{\"path\":\"https://raw.githubusercontent.com/gradio-app/gradio/main/test/test_files/bus.png\"}]}")
                .send()
                .await
                .unwrap()
                .json::<EventResp>()
                .await
                .unwrap();

            let fetched_prediction: String = Request::get(format!("https://reshane-aivid.hf.space/call/predict/{}", event_id.event_id).as_str())
                .send()
                .await
                .unwrap()
                .text()
                .await
                .unwrap();
            link.send_message( Msg::ResultFromSpawn( fetched_prediction ));
        });
    }
    fn view_file(data: &str) -> Html {
        let img = format!("data:image/png;base64,{}", data.to_string());
        let prediction = "fake";
        html! {
            <div>
                <img src={img}/>
                <p> { prediction } </p>
            </div>
        }
    }
}

#[function_component(App)]
fn app() -> Html {
    /*
    let prediction = use_state(|| String::new());
    {
        let prediction = prediction.clone();
        use_effect_with((), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                let event_id: EventResp = Request::post("https://reshane-aivid.hf.space/call/predict")
                    .header("Content-Type", "application/json")
                    .body("{\"data\": [{\"path\":\"https://raw.githubusercontent.com/gradio-app/gradio/main/test/test_files/bus.png\"}]}")
                    .send()
                    .await
                    .unwrap()
                    .json::<EventResp>()
                    .await
                    .unwrap();

                let fetched_prediction: String = Request::get(format!("https://reshane-aivid.hf.space/call/predict/{}", event_id.event_id).as_str())
                    .send()
                    .await
                    .unwrap()
                    .text()
                    .await
                    .unwrap();
                prediction.set(fetched_prediction)
            });
            || ()
        });
    }

    let prediction_html = format!("{:?}", prediction.deref());
    */

    return html! {
        <>
            <h1>{ "Hello World" }</h1>
            <div>
                <h3>{ "Here is my image" }</h3>
                <img src="https://raw.githubusercontent.com/gradio-app/gradio/main/test/test_files/bus.png" alt="bus" />
                <FileDataComponent/>
            </div>
        </>
    };
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    let object = JsValue::from("world");
    info!("Hello {}", object.as_string().unwrap());



    yew::Renderer::<App>::new().render();
}
