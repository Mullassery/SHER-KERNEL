//! Low-level Wayland transport primitives.
//!
//! Owned by SHER-Kernel. This is the kernel-facing substrate: client
//! connection lifecycle and shared buffer handles. It has no concept of
//! surfaces, outputs, input focus, or presentation — that policy belongs
//! to SHER-Display's compositor, which consumes [`WaylandTransport`] as
//! its low-level interface instead of duplicating it.

use sher_common::{Error, ObjectId, Result};
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct WaylandClient {
    pub id: ObjectId,
    pub name: String,
    pub is_connected: bool,
}

#[derive(Clone, Debug)]
pub struct Buffer {
    pub id: ObjectId,
    pub width: u32,
    pub height: u32,
    pub format: u32,
    pub stride: u32,
    pub size_bytes: usize,
}

/// Kernel-owned transport: client connections and shared buffer handles.
pub struct WaylandTransport {
    clients: HashMap<ObjectId, WaylandClient>,
    buffers: HashMap<ObjectId, Buffer>,
}

impl WaylandTransport {
    pub fn new() -> Self {
        WaylandTransport {
            clients: HashMap::new(),
            buffers: HashMap::new(),
        }
    }

    pub fn connect_client(&mut self, client: WaylandClient) -> Result<()> {
        let client_id = client.id;
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

        self.buffers.insert(buffer.id, buffer.clone());
        Ok(buffer)
    }

    pub fn get_buffer(&self, buffer_id: &ObjectId) -> Option<Buffer> {
        self.buffers.get(buffer_id).cloned()
    }

    pub fn release_buffer(&mut self, buffer_id: &ObjectId) -> Result<()> {
        if self.buffers.remove(buffer_id).is_some() {
            Ok(())
        } else {
            Err(Error::Device("Buffer not found".to_string()))
        }
    }

    pub fn client_count(&self) -> usize {
        self.clients.len()
    }

    pub fn buffer_count(&self) -> usize {
        self.buffers.len()
    }
}

impl Default for WaylandTransport {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_connection() {
        let mut transport = WaylandTransport::new();
        let client = WaylandClient {
            id: ObjectId::new(),
            name: "test_app".to_string(),
            is_connected: false,
        };

        let result = transport.connect_client(client.clone());
        assert!(result.is_ok());
        assert_eq!(transport.client_count(), 1);

        let retrieved = transport.get_client(&client.id);
        assert!(retrieved.is_some());
        assert!(retrieved.unwrap().is_connected);
    }

    #[test]
    fn test_client_disconnection() {
        let mut transport = WaylandTransport::new();
        let client = WaylandClient {
            id: ObjectId::new(),
            name: "test_app".to_string(),
            is_connected: false,
        };

        let client_id = client.id;
        let _ = transport.connect_client(client);
        assert_eq!(transport.client_count(), 1);

        let result = transport.disconnect_client(&client_id);
        assert!(result.is_ok());
        assert_eq!(transport.client_count(), 0);
    }

    #[test]
    fn test_buffer_lifecycle() {
        let mut transport = WaylandTransport::new();
        let buffer = transport.create_buffer(1920, 1080, 0x34325241).unwrap();
        assert_eq!(transport.buffer_count(), 1);
        assert!(transport.get_buffer(&buffer.id).is_some());

        let result = transport.release_buffer(&buffer.id);
        assert!(result.is_ok());
        assert_eq!(transport.buffer_count(), 0);
    }
}
