use crate::controller;
use crate::data;
use log::error;
use midimapper;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::Mutex;
mod featuremap;
use serde_json::{json, Value};

pub struct MidiSurface {
    surface_mapping: midimapper::Mapping,
    routing: HashMap<String, featuremap::Out>,

    controller: Arc<Mutex<controller::Controller>>,
    data: data::DataLayer,
}

impl MidiSurface {
    pub fn new(
        surface_map: &Path,
        feature_map: &Path,

        ctrl: Arc<Mutex<controller::Controller>>,
        data: data::DataLayer,
    ) -> Result<MidiSurface, SurfaceError> {
        let smap = match midimapper::Mapping::load_from_file(surface_map) {
            Ok(v) => v,
            Err(e) => return Err(SurfaceError::SurfaceMapError(e.to_string())),
        };

        let fmap = match featuremap::load_from_file(feature_map) {
            Ok(v) => v,
            Err(e) => return Err(SurfaceError::FeatureMapError(e.to_string())),
        };

        let mut routes = HashMap::new();

        for route in fmap {
            routes.insert(route.in_field, route.out);
        }

        Ok(MidiSurface {
            surface_mapping: smap,
            routing: routes,

            controller: ctrl,
            data,
        })
    }

    pub async fn watch(&mut self) {
        let mut input = match midimapper::MIDIMapper::new(self.surface_mapping.clone()) {
            Ok(v) => v,
            Err(e) => {
                error!("{}", e);
                return;
            }
        };

        let mut ch = input.get_channel();

        tokio::spawn(async move {
            input.run(0).await;
        });

        loop {
            let message = match ch.recv().await {
                Some(v) => v,
                None => continue,
            };
            match message {
                midimapper::FeatureResult::Value(name, val) => match self.routing.get(&name) {
                    Some(v) => match v.type_field {
                        featuremap::OutType::Parameter => {
                            let s = self.data.state.get(&v.pattern_key).unwrap().unwrap();
                            let mut decode: Value = serde_json::from_slice(&s).unwrap();
                            if decode.get(&v.state_key).is_some() {
                                let newvalue = MidiSurface::scale_param(val as f64, v.min, v.max);
                                *decode.get_mut(&v.state_key).unwrap() = json!(newvalue);
                                self.data
                                    .state
                                    .insert(&v.pattern_key, serde_json::to_vec(&decode).unwrap());
                            }
                        }
                        featuremap::OutType::Opacity => {
                            let order = self
                                .controller
                                .lock()
                                .await
                                .set_opacity(
                                    &v.path,
                                    MidiSurface::scale_param(val as f64, 0.0, 1.0),
                                )
                                .await;
                        }
                        _ => {}
                    },
                    None => {}
                },
                _ => {}
            }
        }
    }

    fn scale_param(val: f64, min: f64, max: f64) -> f64 {
        if min < max {
            return (max - min) * (val * (1.0 / 127.0)) + min;
        } else {
            return (min - max) * (1.0 - (val * (1.0 / 127.0))) + max;
        }
    }
}

#[derive(Error, Debug)]
pub enum SurfaceError {
    #[error("Cannot load control surface map: {0}")]
    SurfaceMapError(String),
    #[error("Cannot load control feature map: {0}")]
    FeatureMapError(String),
    #[error("MIDI error: {0}")]
    MIDIError(String),
}
