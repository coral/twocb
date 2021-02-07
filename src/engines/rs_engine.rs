use crate::engines;
use anyhow::Result;

mod strobe;

pub struct RSEngine {}

impl engines::Engine for RSEngine {
    fn bootstrap(&mut self) -> anyhow::Result<()> {
        Ok(())
    }

    fn list(&mut self) -> Vec<Box<engines::pattern::Pattern>> {
        vec![Box::new(strobe::Strobe {})]
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
