use reqwest::{
    blocking::Response,
    header::{CONTENT_TYPE, HeaderMap, HeaderValue},
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{
    fs::File,
    io::{Error, Read},
    sync::Arc,
};

const DEFAULT_EVENT: &str = "UPDATE";

// Helper for parsing the json File which holds information on where to find the API endpoint
#[derive(Deserialize, Debug)]
struct SteelSeriesAPIInfo {
    address: String,
    #[serde(rename = "encryptedAddress")]
    _encrypted_address: Option<String>,
    #[serde(rename = "ggEncrypted_address")]
    _gg_encrypted_address: Option<String>,
}

// Helper function which returns the address for the GameSense API
// This address changes with every start of the SteelSeries Application
fn get_api_addr() -> Result<String, Error> {
    #[cfg(target_os = "windows")]
    let engine_path =
        std::env::var("PROGRAMDATA").expect("Could not find env %PROGRAM DATA%") + "/SteelSeries";
    #[cfg(target_os = "macos")]
    let engine_path = "/Library/Application Support/";
    let mut file = File::open(format!(
        "{}/SteelSeries Engine 3/coreProps.json",
        engine_path
    ))
    .expect("Could not open SteelSeries Engine Information. Is SteelSeries Engine running?");
    let mut buff = String::new();
    file.read_to_string(&mut buff)?;

    let data: SteelSeriesAPIInfo = serde_json::from_str(&buff)
        .expect("Could not parse SteelSeries Engine endpoint. Is SteelSeries Engine running?");
    Ok(data.address)
}

// Every game which wants to send data requires a game name and an event name
#[derive(Serialize, Deserialize, Debug)]
struct GameMetadata {
    game: String,
    event: String,
    value_optional: bool,
    game_display_name: Option<String>,
    developer: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct BindGameEvent {
    game: String,
    value_optional: bool,
    handlers: serde_json::Value,
    event: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct GameEvent {
    game: String,
    event: String,
    data: serde_json::Value,
}

pub struct GameSenseAPI {
    game_metadata: GameMetadata,
    client: reqwest::blocking::Client,
    address: String,
    headers: Arc<HeaderMap<HeaderValue>>,
    width: u8,
    height: u8,
}

impl GameSenseAPI {
    pub fn new(game_name: String, width: u8, height: u8) -> GameSenseAPI {
        let game_metadata = GameMetadata {
            developer: None,
            event: DEFAULT_EVENT.to_string(),
            game: game_name,
            game_display_name: None,
            value_optional: true,
        };
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        let headers = Arc::new(headers);
        GameSenseAPI {
            client: reqwest::blocking::Client::new(),
            game_metadata,
            address: get_api_addr().expect("SteelSeries Engine not reachable!"),
            headers,
            width,
            height,
        }
    }

    pub fn developer(&mut self, developer: String) {
        self.game_metadata.developer = Some(developer);
    }

    pub fn game_description(&mut self, description: String) {
        self.game_metadata.game_display_name = Some(description);
    }

    // register our game to the steel series api
    pub fn register(&self) -> Result<(), reqwest::Error> {
        let data = serde_json::to_string(&self.game_metadata)
            .expect("Could not serialize JSON body for registration");
        let res = self
            .client
            .post(format!("http://{}/game_metadata", self.address))
            .body(data)
            .headers((*self.headers).clone())
            .send()?;
        check_response(res)
    }
    // bind an event
    pub fn bind_event(&self) -> Result<(), reqwest::Error> {
        let empty = vec![0; self.width as usize * self.height as usize / 8];
        let data = serde_json::to_string(&BindGameEvent {
            game: self.game_metadata.game.clone(),
            event: DEFAULT_EVENT.to_string(),
            value_optional: true,
            handlers: json!([{
                "zone": "one",
                "device-type": format!("screened-{}x{}", self.width, self.height),
                "mode": "screen",
                "datas": [{
                    "has-text": false,
                    "image-data": empty,
                }],
            }]),
        })
        .unwrap();
        let res = self
            .client
            .post(format!("http://{}/bind_game_event", self.address))
            .body(data)
            .headers((*self.headers).clone())
            .send()?;
        check_response(res)
    }
    // actually send a buffer to the api
    pub fn send_event(&self, img_data: &[u8]) -> Result<(), reqwest::Error> {
        let data = serde_json::to_string(&GameEvent {
            event: DEFAULT_EVENT.to_string(),
            game: self.game_metadata.game.clone(),
            data: json!({
                "frame": {
                    format!("image-data-{}x{}", self.width, self.height): img_data,
                }
            }),
        })
        .unwrap();
        let res = self
            .client
            .post(format!("http://{}/game_event", self.address))
            .body(data)
            .headers((*self.headers).clone())
            .send()?;
        check_response(res)
    }
}

// Helper which panics if the response of the REST request is not 200
fn check_response(res: Response) -> Result<(), reqwest::Error> {
    if !res.status().is_success() {
        panic!("Request failed: {:?}", res.text().unwrap());
    }
    Ok(())
}
