use crate::engines;
use crate::pixels;
use crate::producer;
use crossbeam_channel::select;
use glob::glob;
use log::{error, info};
use notify::{EventKind, Watcher};
use std::convert::TryFrom;
use std::fs;
use std::pin::pin;
use std::sync::Arc;
use thiserror::Error;

pub struct DynamicEngine {
    pattern_folder: String,
    global_scope: String,
    mapping: Vec<pixels::Pixel>,
}

impl engines::Engine for DynamicEngine {
    fn bootstrap(&mut self) -> anyhow::Result<()> {
        initalize_runtime();
        Ok(())
    }

    fn list(&self) -> Vec<String> {
        match glob(&self.pattern_folder) {
            Ok(p) => {
                let mut rp = Vec::new();
                for entry in p {
                    match entry {
                        Ok(path) => {
                            rp.push(path.file_name().unwrap().to_str().unwrap().to_string())
                        }
                        _ => {}
                    }
                }
                rp
            }
            Err(_e) => Vec::new(),
        }
    }

    fn instantiate_pattern(&self, name: &str) -> Option<Box<dyn engines::pattern::Pattern + Send>> {
        let patternpath = std::path::Path::new(&self.pattern_folder).join(name);
        match DynamicPattern::new(patternpath.to_path_buf(), self.mapping.clone()) {
            Ok(d) => return Some(Box::new(d)),
            Err(e) => {
                error!("{}", e);
                return None;
            }
        }
    }
}

impl DynamicEngine {
    pub fn new(
        pattern_folder: &str,
        global_scope: &str,
        mapping: Vec<pixels::Pixel>,
    ) -> DynamicEngine {
        let _global = fs::read_to_string(global_scope)
            .expect("Something went wrong reading the global.js file");
        let code =
            fs::read_to_string(&global_scope).expect("Something went wrong reading the file");

        return DynamicEngine {
            pattern_folder: pattern_folder.to_string(),
            global_scope: code,
            mapping,
        };
    }
}

fn initalize_runtime() {
    let platform = v8::new_default_platform(0, false).make_shared();
    v8::V8::initialize_platform(platform);
    v8::V8::initialize();
    info!("Initalized the V8 platform.");
}

struct DynamicHolder {
    patternname: String,

    frame_channel: crossbeam_channel::Sender<Arc<producer::Frame>>,
    result_channel: crossbeam_channel::Receiver<Result<Vec<vecmath::Vector4<f64>>, DynamicError>>,
    cancel_channel: crossbeam_channel::Sender<bool>,

    setstate_channel: crossbeam_channel::Sender<String>,
    getstate_channel: crossbeam_channel::Receiver<Result<String, DynamicError>>,
    reqstate_channel: crossbeam_channel::Sender<bool>,

    _watcher: notify::RecommendedWatcher,
}

impl engines::pattern::Pattern for DynamicHolder {
    fn name(&self) -> String {
        return self.patternname.clone();
    }

    fn process(&mut self, frame: Arc<producer::Frame>) -> Vec<vecmath::Vector4<f64>> {
        match self.frame_channel.send(frame) {
            Err(e) => {
                error!("Could not send frame to dynamic pattern: {}", e);
            }
            _ => {}
        }

        match self.result_channel.recv() {
            Ok(v) => match v {
                Ok(output) => output,
                Err(_e) => {
                    vec![[1.0, 0.0, 1.0, 1.0]; 864]
                }
            },
            Err(_e) => {
                vec![[1.0, 0.0, 1.0, 1.0]; 864]
            }
        }
    }

    fn get_state(&self) -> Vec<u8> {
        self.reqstate_channel.send(true);

        match self.getstate_channel.recv() {
            Ok(v) => match v {
                Ok(state) => {
                    return state.as_bytes().to_vec();
                }
                Err(e) => {
                    error!("Get state error: {}", e);
                    return "{}".as_bytes().to_vec();
                }
            },
            _ => {}
        }
        return "{}".as_bytes().to_vec();
    }

    fn set_state(&mut self, data: &[u8]) {
        let state = std::str::from_utf8(&data).unwrap().to_string();
        match self.setstate_channel.send(state) {
            Err(e) => {
                error!("Could not send state to dynamic pattern: {}", e);
            }
            _ => {}
        }
    }
}

