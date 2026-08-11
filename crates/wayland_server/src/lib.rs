//! Wayland Compositor - Phase 11 Layer 6 (DEPRECATED)
//!
//! ## Ownership boundary
//!
//! This crate historically implemented a full Wayland compositor —
//! surfaces, outputs, pointer routing, focus — inside SHER-Kernel. That
//! ownership was wrong: the kernel should transport primitives, not decide
//! what they mean or how they're presented.
//!
//! - **SHER-Kernel** (this crate, [`transport`]) owns: client connections,
//!   protocol message handling, shared-memory buffer handles, and
//!   kernel-facing synchronization. See [`WaylandTransport`].
//! - **SHER-Display** owns: surfaces, outputs, pointer/keyboard/seat,
//!   surface lifecycle, compositing, and display policy. It consumes
//!   [`WaylandTransport`] as its low-level substrate rather than
//!   duplicating it — see `sher-display/compatibility-wayland`.
//!
//! [`WaylandCompositor`] below is retained for compatibility during
//! migration and is deprecated: do not add new functionality to it. New
//! compositor policy belongs in SHER-Display.

pub mod transport;
pub use transport::{Buffer, WaylandClient, WaylandTransport};

use sher_common::{ObjectId, Result};
use std::collections::HashMap;

#[derive(Clone, Debug)]
#[deprecated(
    since = "0.1.0",
    note = "surface policy now owned by SHER-Display; see sher-display/surfaces"
)]
pub struct Surface {
    pub id: ObjectId,
    pub client_id: ObjectId,
    pub width: u32,
    pub height: u32,
    pub buffer_id: Option<ObjectId>,
    pub damage_region: bool,
}

#[derive(Clone, Debug)]
#[deprecated(
    since = "0.1.0",
    note = "output policy now owned by SHER-Display; see sher-display/outputs"
)]
pub struct Output {
    pub id: ObjectId,
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub scale: f32,
    pub is_enabled: bool,
}

#[derive(Clone, Debug)]
#[deprecated(
    since = "0.1.0",
    note = "input routing now owned by SHER-Display; see sher-display/input"
)]
pub enum PointerEventType {
    Motion,
    Button,
    Scroll,
    Leave,
    Enter,
}

#[derive(Clone, Debug)]
#[deprecated(
    since = "0.1.0",
    note = "input routing now owned by SHER-Display; see sher-display/input"
)]
pub struct PointerEvent {
    pub surface_id: Option<ObjectId>,
    pub event_type: PointerEventType,
    pub x: i32,
    pub y: i32,
    pub button: Option<u32>,
}

#[deprecated(
    since = "0.1.0",
    note = "compositor policy now owned by SHER-Display; see sher-display/compositor. \
            Use wayland_server::WaylandTransport for the low-level substrate."
)]
pub struct WaylandCompositor {
    clients: HashMap<ObjectId, WaylandClient>,
    surfaces: HashMap<ObjectId, Surface>,
    buffers: HashMap<ObjectId, Buffer>,
    outputs: HashMap<ObjectId, Output>,
    focused_surface: Option<ObjectId>,
    pointer_position: (i32, i32),
    is_running: bool,
}

#[allow(deprecated)]
impl WaylandCompositor {
    pub fn new() -> Self {
        WaylandCompositor {
            clients: HashMap::new(),
            surfaces: HashMap::new(),
            buffers: HashMap::new(),
            outputs: HashMap::new(),
            focused_surface: None,
            pointer_position: (0, 0),
            is_running: false,
        }
    }

    pub fn start(&mut self) -> Result<()> {
        self.is_running = true;
        Ok(())
    }

    pub fn stop(&mut self) -> Result<()> {
        self.is_running = false;
        Ok(())
    }

    pub fn is_running(&self) -> bool {
        self.is_running
    }

    pub fn connect_client(&mut self, client: WaylandClient) -> Result<()> {
        let client_id = client.id.clone();
        let mut client = client;
        client.is_connected = true;
        self.clients.insert(client_id, client);
        Ok(())
    }

