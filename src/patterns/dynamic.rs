use crate::pixels;
use notify::{watcher, RecursiveMode, Watcher};
use rusty_v8 as v8;
use serde::{Deserialize, Serialize};
use serde_json::*;
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
    mapping: std::vec::Vec<pixels::Pixel>,
    loaded: Once,
    isolate: Option<v8::OwnedIsolate>,
    context: v8::Global<v8::Context>,
    setup: Option<v8::Global<v8::Function>>,
    register: Option<v8::Global<v8::Function>>,
    render: Option<v8::Global<v8::Function>>,
}

#[derive(Default, Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct Parameters {
    pub parameters: Vec<String>,
    pub features: Vec<String>,
}

// static DENO_INIT: Once = Once::new();
// DENO_INIT.call_once(|| {
//   unsafe { v8_init() };
// });
// [−][src]Crate lazy_static

///convert array
///         // let m = result.unwrap();
// let r = v8::Local::<v8::Object>::try_from(m).unwrap();
// let par = v8::String::new(try_catch, "parameters").unwrap();
// let something = r.get(try_catch, par.into()).unwrap();
// let array = v8::Local::<v8::Array>::try_from(something).unwrap();
// for i in 0..array.length() {
//     let a = array
//         .get_index(try_catch, i)
//         .unwrap()
//         .to_rust_string_lossy(try_catch);
//     dbg!(a);
// }

// if we _dont_ want to make copies, we could just create the vec in rust,
//build a float64array around the vec's storage, and pass the array into js to have it fill the array

impl Pattern {
    pub fn create(filename: &str, mapping: std::vec::Vec<pixels::Pixel>) -> Self {
        let mut isolate = v8::Isolate::new(v8::CreateParams::default());
        let global_context;
        {
            let handle_scope = &mut v8::HandleScope::new(&mut isolate);
            let context = v8::Context::new(handle_scope);
            global_context = v8::Global::new(handle_scope, context);
        }

        Pattern {
            filename: filename.to_string(),
            mapping,
            loaded: Once::new(),
            isolate: Some(isolate),
            context: global_context,
            setup: None,
            register: None,
            render: None,
        }
    }

    pub fn load(&mut self) {
        let global = fs::read_to_string("files/support/global.js")
            .expect("Something went wrong reading the global.js file");
        let code =
            fs::read_to_string(&self.filename).expect("Something went wrong reading the file");
        let isolate = self.isolate.as_mut().unwrap();
        let scope = &mut v8::HandleScope::with_context(isolate, &self.context);
        let context: &v8::Context = self.context.borrow();
        //load global
        {
            let code = v8::String::new(scope, &global).unwrap();
            let script = v8::Script::compile(scope, code, None).unwrap();
            script.run(scope).unwrap();
        }

        let code = v8::String::new(scope, &code).unwrap();
        let script = v8::Script::compile(scope, code, None).unwrap();
        //Execute script to load functions into memory
        script.run(scope).unwrap();

        //Bind function handlers
        self.setup = Pattern::bind_function(scope, context, "_setup");
        self.register = Pattern::bind_function(scope, context, "_internalRegister");
        self.render = Pattern::bind_function(scope, context, "_internalRender");
    }

    pub fn setup(&mut self) {
        let scope =
            &mut v8::HandleScope::with_context(self.isolate.as_mut().unwrap(), &self.context);
        let context: &v8::Context = self.context.borrow();
        let function_global_handle = self.setup.as_ref().expect("function not loaded");
        let function: &v8::Function = function_global_handle.borrow();

        //Serialize mapping
        let serialized_mapping = serde_json::to_string(&self.mapping).unwrap();
        let mapping = v8::String::new(scope, &serialized_mapping).unwrap().into();

        let mut try_catch = &mut v8::TryCatch::new(scope);
        let global = context.global(try_catch).into();
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

    pub fn register(&mut self) {
        let scope =
            &mut v8::HandleScope::with_context(self.isolate.as_mut().unwrap(), &self.context);
        let context: &v8::Context = self.context.borrow();
        let function_global_handle = self.register.as_ref().expect("function not loaded");
        let function: &v8::Function = function_global_handle.borrow();

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
        let m = result.unwrap().to_rust_string_lossy(try_catch);
        dbg!(m);
        //let v: Parameters = serde_json::from_str(&m).unwrap();
    }

    pub fn process(&mut self) -> Vec<f64> {
        let scope =
            &mut v8::HandleScope::with_context(self.isolate.as_mut().unwrap(), &self.context);
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
        let copied = unsafe {
            let ptr = v.as_mut_ptr();
            let slice = std::slice::from_raw_parts_mut(
                ptr as *mut u8,
                v.len() * std::mem::size_of::<f64>(),
            );
            res.copy_contents(slice)
        };

        return v;
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
