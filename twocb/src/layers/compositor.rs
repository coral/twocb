use crate::layers::blending;
use crate::layers::{Link, LinkAllocation, LinkResult};
use crate::producer;

use atomic_counter::AtomicCounter;
use std::collections::HashMap;
use std::hash::Hash;
use std::mem;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct Compositor {
    pub links: Vec<LinkAllocation>,
    lookup: HashMap<String, Arc<Mutex<Link>>>,
    buffer: Vec<vecmath::Vector4<f64>>,

    counter: atomic_counter::ConsistentCounter,
}

impl Compositor {
    pub fn new() -> Compositor {
        Compositor {
            links: vec![],
            lookup: HashMap::new(),

            buffer: vec![],
            counter: atomic_counter::ConsistentCounter::new(0),
        }
    }

    pub fn add_link(&mut self, link: Link) {
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

        let name = link.name.clone();
        //let arclink = Arc::new(Mutex::new(link));
        let la = LinkAllocation {
            id: self.counter.inc(),
            name,
            link: link,
        };

        //self.lookup.insert(name.clone() + "_" + , v)

        self.links.push(la);
    }

    pub fn remove_link(&mut self, name: &str) -> bool {
        return self
            .links
            .iter()
            .position(|n| n.link.name == name)
            .map(|e| self.links.remove(e))
            .is_some();
    }

    pub async fn write_pattern_state(&mut self, key: &str, data: &[u8]) {
        for l in &mut self.links {
            for step in &mut l.link.steps {
                let nk = l.name.clone() + "_" + &step.pattern.name();
                if &nk == key {
                    step.pattern.set_state(data);
                }
            }
        }
    }

    pub async fn render(&mut self, frame: producer::Frame) -> Vec<vecmath::Vector4<f64>> {
        let f = Arc::new(frame);
        self.buffer.clear();
        let mut handles = vec![];
        for la in &self.links {
            let cid = la.id;
            let link = &la.link;
            let frame = f.clone();
            handles.push(tokio::spawn(async move {
                // let mut l = link.lock();
                // let res = l.render(frame);
                LinkResult {
                    id: cid,
                    output: link.render(frame).await,
                }

                // match link.lock() {
                //     Ok(v) => LinkResult {
                //         id: cid,
                //         output: v.render(frame).await,
                //     },
                //     _ => LinkResult {
                //         id: cid,
                //         output: vec![[1.0, 0.0, 1.0, 1.0]; 1],
                //     },
                // }
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
