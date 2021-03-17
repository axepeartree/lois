# lois

2D renderer prototype written in Rust. Uses a `wgpu-rs` backend.

## Architecture

The `Graphics` struct is responsible for most of the core logic. It must own an implementation of the `Backend` trait.

It works by holding an internal queue which stores multiple commands. `DrawCommand::Clear` clears the render target; `DrawCommand::DrawTextureBatch` creates one or multiple quads, which are ready for rendering, using a given render target and a given texture.

Once the commands are registered, they're passed on to the backend through the `Graphics::present()` function. The backend can then use these commands to render everything to the screen in the correct order. Finally, the command queue is cleared.

## TODOs

- Fonts and text rendering
- Code cleanup and optimizations
