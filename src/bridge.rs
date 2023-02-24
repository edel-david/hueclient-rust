use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::str::FromStr;

use std::marker::PhantomData;

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct GroupState {
    pub all_on: bool,
    pub any_on: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Group {
    pub name: String,
    pub lights: Vec<String>,
    pub sensors: Vec<String>,
    pub r#type: String,
    pub state: GroupState,
    pub recycle: bool,
    pub action: LightState,
}

#[derive(Debug, Clone)]
pub struct IdentifiedGroup {
    pub id: usize,
    pub group: Group,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct LightState {
    pub on: bool,
    pub bri: Option<u8>,
    pub hue: Option<u16>,
    pub sat: Option<u8>,
    pub ct: Option<u16>,
    pub xy: Option<(f32, f32)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Light {
    pub name: String,
    pub modelid: String,
    pub swversion: String,
    pub uniqueid: String,
    pub state: LightState,
}

#[derive(Debug, Clone)]
pub struct IdentifiedLight {
    pub id: usize,
    pub light: Light,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scene {
    pub name: String,
    pub r#type: String,
    pub lights: Vec<String>,
    pub owner: String,
    pub recycle: bool,
    pub locked: bool,
}

#[derive(Debug, Clone)]
pub struct IdentifiedScene {
    pub id: String,
    pub scene: Scene,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandLight {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub on: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bri: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hue: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sat: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ct: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub xy: Option<(f32, f32)>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transitiontime: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alert: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scene: Option<String>,
}

impl Default for CommandLight {
    fn default() -> CommandLight {
        CommandLight {
            on: None,
            bri: None,
            hue: None,
            sat: None,
            transitiontime: None,
            ct: None,
            xy: None,
            alert: None,
            scene: None,
        }
    }
}

impl CommandLight {
    pub fn on(self) -> CommandLight {
        CommandLight {
            on: Some(true),
            ..self
        }
    }
    pub fn off(self) -> CommandLight {
        CommandLight {
            on: Some(false),
            ..self
        }
    }
    pub fn with_bri(self, b: u8) -> CommandLight {
        CommandLight {
            bri: Some(b),
            ..self
        }
    }
    pub fn with_hue(self, h: u16) -> CommandLight {
        CommandLight {
            hue: Some(h),
            ..self
        }
    }
    pub fn with_sat(self, s: u8) -> CommandLight {
        CommandLight {
            sat: Some(s),
            ..self
        }
    }
    pub fn with_ct(self, c: u16) -> CommandLight {
        CommandLight {
            ct: Some(c),
            ..self
        }
    }
    pub fn with_xy(self, x: f32, y: f32) -> CommandLight {
        CommandLight {
            xy: Some((x, y)),
            ..self
        }
    }
    pub fn alert(self) -> CommandLight {
        CommandLight {
            alert: Some("select".into()),
            ..self
        }
    }
    pub fn scene(self, s: String) -> CommandLight {
        CommandLight {
            scene: Some(s),
            ..self
        }
    }
}

#[derive(Debug, Clone)]
pub struct Unauthed;
#[derive(Debug, Clone)]
pub struct Authed;


/// The bridge is the central access point of the lamps in a Hue setup, and also the central access
/// point of this library.
/// A bridge can be authenticated or unauthenticated
#[derive(Debug, Clone)]
pub struct Bridge<State = Unauthed> {
    /// The IP-address of the bridge.
    pub ip: std::net::IpAddr,
    /// This is the username of the currently logged in user.
    pub username: Option<String>,
    pub(self) client: reqwest::blocking::Client,
    state: PhantomData<State>,
}

impl Bridge<Unauthed> {
    /// Create a bridge at this IP. If you know the IP-address, this is the fastest option. Note
    /// that this function does not validate whether a bridge is really present at the IP-address.
    /// ### Example
    /// ```no_run
    /// let bridge = hueclient::Bridge::for_ip([192u8, 168, 0, 4]);
    /// ```
    pub fn for_ip(ip: impl Into<std::net::IpAddr>) -> Bridge<Unauthed> {
        Bridge {
            ip: ip.into(),
            client: reqwest::blocking::Client::new(),
            username: None,
            state: PhantomData::<Unauthed>,
        }
    }

    /// Consumes the bridge and returns a new one with a configured username.
    /// ### Example
    /// ```no_run
    /// let bridge = hueclient::Bridge::for_ip([192u8, 168, 0, 4])
    ///     .with_user("rVV05G0i52vQMMLn6BK3dpr0F3uDiqtDjPLPK2uj");
    /// ```
    pub fn with_user(self, username: impl Into<String>) -> Bridge<Authed> {
        Bridge {
            ip: self.ip,
            username: Some(username.into()),
            client: self.client,
            state: std::marker::PhantomData::<Authed>,
        }
    }





    /// This function registers a new user at the provided brige, using `devicetype` as an
    /// identifier for that user. It returns an error if the button of the bridge was not pressed
    /// shortly before running this function.
    /// ### Example
    /// ```no_run
    /// let mut bridge = hueclient::Bridge::for_ip([192u8, 168, 0, 4]);
    /// let password = bridge.register_user("mylaptop").unwrap();
    /// // now this password can be stored and reused
    /// ```
    pub fn register_user(self, devicetype: &str) -> crate::Result<Bridge<Authed>> {
        #[derive(Serialize)]
        struct PostApi {
            devicetype: String,
        }
        #[derive(Debug, Deserialize)]
        struct Username {
            username: String,
        }
        let obtain = PostApi {
            devicetype: devicetype.to_string(),
        };
        let url = format!("http://{}/api", self.ip);
        let resp: BridgeResponse<SuccessResponse<Username>> =
            self.client.post(&url).json(&obtain).send()?.json()?;
        let resp = resp.get()?;

        Ok(Bridge {
            ip: self.ip,
            username: Some(resp.success.username),
            client: self.client,
            state: PhantomData::<Authed>,
        })
    }

    /// Scans the current network for Bridges, and if there is at least one, returns the first one
    /// that was found.
    /// ### Example
    /// ```no_run
    /// let maybe_bridge = hueclient::Bridge::discover();
    /// ```
    pub fn discover() -> Option<Bridge<Unauthed>> {
        crate::disco::discover_hue_bridge()
            .ok()
            .map(|ip| Bridge {
                ip,
                client: reqwest::blocking::Client::new(),
                username: None,
                state: PhantomData::<Unauthed>,
            })
    }

    /// A convience wrapper around `Bridge::disover`, but panics if there is no bridge present.
    /// ### Example
    /// ```no_run
    /// let brige = hueclient::Bridge::discover_required();
    /// ```
    /// ### Panics
    /// This function panics if there is no brige present.
    pub fn discover_required() -> Bridge<Unauthed> {
        Self::discover().expect("No bridge found!")
    }

}


impl Bridge<Authed> {
    /// Idk if it makes sense to register user on authed Bridge?
    /// This function registers a new user at the provided brige, using `devicetype` as an
    /// identifier for that user. It returns an error if the button of the bridge was not pressed
    /// shortly before running this function.
    /// ### Example
    /// ```no_run
    /// let bridge = hueclient::Bridge::for_ip([192u8, 168, 0, 4])
    ///     .register_user("mylaptop")
    ///     .unwrap();
    /// // now this username d can be stored and reused
    /// println!("the password was {}", bridge.username);
    /// ```
    pub fn register_user(self, devicetype: &str) -> crate::Result<Bridge<Authed>> {
        #[derive(Serialize)]
        struct PostApi {
            devicetype: String,
        }
        #[derive(Debug, Deserialize)]
        struct Username {
            username: String,
        }
        let obtain = PostApi {
            devicetype: devicetype.to_string(),
        };
        let url = format!("http://{}/api", self.ip);
        let resp: BridgeResponse<SuccessResponse<Username>> =
            self.client.post(&url).json(&obtain).send()?.json()?;
        let resp = resp.get()?;

        Ok(Bridge {
            ip: self.ip,
            username: Some(resp.success.username),
            client: self.client,
            state: PhantomData::<Authed>,
        })
    }

    /// Returns a vector of all lights that are registered at this `Bridge`, sorted by their id's.
    /// This function returns an error if `bridge.username` is `None`.
    /// ### Example
    /// ```no_run
    /// let bridge = hueclient::Bridge::for_ip([192u8, 168, 0, 4])
    ///    .with_user("rVV05G0i52vQMMLn6BK3dpr0F3uDiqtDjPLPK2uj");
    /// for light in &bridge.get_all_lights().unwrap() {
    ///     println!("{:?}", light);
    /// }
    /// ```
    pub fn get_all_lights(&self) -> crate::Result<Vec<IdentifiedLight>> {
        let url = format!(
            "http://{}/api/{}/lights",
            self.ip,
            self.username.as_ref().unwrap()
        );
        type Resp = BridgeResponse<HashMap<String, Light>>;
        let resp: Resp = self.client.get(&url).send()?.json()?;
        let mut lights = vec![];
        for (k, light) in resp.get()? {
            let id = usize::from_str(&k)
                .map_err(|_| crate::HueError::protocol_err("Light id should be a number"))?;
            lights.push(IdentifiedLight { id, light });
        }
        lights.sort_by(|a, b| a.id.cmp(&b.id));
        Ok(lights)
    }

    /// Returns a vector of all groups that are registered at this `Bridge`, sorted by their id's.
    /// This function returns an error if `bridge.username` is `None`.
    /// ### Example
    /// ```no_run
    /// let bridge = hueclient::Bridge::for_ip([192u8, 168, 0, 4])
    ///    .with_user("rVV05G0i52vQMMLn6BK3dpr0F3uDiqtDjPLPK2uj");
    /// for group in &bridge.get_all_groups().unwrap() {
    ///     println!("{:?}", group);
    /// }
    /// ```
    pub fn get_all_groups(&self) -> crate::Result<Vec<IdentifiedGroup>> {
        let usernamestring = &self.username.clone().unwrap();
        let url = format!("http://{}/api/{}/groups", self.ip, usernamestring);
        type Resp = BridgeResponse<HashMap<String, Group>>;
        let resp: Resp = self.client.get(&url).send()?.json()?;
        let mut groups = vec![];
        for (k, group) in resp.get()? {
            let id = usize::from_str(&k)
                .map_err(|_| crate::HueError::protocol_err("Group id should be a number"))?;
            groups.push(IdentifiedGroup { id, group });
        }
        groups.sort_by(|a, b| a.id.cmp(&b.id));
        Ok(groups)
    }

    /// Returns a vector of all scenes that are registered at this `Bridge`, sorted by their id's.
    /// This function returns an error if `bridge.username` is `None`.
    /// ### Example
    /// ```no_run
    /// let bridge = hueclient::Bridge::for_ip([192u8, 168, 0, 4])
    ///    .with_user("rVV05G0i52vQMMLn6BK3dpr0F3uDiqtDjPLPK2uj");
    /// for scene in &bridge.get_all_scenes().unwrap() {
    ///     println!("{:?}", scene);
    /// }
    /// ```
    pub fn get_all_scenes(&self) -> crate::Result<Vec<IdentifiedScene>> {
        let url = format!(
            "http://{}/api/{}/scenes",
            self.ip,
            self.username.as_ref().unwrap()
        );
        type Resp = BridgeResponse<HashMap<String, Scene>>;
        let resp: Resp = self.client.get(&url).send()?.json()?;
        let mut scenes = vec![];
        for (k, scene) in resp.get()? {
            scenes.push(IdentifiedScene {
                id: k,
                scene: scene,
            });
        }
        scenes.sort_by(|a, b| a.id.cmp(&b.id));
        Ok(scenes)
    }

    pub fn set_scene(&self, scene: String) -> crate::Result<Value> {
        let url = format!(
            "http://{}/api/{}/groups/0/action",
            self.ip,
            self.username.as_ref().unwrap()
        );
        let command = CommandLight::default().scene(scene);
        let resp: BridgeResponse<Value> = self.client.put(&url).json(&command).send()?.json()?;
        resp.get()
    }

    pub fn set_group_state(&self, group: usize, command: &CommandLight) -> crate::Result<Value> {
        let url = format!(
            "http://{}/api/{}/groups/{}/action",
            self.ip,
            self.username.as_ref().unwrap(),
            group
        );
        let resp: BridgeResponse<Value> = self.client.put(&url).json(command).send()?.json()?;
        resp.get()
    }

    pub fn set_light_state(&self, light: usize, command: &CommandLight) -> crate::Result<Value> {
        let url = format!(
            "http://{}/api/{}/lights/{}/state",
            self.ip,
            self.username.as_ref().unwrap(),
            light
        );
        let resp: BridgeResponse<Value> = self.client.put(&url).json(command).send()?.json()?;
        resp.get()
    }
}
#[derive(Debug, serde::Deserialize)]
#[serde(untagged)]
enum BridgeResponse<T> {
    Element(T),
    List(Vec<T>),
    Errors(Vec<BridgeError>),
}

impl<T> BridgeResponse<T> {
    fn get(self) -> crate::Result<T> {
        match self {
            BridgeResponse::Element(t) => Ok(t),
            BridgeResponse::List(mut ts) => ts
                .pop()
                .ok_or_else(|| crate::HueError::protocol_err("expected non-empty array")),
            BridgeResponse::Errors(mut es) => {
                // it is safe to unwrap here, since any empty lists will be treated as the
                // `BridgeResponse::List` case.
                let BridgeError { error } = es.pop().unwrap();
                Err(crate::HueError::BridgeError {
                    code: error.r#type,
                    msg: error.description,
                })
            }
        }
    }
}

#[derive(Debug, serde::Deserialize)]
struct BridgeError {
    error: BridgeErrorInner,
}

#[derive(Debug, serde::Deserialize)]
struct BridgeErrorInner {
    address: String,
    description: String,
    r#type: usize,
}

#[derive(Debug, serde::Deserialize)]
struct SuccessResponse<T> {
    success: T,
}
