# v8_promise_reject_test

This tests if [rusty_v8](https://github.com/denoland/rusty_v8/) isolates properly call the [set_promise_reject_callback](https://docs.rs/rusty_v8/0.12.0/rusty_v8/struct.Isolate.html#method.set_promise_reject_callback) when a promise rejects then has a handler added. I built this test because Deno core doesn't properly call `set_promise_reject_callback` when a promise handler is added in a `setTimeout` function, so I was checking to see if it was an upstream issue. The tests should run successfully.

View the [promise.rs](/tests/promises.rs) file to see the tests.