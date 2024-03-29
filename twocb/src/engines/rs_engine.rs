use crate::engines;

use log::info;
use std::collections::HashMap;

mod foldeddemo;
mod strobe;

pub struct RSEngine {
    inventory: HashMap<String, Box<dyn Fn() -> Box<dyn engines::pattern::Pattern + Send>>>,
}

impl engines::Engine for RSEngine {
    fn bootstrap(&mut self) -> anyhow::Result<()> {
        self.inventory.insert(
            "strobe".to_string(),
            Box::new(|| Box::new(strobe::Strobe::new())),
        );
        self.inventory.insert(
            "foldeddemo".to_string(),
            Box::new(|| Box::new(foldeddemo::FoldedDemo::new())),
        );

        info!("Started RSEngine: {:?}", self.list());
        Ok(())
    }

    fn list(&self) -> Vec<String> {
        self.inventory.keys().cloned().collect()
    }

    fn instantiate_pattern(&self, name: &str) -> Option<Box<dyn engines::pattern::Pattern + Send>> {
        self.inventory.get(name).map(|p| p())
    }
}

impl RSEngine {
    pub fn new() -> RSEngine {
        return RSEngine {
            inventory: HashMap::new(),
        };
    }
}
