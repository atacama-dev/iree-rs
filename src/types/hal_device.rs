use crate::types::utils::iree_string_view_to_string;
use iree_sys::iree::runtime::api::{
    iree_hal_device_id_t, iree_hal_device_info_t, iree_hal_device_release, iree_hal_device_t,
    iree_string_view_t,
};
use std::fmt::Display;

use super::hal_buffer::IreeHalBufferView;

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
