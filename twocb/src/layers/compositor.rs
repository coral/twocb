use crate::layers::blending;
use crate::layers::{Link, LinkAllocation, LinkResult};
use crate::producer;

use atomic_counter::AtomicCounter;
use std::collections::HashMap;
use std::hash::Hash;
use std::mem;
use std::sync::{Arc, Mutex};

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
        let name = link.name.clone();
        let arclink = Arc::new(Mutex::new(link));
        let la = LinkAllocation {
            id: self.counter.inc(),
            name,
            link: arclink.clone(),
        };

        self.links.push(la);
    }

    pub fn remove_link(&mut self, name: &str) -> bool {
        return self
            .links
            .iter()
            .position(|n| n.link.lock().unwrap().name == name)
            .map(|e| self.links.remove(e))
            .is_some();
    }

    pub fn write_pattern_state(&mut self, key: &str, data: &[u8]) {
        for l in &mut self.links {
            for step in &mut l.link.lock().unwrap().steps {
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
            let link = la.link.clone();
            let frame = f.clone();
            handles.push(tokio::spawn(async move {
                let mut re = link.lock().unwrap();
                LinkResult {
                    id: cid,
                    opacity: re.opacity,
                    output: re.render(frame),
                }
            }));
        }

        let ok = futures::future::join_all(handles).await;
        for r in ok {
            match r {
                Ok(result) => {
                    self.buffer = blending::blend(
                        blending::BlendModes::Add,
                        mem::take(&mut self.buffer),
                        result.output,
                        1.0,
                    );
                }
                Err(e) => {}
            }
        }

        return self.buffer.clone();
    }
}
