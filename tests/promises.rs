use rusty_v8 as v8;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Once;

pub struct State {
    reject_no_handler_count: u8,
    handler_after_reject_count: u8,
}

struct TestIsolate(v8::OwnedIsolate);

impl TestIsolate {
    fn new() -> Self {
        static STARTED: Once = Once::new();

        STARTED.call_once(|| {
            let platform = v8::new_default_platform().unwrap();
            v8::V8::initialize_platform(platform);
            v8::V8::initialize();
        });

        let mut isolate = v8::Isolate::new(Default::default());

        isolate.set_promise_reject_callback(promise_reject_callback);

        let state = State {
            reject_no_handler_count: 0,
            handler_after_reject_count: 0,
        };

        isolate.set_slot(Rc::new(RefCell::new(state)));

        Self(isolate)
    }

    fn execute(&mut self, code: &str) {
        let handle_scope = &mut v8::HandleScope::new(&mut self.0);
        let context = v8::Context::new(handle_scope);
        let scope = &mut v8::ContextScope::new(handle_scope, context);

        let object_templ = v8::ObjectTemplate::new(scope);
        let function_templ = v8::FunctionTemplate::new(scope, sleep_one_second);
        let name = v8::String::new(scope, "sleep_one_second").unwrap();
        object_templ.set(name.into(), function_templ.into());

        let context = v8::Context::new_from_template(scope, object_templ);
        let scope = &mut v8::ContextScope::new(scope, context);

        let source = v8::String::new(scope, code).unwrap();
        let script = v8::Script::compile(scope, source, None).unwrap();
        script.run(scope);
    }

    fn get_reject_no_handler_count(&self) -> u8 {
        let state = self.0.get_slot::<Rc<RefCell<State>>>().unwrap().borrow();
        state.reject_no_handler_count
    }

    fn get_handler_after_reject_count(&self) -> u8 {
        let state = self.0.get_slot::<Rc<RefCell<State>>>().unwrap().borrow();
        state.handler_after_reject_count
    }
}

// Callback for promises
pub extern "C" fn promise_reject_callback(msg: v8::PromiseRejectMessage) {
    let scope = &mut unsafe { v8::CallbackScope::new(&msg) };

    let state_rc = scope.get_slot::<Rc<RefCell<State>>>().unwrap();
    let mut state = state_rc.borrow_mut();

    match msg.get_event() {
        v8::PromiseRejectEvent::PromiseRejectWithNoHandler => {
            state.reject_no_handler_count += 1;
        }
        v8::PromiseRejectEvent::PromiseHandlerAddedAfterReject => {
            state.handler_after_reject_count += 1;
        }
        v8::PromiseRejectEvent::PromiseRejectAfterResolved => {}
        v8::PromiseRejectEvent::PromiseResolveAfterResolved => {}
    };
}

// Sleep for one second when called
pub fn sleep_one_second(
    _: &mut v8::HandleScope,
    _: v8::FunctionCallbackArguments,
    _: v8::ReturnValue,
) {
    std::thread::sleep(std::time::Duration::from_secs(1));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn revocation_test() {
        // Example sleep code
        // Should reject then send revocation notification
        let code = r#"
            async function sleep(fn){
                await sleep_one_second();
                fn();
            }

            let promise = new Promise((res, rej) => {
                rej("test");
            })

            sleep(() => {
                promise.catch(data => {  })
            })
        "#;

        let mut isolate = TestIsolate::new();
        isolate.execute(&code);

        let expected_state = State {
            reject_no_handler_count: 1,
            handler_after_reject_count: 1,
        };

        let gen_reject_no_handler_count = isolate.get_reject_no_handler_count();
        let gen_handler_after_reject_count = isolate.get_handler_after_reject_count();

        assert!(
            expected_state.reject_no_handler_count == gen_reject_no_handler_count &&
                expected_state.handler_after_reject_count == gen_handler_after_reject_count
        );
    }

    #[test]
    fn reject_no_handler_test() {
        // Should reject then end
        let code = r#"
            let promise = new Promise((res, rej) => {
                rej("test");
            })
        "#;

        let mut isolate = TestIsolate::new();
        isolate.execute(&code);

        let expected_state = State {
            reject_no_handler_count: 1,
            handler_after_reject_count: 0,
        };

        let gen_reject_no_handler_count = isolate.get_reject_no_handler_count();
        let gen_handler_after_reject_count = isolate.get_handler_after_reject_count();

        assert!(
            expected_state.reject_no_handler_count == gen_reject_no_handler_count &&
                expected_state.handler_after_reject_count == gen_handler_after_reject_count
        );
    }
}