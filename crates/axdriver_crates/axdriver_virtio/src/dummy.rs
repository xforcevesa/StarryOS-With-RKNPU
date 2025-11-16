use core::ptr::NonNull;

use virtio_drivers::{
    Error, PhysAddr, Result,
    transport::{DeviceStatus, DeviceType, Transport},
};

pub struct DummyTransport;

impl Transport for DummyTransport {
    fn device_type(&self) -> DeviceType {
        DeviceType::Invalid
    }

    fn read_device_features(&mut self) -> u64 {
        0
    }

    fn write_driver_features(&mut self, _driver_features: u64) {}

    fn max_queue_size(&mut self, _queue: u16) -> u32 {
        0
    }

    fn notify(&mut self, _queue: u16) {}

    fn get_status(&self) -> DeviceStatus {
        DeviceStatus::empty()
    }

    fn set_status(&mut self, _status: DeviceStatus) {}

    fn set_guest_page_size(&mut self, _guest_page_size: u32) {}

    fn requires_legacy_layout(&self) -> bool {
        false
    }

    fn queue_set(
        &mut self,
        _queue: u16,
        _size: u32,
        _descriptors: PhysAddr,
        _driver_area: PhysAddr,
        _device_area: PhysAddr,
    ) {
    }

    fn queue_unset(&mut self, _queue: u16) {}

    fn queue_used(&mut self, _queue: u16) -> bool {
        false
    }

    fn ack_interrupt(&mut self) -> bool {
        false
    }

    fn config_space<T: 'static>(&self) -> Result<NonNull<T>> {
        Err(Error::ConfigSpaceMissing)
    }
}
