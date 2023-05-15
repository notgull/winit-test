// SPDX-License-Identifier: MIT OR Apache-2.0 OR Zlib

//! Run tests using a `winit` context.

#![forbid(unsafe_code)]

/// Re-exporting `winit` for the sake of convenience.
pub use winit;

/// The whole point.
#[macro_export]
macro_rules! main {
    ($ty:ty => $($tt:tt)*) => {
        #[cfg(not(target_os = "android"))]
        fn main() -> Result<(), Box<dyn std::error::Error>> {
            const TESTS: &[$crate::__private::WinitBasedTest<$ty>] = &[
                $crate::__winit_test_internal_collect_test!($($tt)*)
            ];

            $crate::__private::run(TESTS, ());
            Ok(())
        }

        #[cfg(target_os = "android")]
        #[no_mangle]
        fn android_main(app: $crate::__private::Context) {
            pub(super) const TESTS: &[$crate::__private::WinitBasedTest<$ty>] = &[
                $crate::__winit_test_internal_collect_test!($($tt)*)
            ];

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
    () => {};
    ($name:ident $(, $($tt:tt)*)?) => {
        $crate::__private::WinitBasedTest {
            name: stringify!($name),
            function: $crate::__private::TestFunction::Oneoff($name),
        }

        $(
            ,
            $crate::__winit_test_internal_collect_test!($($tt)*);
        )?
    }
}

#[doc(hidden)]
// This part is semver-exempt.
pub mod __private {
    use winit::event_loop::EventLoopBuilder;
    pub use winit::event_loop::EventLoopWindowTarget;

    use owo_colors::OwoColorize;
    use std::panic::{catch_unwind, AssertUnwindSafe};
    use std::time::Instant;

    #[cfg(target_os = "android")]
    pub use winit::platform::android::{
        activity::AndroidApp as Context, EventLoopBuilderExtAndroid,
    };
    #[cfg(not(target_os = "android"))]
    pub type Context = ();

    /// Run a set of tests using a `winit` context.
    pub fn run<T: 'static>(tests: &'static [WinitBasedTest<T>], _ctx: Context) {
        // Create a new event loop and obtain a window target.
        let mut builder = EventLoopBuilder::<T>::with_user_event();

        // Install the Android event loop extension if necessary.
        #[cfg(target_os = "android")]
        {
            builder = builder.with_android_app(_ctx);
        }

        let event_loop = builder.build();

        println!("\nRunning {} tests...", tests.len());
        let mut passed = 0;
        let mut panics = vec![];
        let start = Instant::now();
        let mut run = false;
        let mut code = 0;

        // Run the tests.
        event_loop.run(move |_, elwt, control_flow| {
            if run {
                control_flow.set_exit_with_code(code);
                return;
            }
            run = true;

            for test in tests {
                print!("test {} ... ", test.name);

                match test.function {
                    TestFunction::Oneoff(f) => {
                        match catch_unwind(AssertUnwindSafe(move || f(elwt))) {
                            Ok(()) => {
                                println!("{}", "ok".green());
                                passed += 1;
                            }

                            Err(e) => {
                                println!("{}", "FAILED".red());
                                panics.push((test.name, e));
                            }
                        }
                    }
                }
            }

            let failures = panics.len();
            println!();
            if !panics.is_empty() {
                println!("failures:\n");
                for (name, e) in panics.drain(..) {
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

            let elapsed = start.elapsed();
            println!(
                ". {} passed; {} failed; finished in {:?}",
                passed, failures, elapsed
            );

            code = if failures == 0 { 0 } else { 1 };
            control_flow.set_exit_with_code(code);
        });
    }

    pub struct WinitBasedTest<T: 'static> {
        pub name: &'static str,
        pub function: TestFunction<T>,
    }

    pub enum TestFunction<T: 'static> {
        Oneoff(fn(&EventLoopWindowTarget<T>)),
    }
}
