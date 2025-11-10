use std::{num::NonZeroU32, slice};

use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use tech_paws_ui::{
    View,
    render::{RenderCommand, RenderState, Renderer},
};
use tiny_skia::PixmapMut;

pub struct TinySkiaRenderer<D, W> {
    context: softbuffer::Context<D>,
    surface: softbuffer::Surface<D, W>,
}

impl<D: HasDisplayHandle, W: HasWindowHandle> TinySkiaRenderer<D, W> {
    pub fn new(display: D, window: W) -> Self {
        let context = softbuffer::Context::new(display).unwrap();
        let mut surface = softbuffer::Surface::new(&context, window).unwrap();

        Self { context, surface }
    }
}

impl<D: HasDisplayHandle, W: HasWindowHandle> Renderer for TinySkiaRenderer<D, W> {
    fn process_commands(&mut self, view: &View, state: &RenderState, commands: &[RenderCommand]) {
        let width = view.size.width as u32;
        let height = view.size.width as u32;

        self.surface
            .resize(
                NonZeroU32::new(width).unwrap(),
                NonZeroU32::new(height).unwrap(),
            )
            .unwrap();

        let mut surface_buffer = self.surface.buffer_mut().unwrap();
        let surface_buffer_u8 = unsafe {
            slice::from_raw_parts_mut(
                surface_buffer.as_mut_ptr() as *mut u8,
                surface_buffer.len() * 4,
            )
        };
        let mut pixmap = PixmapMut::from_bytes(surface_buffer_u8, width, height).unwrap();
        pixmap.fill(tiny_skia::Color::from_rgba8(0, 0, 0, 0xFF));

        surface_buffer.present().unwrap();
    }
}