struct DynamicPattern {
    isolate: v8::OwnedIsolate,
    context: v8::Global<v8::Context>,
    setup: Option<v8::Global<v8::Function>>,
    get_state: Option<v8::Global<v8::Function>>,
    set_state: Option<v8::Global<v8::Function>>,
    render: Option<v8::Global<v8::Function>>,
}

impl DynamicPattern {
    pub fn new(
        path: std::path::PathBuf,
        mapping: Vec<pixels::Pixel>,
    ) -> Result<DynamicHolder, std::io::Error> {
        let global = match fs::read_to_string("files/support/global.js") {
            Ok(v) => v,
            Err(e) => return Err(e),
        };

        //Fix this later, this is so dumb
        let codepath = path.as_path();
        let patternname = match fs::read_to_string(codepath) {
            Ok(_v) => String::from(path.file_name().unwrap().to_str().unwrap()),
            Err(e) => return Err(e),
        };

        let (frame_tx, frame_rx) = crossbeam_channel::unbounded();
        let (result_tx, result_rx) = crossbeam_channel::unbounded();
        let (cancel_tx, cancel_rx) = crossbeam_channel::unbounded();
        let (reload_tx, reload_rx) = crossbeam_channel::unbounded();

        let (setstate_tx, setstate_rx) = crossbeam_channel::unbounded();
        let (getstate_tx, getstate_rx) = crossbeam_channel::unbounded();
        let (reqstate_tx, reqstate_rx) = crossbeam_channel::unbounded();

        let mut watcher: notify::RecommendedWatcher =
            notify::recommended_watcher(move |res| match res {
                Ok(event) => match reload_tx.send(event) {
                    Err(_e) => return,
                    _ => {}
                },
                Err(e) => println!("watch error: {:?}", e),
            })
            .unwrap();

        watcher
            .watch(
                &fs::canonicalize(codepath).unwrap(),
                notify::RecursiveMode::Recursive,
            )
            .unwrap();

        let c = codepath.to_path_buf();

        std::thread::spawn(move || {
            loop {
                let code = match fs::read_to_string(c.clone()) {
                    Ok(v) => v,
                    _ => "".to_string(),
                };

                let mut isolate = v8::Isolate::new(v8::CreateParams::default());
                let global_context;
                {
                    v8::scope!(let scope, &mut isolate);
                    let context = v8::Context::new(&scope, Default::default());
                    global_context = v8::Global::new(&scope, context);
                }

                let mut d = DynamicPattern {
                    isolate: isolate,
                    context: global_context,
                    setup: None,
                    get_state: None,
                    set_state: None,
                    render: None,
                };

                match d.load(&global, &code) {
                    Ok(_v) => {}
                    Err(e) => {
                        error!("{}", e);
                        match reload_rx.recv() {
                            Ok(_) => continue,
                            _ => {}
                        }
                    }
                }
                d.setup(mapping.clone());

                loop {
                    select! {

                        //Render frame
                        recv(frame_rx) -> frame => {
                            match frame {
                                Ok(frame) => match d.dynamic_process(frame) {
                                    Ok(output) => match result_tx.send(Ok(output)) {
                                        Err(e) => {
                                            error!("Dynamic pattern send error: {}", e);
                                        }
                                        _ => {}
                                    },
                                    Err(e) => {
                                        error!("Dynamic error: {}", e);
                                        match result_tx.send(Err(DynamicError::ProduceError)) {
                                            Err(e) => {
                                                error!("Dynamic pattern send error: {}", e);
                                            }
                                            _ => {}
                                        }
                                        match reload_rx.recv() {
                                            Ok(_) => break,
                                            _ => {}
                                        }
                                    }
                                },
                                Err(e) => {
                                    error!("Dynamic pattern recieve error: {}", e);
                                }
                            }
                        },

                        //Cancel (kill pattern)
                        recv(cancel_rx) -> cancel => {
                            match cancel {
                                Ok(_) => {
                                    return;
                                }
                                _ => {}
                            }
                        }

                        //Set state
                        recv(setstate_rx) -> setstate => {
                            match setstate {
                                Ok(state) => d.inject_state(state),
                                _ => {}
                            }
                        }

                        //Request for internal state
                        recv(reqstate_rx) -> reqstate => {
                            match reqstate {
                                Ok(_) => {
                                    match d.extract_state() {
                                        Ok(state) => {
                                            getstate_tx.send(Ok(state));
                                        },
                                        Err(e) => {
                                            getstate_tx.send(Err(DynamicError::StateError(e.to_string())));
                                        }
                                    }
                                },
                                _ => {}
                            }
                        }

                        //Reload on file update
                        recv(reload_rx) -> reload => {
                            match reload {
                                Ok(event) => {
                                    match event.kind {
                                        EventKind::Modify(mf) => match mf {
                                            notify::event::ModifyKind::Data(_) => {
                                                break;
                                            }
                                            _ => {}
                                        },
                                        _ => {}
                                    };
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
        });

        return Ok(DynamicHolder {
            patternname: patternname,

            frame_channel: frame_tx,
            result_channel: result_rx,
            cancel_channel: cancel_tx,

            setstate_channel: setstate_tx,
            getstate_channel: getstate_rx,
            reqstate_channel: reqstate_tx,

            _watcher: watcher,
        });
    }

    fn load(&mut self, global: &str, code: &str) -> Result<(), DynamicError> {
        v8::scope!(let scope, &mut self.isolate);
        let context = v8::Local::new(&scope, &self.context);
        let scope = &mut v8::ContextScope::new(scope, context);

        //Load global
        match DynamicPattern::execute(scope, global) {
            Err(e) => return Err(e),
            _ => {}
        }

        //Load code
        match DynamicPattern::execute(scope, code) {
            Err(e) => return Err(e),
            _ => {}
        }

        //Bind function handlers
        self.setup = DynamicPattern::bind_function(scope, context, "_setup");
        self.get_state = DynamicPattern::bind_function(scope, context, "_getState");
        self.set_state = DynamicPattern::bind_function(scope, context, "_setState");
        self.render = DynamicPattern::bind_function(scope, context, "_internalRender");

        Ok(())
    }

    fn execute(scope: &mut v8::PinScope<'_, '_>, code: &str) -> Result<(), DynamicError> {
        let code = v8::String::new(scope, code).unwrap();
        let script = match v8::Script::compile(scope, code, None) {
            Some(script) => script,
            None => {
                return Err(DynamicError::CompileError(
                    "I haven't figured out how to extract compilation errors yet".to_string(),
                ));
            }
        };
        //Execute script to load functions into memory
        match script.run(scope) {
            Some(_v) => {}
            None => {
                return Err(DynamicError::ScriptRunError(
                    "I haven't figured out how to extract run errors yet".to_string(),
                ));
            }
        }

        Ok(())
    }

    fn inject_state(&mut self, state: String) {
        v8::scope!(let scope, &mut self.isolate);
        let context = v8::Local::new(&scope, &self.context);
        let scope = &mut v8::ContextScope::new(scope, context);
        let function = v8::Local::new(scope, self.set_state.as_ref().expect("function not loaded"));
        let state = v8::String::new(scope, &state).unwrap().into();

        let try_catch = pin!(v8::TryCatch::new(scope));
        let mut try_catch = try_catch.init();
        let global = context.global(&try_catch).into();
        let result = function.call(&mut try_catch, global, &[state]);
        if result.is_none() {
            let exception = try_catch.exception().unwrap();
            let exception_string = exception
                .to_string(&mut try_catch)
                .unwrap()
                .to_rust_string_lossy(&mut try_catch);

            panic!("{}", exception_string);
        }
    }

    fn extract_state(&mut self) -> Result<String, DynamicError> {
        v8::scope!(let scope, &mut self.isolate);
        let context = v8::Local::new(&scope, &self.context);
        let scope = &mut v8::ContextScope::new(scope, context);
        let function = v8::Local::new(scope, self.get_state.as_ref().expect("function not loaded"));

        let try_catch = pin!(v8::TryCatch::new(scope));
        let mut try_catch = try_catch.init();
        let global = context.global(&try_catch).into();
        let result = function.call(&mut try_catch, global, &[]);

        match result {
            Some(result) => {
                return Ok(result.to_rust_string_lossy(&try_catch));
            }
            None => {
                let exception = try_catch.exception().unwrap();
                let exception_string = exception
                    .to_string(&mut try_catch)
                    .unwrap()
                    .to_rust_string_lossy(&mut try_catch);

                return Err(DynamicError::StateError(exception_string));
            }
        }
    }

    fn setup(&mut self, mapping: Vec<pixels::Pixel>) {
        v8::scope!(let scope, &mut self.isolate);
        let context = v8::Local::new(&scope, &self.context);
        let scope = &mut v8::ContextScope::new(scope, context);
        let function = v8::Local::new(scope, self.setup.as_ref().expect("function not loaded"));

        //Serialize mapping
        let serialized_mapping = serde_json::to_string(&mapping).unwrap();
        let mapping = v8::String::new(scope, &serialized_mapping).unwrap().into();

        let try_catch = pin!(v8::TryCatch::new(scope));
        let mut try_catch = try_catch.init();
        let global = context.global(&try_catch).into();
        let result = function.call(&mut try_catch, global, &[mapping]);
        if result.is_none() {
            let exception = try_catch.exception().unwrap();
            let exception_string = exception
                .to_string(&mut try_catch)
                .unwrap()
                .to_rust_string_lossy(&mut try_catch);

            panic!("{}", exception_string);
        }
    }

    fn dynamic_process(
        &mut self,
        frame: Arc<producer::Frame>,
    ) -> Result<Vec<vecmath::Vector4<f64>>, DynamicError> {
        v8::scope!(let scope, &mut self.isolate);
        let context = v8::Local::new(&scope, &self.context);
        let scope = &mut v8::ContextScope::new(scope, context);
        let function = v8::Local::new(scope, self.render.as_ref().expect("function not loaded"));

        // Serialize frame to JSON and parse it in V8
        let json_str = match serde_json::to_string(&*frame) {
            Ok(s) => s,
            Err(e) => {
                return Err(DynamicError::SerializeError(e.to_string()));
            }
        };
        let v8_str = v8::String::new(scope, &json_str).unwrap();
        let res = v8::json::parse(scope, v8_str.into()).unwrap();

        let try_catch = pin!(v8::TryCatch::new(scope));
        let mut try_catch = try_catch.init();
        let global = context.global(&try_catch).into();
        let result = function.call(&mut try_catch, global, &[res]);
        if result.is_none() {
            let exception = try_catch.exception().unwrap();
            let exception_string = exception
                .to_string(&mut try_catch)
                .unwrap()
                .to_rust_string_lossy(&mut try_catch);

            return Err(DynamicError::ScriptRunError(exception_string));
        }

        let res = v8::Local::<v8::Float64Array>::try_from(result.unwrap()).unwrap();
        let data_ptr = res.data();
        let byte_len = res.byte_length();
        let slice: &[f64] = unsafe {
            std::slice::from_raw_parts(
                data_ptr as *const f64,
                byte_len / std::mem::size_of::<f64>(),
            )
        };

        Ok(slice.chunks(4).map(|s| [s[0], s[1], s[2], s[3]]).collect())
    }

    fn bind_function(
        scope: &mut v8::PinScope<'_, '_>,
        context: v8::Local<v8::Context>,
        name: &str,
    ) -> Option<v8::Global<v8::Function>> {
        let fn_name = v8::String::new(scope, name).unwrap();
        let fn_value = context
            .global(scope)
            .get(scope, fn_name.into())
            .expect("missing function Process");
        let function = v8::Local::<v8::Function>::try_from(fn_value).expect("function expected");
        let function_global_handle = v8::Global::new(scope, function);
        Some(function_global_handle)
    }
}

impl Drop for DynamicHolder {
    fn drop(&mut self) {
        self.cancel_channel.send(true).unwrap();
    }
}

#[derive(Error, Debug)]
pub enum DynamicError {
    #[error("Compile error: {0}")]
    CompileError(String),
    #[error("Script run error: {0}")]
    ScriptRunError(String),
    #[error("Produce error")]
    ProduceError,
    #[error("Could not get state: {0}")]
    StateError(String),
    #[error("Could not serialize Frame: {0}")]
    SerializeError(String),
}
