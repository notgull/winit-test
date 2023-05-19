// SPDX-License-Identifier: MIT OR Apache-2.0 OR Zlib

//! Run tests using a `winit` context.

#![forbid(unsafe_code)]

/// Re-exporting `winit` for the sake of convenience.
pub use winit;

/// The whole point.
#[macro_export]
macro_rules! main {
    ($ty:ty => $($test:expr),*) => {
        #[cfg(target_arch = "wasm32")]
        $crate::__private::wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

        #[cfg(not(target_os = "android"))]
        #[cfg_attr(target_arch = "wasm32", $crate::__private::wasm_bindgen_test::wasm_bindgen_test)]
        fn main() -> Result<(), Box<dyn std::error::Error>> {
            const TESTS: &[$crate::__private::WinitBasedTest<$ty>] = &[$(
                $crate::__winit_test_internal_collect_test!($test)
            ),*];

            $crate::__private::run(TESTS, ());
            Ok(())
        }

        #[cfg(target_os = "android")]
        #[no_mangle]
        fn android_main(app: $crate::__private::Context) {
            pub(super) const TESTS: &[$crate::__private::WinitBasedTest<$ty>] = &[$(
                $crate::__winit_test_internal_collect_test!($test)
            ),*];

            $crate::__private::run(TESTS, app);
        }
    };
    ($($tt:tt)*) => {
        $crate::main!(() => $($tt)*);
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! __winit_test_internal_collect_test {
    ($name:expr) => {
        $crate::__private::WinitBasedTest {
            name: stringify!($name),
            function: $crate::__private::TestFunction::Oneoff($name),
        }
    };
}

#[doc(hidden)]
// This part is semver-exempt.
pub mod __private {
    #[cfg(target_arch = "wasm32")]
    pub use wasm_bindgen_test;

    pub use winit::event_loop::EventLoopWindowTarget;
    use winit::event_loop::{ControlFlow, EventLoopBuilder};

    use owo_colors::OwoColorize;
    use std::any::Any;
    use std::panic::{catch_unwind, AssertUnwindSafe};
    #[cfg(not(target_arch = "wasm32"))]
    use std::time::Instant;
    #[cfg(target_arch = "wasm32")]
    use web_time::Instant;

    #[cfg(target_os = "android")]
    pub use winit::platform::android::{
        activity::AndroidApp as Context, EventLoopBuilderExtAndroid,
    };
    #[cfg(target_arch = "wasm32")]
    use winit::platform::web::EventLoopExtWebSys;
    #[cfg(not(target_os = "android"))]
    pub type Context = ();

    struct State {
        passed: i32,
        panics: Vec<(&'static str, Box<dyn Any + Send>)>,
        start: Instant,
        run: bool,
        code: i32,
    }

    /// Run a set of tests using a `winit` context.
    pub fn run<T: 'static>(tests: &'static [WinitBasedTest<T>], _ctx: Context) {
        // If we're on Miri, we can't run the tests.
        if cfg!(miri) {
            eprintln!();
            eprintln!(
                "{}: tests cannot be run under miri",
                "warning(winit-test)".yellow().bold()
            );
            eprintln!("    = See this issue for more information: https://github.com/notgull/winit-test/issues/2");
            eprintln!();
            return;
        }

        // Create a new event loop and obtain a window target.
        let mut builder = EventLoopBuilder::<T>::with_user_event();

        // Install the Android event loop extension if necessary.
        #[cfg(target_os = "android")]
        {
            builder = builder.with_android_app(_ctx);
        }

        let event_loop = builder.build();

        println!("\nRunning {} tests...", tests.len());
        let mut state = State {
            passed: 0,
            panics: vec![],
            start: Instant::now(),
            run: false,
            code: 0,
        };

        // Run the tests.
        #[cfg(not(target_arch = "wasm32"))]
        event_loop.run(move |_, elwt, control_flow| {
            run_internal(tests, &mut state, elwt, control_flow);
        });
        #[cfg(target_arch = "wasm32")]
        event_loop.spawn(move |_, elwt, control_flow| {
            run_internal(tests, &mut state, elwt, control_flow);
        });
    }

    /// Run a set of tests using a `winit` context.
    fn run_internal<T: 'static>(
        tests: &'static [WinitBasedTest<T>],
        state: &mut State,
        elwt: &EventLoopWindowTarget<T>,
        control_flow: &mut ControlFlow,
    ) {
        if state.run {
            control_flow.set_exit_with_code(state.code);
            return;
        }
        state.run = true;

        for test in tests {
            print!("test {} ... ", test.name);

            match test.function {
                TestFunction::Oneoff(f) => match catch_unwind(AssertUnwindSafe(move || f(elwt))) {
                    Ok(()) => {
                        println!("{}", "ok".green());
                        state.passed += 1;
                    }

                    Err(e) => {
                        println!("{}", "FAILED".red());
                        state.panics.push((test.name, e));
                    }
                },
            }
        }

        let failures = state.panics.len();
        println!();
        if !state.panics.is_empty() {
            println!("failures:\n");
            for (name, e) in state.panics.drain(..) {
                println!("---- {} panic ----", name);

                if let Some(s) = e.downcast_ref::<&'static str>() {
                    println!("{}", s.red());
                } else if let Some(s) = e.downcast_ref::<String>() {
                    println!("{}", s.red());
                } else {
                    println!("{}", "unknown panic type".red());
                }

                println!();
            }

            print!("test result: {}", "FAILED".red());
        } else {
            print!("test result: {}", "ok".green());
        }

        let elapsed = state.start.elapsed();
        println!(
            ". {} passed; {} failed; finished in {:?}",
            state.passed, failures, elapsed
        );

        state.code = if failures == 0 { 0 } else { 1 };
        control_flow.set_exit_with_code(state.code);
    }

    pub struct WinitBasedTest<T: 'static> {
        pub name: &'static str,
        pub function: TestFunction<T>,
    }

    pub enum TestFunction<T: 'static> {
        Oneoff(fn(&EventLoopWindowTarget<T>)),
    }
}
