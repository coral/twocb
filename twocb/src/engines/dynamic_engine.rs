use crate::engines;
use crate::engines::params::{PARAM_PREFIXES, ParamKind, ParamValue, PatternParam};
use crate::pixels;
use crate::producer;
use crate::world_state::{self, BUFFER_LEN, MAX_NOTES, WorldStateBuffer};
use crossbeam_channel::select;
use glob::glob;
use log::{error, info, warn};
use notify::{EventKind, Watcher};
use serde::{Deserialize, Serialize};
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
        let code = fs::read_to_string(&global_scope)
            .expect("Something went wrong reading the global.js file");

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

// --- Error types ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternError {
    pub kind: ErrorKind,
    pub message: String,
    pub filename: String,
    pub line: i32,
    pub column: i32,
    pub source_line: String,
    pub stack_trace: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorKind {
    Compile,
    Runtime,
}

/// No-op deleter for ArrayBuffer backing stores we manage ourselves.
unsafe extern "C" fn noop_deleter(
    _data: *mut std::ffi::c_void,
    _byte_length: usize,
    _deleter_data: *mut std::ffi::c_void,
) {
    // Intentionally empty - Rust owns the memory
}

/// Extract V8 error details from a TryCatch scope.
/// Uses a macro because PinnedRef<TryCatch<...>> has complex generics.
macro_rules! extract_v8_error {
    ($try_catch:expr, $filename:expr) => {{
        let exception = $try_catch.exception().unwrap();
        let message_str = exception
            .to_string(&mut $try_catch)
            .unwrap()
            .to_rust_string_lossy(&mut $try_catch);

        let mut err = PatternError {
            kind: ErrorKind::Runtime,
            message: message_str,
            filename: $filename.to_string(),
            line: 0,
            column: 0,
            source_line: String::new(),
            stack_trace: None,
        };

        if let Some(msg) = $try_catch.message() {
            err.line = msg.get_line_number(&mut $try_catch).unwrap_or(0) as i32;
            err.column = msg.get_start_column() as i32;
            if let Some(src) = msg.get_source_line(&mut $try_catch) {
                err.source_line = src.to_rust_string_lossy(&mut $try_catch);
            }
        }

        if let Some(stack) = $try_catch.stack_trace() {
            let stack_str = stack
                .to_string(&mut $try_catch)
                .unwrap()
                .to_rust_string_lossy(&mut $try_catch);
            err.stack_trace = Some(stack_str);
        }

        err
    }};
}

// --- DynamicHolder: the thread-safe handle sent to the compositor ---

struct DynamicHolder {
    patternname: String,

    frame_tx: crossbeam_channel::Sender<Arc<producer::Frame>>,
    result_rx: crossbeam_channel::Receiver<Result<Vec<vecmath::Vector4<f64>>, DynamicError>>,
    cmd_tx: crossbeam_channel::Sender<Command>,

    _watcher: notify::RecommendedWatcher,

    pixel_count: usize,
}

enum Command {
    SetState(String),
    GetState {
        reply: crossbeam_channel::Sender<Result<String, DynamicError>>,
    },
    GetParams {
        reply: crossbeam_channel::Sender<Vec<PatternParam>>,
    },
    SetParam {
        name: String,
        value: ParamValue,
        reply: crossbeam_channel::Sender<Vec<PatternParam>>,
    },
}

impl engines::pattern::Pattern for DynamicHolder {
    fn name(&self) -> String {
        return self.patternname.clone();
    }

    fn process(&mut self, frame: Arc<producer::Frame>) -> Vec<vecmath::Vector4<f64>> {
        match self.frame_tx.send(frame) {
            Err(e) => {
                error!("Could not send frame to dynamic pattern: {}", e);
            }
            _ => {}
        }

        match self.result_rx.recv() {
            Ok(v) => match v {
                Ok(output) => output,
                Err(_e) => {
                    vec![[1.0, 0.0, 1.0, 1.0]; self.pixel_count]
                }
            },
            Err(_e) => {
                vec![[1.0, 0.0, 1.0, 1.0]; self.pixel_count]
            }
        }
    }