    pub fn disconnect_client(&mut self, client_id: &ObjectId) -> Result<()> {
        if let Some(mut client) = self.clients.remove(client_id) {
            client.is_connected = false;
        }
        Ok(())
    }

    pub fn get_client(&self, client_id: &ObjectId) -> Option<WaylandClient> {
        self.clients.get(client_id).cloned()
    }

    pub fn create_surface(&mut self, client_id: &ObjectId) -> Result<Surface> {
        if !self.clients.contains_key(client_id) {
            return Err(sher_common::Error::Device("Client not found".to_string()));
        }

        let surface = Surface {
            id: ObjectId::new(),
            client_id: client_id.clone(),
            width: 0,
            height: 0,
            buffer_id: None,
            damage_region: false,
        };

        self.surfaces.insert(surface.id.clone(), surface.clone());
        Ok(surface)
    }

    pub fn get_surface(&self, surface_id: &ObjectId) -> Option<Surface> {
        self.surfaces.get(surface_id).cloned()
    }

    pub fn configure_surface(&mut self, surface_id: &ObjectId, width: u32, height: u32) -> Result<()> {
        if let Some(surface) = self.surfaces.get_mut(surface_id) {
            surface.width = width;
            surface.height = height;
            Ok(())
        } else {
            Err(sher_common::Error::Device("Surface not found".to_string()))
        }
    }

    pub fn destroy_surface(&mut self, surface_id: &ObjectId) -> Result<()> {
        if self.surfaces.remove(surface_id).is_some() {
            if self.focused_surface.as_ref() == Some(surface_id) {
                self.focused_surface = None;
            }
            Ok(())
        } else {
            Err(sher_common::Error::Device("Surface not found".to_string()))
        }
    }

    pub fn create_buffer(&mut self, width: u32, height: u32, format: u32) -> Result<Buffer> {
        let stride = width * 4;
        let size_bytes = (stride * height) as usize;

        let buffer = Buffer {
            id: ObjectId::new(),
            width,
            height,
            format,
            stride,
            size_bytes,
        };

        self.buffers.insert(buffer.id.clone(), buffer.clone());
        Ok(buffer)
    }

    pub fn get_buffer(&self, buffer_id: &ObjectId) -> Option<Buffer> {
        self.buffers.get(buffer_id).cloned()
    }

    pub fn attach_buffer(&mut self, surface_id: &ObjectId, buffer_id: &ObjectId) -> Result<()> {
        if !self.buffers.contains_key(buffer_id) {
            return Err(sher_common::Error::Device("Buffer not found".to_string()));
        }

        if let Some(surface) = self.surfaces.get_mut(surface_id) {
            surface.buffer_id = Some(buffer_id.clone());
            Ok(())
        } else {
            Err(sher_common::Error::Device("Surface not found".to_string()))
        }
    }

    pub fn commit_surface(&mut self, surface_id: &ObjectId) -> Result<()> {
        if let Some(surface) = self.surfaces.get_mut(surface_id) {
            surface.damage_region = false;
            Ok(())
        } else {
            Err(sher_common::Error::Device("Surface not found".to_string()))
        }
    }

    pub fn register_output(&mut self, output: Output) -> Result<()> {
        self.outputs.insert(output.id.clone(), output);
        Ok(())
    }

    pub fn get_output(&self, output_id: &ObjectId) -> Option<Output> {
        self.outputs.get(output_id).cloned()
    }

    pub fn set_focus(&mut self, surface_id: Option<ObjectId>) -> Result<()> {
        self.focused_surface = surface_id;
        Ok(())
    }

    pub fn get_focused_surface(&self) -> Option<ObjectId> {
        self.focused_surface.clone()
    }

    pub fn route_pointer_event(&mut self, event: PointerEvent) -> Result<()> {
        match event.event_type {
            PointerEventType::Motion => {
                self.pointer_position = (event.x, event.y);
            }
            PointerEventType::Button => {
                if let Some(surface_id) = event.surface_id {
                    self.set_focus(Some(surface_id))?;
                }
            }
            PointerEventType::Leave => {
                self.set_focus(None)?;
            }
            PointerEventType::Enter => {
                if let Some(surface_id) = event.surface_id {
                    self.set_focus(Some(surface_id))?;
                }
            }
            PointerEventType::Scroll => {}
        }
        Ok(())
    }

