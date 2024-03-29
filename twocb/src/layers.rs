use crate::engines::{DynamicEngine, Engine, Pattern, RSEngine};
use crate::producer;
use serde::{
    ser::{SerializeStruct, Serializer},
    Deserialize, Serialize,
};

use std::fmt;
use std::mem;
use std::sync::{Arc, Mutex};
use strum_macros;

pub mod blending;
pub mod compositor;

#[derive(Serialize)]
pub struct LinkAllocation {
    #[serde(skip_serializing)]
    id: usize,
    #[serde(skip_serializing)]
    name: String,

    #[serde(flatten)]
    pub link: Arc<Mutex<Link>>,
}

unsafe impl Send for LinkAllocation {}

#[derive(Debug)]
struct LinkResult {
    id: usize,
    name: String,
    opacity: f64,
    output: Vec<vecmath::Vector4<f64>>,
}

#[derive(Serialize)]
pub struct Link {
    pub name: String,
    pub opacity: f64,
    pub steps: Vec<Step>,

    #[serde(skip_serializing)]
    output: Vec<vecmath::Vector4<f64>>,
}

unsafe impl Send for Link {}

impl Link {
    pub fn create(name: String, steps: Vec<Step>) -> Link {
        Link {
            name,
            steps,
            opacity: 1.0,
            output: vec![[0.0; 4]; 700],
        }
    }

    pub fn render(&mut self, frame: Arc<producer::Frame>) -> Vec<vecmath::Vector4<f64>> {
        for (i, stp) in self.steps.iter_mut().enumerate() {
            let out = stp.pattern.process(frame.clone());
            if i == 0 {
                self.output = out
            } else {
                self.output =
                    blending::blend(stp.blend_mode, mem::take(&mut self.output), out, 1.0);
            }
        }

        return mem::take(&mut self.output);
    }
}

unsafe impl Send for Step {}

pub struct Step {
    pub pattern: Box<dyn Pattern + Send>,
    pub engine_type: EngineType,
    pub blend_mode: blending::BlendModes,

    pub drx: tokio::sync::mpsc::Receiver<Vec<u8>>,
}
impl Serialize for Step {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Step", 3)?;
        state.serialize_field("pattern", &self.pattern.name())?;
        state.serialize_field("engine_type", &self.engine_type.to_string())?;
        state.serialize_field("blendmode", &self.blend_mode.to_string())?;
        state.end()
    }
}

impl fmt::Debug for Step {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Step")
            .field("pattern", &self.pattern.name())
            .field("engine_type", &self.engine_type.to_string())
            .field("blendmode", &self.blend_mode.to_string())
            .finish()
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
pub struct DeLink {
    pub name: String,
    pub steps: Vec<DeStep>,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct DeStep {
    pub pattern: String,
    pub engine_type: EngineType,
    pub blendmode: blending::BlendModes,
}
