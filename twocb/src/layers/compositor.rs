use crate::layers::blending;
use crate::layers::{Link, LinkAllocation, LinkResult};
use crate::producer;

use atomic_counter::AtomicCounter;
use std::collections::HashMap;
use std::mem;
use std::sync::{Arc, Mutex};

pub struct Compositor {
    pub links: Vec<LinkAllocation>,
    buffer: Vec<vecmath::Vector4<f64>>,

    counter: atomic_counter::ConsistentCounter,
}

impl Compositor {
    pub fn new() -> Compositor {
        Compositor {
            links: vec![],

            buffer: vec![],
            counter: atomic_counter::ConsistentCounter::new(0),
        }
    }

    pub async fn add_link(&mut self, link: Link) {
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

        let la = LinkAllocation {
            id: self.counter.inc(),
            name: link.name.clone(),
            link: Arc::new(Mutex::new(link)),
        };

        self.links.push(la);
    }

    pub fn remove_link(&mut self, name: String) -> bool {
        return self
            .links
            .iter()
            .position(|n| n.link.lock().unwrap().name == name)
            .map(|e| self.links.remove(e))
            .is_some();
    }

    pub fn get_pattern(&mut self, key: &str) {}

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
