// SPDX-License-Identifier: MIT OR Apache-2.0 OR Zlib

use winit::event_loop::EventLoopWindowTarget;
use winit_test::main;

#[allow(dead_code)]
fn smoke(_elwt: &EventLoopWindowTarget<()>) {}
#[allow(dead_code)]
fn other_smoke(_elwt: &EventLoopWindowTarget<()>) {}

main!(smoke, other_smoke);
