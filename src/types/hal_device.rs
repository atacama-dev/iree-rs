use crate::types::utils::iree_string_view_to_string;

use iree_sys::iree::runtime::api::{
    iree_hal_device_id_t, iree_hal_device_info_t, iree_hal_device_release, iree_hal_device_t,
    iree_hal_device_transfer_d2h, iree_string_view_t, iree_timeout_t,
    iree_timeout_type_e_IREE_TIMEOUT_ABSOLUTE,
};

use std::ffi::c_void;
use std::fmt::Display;
use std::ptr::null_mut;

use super::bytespan::IreeConstByteSpan;
use super::hal_buffer::{IreeHalBuffer, IreeHalBufferView};

#[derive(Debug)]
pub struct IreeHalDevice {
    pub(crate) device_ptr: *mut iree_hal_device_t,
}

impl Drop for IreeHalDevice {
    fn drop(&mut self) {
        unsafe {
            iree_hal_device_release(self.device_ptr);
        }
    }
}

pub fn iree_infinite_timeout() -> iree_timeout_t {
    iree_timeout_t {
        type_: iree_timeout_type_e_IREE_TIMEOUT_ABSOLUTE,
        nanos: 0i64,
    }
}

impl IreeHalDevice {
    pub fn transfer_d2h<T>(
        &self,
        source: &IreeHalBufferView,
        source_offset: usize,
        target: *mut c_void,
        data_length: usize,
        flags: u32,
        timeout: iree_timeout_t,
    ) {
        unsafe {
            iree_hal_device_transfer_d2h(
                self.device_ptr,
                source.buffer().unwrap().buffer_ptr,
                source_offset,
                target,
                data_length,
                flags,
                timeout,
            );
        }
    }
}

pub struct IreeDeviceInfo {
    pub device_id: iree_hal_device_id_t,
    pub path: iree_string_view_t,
    pub name: iree_string_view_t,
}

impl Display for IreeDeviceInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        unsafe {
            write!(
                f,
                "path: {}, name: {}",
                if self.path.size > 0 {
                    iree_string_view_to_string(self.path)
                } else {
                    "none".to_string()
                },
                iree_string_view_to_string(self.name)
            )
        }
    }
}

impl From<iree_hal_device_info_t> for IreeDeviceInfo {
    fn from(device_info: iree_hal_device_info_t) -> Self {
        Self {
            device_id: device_info.device_id,
            path: device_info.path,
            name: device_info.name,
        }
    }
}
