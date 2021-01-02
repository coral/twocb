use notify::{watcher, RecursiveMode, Watcher};
use rusty_v8 as v8;
use std::borrow::BorrowMut;
use std::cell::Cell;
use std::cell::RefCell;
use std::convert::TryFrom;
use std::fs;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::Once;

pub fn initalize_runtime() {
    let platform = v8::new_default_platform().unwrap();
    v8::V8::initialize_platform(platform);
    v8::V8::initialize();
}

pub fn shutdown_runtime() {
    v8::V8::shutdown_platform();
}

pub(crate) struct JsRuntimeState {
    pub global_context: Option<v8::Global<v8::Context>>,
}

pub struct Pattern {
    filename: String,
    handle: String,
    loaded: Once,
    isolate: Option<v8::OwnedIsolate>,
}

// static DENO_INIT: Once = Once::new();
// DENO_INIT.call_once(|| {
//   unsafe { v8_init() };
// });
// [−][src]Crate lazy_static

impl Pattern {
    pub fn create(filename: &str) -> Self {
        let mut isolate = v8::Isolate::new(v8::CreateParams::default());
        let global_context;
        {
            let handle_scope = &mut v8::HandleScope::new(&mut isolate);
            let context = v8::Context::new(handle_scope);
            global_context = v8::Global::new(handle_scope, context);
        }
        isolate.set_slot(Rc::new(RefCell::new(JsRuntimeState {
            global_context: Some(global_context),
        })));

        Pattern {
            filename: filename.to_string(),
            loaded: Once::new(),
            handle: "hello".to_string(),
            isolate: Some(isolate),
        }
    }

    pub(crate) fn state(isolate: &v8::Isolate) -> Rc<RefCell<JsRuntimeState>> {
        let s = isolate.get_slot::<Rc<RefCell<JsRuntimeState>>>().unwrap();
        s.clone()
    }

    pub(crate) fn global_context(&mut self) -> v8::Global<v8::Context> {
        let state = Self::state(self.v8_isolate());
        let state = state.borrow();
        state.global_context.clone().unwrap()
    }

    pub(crate) fn v8_isolate(&mut self) -> &mut v8::OwnedIsolate {
        self.isolate.as_mut().unwrap()
    }

    pub fn init(&mut self) {}

    pub fn load(&mut self) {
        let code =
            fs::read_to_string(&self.filename).expect("Something went wrong reading the file");
        let context = self.global_context();
        let scope = &mut v8::HandleScope::with_context(self.v8_isolate(), context);
        //Make a v8 string of the blah
        let code = v8::String::new(scope, &code).unwrap();
        let script = v8::Script::compile(scope, code, None).unwrap();
        //Execute script to load functions into memory
        let result = script.run(scope).unwrap();
    }

    pub fn process(&mut self) {
        let mut handle = &self.handle.clone();
        let context = self.global_context();
        let scope = &mut v8::HandleScope::with_context(self.v8_isolate(), context);
        let process_str = v8::String::new(scope, handle).unwrap();
        let process_fn = scope
            .get_current_context()
            .global(scope)
            .get(scope, process_str.into())
            .expect("missing function Process");
        let process_fn =
            v8::Local::<v8::Function>::try_from(process_fn).expect("function expected");

        let name = v8::Number::new(scope, 5.0).into();
        let result = {
            let mut try_catch = v8::TryCatch::new(scope);
            let mut global = try_catch
                .get_current_context()
                .global(&mut try_catch)
                .into();
            let result = process_fn.call(&mut try_catch, global, &[name]);
            if result.is_none() {
                let exception = try_catch.exception().unwrap();
                let exception_string = exception
                    .to_string(&mut try_catch)
                    .unwrap()
                    .to_rust_string_lossy(&mut try_catch);

                panic!("{}", exception_string);
            }
            result
        };
        let m = result.unwrap().to_number(scope).unwrap();
        //dbg!(m.value());
    }
}
