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

        // Run the tests.
        event_loop.run(move |_, elwt, control_flow| {
            for test in tests {
                match test.function {
                    TestFunction::Oneoff(f) => f(elwt),
                }
            }

            control_flow.set_exit();
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
