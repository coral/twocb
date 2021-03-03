use crate::engines::Pattern;
use futures::future;
use std::mem;
use std::sync::Arc;
use std::sync::Mutex;
use tokio::spawn;

pub mod blending;

pub struct Manager {
    links: Vec<Arc<Mutex<Link>>>,
}

impl Manager {
    pub fn new() -> Manager {
        Manager { links: vec![] }
    }

    pub fn add_link(&mut self, link: Link) {
        self.links.push(Arc::new(Mutex::new(link)));
    }

    pub fn remove_link(&mut self, name: String) -> bool {
        return self
            .links
            .iter()
            .position(|n| n.lock().unwrap().name == name)
            .map(|e| self.links.remove(e))
            .is_some();
    }

    pub async fn render(&mut self) {
        let mut handles = vec![];
        for link in &self.links {
            let link = link.clone();
            handles.push(tokio::spawn(async move {
                let otp = link.lock().unwrap().render();
                dbg!(otp);
            }));
        }

        futures::future::join_all(handles).await;
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
            output: vec![[0.0; 4]; 100],
        }
    }

    pub fn render(&mut self) -> Vec<vecmath::Vector4<f64>> {
        for (i, stp) in self.steps.iter().enumerate() {
            let mut out = stp.pattern.as_ref().process();
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
    pub pattern: Arc<dyn Pattern>,
    pub blendmode: blending::BlendModes,
}

// pub trait Step {
//     fn init(&self);
//     fn query_parameters(&self) -> Vec<String>;
//     fn query_requirements(&self);
//     fn render(&self);
// }
