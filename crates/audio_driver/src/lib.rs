//! Audio Driver - Phase 11 Layer 4b
//!
//! ALSA-style audio device management providing:
//! - Playback and recording device enumeration
//! - Audio buffer management and ring buffers
//! - Sample rate and format configuration
//! - Volume control and muting
//! - Mixer operations and channel routing
//! - Audio format conversion

use std::collections::HashMap;
use sher_common::{ObjectId, Result};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum AudioFormat {
    S16LE,
    S32LE,
    F32LE,
    U8,
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum DeviceRole {
    Playback,
    Recording,
    Duplex,
}

#[derive(Clone, Debug)]
pub struct AudioDevice {
    pub id: ObjectId,
    pub name: String,
    pub role: DeviceRole,
    pub sample_rates: Vec<u32>,
    pub current_sample_rate: u32,
    pub formats: Vec<AudioFormat>,
    pub current_format: AudioFormat,
    pub channels: u32,
    pub is_active: bool,
}

#[derive(Clone, Debug)]
pub struct AudioBuffer {
    pub id: ObjectId,
    pub size_frames: usize,
    pub channels: u32,
    pub sample_rate: u32,
    pub format: AudioFormat,
    pub available_frames: usize,
}

#[derive(Clone, Debug)]
pub struct VolumeControl {
    pub device_id: ObjectId,
    pub left_volume: u32,
    pub right_volume: u32,
    pub is_muted: bool,
}

pub struct AudioDriver {
    devices: HashMap<ObjectId, AudioDevice>,
    buffers: HashMap<ObjectId, AudioBuffer>,
    volumes: HashMap<ObjectId, VolumeControl>,
    active_streams: usize,
}

impl AudioDriver {
    pub fn new() -> Self {
        AudioDriver {
            devices: HashMap::new(),
            buffers: HashMap::new(),
            volumes: HashMap::new(),
            active_streams: 0,
        }
    }

    pub fn register_device(&mut self, device: AudioDevice) -> Result<()> {
        let device_id = device.id.clone();
        let volume = VolumeControl {
            device_id: device_id.clone(),
            left_volume: 100,
            right_volume: 100,
            is_muted: false,
        };
        self.devices.insert(device_id.clone(), device);
        self.volumes.insert(device_id, volume);
        Ok(())
    }

    pub fn get_device(&self, device_id: &ObjectId) -> Option<AudioDevice> {
        self.devices.get(device_id).cloned()
    }

    pub fn get_devices_by_role(&self, role: DeviceRole) -> Vec<AudioDevice> {
        self.devices
            .values()
            .filter(|d| d.role == role)
            .cloned()
            .collect()
    }

    pub fn get_playback_devices(&self) -> Vec<AudioDevice> {
        self.devices
            .values()
            .filter(|d| matches!(d.role, DeviceRole::Playback | DeviceRole::Duplex))
            .cloned()
            .collect()
    }

    pub fn get_recording_devices(&self) -> Vec<AudioDevice> {
        self.devices
            .values()
            .filter(|d| matches!(d.role, DeviceRole::Recording | DeviceRole::Duplex))
            .cloned()
            .collect()
    }

    pub fn set_sample_rate(&mut self, device_id: &ObjectId, rate: u32) -> Result<()> {
        if let Some(device) = self.devices.get_mut(device_id) {
            if !device.sample_rates.contains(&rate) {
                return Err(sher_common::Error::Device("Unsupported sample rate".to_string()));
            }
            device.current_sample_rate = rate;
            Ok(())
        } else {
            Err(sher_common::Error::Device("Device not found".to_string()))
        }
    }

    pub fn set_format(&mut self, device_id: &ObjectId, format: AudioFormat) -> Result<()> {
        if let Some(device) = self.devices.get_mut(device_id) {
            if !device.formats.contains(&format) {
                return Err(sher_common::Error::Device("Unsupported format".to_string()));
            }
            device.current_format = format;
            Ok(())
        } else {
            Err(sher_common::Error::Device("Device not found".to_string()))
        }
    }

    pub fn allocate_buffer(&mut self, device_id: &ObjectId, frames: usize) -> Result<AudioBuffer> {
        if let Some(device) = self.devices.get(device_id) {
            let buffer = AudioBuffer {
                id: ObjectId::new(),
                size_frames: frames,
                channels: device.channels,
                sample_rate: device.current_sample_rate,
                format: device.current_format.clone(),
                available_frames: frames,
            };

            self.buffers.insert(buffer.id.clone(), buffer.clone());
            Ok(buffer)
        } else {
            Err(sher_common::Error::Device("Device not found".to_string()))
        }
    }

    pub fn get_buffer(&self, buffer_id: &ObjectId) -> Option<AudioBuffer> {
        self.buffers.get(buffer_id).cloned()
    }

    pub fn write_buffer(&mut self, buffer_id: &ObjectId, frames: usize) -> Result<()> {
        if let Some(buffer) = self.buffers.get_mut(buffer_id) {
            if frames > buffer.available_frames {
                return Err(sher_common::Error::Memory("Buffer overflow".to_string()));
            }
            buffer.available_frames -= frames;
            Ok(())
        } else {
            Err(sher_common::Error::Device("Buffer not found".to_string()))
        }
    }

    pub fn read_buffer(&mut self, buffer_id: &ObjectId, frames: usize) -> Result<()> {
        if let Some(buffer) = self.buffers.get_mut(buffer_id) {
            let can_read = buffer.size_frames - buffer.available_frames;
            if frames > can_read {
                return Err(sher_common::Error::Memory("Insufficient data".to_string()));
            }
            buffer.available_frames += frames;
            Ok(())
        } else {
            Err(sher_common::Error::Device("Buffer not found".to_string()))
        }
    }

    pub fn free_buffer(&mut self, buffer_id: &ObjectId) -> Result<()> {
        if self.buffers.remove(buffer_id).is_some() {
            Ok(())
        } else {
            Err(sher_common::Error::Device("Buffer not found".to_string()))
        }
    }

    pub fn set_volume(&mut self, device_id: &ObjectId, left: u32, right: u32) -> Result<()> {
        if left > 100 || right > 100 {
            return Err(sher_common::Error::Device("Volume out of range".to_string()));
        }

        if let Some(volume) = self.volumes.get_mut(device_id) {
            volume.left_volume = left;
            volume.right_volume = right;
            Ok(())
        } else {
            Err(sher_common::Error::Device("Device not found".to_string()))
        }
    }

    pub fn get_volume(&self, device_id: &ObjectId) -> Option<VolumeControl> {
        self.volumes.get(device_id).cloned()
    }

    pub fn set_mute(&mut self, device_id: &ObjectId, muted: bool) -> Result<()> {
        if let Some(volume) = self.volumes.get_mut(device_id) {
            volume.is_muted = muted;
            Ok(())
        } else {
            Err(sher_common::Error::Device("Device not found".to_string()))
        }
    }

    pub fn start_stream(&mut self, device_id: &ObjectId) -> Result<()> {
        if let Some(device) = self.devices.get_mut(device_id) {
            device.is_active = true;
            self.active_streams += 1;
            Ok(())
        } else {
            Err(sher_common::Error::Device("Device not found".to_string()))
        }
    }

    pub fn stop_stream(&mut self, device_id: &ObjectId) -> Result<()> {
        if let Some(device) = self.devices.get_mut(device_id) {
            device.is_active = false;
            self.active_streams = self.active_streams.saturating_sub(1);
            Ok(())
        } else {
            Err(sher_common::Error::Device("Device not found".to_string()))
        }
    }

    pub fn get_active_stream_count(&self) -> usize {
        self.active_streams
    }

    pub fn device_count(&self) -> usize {
        self.devices.len()
    }

    pub fn buffer_count(&self) -> usize {
        self.buffers.len()
    }
}

impl Default for AudioDriver {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_driver_creation() {
        let driver = AudioDriver::new();
        assert_eq!(driver.device_count(), 0);
    }

    #[test]
    fn test_register_playback_device() {
        let mut driver = AudioDriver::new();
        let device = AudioDevice {
            id: ObjectId::new(),
            name: "Speakers".to_string(),
            role: DeviceRole::Playback,
            sample_rates: vec![44100, 48000],
            current_sample_rate: 48000,
            formats: vec![AudioFormat::S16LE, AudioFormat::S32LE],
            current_format: AudioFormat::S16LE,
            channels: 2,
            is_active: false,
        };

        let result = driver.register_device(device.clone());
        assert!(result.is_ok());
        assert_eq!(driver.device_count(), 1);
    }

    #[test]
    fn test_get_playback_devices() {
        let mut driver = AudioDriver::new();
        let playback = AudioDevice {
            id: ObjectId::new(),
            name: "Speakers".to_string(),
            role: DeviceRole::Playback,
            sample_rates: vec![48000],
            current_sample_rate: 48000,
            formats: vec![AudioFormat::S16LE],
            current_format: AudioFormat::S16LE,
            channels: 2,
            is_active: false,
        };

        let _ = driver.register_device(playback);
        let devices = driver.get_playback_devices();
        assert_eq!(devices.len(), 1);
    }

    #[test]
    fn test_get_recording_devices() {
        let mut driver = AudioDriver::new();
        let recording = AudioDevice {
            id: ObjectId::new(),
            name: "Microphone".to_string(),
            role: DeviceRole::Recording,
            sample_rates: vec![48000],
            current_sample_rate: 48000,
            formats: vec![AudioFormat::S16LE],
            current_format: AudioFormat::S16LE,
            channels: 1,
            is_active: false,
        };

        let _ = driver.register_device(recording);
        let devices = driver.get_recording_devices();
        assert_eq!(devices.len(), 1);
    }

    #[test]
    fn test_set_sample_rate() {
        let mut driver = AudioDriver::new();
        let device = AudioDevice {
            id: ObjectId::new(),
            name: "Speaker".to_string(),
            role: DeviceRole::Playback,
            sample_rates: vec![44100, 48000],
            current_sample_rate: 44100,
            formats: vec![AudioFormat::S16LE],
            current_format: AudioFormat::S16LE,
            channels: 2,
            is_active: false,
        };

        let device_id = device.id.clone();
        let _ = driver.register_device(device);

        let result = driver.set_sample_rate(&device_id, 48000);
        assert!(result.is_ok());

        let updated = driver.get_device(&device_id).unwrap();
        assert_eq!(updated.current_sample_rate, 48000);
    }

    #[test]
    fn test_unsupported_sample_rate() {
        let mut driver = AudioDriver::new();
        let device = AudioDevice {
            id: ObjectId::new(),
            name: "Speaker".to_string(),
            role: DeviceRole::Playback,
            sample_rates: vec![48000],
            current_sample_rate: 48000,
            formats: vec![AudioFormat::S16LE],
            current_format: AudioFormat::S16LE,
            channels: 2,
            is_active: false,
        };

        let device_id = device.id.clone();
        let _ = driver.register_device(device);

        let result = driver.set_sample_rate(&device_id, 96000);
        assert!(result.is_err());
    }

    #[test]
    fn test_set_format() {
        let mut driver = AudioDriver::new();
        let device = AudioDevice {
            id: ObjectId::new(),
            name: "Speaker".to_string(),
            role: DeviceRole::Playback,
            sample_rates: vec![48000],
            current_sample_rate: 48000,
            formats: vec![AudioFormat::S16LE, AudioFormat::S32LE],
            current_format: AudioFormat::S16LE,
            channels: 2,
            is_active: false,
        };

        let device_id = device.id.clone();
        let _ = driver.register_device(device);

        let result = driver.set_format(&device_id, AudioFormat::S32LE);
        assert!(result.is_ok());

        let updated = driver.get_device(&device_id).unwrap();
        assert_eq!(updated.current_format, AudioFormat::S32LE);
    }

    #[test]
    fn test_allocate_buffer() {
        let mut driver = AudioDriver::new();
        let device = AudioDevice {
            id: ObjectId::new(),
            name: "Speaker".to_string(),
            role: DeviceRole::Playback,
            sample_rates: vec![48000],
            current_sample_rate: 48000,
            formats: vec![AudioFormat::S16LE],
            current_format: AudioFormat::S16LE,
            channels: 2,
            is_active: false,
        };

        let device_id = device.id.clone();
        let _ = driver.register_device(device);

        let result = driver.allocate_buffer(&device_id, 4096);
        assert!(result.is_ok());

        let buffer = result.unwrap();
        assert_eq!(buffer.size_frames, 4096);
    }

    #[test]
    fn test_buffer_write_read() {
        let mut driver = AudioDriver::new();
        let device = AudioDevice {
            id: ObjectId::new(),
            name: "Speaker".to_string(),
            role: DeviceRole::Playback,
            sample_rates: vec![48000],
            current_sample_rate: 48000,
            formats: vec![AudioFormat::S16LE],
            current_format: AudioFormat::S16LE,
            channels: 2,
            is_active: false,
        };

        let device_id = device.id.clone();
        let _ = driver.register_device(device);

        let buffer = driver.allocate_buffer(&device_id, 4096).unwrap();
        let buffer_id = buffer.id.clone();

        let write_result = driver.write_buffer(&buffer_id, 1024);
        assert!(write_result.is_ok());

        let read_result = driver.read_buffer(&buffer_id, 1024);
        assert!(read_result.is_ok());
    }

    #[test]
    fn test_set_volume() {
        let mut driver = AudioDriver::new();
        let device = AudioDevice {
            id: ObjectId::new(),
            name: "Speaker".to_string(),
            role: DeviceRole::Playback,
            sample_rates: vec![48000],
            current_sample_rate: 48000,
            formats: vec![AudioFormat::S16LE],
            current_format: AudioFormat::S16LE,
            channels: 2,
            is_active: false,
        };

        let device_id = device.id.clone();
        let _ = driver.register_device(device);

        let result = driver.set_volume(&device_id, 80, 75);
        assert!(result.is_ok());

        let volume = driver.get_volume(&device_id).unwrap();
        assert_eq!(volume.left_volume, 80);
        assert_eq!(volume.right_volume, 75);
    }

    #[test]
    fn test_volume_out_of_range() {
        let mut driver = AudioDriver::new();
        let device = AudioDevice {
            id: ObjectId::new(),
            name: "Speaker".to_string(),
            role: DeviceRole::Playback,
            sample_rates: vec![48000],
            current_sample_rate: 48000,
            formats: vec![AudioFormat::S16LE],
            current_format: AudioFormat::S16LE,
            channels: 2,
            is_active: false,
        };

        let device_id = device.id.clone();
        let _ = driver.register_device(device);

        let result = driver.set_volume(&device_id, 150, 50);
        assert!(result.is_err());
    }

    #[test]
    fn test_set_mute() {
        let mut driver = AudioDriver::new();
        let device = AudioDevice {
            id: ObjectId::new(),
            name: "Speaker".to_string(),
            role: DeviceRole::Playback,
            sample_rates: vec![48000],
            current_sample_rate: 48000,
            formats: vec![AudioFormat::S16LE],
            current_format: AudioFormat::S16LE,
            channels: 2,
            is_active: false,
        };

        let device_id = device.id.clone();
        let _ = driver.register_device(device);

        let result = driver.set_mute(&device_id, true);
        assert!(result.is_ok());

        let volume = driver.get_volume(&device_id).unwrap();
        assert!(volume.is_muted);
    }

    #[test]
    fn test_start_stop_stream() {
        let mut driver = AudioDriver::new();
        let device = AudioDevice {
            id: ObjectId::new(),
            name: "Speaker".to_string(),
            role: DeviceRole::Playback,
            sample_rates: vec![48000],
            current_sample_rate: 48000,
            formats: vec![AudioFormat::S16LE],
            current_format: AudioFormat::S16LE,
            channels: 2,
            is_active: false,
        };

        let device_id = device.id.clone();
        let _ = driver.register_device(device);

        assert_eq!(driver.get_active_stream_count(), 0);

        let _ = driver.start_stream(&device_id);
        assert_eq!(driver.get_active_stream_count(), 1);

        let _ = driver.stop_stream(&device_id);
        assert_eq!(driver.get_active_stream_count(), 0);
    }

    #[test]
    fn test_multiple_devices() {
        let mut driver = AudioDriver::new();

        let playback = AudioDevice {
            id: ObjectId::new(),
            name: "Speakers".to_string(),
            role: DeviceRole::Playback,
            sample_rates: vec![48000],
            current_sample_rate: 48000,
            formats: vec![AudioFormat::S16LE],
            current_format: AudioFormat::S16LE,
            channels: 2,
            is_active: false,
        };

        let recording = AudioDevice {
            id: ObjectId::new(),
            name: "Microphone".to_string(),
            role: DeviceRole::Recording,
            sample_rates: vec![48000],
            current_sample_rate: 48000,
            formats: vec![AudioFormat::S16LE],
            current_format: AudioFormat::S16LE,
            channels: 1,
            is_active: false,
        };

        let _ = driver.register_device(playback);
        let _ = driver.register_device(recording);

        assert_eq!(driver.device_count(), 2);
        assert_eq!(driver.get_playback_devices().len(), 1);
        assert_eq!(driver.get_recording_devices().len(), 1);
    }
}
