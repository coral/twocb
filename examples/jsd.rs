use notify::{watcher, RecursiveMode, Watcher};
use rusty_v8 as v8;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::fs;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use std::thread::sleep;
use std::time::{Duration, Instant};

fn main() {
    dbg!("party");

    let platform = v8::new_default_platform().unwrap();
    v8::V8::initialize_platform(platform);
    v8::V8::initialize();

    // Create a channel to receive the events.
    let (tx, rx) = channel();

    let mut watcher = watcher(tx, Duration::from_secs(1)).unwrap();

    watcher
        .watch("examples/", RecursiveMode::Recursive)
        .unwrap();

    //Watcher
    loop {
        let (sendstop, stopchan) = channel();
        let contents =
            fs::read_to_string("examples/fn.js").expect("Something went wrong reading the file");
        thread::spawn(move || {
            start_script(&contents.clone(), stopchan);
        });
        loop {
            match rx.recv() {
                Ok(event) => match event {
                    notify::DebouncedEvent::Write(whatever) => {
                        //let newchannel
                        sendstop.send(true);
                        dbg!(whatever);
                        break;
                    }
                    _ => {}
                },
                Err(e) => println!("watch error: {:?}", e),
            }
        }
    }

    unsafe {
        v8::V8::dispose();
    }
    v8::V8::shutdown_platform();
}

pub fn start_script(code: &str, stopchan: Receiver<bool>) {
    // Create a new Isolate and make it the current one.
    let isolate = &mut v8::Isolate::new(v8::CreateParams::default());
    // Create a stack-allocated handle scope.
    let handle_scope = &mut v8::HandleScope::new(isolate);
    // Create a new context.
    let context = v8::Context::new(handle_scope);
    // Enter the context for compiling and running the hello world script.
    let scope = &mut v8::ContextScope::new(handle_scope, context);
    // Create a string containing the JavaScript source code.
    let code = v8::String::new(scope, code).unwrap();
    // Compile the source code.
    let script = v8::Script::compile(scope, code, None).unwrap();
    let result = script.run(scope).unwrap();

    // v8::Object::New()
    let process_str = v8::String::new(scope, "hello").unwrap();
    let process_fn = context
        .global(scope)
        .get(scope, process_str.into())
        .expect("missing function Process");
    let process_fn = v8::Local::<v8::Function>::try_from(process_fn).expect("function expected");
    let name = v8::Number::new(scope, 5.0).into();

    loop {
        if stopchan.try_recv().is_ok() {
            dbg!("BREAKING");
            break;
        }
        let result = {
            let mut try_catch = v8::TryCatch::new(scope);
            let global = context.global(&mut try_catch).into();
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
        dbg!(m.value());
        sleep(Duration::new(1, 0));
    }
}
