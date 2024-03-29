use crate::data;
use crate::engines::{DynamicEngine, Engine, Pattern, RSEngine};
use crate::layers::{compositor, DeLink, EngineType, Link, Step};
use crate::pixels;

use log::error;
use std::sync::Arc;
use tokio::sync::mpsc;
pub struct Controller {
    rse: RSEngine,
    dse: DynamicEngine,

    compositor: Arc<tokio::sync::Mutex<compositor::Compositor>>,
    data: data::DataLayer,
}
unsafe impl Send for Controller {}
unsafe impl Sync for Controller {}

impl Controller {
    pub fn new(
        data: data::DataLayer,
        compositor: Arc<tokio::sync::Mutex<compositor::Compositor>>,
        mapping: Vec<pixels::Pixel>,
    ) -> Controller {
        let mut rse = RSEngine::new();
        rse.bootstrap().unwrap();

        let mut dse = DynamicEngine::new("files/dynamic/", "files/support/global.js", mapping);
        dse.bootstrap().unwrap();

        return Controller {
            rse,
            dse,

            compositor,
            data,
        };
    }

    pub async fn bootstrap(&mut self) {
        let order = self.lookup_order().await.unwrap();
        for (index, entry) in order.iter().enumerate() {
            let result = self.data.links.get(&entry.name).unwrap();
            match result {
                Some(v) => {
                    let link: DeLink = serde_json::from_slice(&v).unwrap();
                    println!("{:?}", link);
                    match self.load_link(link).await {
                        Err(e) => {
                            error!("Could not load link: {}", e);
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    }

    pub fn watch_state_changes(
        db: data::DataLayer,
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
                    _ => {}
                }
            }
        });
    }

    async fn load_link(&mut self, link: DeLink) -> Result<(), &str> {
        let store = serde_json::to_vec(&link).unwrap();
        let mut steps = Vec::new();
        for step in link.steps {
            let (tx, rx) = mpsc::channel(5);
            let pt = match self.instantiate(&step.pattern, step.engine_type) {
                Ok(pattern) => pattern,
                Err(e) => {
                    return Err(e);
                }
            };
            let mut newstep = Step {
                pattern: pt,
                blend_mode: step.blendmode,
                engine_type: step.engine_type,

                drx: rx,
            };

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

        let key = link.name.clone();
        let newlink = Link::create(link.name, steps);
        self.compositor.lock().await.remove_link(&key);
        self.compositor.lock().await.push(newlink);
        self.data.write_layer(&key, &store);

        self.sync_order().await;

        Ok(())
    }

    async fn sync_order(&mut self) {
        let order = serde_json::to_vec(self.compositor.lock().await.get_order()).unwrap();
        self.data.global.insert("order", order);
    }

    pub async fn lookup_order(&self) -> Result<Vec<compositor::Order>, &str> {
        match self.data.global.get("order").unwrap() {
            Some(v) => {
                let order: Vec<compositor::Order> = serde_json::from_slice(&v).unwrap();
                Ok(order)
            }
            None => return Err("lookup order error"),
        }
    }

    pub async fn remove_link(&mut self, key: &str) -> bool {
        self.data.links.remove(&key);
        self.data.clear_states_for_link(&key);
        let k = self.compositor.lock().await.remove_link(key);
        self.sync_order().await;
        return k;
    }

    pub async fn add_link(&mut self, new_link: DeLink) -> Result<(), &str> {
        self.load_link(new_link).await
    }

    pub async fn get_links_string(&self) -> String {
        serde_json::to_string(&self.compositor.lock().await.links).unwrap()
    }

    pub async fn set_opacity(&mut self, link: &str, opacity: f64) {
        match self
            .compositor
            .lock()
            .await
            .links
            .iter()
            .find(|n| n.link.lock().unwrap().name == link)
        {
            Some(v) => {
                v.link.lock().unwrap().opacity = opacity;
            }
            None => {
                error!("could not find shit");
            }
        }
    }

    fn instantiate(
        &mut self,
        name: &str,
        engine_type: EngineType,
    ) -> Result<Box<dyn Pattern + Send>, &'static str> {
        match engine_type {
            EngineType::Rse => match self.rse.instantiate_pattern(name) {
                Some(v) => return Ok(v),
                None => return Err("Could not find RSE pattern"),
            },
            EngineType::Dse => match self.dse.instantiate_pattern(name) {
                Some(v) => return Ok(v),
                None => return Err("Could not find DSE pattern"),
            },
        }
    }
}
