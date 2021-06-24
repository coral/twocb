use crate::engines;
use crate::producer;
use async_trait::async_trait;
use glob::glob;
use log::debug;
use log::error;
use rusty_v8 as v8;
use std::borrow::Borrow;
use std::convert::TryFrom;
use std::fs;
use std::path::PathBuf;
use std::sync::mpsc;
use std::sync::Arc;
use tokio::sync::oneshot;
use tokio::task;

pub struct DynamicEngine {
    pattern_folder: String,
    global_scope: String,
}

impl engines::Engine for DynamicEngine {
    fn bootstrap(&mut self) -> anyhow::Result<()> {
        //self.init_patterns();
        self.watch();
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
            Err(e) => Vec::new(),
        }
    }

    fn instantiate_pattern(&self, name: &str) -> Option<Box<dyn engines::pattern::Pattern + Send>> {
        let patternpath = std::path::Path::new(&self.pattern_folder).join(name);
        match DynamicPattern::new(patternpath.to_path_buf()) {
            Ok(d) => return Some(Box::new(d)),
            Err(e) => {
                error!("{}", e);
                return None;
            }
        }
    }
}

impl DynamicEngine {
    pub fn new(pattern_folder: &str, global_scope: &str) -> DynamicEngine {
        let _global = fs::read_to_string(global_scope)
            .expect("Something went wrong reading the global.js file");
        let code =
            fs::read_to_string(&global_scope).expect("Something went wrong reading the file");

        return DynamicEngine {
            pattern_folder: pattern_folder.to_string(),
            global_scope: code,
        };
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

struct DynamicHolder {
    frame_channel: mpsc::Sender<Arc<producer::Frame>>,
    result_channel: mpsc::Receiver<Vec<vecmath::Vector4<f64>>>,
}

impl engines::pattern::Pattern for DynamicHolder {
    fn name(&self) -> String {
        return "ok".to_string();
    }

    fn process(&mut self, _frame: Arc<producer::Frame>) -> Vec<vecmath::Vector4<f64>> {
        // Ok this sucks
        // This needs to be called on the same thread as we initialized the pattern on.
        // Really sad
        // But w/e we can come up with some dumb thread pool.

        self.frame_channel.send(_frame);

        match self.result_channel.recv() {
            Ok(v) => v,
            Err(_) => vec![[1.0, 0.0, 1.0, 1.0]; 864],
        }
        //vec![[1.0, 0.0, 1.0, 1.0]; 864]
    }
    fn get_state(&self) -> Vec<u8> {
        return Vec::new();
    }

    fn set_state(&mut self, data: &[u8]) {}
}

struct DynamicPattern {
    path: std::path::PathBuf,
    //tp: tokio::runtime::Runtime,
    isolate: v8::OwnedIsolate,
    context: v8::Global<v8::Context>,
    setup: Option<v8::Global<v8::Function>>,
    register: Option<v8::Global<v8::Function>>,
    render: Option<v8::Global<v8::Function>>,
}

impl DynamicPattern {
    pub fn new(path: std::path::PathBuf) -> Result<DynamicHolder, std::io::Error> {
        let global = match fs::read_to_string("files/support/global.js") {
            Ok(v) => v,
            Err(e) => return Err(e),
        };

        let codepath = path.as_path();
        let code = match fs::read_to_string(codepath) {
            Ok(v) => v,
            Err(e) => return Err(e),
        };

        let (frame_tx, mut frame_rx) = mpsc::channel();
        let (result_tx, mut result_rx) = mpsc::channel();

        std::thread::spawn(move || {
            {
                let mut isolate = v8::Isolate::new(v8::CreateParams::default());
                let global_context;
                {
                    let handle_scope = &mut v8::HandleScope::new(&mut isolate);
                    let context = v8::Context::new(handle_scope);
                    global_context = v8::Global::new(handle_scope, context);
                }

                //tx.send(isolate.thread_safe_handle());

                let mut d = DynamicPattern {
                    path,

                    isolate: isolate,
                    context: global_context,
                    setup: None,
                    register: None,
                    render: None,
                };

                d.load(&global, &code);

                loop {
                    let frame = frame_rx.recv();
                    d.dynamic_process();
                }
            }
        });

        return Ok(DynamicHolder {
            frame_channel: frame_tx,
            result_channel: result_rx,
        });
    }

    fn load(&mut self, global: &str, code: &str) {
        let scope = &mut v8::HandleScope::with_context(&mut self.isolate, &self.context);
        let context: &v8::Context = self.context.borrow();
        //load global
        {
            let code = v8::String::new(scope, global).unwrap();
            let script = v8::Script::compile(scope, code, None).unwrap();
            script.run(scope).unwrap();
        }

        let code = v8::String::new(scope, code).unwrap();
        let script = v8::Script::compile(scope, code, None).unwrap();
        //Execute script to load functions into memory
        script.run(scope).unwrap();

        //Bind function handlers
        self.setup = DynamicPattern::bind_function(scope, context, "_setup");
        self.register = DynamicPattern::bind_function(scope, context, "_internalRegister");
        self.render = DynamicPattern::bind_function(scope, context, "_internalRender");
    }

    //This function is for the pattern to bind parameters
    //it's kinda obvious i haven't really ported it yet.
    fn register(&mut self) {
        //Register functions
        // let scope =
        //     &mut v8::HandleScope::with_context(self.isolate.as_mut().unwrap(), &self.context);
        // let context: &v8::Context = self.context.borrow();
        // let function_global_handle = self.register.as_ref().expect("function not loaded");
        // let function: &v8::Function = function_global_handle.borrow();

        // let mut try_catch = &mut v8::TryCatch::new(scope);
        // let global = context.global(try_catch).into();
        // let result = function.call(&mut try_catch, global, &[]);
        // if result.is_none() {
        //     let exception = try_catch.exception().unwrap();
        //     let exception_string = exception
        //         .to_string(&mut try_catch)
        //         .unwrap()
        //         .to_rust_string_lossy(&mut try_catch);

        //     panic!("PENIS : {}", exception_string);
        // }
        // let m = result.unwrap().to_rust_string_lossy(try_catch);
        // dbg!(m);
    }

    fn dynamic_process(&mut self) {
        let scope = &mut v8::HandleScope::with_context(&mut self.isolate, &self.context);
        let context: &v8::Context = self.context.borrow();
        let function_global_handle = self.render.as_ref().expect("function not loaded");
        let function: &v8::Function = function_global_handle.borrow();

        // let buf = v8::ArrayBuffer::new_backing_store_from_boxed_slice(baked.into_boxed_slice())

        //        let mapping = v8::Float64Array::new(scope, baked, 0, 0);

        //let name = v8::Number::new(scope, 5.0).into();

        //let pixelbuffer: Vec<f64> = vec![0., self.mapping.len()];

        let mut try_catch = &mut v8::TryCatch::new(scope);
        let global = context.global(try_catch).into();
        let result = function.call(&mut try_catch, global, &[]);
        if result.is_none() {
            let exception = try_catch.exception().unwrap();
            let exception_string = exception
                .to_string(&mut try_catch)
                .unwrap()
                .to_rust_string_lossy(&mut try_catch);

            panic!("{}", exception_string);
        }

        let res = v8::Local::<v8::Float64Array>::try_from(result.unwrap()).unwrap();
        // let backing = res.buffer(try_catch).unwrap().get_backing_store();
        // let slice: &[f64] = unsafe {
        //     let ptr = backing.data().offset(res.byte_offset() as isize);
        //     let len = res.byte_length();
        //     std::slice::from_raw_parts(ptr as *const f64, len / std::mem::size_of::<f64>())
        // };

        // return slice;
        // dbg!(slice);
        // let mut m = vec![0; res.byte_length()];

        let mut v = vec![0.0f64; res.byte_length() / std::mem::size_of::<f64>()];
        let _copied = unsafe {
            let ptr = v.as_mut_ptr();
            let slice = std::slice::from_raw_parts_mut(
                ptr as *mut u8,
                v.len() * std::mem::size_of::<f64>(),
            );
            res.copy_contents(slice)
        };
    }

    fn bind_function(
        scope: &mut v8::HandleScope,
        context: &rusty_v8::Context,
        name: &str,
    ) -> Option<v8::Global<v8::Function>> {
        let fn_name = v8::String::new(scope, &name).unwrap();
        let fn_value = context
            .global(scope)
            .get(scope, fn_name.into())
            .expect("missing function Process");
        let function = v8::Local::<v8::Function>::try_from(fn_value).expect("function expected");
        let function_global_handle = v8::Global::new(scope, function);
        Some(function_global_handle)
    }
}
