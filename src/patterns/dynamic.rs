use notify::{watcher, RecursiveMode, Watcher};
use rusty_v8 as v8;
use std::borrow::Borrow;
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
    context: v8::Global<v8::Context>,
    before_render: Option<v8::Global<v8::Function>>,
    render: Option<v8::Global<v8::Function>>,
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

        Pattern {
            filename: filename.to_string(),
            loaded: Once::new(),
            handle: "hello".to_string(),
            isolate: Some(isolate),
            context: global_context,
            before_render: None,
            render: None,
        }
    }

    pub fn init(&mut self) {}

    pub fn load(&mut self) {
        let code =
            fs::read_to_string(&self.filename).expect("Something went wrong reading the file");
        let isolate = self.isolate.as_mut().unwrap();
        let scope = &mut v8::HandleScope::with_context(isolate, &self.context);
        let context: &v8::Context = self.context.borrow();
        //Make a v8 string of the blah
        let code = v8::String::new(scope, &code).unwrap();
        let script = v8::Script::compile(scope, code, None).unwrap();
        //Execute script to load functions into memory
        script.run(scope).unwrap();

        //RegisterParameters
        let param = v8::String::new(scope, "register()").unwrap();
        v8::Script::compile(scope, param, None);
        let res = script.run(scope).unwrap();
        let result = res.to_string(scope).unwrap();
        println!("{}", result.to_rust_string_lossy(scope));

        // Bind before render handle
        let fn_name = v8::String::new(scope, "beforeRender").unwrap();
        let fn_value = context
            .global(scope)
            .get(scope, fn_name.into())
            .expect("missing function Process");
        let function = v8::Local::<v8::Function>::try_from(fn_value).expect("function expected");
        let function_global_handle = v8::Global::new(scope, function);
        self.before_render = Some(function_global_handle);

        // Bind render handle
        let fn_name = v8::String::new(scope, "render").unwrap();
        let fn_value = context
            .global(scope)
            .get(scope, fn_name.into())
            .expect("missing function Process");
        let function = v8::Local::<v8::Function>::try_from(fn_value).expect("function expected");
        let function_global_handle = v8::Global::new(scope, function);
        self.render = Some(function_global_handle);
    }

    pub fn process(&mut self) {
        let scope =
            &mut v8::HandleScope::with_context(self.isolate.as_mut().unwrap(), &self.context);
        let context: &v8::Context = self.context.borrow();
        let function_global_handle = self.render.as_ref().expect("function not loaded");
        let function: &v8::Function = function_global_handle.borrow();

        let name = v8::Number::new(scope, 5.0).into();
        let mut try_catch = &mut v8::TryCatch::new(scope);
        let global = context.global(try_catch).into();
        let result = function.call(&mut try_catch, global, &[name]);
        if result.is_none() {
            let exception = try_catch.exception().unwrap();
            let exception_string = exception
                .to_string(&mut try_catch)
                .unwrap()
                .to_rust_string_lossy(&mut try_catch);

            panic!("{}", exception_string);
        }
        let m = result.unwrap().to_number(try_catch).unwrap();
        //dbg!(m.value());
    }
}
