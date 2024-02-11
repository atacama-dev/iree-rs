use super::allocator::IreeAllocator;
use super::hal_device::{IreeDeviceInfo, IreeHalDevice};
use super::utils::iree_string_view_to_string;
use crate::err::IreeError;
use crate::types::status::IreeStatus;
use iree_sys::helper::IREE_CHECK_OK;
use iree_sys::iree::runtime::api::{
    iree_hal_device_id_t, iree_hal_driver_create_device_by_id, iree_hal_driver_info_t,
    iree_hal_driver_query_available_devices, iree_hal_driver_registry_enumerate,
    iree_hal_driver_registry_t, iree_hal_driver_registry_try_create, iree_hal_driver_t,
    iree_string_view_t,
};

use std::fmt::Display;
use std::ptr::slice_from_raw_parts;

#[derive(Debug)]
pub struct IreeHalDriver {
    name: iree_string_view_t,
    pub(crate) driver_ptr: *mut iree_hal_driver_t,
}

impl IreeHalDriver {
    pub fn name(&self) -> String {
        iree_string_view_to_string(self.name)
    }

    pub fn query_available_devices(
        &self,
        allocator: &IreeAllocator,
    ) -> Result<Vec<IreeDeviceInfo>, IreeError> {
        let mut out_device_infos = std::ptr::null_mut();
        let mut out_device_info_count = 0usize;
        unsafe {
            let status = iree_hal_driver_query_available_devices(
                self.driver_ptr,
                allocator.allocator,
                &mut out_device_info_count,
                &mut out_device_infos,
            );
            if !IREE_CHECK_OK(status) {
                return Err(IreeError::from_status(IreeStatus { status }, allocator));
            }
        };
        let device_infos = slice_from_raw_parts(out_device_infos, out_device_info_count);
        let device_infos: Vec<IreeDeviceInfo> = unsafe { &*device_infos }
            .iter()
            .map(|&x| x.into())
            .collect();
        Ok(device_infos)
    }
    pub fn create_device_by_id(
        &self,
        device_id: iree_hal_device_id_t,
        _params: Vec<String>,
        allocator: &IreeAllocator,
    ) -> Result<IreeHalDevice, IreeError> {
        let mut out_device = std::ptr::null_mut();
        unsafe {
            let status = iree_hal_driver_create_device_by_id(
                self.driver_ptr,
                device_id,
                0,                    // params.len,
                std::ptr::null_mut(), // params,
                allocator.allocator,
                &mut out_device,
            );
            if !IREE_CHECK_OK(status) {
                return Err(IreeError::from_status(IreeStatus { status }, allocator));
            }
        };
        Ok(IreeHalDevice {
            device_ptr: out_device,
        })
    }
}

pub struct IreeDriverInfo {
    pub driver_name: iree_string_view_t,
    pub full_name: iree_string_view_t,
}

impl Display for IreeDriverInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "driver_name: {}, full_name: {}",
            iree_string_view_to_string(self.driver_name),
            iree_string_view_to_string(self.full_name)
        )
    }
}

impl From<iree_hal_driver_info_t> for IreeDriverInfo {
    fn from(driver_info: iree_hal_driver_info_t) -> Self {
        Self {
            driver_name: driver_info.driver_name,
            full_name: driver_info.full_name,
        }
    }
}

pub struct IreeHalDriverRegistry {
    pub(crate) driver_registry_ptr: *mut iree_hal_driver_registry_t,
}

impl IreeHalDriverRegistry {
    pub fn enumerate(&self, allocator: &IreeAllocator) -> Result<Vec<IreeDriverInfo>, IreeError> {
        let mut out_driver_infos = std::ptr::null_mut();
        let mut out_driver_info_count = 0usize;
        unsafe {
            let status = iree_hal_driver_registry_enumerate(
                self.driver_registry_ptr,
                allocator.allocator,
                &mut out_driver_info_count,
                &mut out_driver_infos,
            );
            if !IREE_CHECK_OK(status) {
                return Err(IreeError::from_status(IreeStatus { status }, allocator));
            }
        };
        let driver_infos = slice_from_raw_parts(out_driver_infos, out_driver_info_count);
        let driver_infos: Vec<IreeDriverInfo> = unsafe { &*driver_infos }
            .iter()
            .map(|&x| x.into())
            .collect();

        Ok(driver_infos)
    }

    pub fn try_create(
        &self,
        driver_name: iree_string_view_t,
        allocator: &IreeAllocator,
    ) -> Result<IreeHalDriver, IreeError> {
        let mut out_driver = std::ptr::null_mut();
        unsafe {
            iree_hal_driver_registry_try_create(
                self.driver_registry_ptr,
                driver_name,
                allocator.allocator,
                &mut out_driver,
            )
        };
        Ok(IreeHalDriver {
            name: driver_name,
            driver_ptr: out_driver,
        })
    }
}