    fn get_state(&self) -> Vec<u8> {
        let (reply_tx, reply_rx) = crossbeam_channel::bounded(1);
        if self
            .cmd_tx
            .send(Command::GetState { reply: reply_tx })
            .is_err()
        {
            return "{}".as_bytes().to_vec();
        }
        match reply_rx.recv() {
            Ok(Ok(state)) => state.as_bytes().to_vec(),
            Ok(Err(e)) => {
                error!("Get state error: {}", e);
                "{}".as_bytes().to_vec()
            }
            Err(_) => "{}".as_bytes().to_vec(),
        }
    }

    fn set_state(&mut self, data: &[u8]) {
        let state = std::str::from_utf8(&data).unwrap().to_string();
        if let Err(e) = self.cmd_tx.send(Command::SetState(state)) {
            error!("Could not send state to dynamic pattern: {}", e);
        }
    }

    fn params(&self) -> Vec<PatternParam> {
        let (reply_tx, reply_rx) = crossbeam_channel::bounded(1);
        if self
            .cmd_tx
            .send(Command::GetParams { reply: reply_tx })
            .is_err()
        {
            return Vec::new();
        }
        reply_rx.recv().unwrap_or_default()
    }

    fn set_param(&mut self, name: &str, value: ParamValue) {
        let (reply_tx, reply_rx) = crossbeam_channel::bounded(1);
        let _ = self.cmd_tx.send(Command::SetParam {
            name: name.to_string(),
            value,
            reply: reply_tx,
        });
        // Consume the response to keep channels in sync
        let _ = reply_rx.recv();
    }
}

// --- DynamicPattern: lives on its own thread, owns the V8 isolate ---

struct DynamicPattern {
    isolate: v8::OwnedIsolate,
    context: v8::Global<v8::Context>,
    setup: Option<v8::Global<v8::Function>>,
    get_state: Option<v8::Global<v8::Function>>,
    set_state: Option<v8::Global<v8::Function>>,
    render: Option<v8::Global<v8::Function>>,

    // Zero-copy buffers
    world_buffer: Box<WorldStateBuffer>,
    pixel_buffer: Vec<f64>,
    pixel_count: usize,

    // Discovered parameters
    params: Vec<DiscoveredParam>,

    filename: String,
}

struct DiscoveredParam {
    pub info: PatternParam,
    pub function: v8::Global<v8::Function>,
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

        let codepath = path.as_path();
        let patternname = match fs::read_to_string(codepath) {
            Ok(_v) => String::from(path.file_name().unwrap().to_str().unwrap()),
            Err(e) => return Err(e),
        };

        let pixel_count = mapping.len();

