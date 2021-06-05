use crate::data;
use crate::engines::{DynamicEngine, Engine, Pattern, RSEngine};
use crate::layers::{compositor, DeLink, DeStep, EngineType, Link, Step};

use std::sync::{Arc, Mutex};

pub struct Controller {
    rse: RSEngine,

    compositor: Arc<tokio::sync::Mutex<compositor::Compositor>>,
    db: data::DataLayer,
}

impl Controller {
    pub fn new(
        db: data::DataLayer,
        compositor: Arc<tokio::sync::Mutex<compositor::Compositor>>,
    ) -> Controller {
        let mut rse = RSEngine::new();
        rse.bootstrap().unwrap();

        return Controller {
            rse,

            compositor,
            db,
        };
    }

    pub async fn bootstrap(&mut self) {
        for result in self.db.links.iter() {
            match result {
                Ok((_, v)) => {
                    let link: DeLink = serde_json::from_slice(&v).unwrap();
                    self.load_link(link).await;
                }
                _ => {}
            }
        }
    }

    async fn load_link(&mut self, link: DeLink) {
        let mut steps = Vec::new();
        for step in link.steps {
            steps.push(Step {
                pattern: self.instantiate(&step.pattern, step.engine_type).unwrap(),
                blend_mode: step.blendmode,
                engine_type: step.engine_type,
            });
        }
        let newlink = Link::create(link.name, steps);
        self.compositor.lock().await.add_link(newlink).await;
    }

    fn instantiate(
        &mut self,
        name: &str,
        engine_type: EngineType,
    ) -> Result<Box<dyn Pattern>, &'static str> {
        match engine_type {
            Rse => match self.rse.instantiate_pattern(name) {
                Some(v) => return Ok(v),
                None => return Err("Could not find RSE pattern"),
            },
            _ => Err("Could not find pattern"),
        }
    }
}
