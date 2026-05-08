use core_graphics_types::geometry::CGSize;
use metal::foreign_types::{ForeignType, ForeignTypeRef};
use metal::{CommandQueue, Device, MetalLayer};
use objc::{class, msg_send, sel, sel_impl};
use skia_safe::{
    gpu::{self, mtl, SurfaceOrigin},
    ColorType, Surface,
};

/// Owns the Metal device, command queue, Skia GPU context, and the
/// CAMetalLayer attached to the NSPanel's content view.
pub struct MetalCanvas {
    pub device: Device,
    pub command_queue: CommandQueue,
    pub gr_context: gpu::DirectContext,
    pub layer: MetalLayer,
    pub width: i32,
    pub height: i32,
    pub scale: f32,
}

impl MetalCanvas {
    /// Attach a Metal drawing layer to the given NSPanel.
    ///
    /// `width`/`height` are logical points; `scale` is the Retina backing
    /// scale factor (2.0 on most Retina displays).
    ///
    /// # Safety
    /// `ns_panel` must be a valid NSPanel pointer. Must be called on the main thread.
    pub unsafe fn new(
        ns_panel: *mut objc::runtime::Object,
        width: i32,
        height: i32,
        scale: f32,
    ) -> Self {
        let device = Device::system_default().expect("no Metal device found");
        let command_queue = device.new_command_queue();

        // Attach a CAMetalLayer to the panel's content view
        let content_view: *mut objc::runtime::Object = msg_send![ns_panel, contentView];
        let _: () = msg_send![content_view, setWantsLayer: true];

        let layer = MetalLayer::new();
        layer.set_device(&device);
        layer.set_pixel_format(metal::MTLPixelFormat::BGRA8Unorm);
        layer.set_presents_with_transaction(false);
        // Disabling framebufferOnly allows Skia blend modes to work correctly.
        layer.set_framebuffer_only(false);
        layer.set_opaque(false); // transparent overlay

        let physical_w = (width as f64) * (scale as f64);
        let physical_h = (height as f64) * (scale as f64);
        layer.set_drawable_size(CGSize::new(physical_w, physical_h));

        // Attach layer to the content view
        let layer_ptr = layer.as_ref() as *const _ as *mut objc::runtime::Object;
        let _: () = msg_send![content_view, setLayer: layer_ptr];

        // Build Skia Metal backend context
        let backend = mtl::BackendContext::new(
            device.as_ptr() as mtl::Handle,
            command_queue.as_ptr() as mtl::Handle,
        );
        let gr_context = gpu::direct_contexts::make_metal(&backend, None)
            .expect("failed to create Skia Metal context");

        Self {
            device,
            command_queue,
            gr_context,
            layer,
            width,
            height,
            scale,
        }
    }

    /// Acquire the next drawable and wrap it in a Skia Surface.
    /// Returns `None` if no drawable is available (display throttle / offscreen).
    ///
    /// IMPORTANT: The caller must call `flush()` on the returned surface's context,
    /// then present the drawable before dropping. Use `render_frame` for a complete
    /// draw-flush-present cycle.
    pub fn next_surface(&mut self) -> Option<Surface> {
        let drawable = self.layer.next_drawable()?;

        let texture_info = unsafe {
            mtl::TextureInfo::new(drawable.texture().as_ptr() as mtl::Handle)
        };

        let drawable_size = self.layer.drawable_size();
        let physical_w = drawable_size.width as i32;
        let physical_h = drawable_size.height as i32;

        let backend_rt = gpu::backend_render_targets::make_mtl(
            (physical_w, physical_h),
            &texture_info,
        );

        gpu::surfaces::wrap_backend_render_target(
            &mut self.gr_context,
            &backend_rt,
            SurfaceOrigin::TopLeft,
            ColorType::BGRA8888,
            None,
            None,
        )
    }

    /// Flush pending Skia work and submit to GPU.
    pub fn flush(&mut self) {
        self.gr_context.flush_and_submit();
    }

    /// Execute a complete render frame: acquire drawable, invoke the draw callback,
    /// flush the Skia context, then present the drawable.
    ///
    /// Returns `true` if a drawable was available and the frame was rendered.
    pub fn render_frame<F>(&mut self, draw_fn: F) -> bool
    where
        F: FnOnce(&skia_safe::Canvas),
    {
        let drawable = match self.layer.next_drawable() {
            Some(d) => d,
            None => return false,
        };

        let texture_info = unsafe {
            mtl::TextureInfo::new(drawable.texture().as_ptr() as mtl::Handle)
        };

        let drawable_size = self.layer.drawable_size();
        let physical_w = drawable_size.width as i32;
        let physical_h = drawable_size.height as i32;

        let backend_rt = gpu::backend_render_targets::make_mtl(
            (physical_w, physical_h),
            &texture_info,
        );

        if let Some(mut surface) = gpu::surfaces::wrap_backend_render_target(
            &mut self.gr_context,
            &backend_rt,
            SurfaceOrigin::TopLeft,
            ColorType::BGRA8888,
            None,
            None,
        ) {
            draw_fn(surface.canvas());
            self.gr_context.flush_and_submit();
            drop(surface);

            let command_buffer = self.command_queue.new_command_buffer();
            command_buffer.present_drawable(drawable);
            command_buffer.commit();
            true
        } else {
            false
        }
    }

    /// Resize the drawable layer to new logical dimensions.
    pub fn resize(&mut self, width: i32, height: i32, scale: f32) {
        self.width = width;
        self.height = height;
        self.scale = scale;
        let physical_w = (width as f64) * (scale as f64);
        let physical_h = (height as f64) * (scale as f64);
        self.layer.set_drawable_size(CGSize::new(physical_w, physical_h));
    }
}
