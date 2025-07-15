use reqwest::{
    blocking::Response,
    header::{CONTENT_TYPE, HeaderMap, HeaderValue},
};
use serde::{Deserialize, Serialize};
use serde_json::{Map, json};
use std::{
    collections::HashMap,
    fs::File,
    io::{Error, Read},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

use crate::display::{SteelSeriesDisplay, SteelSeriesLCDType};

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
    client: Arc<reqwest::blocking::Client>,
    address: String,
    headers: Arc<HeaderMap<HeaderValue>>,
    displays: HashMap<SteelSeriesLCDType, SteelSeriesDisplay>,
    send_heartbeat: Arc<AtomicBool>,
}

impl GameSenseAPI {
    /// Create a new instance of the GameSense API
    /// An instance of the API will hold all displays for each type of device. Currently the GameSense API supports 4 types
    /// of displays/sizes:
    /// * 128x40 display for Apex7, Apex 7 TKL, Apex Pro and Apex Pro TKL
    /// * 128x48 display for Arctis Pro Wireless
    /// * 128x52 display for GameDAC or Arctis Pro
    /// * 128x36 display for Rival 700 and 710
    ///
    /// You must call `register()` and `bind_event()` manually.
    /// In order to update the screen (-> send data to the device(s)), call `update_displays()`
    ///
    /// # Arguments
    ///
    /// * `game_name` - A game name which will be shown in the SteelSeries Desktop Application. Allowed are upper-case A-Z, 0-9, hyphen, and underscore.
    ///
    pub fn new(game_name: String) -> GameSenseAPI {
        let game_metadata = GameMetadata {
            developer: None,
            event: DEFAULT_EVENT.to_string(),
            game: game_name,
            game_display_name: None,
            value_optional: true,
        };

        // create a hashmap with all display sizes currently known
        let displays: HashMap<_, _> = SteelSeriesLCDType::all()
            .iter()
            .map(|lcd_type| (*lcd_type, SteelSeriesDisplay::new(*lcd_type)))
            .collect();

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        let headers = Arc::new(headers);

        GameSenseAPI {
            client: Arc::new(reqwest::blocking::Client::new()),
            game_metadata,
            address: get_api_addr().expect("SteelSeries Engine not reachable!"),
            headers,
            displays,
            send_heartbeat: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Optionally set a developer name for this game. Will be shown in SteelSeries GG Client.
    pub fn developer(&mut self, developer: String) {
        self.game_metadata.developer = Some(developer);
    }

    /// Optionally set a game description for this game. Will be shown in SteelSeries GG Client.
    pub fn game_description(&mut self, description: String) {
        self.game_metadata.game_display_name = Some(description);
    }

    /// Register our game to the GameSense API.
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

    /// Bind the UPDATE event. This must be called AFTER the registration of the game.
    pub fn bind_event(&self) -> Result<(), reqwest::Error> {
        let mut handler_datas: Vec<serde_json::Value> = vec![];

        for lcd_type in self.displays.keys() {
            let dimensions = lcd_type.dimensions();
            let empty_data = vec![0; dimensions.width as usize * dimensions.height as usize / 8];
            handler_datas.push(json!({
                "zone": "one",
                "device-type": format!("screened-{}x{}", dimensions.width, dimensions.height),
                "mode": "screen",
                "datas": [{
                    "has-text": false,
                    "image-data": empty_data
                }]
            }));
        }
        let data = serde_json::to_string(&BindGameEvent {
            game: self.game_metadata.game.clone(),
            event: DEFAULT_EVENT.to_string(),
            value_optional: true,
            handlers: handler_datas.into(),
        })
        .expect("Could not serialize data for JSON bind event");

        let res = self
            .client
            .post(format!("http://{}/bind_game_event", self.address))
            .body(data)
            .headers((*self.headers).clone())
            .send()?;
        check_response(res)
    }

    /// Call this function to update the screens.
    pub fn update_displays(&self) -> Result<(), reqwest::Error> {
        let mut img_datas: Map<String, serde_json::Value> = Map::new();
        for (lcd_type, display) in &self.displays {
            let dimensions = lcd_type.dimensions();
            img_datas.insert(
                format!("image-data-{}x{}", dimensions.width, dimensions.height),
                display.framebuffer.as_slice().into(),
            );
        }
        let data = serde_json::to_string(&GameEvent {
            event: DEFAULT_EVENT.to_string(),
            game: self.game_metadata.game.clone(),
            data: json!({
                "frame": img_datas
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

    /// 128x40 display for Apex7, Apex 7 TKL, Apex Pro and Apex Pro TKL.
    pub fn display_apex_mut(&mut self) -> &mut SteelSeriesDisplay {
        self.displays.get_mut(&SteelSeriesLCDType::Apex).unwrap()
    }

    /// 128x48 display for Arctis Pro Wireless
    pub fn display_arctis_mut(&mut self) -> &mut SteelSeriesDisplay {
        self.displays.get_mut(&SteelSeriesLCDType::Arctis).unwrap()
    }

    /// 128x52 display for GameDAC or Arctis Pro
    pub fn display_game_dac_mut(&mut self) -> &mut SteelSeriesDisplay {
        self.displays.get_mut(&SteelSeriesLCDType::GameDAC).unwrap()
    }

    /// 128x36 display for Rival 700 and 710
    pub fn display_rival_mut(&mut self) -> &mut SteelSeriesDisplay {
        self.displays
            .get_mut(&SteelSeriesLCDType::Rival7x0)
            .unwrap()
    }

    /// The GameSense API expects us to send a heartbeat every ~15seconds. Use this function to continously
    /// send a heartbeat every 10 seconds.
    /// Note that this is not required if you're updating the screen within the 15 seconds time interval
    /// If you send data only periodically, you should send the heartbeat in order to prevent the device
    /// from resetting the screen automatically.
    pub fn register_heartbeat(&mut self) {
        self.send_heartbeat = Arc::new(AtomicBool::new(true));
        let client = Arc::clone(&self.client);
        let send_heartbeat = Arc::clone(&self.send_heartbeat);
        let address = self.address.clone();
        let data = serde_json::to_string(&json!({
            "game": self.game_metadata.game
        }))
        .unwrap();
        let headers = (*self.headers).clone();
        std::thread::spawn(move || {
            while send_heartbeat.load(Ordering::Relaxed) {
                let _ = client
                    .post(format!("http://{}/game_heartbeat", address))
                    .body(data.clone())
                    .headers(headers.clone())
                    .send();
                std::thread::sleep(Duration::from_secs(10));
            }
        });
    }

    /// Stop sending the heartbeat
    pub fn unregister_heartbeat(&mut self) {
        self.send_heartbeat.store(false, Ordering::Relaxed);
    }
}

// Helper which panics if the response of the REST request is not 200
fn check_response(res: Response) -> Result<(), reqwest::Error> {
    if !res.status().is_success() {
        panic!("Request failed: {:?}", res.text().unwrap());
    }
    Ok(())
}
