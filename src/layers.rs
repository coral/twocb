use crate::engines::Pattern;
use crate::producer;
use atomic_counter::{AtomicCounter, ConsistentCounter};
use std::mem;
use std::sync::Arc;
use std::sync::Mutex;
use tokio::spawn;

pub mod blending;

pub struct Manager {
    links: Vec<LinkAllocation>,
    buffer: Vec<vecmath::Vector4<f64>>,

    counter: atomic_counter::ConsistentCounter,
}

struct LinkAllocation {
    id: usize,
    link: Arc<Mutex<Link>>,
}

#[derive(Debug, Clone)]
struct LinkResult {
    id: usize,
    output: Vec<vecmath::Vector4<f64>>,
}

impl Manager {
    pub fn new() -> Manager {
        Manager {
            links: vec![],
            buffer: vec![],
            counter: atomic_counter::ConsistentCounter::new(0),
        }
    }

    pub fn add_link(&mut self, link: Link) {
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
pub struct Link {
    name: String,
    steps: Vec<Step>,
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
            let mut out = stp.pattern.process(frame.clone());
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
    pub blendmode: blending::BlendModes,
}

// pub trait Step {
//     fn init(&self);
//     fn query_parameters(&self) -> Vec<String>;
//     fn query_requirements(&self);
//     fn render(&self);
// }