        let (frame_tx, frame_rx) = crossbeam_channel::bounded(1);
        let (result_tx, result_rx) = crossbeam_channel::bounded(1);
        let (cmd_tx, cmd_rx) = crossbeam_channel::unbounded();
        let (reload_tx, reload_rx) = crossbeam_channel::unbounded();

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
                    isolate,
                    context: global_context,
                    setup: None,
                    get_state: None,
                    set_state: None,
                    render: None,
                    world_buffer: Box::new(WorldStateBuffer::new()),
                    pixel_buffer: vec![0.0; pixel_count * 4],
                    pixel_count,
                    params: Vec::new(),
                    filename: c.file_name().unwrap().to_str().unwrap().to_string(),
                };

                d.setup_shared_buffers();
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
                d.setup_pattern(mapping.clone());
                d.discover_params();

                loop {
                    select! {
                        // Render frame (hot path)
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
                                        // Wait for reload before retrying
                                        match reload_rx.recv() {
                                            Ok(_) => break,
                                            _ => return,
                                        }
                                    }
                                },
                                Err(_) => return, // sender dropped, shut down
                            }
                        },

                        // Control commands
                        recv(cmd_rx) -> cmd => {
                            match cmd {
                                Ok(Command::SetState(state)) => {
                                    d.inject_state(state);
                                }
                                Ok(Command::GetState { reply }) => {
                                    let result = match d.extract_state() {
                                        Ok(state) => Ok(state),
                                        Err(e) => Err(DynamicError::StateError(e.to_string())),
                                    };
                                    let _ = reply.send(result);
                                }
                                Ok(Command::GetParams { reply }) => {
                                    let params: Vec<PatternParam> = d.params.iter().map(|p| p.info.clone()).collect();
                                    let _ = reply.send(params);
                                }
                                Ok(Command::SetParam { name, value, reply }) => {
                                    d.set_param_value(&name, &value);
                                    let params: Vec<PatternParam> = d.params.iter().map(|p| p.info.clone()).collect();
                                    let _ = reply.send(params);
                                }
                                Err(_) => return, // sender dropped, shut down
                            }
                        }

                        // Reload on file update
                        recv(reload_rx) -> reload => {
                            match reload {
                                Ok(event) => {
                                    match event.kind {
                                        EventKind::Modify(mf) => match mf {
                                            notify::event::ModifyKind::Data(_) => {
                                                info!("Reloading pattern: {}", d.filename);
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
            patternname,
            frame_tx,
            result_rx,
            cmd_tx,
            _watcher: watcher,
            pixel_count,
        });
    }

    fn load(&mut self, global: &str, code: &str) -> Result<(), DynamicError> {
        v8::scope!(let scope, &mut self.isolate);
        let context = v8::Local::new(&scope, &self.context);
        let scope = &mut v8::ContextScope::new(scope, context);

        // Load global
        DynamicPattern::execute(scope, global, &self.filename)?;

        // Load code
        DynamicPattern::execute(scope, code, &self.filename)?;

        // Bind function handlers
        self.setup = DynamicPattern::bind_function(scope, context, "_setup");
        self.get_state = DynamicPattern::bind_function(scope, context, "_getState");
        self.set_state = DynamicPattern::bind_function(scope, context, "_setState");
        self.render = DynamicPattern::bind_function(scope, context, "_internalRender");

        Ok(())
    }

    fn execute(
        scope: &mut v8::PinScope<'_, '_>,
        code: &str,
        filename: &str,
    ) -> Result<(), DynamicError> {
        let code_str = v8::String::new(scope, code).unwrap();

        let try_catch = pin!(v8::TryCatch::new(scope));
        let mut try_catch = try_catch.init();

        let script = match v8::Script::compile(&mut try_catch, code_str, None) {
            Some(script) => script,
            None => {
                let err = extract_v8_error!(try_catch, filename);
                return Err(DynamicError::CompileError(format!(
                    "{}:{}: {} | {}",
                    err.filename, err.line, err.message, err.source_line
                )));
            }
        };

        match script.run(&mut try_catch) {
            Some(_v) => {}
            None => {
                let err = extract_v8_error!(try_catch, filename);
                return Err(DynamicError::ScriptRunError(format!(
                    "{}:{}: {} | {}",
                    err.filename,
                    err.line,
                    err.message,
                    err.stack_trace.unwrap_or_default()
                )));
            }
        }

        Ok(())
    }

    /// Set up the shared Float64Array buffers in the V8 context.
    fn setup_shared_buffers(&mut self) {
        v8::scope!(let scope, &mut self.isolate);
        let context = v8::Local::new(&scope, &self.context);
        let scope = &mut v8::ContextScope::new(scope, context);
        let global = context.global(scope);

        // World state buffer -> _worldRaw Float64Array
        {
            let ptr = self.world_buffer.as_f64_slice_mut().as_mut_ptr();
            let byte_len = BUFFER_LEN * std::mem::size_of::<f64>();
            let backing_store = unsafe {
                v8::ArrayBuffer::new_backing_store_from_ptr(
                    ptr as *mut std::ffi::c_void,
                    byte_len,
                    noop_deleter,
                    std::ptr::null_mut(),
                )
            };
            let backing_store = backing_store.make_shared();
            let ab = v8::ArrayBuffer::with_backing_store(scope, &backing_store);
            let f64array = v8::Float64Array::new(scope, ab, 0, BUFFER_LEN).unwrap();
            let key = v8::String::new(scope, "_worldRaw").unwrap();
            global.set(scope, key.into(), f64array.into());
        }

        // Pixel buffer -> _pixelBuffer Float64Array (overrides the JS-created one)
        {
            let ptr = self.pixel_buffer.as_mut_ptr();
            let byte_len = self.pixel_buffer.len() * std::mem::size_of::<f64>();
            let backing_store = unsafe {
                v8::ArrayBuffer::new_backing_store_from_ptr(
                    ptr as *mut std::ffi::c_void,
                    byte_len,
                    noop_deleter,
                    std::ptr::null_mut(),
                )
            };
            let backing_store = backing_store.make_shared();
            let ab = v8::ArrayBuffer::with_backing_store(scope, &backing_store);
            let f64array = v8::Float64Array::new(scope, ab, 0, self.pixel_count * 4).unwrap();
            let key = v8::String::new(scope, "_pixelBuffer").unwrap();
            global.set(scope, key.into(), f64array.into());
        }

        // Set pixelCount global
        {
            let key = v8::String::new(scope, "pixelCount").unwrap();
            let val = v8::Number::new(scope, self.pixel_count as f64);
            global.set(scope, key.into(), val.into());
        }

        // Create the `world` object with native V8 accessors
        {
            let world_obj = v8::Object::new(scope);

            // Register auto-generated scalar f64 accessors
            self.world_buffer.register_v8_accessors(scope, world_obj);

            // Register custom colorchord accessor
            let cc_key = v8::String::new(scope, "colorchord").unwrap();
            let buf_ptr = &*self.world_buffer as *const WorldStateBuffer as *mut std::ffi::c_void;
            let ext = v8::External::new(scope, buf_ptr);
            let config = v8::AccessorConfiguration::new(world_state::colorchord_getter)
                .data(ext.into())
                .property_attribute(v8::PropertyAttribute::READ_ONLY);
            world_obj.set_accessor_with_configuration(scope, cc_key.into(), config);

            let world_key = v8::String::new(scope, "world").unwrap();
            global.set(scope, world_key.into(), world_obj.into());
        }
    }

    /// Discover Pixelblaze-style parameter functions from the global scope.
    fn discover_params(&mut self) {
        v8::scope!(let scope, &mut self.isolate);
        let context = v8::Local::new(&scope, &self.context);
        let scope = &mut v8::ContextScope::new(scope, context);
        let global = context.global(scope);

        let names = match global.get_own_property_names(scope, v8::GetPropertyNamesArgs::default())
        {
            Some(n) => n,
            None => return,
        };

        let mut discovered = Vec::new();

        for i in 0..names.length() {
            let key = names.get_index(scope, i).unwrap();
            let name_str = key.to_string(scope).unwrap().to_rust_string_lossy(scope);

            for (prefix, kind) in PARAM_PREFIXES {
                if name_str.starts_with(prefix) && name_str.len() > prefix.len() {
                    // Check it's actually a function
                    let val = global.get(scope, key).unwrap();
                    if let Ok(func) = v8::Local::<v8::Function>::try_from(val) {
                        let func_global = v8::Global::new(scope, func);
                        // Extract the human-readable name after the prefix
                        let param_name = name_str[prefix.len()..].to_string();
                        discovered.push(DiscoveredParam {
                            info: PatternParam {
                                name: param_name,
                                kind: *kind,
                                value: kind.default_value(),
                            },
                            function: func_global,
                        });
                    }
                    break;
                }
            }
        }

        info!(
            "Discovered {} params for {}",
            discovered.len(),
            self.filename
        );
        self.params = discovered;
    }

    /// Call a parameter setter function in V8.
    fn set_param_value(&mut self, name: &str, value: &ParamValue) {
        let idx = match self.params.iter().position(|p| p.info.name == name) {
            Some(i) => i,
            None => {
                warn!("Unknown param: {}", name);
                return;
            }
        };

        v8::scope!(let scope, &mut self.isolate);
        let context = v8::Local::new(&scope, &self.context);
        let scope = &mut v8::ContextScope::new(scope, context);
        let global_obj = context.global(scope).into();

        let func = v8::Local::new(scope, &self.params[idx].function);

        match value {
            ParamValue::Float(v) => {
                let arg = v8::Number::new(scope, *v);
                func.call(scope, global_obj, &[arg.into()]);
            }
            ParamValue::Color3(a, b, c) => {
                let a = v8::Number::new(scope, *a);
                let b = v8::Number::new(scope, *b);
                let c = v8::Number::new(scope, *c);
                func.call(scope, global_obj, &[a.into(), b.into(), c.into()]);
            }
            ParamValue::Bool(v) => {
                let arg = v8::Boolean::new(scope, *v);
                func.call(scope, global_obj, &[arg.into()]);
            }
        }

        self.params[idx].info.value = value.clone();
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
            let err = extract_v8_error!(try_catch, &self.filename);
            error!("inject_state error: {}", err.message);
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
                let err = extract_v8_error!(try_catch, &self.filename);
                return Err(DynamicError::StateError(err.message));
            }
        }
    }

    fn setup_pattern(&mut self, mapping: Vec<pixels::Pixel>) {
        v8::scope!(let scope, &mut self.isolate);
        let context = v8::Local::new(&scope, &self.context);
        let scope = &mut v8::ContextScope::new(scope, context);
        let function = v8::Local::new(scope, self.setup.as_ref().expect("function not loaded"));

        let serialized_mapping = serde_json::to_string(&mapping).unwrap();
        let mapping = v8::String::new(scope, &serialized_mapping).unwrap().into();

        let try_catch = pin!(v8::TryCatch::new(scope));
        let mut try_catch = try_catch.init();
        let global = context.global(&try_catch).into();
        let result = function.call(&mut try_catch, global, &[mapping]);
        if result.is_none() {
            let err = extract_v8_error!(try_catch, &self.filename);
            panic!("setup error: {}: {}", self.filename, err.message);
        }
    }

    /// Write frame data into the shared WorldStateBuffer and call _internalRender().
    /// Reads pixels back from the shared pixel buffer - zero copy both directions.
    fn dynamic_process(
        &mut self,
        frame: Arc<producer::Frame>,
    ) -> Result<Vec<vecmath::Vector4<f64>>, DynamicError> {
        // Write frame data directly into the shared buffer
        let wb = &mut *self.world_buffer;
        wb.framerate = frame.framerate;
        wb.frame_index = frame.index as f64;
        wb.delta = frame.delta;
        wb.phase = frame.phase;
        wb.bar = frame.bar;

        // Tempo
        wb.bpm = frame.tempo.bpm as f64;
        wb.tempo_confidence = frame.tempo.confidence as f64;
        wb.tempo_period = frame.tempo.period as f64;

        // Colorchord notes
        let note_count = frame.colorchord.notes.len().min(MAX_NOTES);
        wb.note_count = note_count as f64;
        for i in 0..note_count {
            let note = &frame.colorchord.notes[i];
            let base = i * 3;
            wb.notes[base] = note.amplitude_out as f64;
            wb.notes[base + 1] = note.amplitude_iir2 as f64;
            wb.notes[base + 2] = note.id as f64;
        }
        // Zero out unused note slots
        for i in note_count..MAX_NOTES {
            let base = i * 3;
            wb.notes[base] = 0.0;
            wb.notes[base + 1] = 0.0;
            wb.notes[base + 2] = 0.0;
        }

        // Folded spectrum
        let folded_len = frame.colorchord.folded.len().min(wb.folded.len());
        for i in 0..folded_len {
            wb.folded[i] = frame.colorchord.folded[i] as f64;
        }

        wb.pixel_count = self.pixel_count as f64;

        // Call _internalRender() with no arguments (it reads from _worldRaw)
        v8::scope!(let scope, &mut self.isolate);
        let context = v8::Local::new(&scope, &self.context);
        let scope = &mut v8::ContextScope::new(scope, context);
        let function = v8::Local::new(scope, self.render.as_ref().expect("function not loaded"));

        let try_catch = pin!(v8::TryCatch::new(scope));
        let mut try_catch = try_catch.init();
        let global = context.global(&try_catch).into();
        let result = function.call(&mut try_catch, global, &[]);
        if result.is_none() {
            let err = extract_v8_error!(try_catch, &self.filename);
            return Err(DynamicError::ScriptRunError(format!(
                "{}:{}: {}",
                err.filename, err.line, err.message
            )));
        }

        // Read pixels directly from the shared buffer - no copy from V8
        Ok(self
            .pixel_buffer
            .chunks(4)
            .map(|s| [s[0], s[1], s[2], s[3]])
            .collect())
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
}
