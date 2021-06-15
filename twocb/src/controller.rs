use crate::data;
use crate::engines::{DynamicEngine, Engine, Pattern, RSEngine};
use crate::layers::{compositor, DeLink, DeStep, EngineType, Link, Step};

use log::error;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
pub struct Controller {
    rse: RSEngine,
    dse: DynamicEngine,

    pub updates: Arc<Mutex<HashMap<String, mpsc::Sender<Vec<u8>>>>>,

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

            updates: Arc::new(Mutex::new(HashMap::new())),

            compositor,
            data,
        };
    }

    pub async fn bootstrap(&mut self) {
        let mut subscriber = self.data.clone().state.watch_prefix(vec![]);

        for result in self.data.links.iter() {
            match result {
                Ok((k, v)) => {
                    let link: DeLink = serde_json::from_slice(&v).unwrap();
                    println!("{:?}", link);
                    self.load_link(link).await;
                }
                _ => {}
            }
        }

        // let m = Step {
        //     pattern: self.instantiate("foldeddemo", EngineType::Rse).unwrap(),
        //     blend_mode: crate::layers::blending::BlendModes::Add,
        //     engine_type: EngineType::Rse,
        // };

        // let k = Link::create("wooof".to_string(), vec![m]);
        // self.compositor.lock().await.add_link(k).await;

        let knuck = DeLink {
            name: "woo".to_string(),
            steps: vec![DeStep {
                pattern: "foldeddemo".to_string(),
                blendmode: crate::layers::blending::BlendModes::Add,
                engine_type: EngineType::Rse,
            }],
        };

        self.load_link(knuck).await;
    }

    pub fn watch_state_changes(
        db: data::DataLayer,
        changes: Arc<Mutex<HashMap<String, mpsc::Sender<Vec<u8>>>>>,
        compositor: Arc<tokio::sync::Mutex<compositor::Compositor>>,
    ) {
        tokio::spawn(async move {
            let subscriber = db.state.watch_prefix(vec![]);
            for event in subscriber {
                match event {
                    sled::Event::Insert { key, value } => {
                        let k = std::str::from_utf8(&key).unwrap();
                        let l = &mut compositor.lock().await;
                        l.write_pattern_state(k, &value)
                    }
                    _ => {
                        dbg!("SOMETHING ELSE");
                    }
                }
            }
        });
    }

    pub fn watch_layer_changes(&mut self, 
    ) {
        let lc_db = self.data.clone();
        tokio::spawn(async move {
            let subscriber = lc_db.links.watch_prefix(vec![]);
            for event in subscriber {
                match event {
                    sled::Event::Insert { key, value } => {
                        let k = std::str::from_utf8(&key).unwrap();
                        //l.write_pattern_state(k, &value)
                    }
                    sled::Event::Remove  { key }=> {
                       self.remove_link(std::str::from_utf8(&key).unwrap()); 
                    }
                }
             }
        });
    }

    async fn load_link(&mut self, link: DeLink) {
        let mut steps = Vec::new();
        for step in link.steps {
            let (mut tx, mut rx) = mpsc::channel(5);
            let mut newstep = Step {
                pattern: self.instantiate(&step.pattern, step.engine_type).unwrap(),
                blend_mode: step.blendmode,
                engine_type: step.engine_type,

                drx: rx,
            };

            self.updates
                .lock()
                .await
                .insert(link.name.clone() + "_" + &step.pattern, tx);

            let key = &format!("{}_{}", &link.name, &step.pattern);
            match self.data.get_state(key) {
                Some(d) => newstep.pattern.set_state(&d),
                None => {
                    let newstate = newstep.pattern.get_state();
                    self.data.write_state(key, &newstate);
                }
            }

            steps.push(newstep);
        }
        let newlink = Link::create(link.name, steps);
        self.compositor.lock().await.add_link(newlink).await;
    }

    async fn remove_link(&mut self, key: &str) {
        self.compositor.lock().await.remove_link(key);
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
