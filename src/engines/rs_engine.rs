use crate::engines;
use anyhow::Result;
use std::sync::Arc;

mod strobe;

pub struct RSEngine {}

impl engines::Engine for RSEngine {
    fn bootstrap(&mut self) -> anyhow::Result<()> {
        Ok(())
    }

    fn list(&mut self) -> Vec<Arc<dyn engines::pattern::Pattern>> {
        vec![Arc::new(strobe::Strobe::new())]
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
        return RSEngine {};
    }

    pub fn hello(&mut self) {
        println!("OKIDOKI");
    }
}