    pub fn get_pointer_position(&self) -> (i32, i32) {
        self.pointer_position
    }

    pub fn client_count(&self) -> usize {
        self.clients.len()
    }

    pub fn surface_count(&self) -> usize {
        self.surfaces.len()
    }

    pub fn buffer_count(&self) -> usize {
        self.buffers.len()
    }

    pub fn output_count(&self) -> usize {
        self.outputs.len()
    }
}

#[allow(deprecated)]
impl Default for WaylandCompositor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[allow(deprecated)]
mod tests {
    use super::*;

    #[test]
    fn test_compositor_creation() {
        let compositor = WaylandCompositor::new();
        assert!(!compositor.is_running());
    }

    #[test]
    fn test_start_stop() {
        let mut compositor = WaylandCompositor::new();
        let _ = compositor.start();
        assert!(compositor.is_running());

        let _ = compositor.stop();
        assert!(!compositor.is_running());
    }

    #[test]
    fn test_client_connection() {
        let mut compositor = WaylandCompositor::new();
        let client = WaylandClient {
            id: ObjectId::new(),
            name: "test_app".to_string(),
            is_connected: false,
        };

        let result = compositor.connect_client(client.clone());
        assert!(result.is_ok());
        assert_eq!(compositor.client_count(), 1);

        let retrieved = compositor.get_client(&client.id);
        assert!(retrieved.is_some());
        assert!(retrieved.unwrap().is_connected);
    }

    #[test]
    fn test_client_disconnection() {
        let mut compositor = WaylandCompositor::new();
        let client = WaylandClient {
            id: ObjectId::new(),
            name: "test_app".to_string(),
            is_connected: false,
        };

        let client_id = client.id.clone();
        let _ = compositor.connect_client(client);
        assert_eq!(compositor.client_count(), 1);

        let result = compositor.disconnect_client(&client_id);
        assert!(result.is_ok());
        assert_eq!(compositor.client_count(), 0);
    }

    #[test]
    fn test_surface_creation() {
        let mut compositor = WaylandCompositor::new();
        let client = WaylandClient {
            id: ObjectId::new(),
            name: "test_app".to_string(),
            is_connected: false,
        };

        let client_id = client.id.clone();
        let _ = compositor.connect_client(client);

        let result = compositor.create_surface(&client_id);
        assert!(result.is_ok());
        assert_eq!(compositor.surface_count(), 1);
    }

    #[test]
    fn test_surface_configuration() {
        let mut compositor = WaylandCompositor::new();
        let client = WaylandClient {
            id: ObjectId::new(),
            name: "test_app".to_string(),
            is_connected: false,
        };

        let client_id = client.id.clone();
        let _ = compositor.connect_client(client);

        let surface = compositor.create_surface(&client_id).unwrap();
        let result = compositor.configure_surface(&surface.id, 1920, 1080);
        assert!(result.is_ok());

        let configured = compositor.get_surface(&surface.id).unwrap();
        assert_eq!(configured.width, 1920);
        assert_eq!(configured.height, 1080);
    }

    #[test]
    fn test_surface_destruction() {
        let mut compositor = WaylandCompositor::new();
        let client = WaylandClient {
            id: ObjectId::new(),
            name: "test_app".to_string(),
            is_connected: false,
        };

        let client_id = client.id.clone();
        let _ = compositor.connect_client(client);

        let surface = compositor.create_surface(&client_id).unwrap();
        assert_eq!(compositor.surface_count(), 1);

        let result = compositor.destroy_surface(&surface.id);
        assert!(result.is_ok());
        assert_eq!(compositor.surface_count(), 0);
    }

    #[test]
    fn test_buffer_creation() {
        let mut compositor = WaylandCompositor::new();
        let result = compositor.create_buffer(1920, 1080, 0x34325241);
        assert!(result.is_ok());

        let buffer = result.unwrap();
        assert_eq!(buffer.width, 1920);
        assert_eq!(buffer.height, 1080);
        assert_eq!(compositor.buffer_count(), 1);
    }

