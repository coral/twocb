use crate::data;
use crate::engines::{DynamicEngine, Engine, Pattern, RSEngine};
use crate::layers::{compositor, DeLink, DeStep, EngineType, Link, Step};

use log::error;
use std::sync::{Arc, Mutex};
pub struct Controller {
    rse: RSEngine,
    dse: DynamicEngine,

    compositor: Arc<tokio::sync::Mutex<compositor::Compositor>>,
    data: data::DataLayer,
}

impl Controller {
    pub fn new(
        data: data::DataLayer,
        compositor: Arc<tokio::sync::Mutex<compositor::Compositor>>,
    ) -> Controller {
        let mut rse = RSEngine::new();
        rse.bootstrap().unwrap();

        let mut dse = DynamicEngine::new("files/dynamic/*.js", "files/support/global.js");
        dse.bootstrap().unwrap();

        return Controller {
            rse,
            dse,

            compositor,
            data,
        };
    }

    pub async fn bootstrap(&mut self) {
        let mut subscriber = self.data.clone().db.watch_prefix(vec![]);
        tokio::spawn(async move {
            for event in subscriber.take(1) {
                match event {
                    sled::Event::Insert { key, value } => {
                        dbg!(key, value);
                    }
                    _ => {}
                }
            }
        });

        for result in self.data.links.iter() {
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
            let mut newstep = Step {
                pattern: self.instantiate(&step.pattern, step.engine_type).unwrap(),
                blend_mode: step.blendmode,
                engine_type: step.engine_type,
            };

            let key = &format!("{}_{}", &link.name, &step.pattern);
            match self.data.subscribe(key).await {
                Ok(v) => match self.data.get_state(key) {
                    Some(d) => newstep.pattern.set_state(d),
                    None => {
                        let newstate = newstep.pattern.get_state();
                        self.data.write_state(key, &newstate);
                    }
                },
                Err(err) => error!("Could not subscribe to key updates: {}", err),
            }

            steps.push(newstep);
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
            Dse => match self.dse.instantiate_pattern(name) {
                Some(v) => return Ok(v),
                None => return Err("Could not find DSE pattern"),
            },
            _ => Err("Could not find pattern"),
        }
    }
}
