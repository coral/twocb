use crate::engines;
use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;

mod strobe;

pub struct RSEngine {
    inventory: HashMap<String, Box<dyn Fn() -> Box<dyn engines::pattern::Pattern>>>,
}

impl engines::Engine for RSEngine {
    fn bootstrap(&mut self) -> anyhow::Result<()> {
        self.inventory.insert(
            "strobe".to_string(),
            Box::new(|| Box::new(strobe::Strobe::new())),
        );
        Ok(())
    }

    // fn list(&mut self) -> Vec<Arc<dyn engines::pattern::Pattern>> {
    //     vec![Arc::new(strobe::Strobe::new())]
    // }

    fn list(&self) -> Vec<String> {
        self.inventory.keys().cloned().collect()
    }

    fn instantiate_pattern(&self, name: &str) -> Option<Box<dyn engines::pattern::Pattern>> {
        self.inventory.get(name).map(|p| p())
    }

    // fn list(&mut self) -> Vec<engines::Pattern> {
    //     vec![
    //         engines::Pattern {
    //             name: "first pattern".to_string(),
    //         },
    //         engines::Pattern {
    //             name: "second pattern".to_string(),
    //         },
    //     ]
    // }
}

impl RSEngine {
    pub fn new() -> RSEngine {
        return RSEngine {
            inventory: HashMap::new(),
        };
    }

    pub fn hello(&mut self) {
        println!("OKIDOKI");
    }
}
