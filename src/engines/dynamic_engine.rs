use crate::engines;
use crate::producer;
use glob::glob;
use log::{debug, info, warn};
use rusty_v8 as v8;
use std::collections::HashMap;
use std::sync::Arc;

pub struct DynamicEngine {
    inventory: HashMap<String, Box<dyn Fn() -> Box<dyn engines::pattern::Pattern>>>,
    pattern_folder: String,
}

impl engines::Engine for DynamicEngine {
    fn bootstrap(&mut self) -> anyhow::Result<()> {
        self.init_patterns();
        self.watch();
        initalize_runtime();
        Ok(())
    }

    fn list(&self) -> Vec<String> {
        self.inventory.keys().cloned().collect()
    }

    fn instantiate_pattern(&self, name: &str) -> Option<Box<dyn engines::pattern::Pattern>> {
        self.inventory.get(name).map(|p| p())
    }
}

impl DynamicEngine {
    pub fn new(pattern_folder: &str) -> DynamicEngine {
        return DynamicEngine {
            inventory: HashMap::new(),
            pattern_folder: pattern_folder.to_string(),
        };
    }

    fn init_patterns(&mut self) {
        for entry in glob(&self.pattern_folder).expect("Failed to read dynamic pattern") {
            match entry {
                Ok(path) => {
                    self.inventory.insert(
                        path.file_name().unwrap().to_str().unwrap().to_string(),
                        Box::new(move || Box::new(DynamicPattern::new(path.clone()))),
                    );
                }
                _ => {}
            }
        }
    }

    fn watch(&mut self) {}
}

fn initalize_runtime() {
    let platform = v8::new_default_platform().unwrap();
    v8::V8::initialize_platform(platform);
    v8::V8::initialize();
    debug!("Initalized the V8 platform.");
}

fn shutdown_runtime() {
    v8::V8::shutdown_platform();
}

struct DynamicPattern {
    path: std::path::PathBuf,

    isolate: Option<v8::OwnedIsolate>,
    context: v8::Global<v8::Context>,
}

impl DynamicPattern {
    pub fn new(path: std::path::PathBuf) -> DynamicPattern {
        let mut isolate = v8::Isolate::new(v8::CreateParams::default());
        let global_context;
        {
            let handle_scope = &mut v8::HandleScope::new(&mut isolate);
            let context = v8::Context::new(handle_scope);
            global_context = v8::Global::new(handle_scope, context);
        }
        return DynamicPattern {
            path,
            isolate: Some(isolate),
            context: global_context,
        };
    }
}

impl engines::pattern::Pattern for DynamicPattern {
    fn name(&self) -> String {
        return "ok".to_string();
    }

    fn process(&mut self, _frame: Arc<producer::Frame>) -> Vec<vecmath::Vector4<f64>> {
        return vec![[1.0, 0.0, 1.0, 1.0]; 864];
    }
}
