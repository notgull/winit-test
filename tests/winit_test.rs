// SPDX-License-Identifier: MIT OR Apache-2.0 OR Zlib

use winit::event_loop::EventLoopWindowTarget;
use winit_test::main;

fn smoke(_elwt: &EventLoopWindowTarget<()>) {}

main!(smoke);
