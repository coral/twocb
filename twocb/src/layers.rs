use crate::data;
use crate::engines::{DynamicEngine, Engine, Pattern, RSEngine};
use crate::producer;
use atomic_counter::AtomicCounter;
use log::error;
use serde::{
    ser::{SerializeStruct, Serializer},
    Deserialize, Serialize,
};
use serde_json;
use std::mem;
use std::sync::{Arc, Mutex};
use strum_macros;

use self::blending::BlendModes;

pub mod blending;
pub struct Compositor {
    pub links: Vec<LinkAllocation>,
    buffer: Vec<vecmath::Vector4<f64>>,

    counter: atomic_counter::ConsistentCounter,
}

#[derive(Serialize)]
pub struct LinkAllocation {
    id: usize,
    link: Arc<Mutex<Link>>,
}

#[derive(Debug, Clone)]
struct LinkResult {
    id: usize,
    output: Vec<vecmath::Vector4<f64>>,
}

impl Compositor {
    pub fn new() -> Compositor {
        Compositor {
            links: vec![],
            buffer: vec![],
            counter: atomic_counter::ConsistentCounter::new(0),
        }
    }

    pub async fn add_link(&mut self, mut link: Link) {
        // for s in link.steps.iter_mut() {
        //     let key = &format!("{}_{}", &link.name, s.pattern.name());
        //     match self.db.subscribe(key).await {
        //         Ok(v) => match self.db.get_state(key) {
        //             Some(d) => s.pattern.set_state(d),
        //             None => {
        //                 let newstate = &s.pattern.get_state();
        //                 self.db.write_state(key, &newstate);
        //             }
        //         },
        //         Err(err) => error!("Could not subscribe to key updates: {}", err),
        //     }
        // }

        self.links.push(LinkAllocation {
            id: self.counter.inc(),
            link: Arc::new(Mutex::new(link)),
        });
    }

    pub fn remove_link(&mut self, name: String) -> bool {
        return self
            .links
            .iter()
            .position(|n| n.link.lock().unwrap().name == name)
            .map(|e| self.links.remove(e))
            .is_some();
    }

    pub async fn render(&mut self, frame: producer::Frame) -> Vec<vecmath::Vector4<f64>> {
        let f = Arc::new(frame);
        self.buffer.clear();
        let mut handles = vec![];
        for la in &self.links {
            let cid = la.id;
            let link = la.link.clone();
            let frame = f.clone();
            handles.push(tokio::spawn(async move {
                LinkResult {
                    id: cid,
                    output: link.lock().unwrap().render(frame),
                }
            }));
        }

        let ok = futures::future::join_all(handles).await;
        for r in ok {
            self.buffer = blending::blend(
                blending::BlendModes::Add,
                mem::take(&mut self.buffer),
                r.unwrap().output,
                1.0,
            );
        }

        return self.buffer.clone();
    }
}

unsafe impl Send for Link {}
#[derive(Serialize)]
pub struct Link {
    name: String,
    steps: Vec<Step>,

    #[serde(skip_serializing)]
    output: Vec<vecmath::Vector4<f64>>,
}

impl Link {
    pub fn create(name: String, steps: Vec<Step>) -> Link {
        Link {
            name,
            steps,
            output: vec![[0.0; 4]; 700],
        }
    }

    pub fn render(&mut self, frame: Arc<producer::Frame>) -> Vec<vecmath::Vector4<f64>> {
        for (i, stp) in self.steps.iter_mut().enumerate() {
            let out = stp.pattern.process(frame.clone());
            if i == 0 {
                self.output = out
            } else {
                self.output = blending::blend(stp.blendmode, mem::take(&mut self.output), out, 1.0);
            }
        }

        return mem::take(&mut self.output);
    }
}

pub struct Step {
    pub pattern: Box<dyn Pattern>,
    pub engine_type: EngineType,
    pub blendmode: blending::BlendModes,
}
impl Serialize for Step {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Step", 3)?;
        state.serialize_field("pattern", &self.pattern.name())?;
        state.serialize_field("engine_type", &self.engine_type.to_string())?;
        state.serialize_field("blendmode", &self.blendmode.to_string())?;
        state.end()
    }
}

#[derive(
    Clone, Copy, Debug, Serialize, Deserialize, strum_macros::ToString, strum_macros::EnumString,
)]
pub enum EngineType {
    Rse,
    Dse,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DeLayer {
    id: i64,
    link: DeLink,
}

#[derive(Serialize, Deserialize, Debug)]
struct DeLink {
    name: String,
    steps: Vec<DeStep>,
}
#[derive(Serialize, Deserialize, Debug)]
struct DeStep {
    pub pattern: String,
    pub engine_type: EngineType,
    pub blendmode: blending::BlendModes,
}

pub struct Controller {
    rse: RSEngine,

    compositor: Arc<tokio::sync::Mutex<Compositor>>,
    db: data::DataLayer,
}

impl Controller {
    pub fn new(db: data::DataLayer, compositor: Arc<tokio::sync::Mutex<Compositor>>) -> Controller {
        let mut rse = RSEngine::new();
        rse.bootstrap().unwrap();

        return Controller {
            rse,

            compositor,
            db,
        };
    }

    pub async fn bootstrap(&mut self) {
        let m = self.compositor.clone();
        let k = m.lock().await;
        let wtf = serde_json::to_string(&k.links).unwrap();

        print!("{}", wtf);
        let p: Vec<DeLayer> = serde_json::from_str(&wtf).unwrap();
        dbg!(p);
    }

    // pub fn add_pattern(&mut self, pattern: &str, engine: EngineType, blendmode: ) {

    // }
}