    #[test]
    fn test_attach_buffer_to_surface() {
        let mut compositor = WaylandCompositor::new();
        let client = WaylandClient {
            id: ObjectId::new(),
            name: "test_app".to_string(),
            is_connected: false,
        };

        let client_id = client.id.clone();
        let _ = compositor.connect_client(client);

        let surface = compositor.create_surface(&client_id).unwrap();
        let buffer = compositor.create_buffer(1920, 1080, 0x34325241).unwrap();

        let result = compositor.attach_buffer(&surface.id, &buffer.id);
        assert!(result.is_ok());

        let attached = compositor.get_surface(&surface.id).unwrap();
        assert!(attached.buffer_id.is_some());
    }

    #[test]
    fn test_commit_surface() {
        let mut compositor = WaylandCompositor::new();
        let client = WaylandClient {
            id: ObjectId::new(),
            name: "test_app".to_string(),
            is_connected: false,
        };

        let client_id = client.id.clone();
        let _ = compositor.connect_client(client);

        let surface = compositor.create_surface(&client_id).unwrap();
        let result = compositor.commit_surface(&surface.id);
        assert!(result.is_ok());
    }

    #[test]
    fn test_output_registration() {
        let mut compositor = WaylandCompositor::new();
        let output = Output {
            id: ObjectId::new(),
            name: "HDMI-1".to_string(),
            width: 1920,
            height: 1080,
            scale: 1.0,
            is_enabled: true,
        };

        let result = compositor.register_output(output.clone());
        assert!(result.is_ok());
        assert_eq!(compositor.output_count(), 1);

        let retrieved = compositor.get_output(&output.id);
        assert!(retrieved.is_some());
    }

    #[test]
    fn test_focus_management() {
        let mut compositor = WaylandCompositor::new();
        let client = WaylandClient {
            id: ObjectId::new(),
            name: "test_app".to_string(),
            is_connected: false,
        };

        let client_id = client.id.clone();
        let _ = compositor.connect_client(client);

        let surface = compositor.create_surface(&client_id).unwrap();
        let _ = compositor.set_focus(Some(surface.id.clone()));

        assert_eq!(compositor.get_focused_surface(), Some(surface.id));
    }

    #[test]
    fn test_pointer_position_tracking() {
        let mut compositor = WaylandCompositor::new();
        let event = PointerEvent {
            surface_id: None,
            event_type: PointerEventType::Motion,
            x: 100,
            y: 200,
            button: None,
        };

        let _ = compositor.route_pointer_event(event);
        assert_eq!(compositor.get_pointer_position(), (100, 200));
    }

    #[test]
    fn test_multiple_surfaces() {
        let mut compositor = WaylandCompositor::new();
        let client = WaylandClient {
            id: ObjectId::new(),
            name: "test_app".to_string(),
            is_connected: false,
        };

        let client_id = client.id.clone();
        let _ = compositor.connect_client(client);

        let _surface1 = compositor.create_surface(&client_id).unwrap();
        let _surface2 = compositor.create_surface(&client_id).unwrap();
        let _surface3 = compositor.create_surface(&client_id).unwrap();

        assert_eq!(compositor.surface_count(), 3);
    }

    #[test]
    fn test_render_pipeline() {
        let mut compositor = WaylandCompositor::new();
        let client = WaylandClient {
            id: ObjectId::new(),
            name: "app".to_string(),
            is_connected: false,
        };

        let client_id = client.id.clone();
        let _ = compositor.connect_client(client);

        let surface = compositor.create_surface(&client_id).unwrap();
        let buffer = compositor.create_buffer(1920, 1080, 0x34325241).unwrap();

        let _ = compositor.attach_buffer(&surface.id, &buffer.id);
        let _ = compositor.commit_surface(&surface.id);

        assert_eq!(compositor.surface_count(), 1);
        assert_eq!(compositor.buffer_count(), 1);
    }
}
