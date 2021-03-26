use crate::engines;
use glob::glob;
use log::{debug, info, warn};
use rusty_v8 as v8;
use std::collections::HashMap;

pub struct DynamicEngine {
    inventory: HashMap<String, Box<dyn Fn() -> Box<dyn engines::pattern::Pattern>>>,
    pattern_folder: String,
}

impl engines::Engine for DynamicEngine {
    fn bootstrap(&mut self) -> anyhow::Result<()> {
        self.list_patterns();
        self.watch();
        initalize_runtime();
        Ok(())
    }

    fn list(&self) -> Vec<String> {
        return vec![];
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

    fn list_patterns(&mut self) {
        let patterns = glob(&self.pattern_folder).expect("Failed to read dynamic pattern");
        for entry in glob(&self.pattern_folder).expect("Failed to read dynamic pattern") {
            match entry {
                Ok(path) => println!("{:?}", path.display()),
                Err(e) => println!("{:?}", e),
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
