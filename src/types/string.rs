use std::{ffi::CStr, os::raw::c_char};

use iree_sys::iree::runtime::api::{
    iree_string_builder_append_cstring, iree_string_builder_buffer,
    iree_string_builder_deinitialize, iree_string_builder_initialize,
    iree_string_builder_initialize_with_storage, iree_string_builder_size, iree_string_builder_t,
    iree_string_view_t, iree_string_view_to_cstring,
};

use super::allocator::IreeAllocator;

#[derive(Debug)]
pub struct IreeStringView {
    pub(crate) iree_string_view_ptr: *mut iree_string_view_t,
}

impl IreeStringView {
    pub fn new(iree_string_view_ptr: iree_string_view_t) -> Self {
        Self {
            iree_string_view_ptr: Box::into_raw(Box::new(iree_string_view_ptr)),
        }
    }

    pub fn to_string(&self) -> String {
        let mut cstr: *mut c_char = [0; 1024].as_mut_ptr() as *mut c_char;
        unsafe {
            iree_string_view_to_cstring(*self.iree_string_view_ptr, cstr, 1024);

            CStr::from_ptr(cstr).to_string_lossy().into_owned()
        }
    }

    pub fn from_string(string: String) -> Self {
        let cstr = std::ffi::CString::new(string).unwrap();
        let iree_string_view_ptr = Box::into_raw(Box::new(iree_string_view_t {
            data: cstr.as_ptr(),
            size: cstr.as_bytes().len(),
        }));
        Self {
            iree_string_view_ptr,
        }
    }
}

pub struct IreeStringBuilder {
    pub(crate) iree_string_builder_ptr: *mut iree_string_builder_t,
}

impl IreeStringBuilder {
    pub fn initialize(allocator: IreeAllocator) -> Self {
        let iree_string_builder_ptr = unsafe {
            let mut iree_string_builder_ptr = std::ptr::null_mut();
            iree_string_builder_initialize(allocator.allocator, iree_string_builder_ptr);
            iree_string_builder_ptr
        };
        Self {
            iree_string_builder_ptr,
        }
    }

    pub fn buffer(&self) -> String {
        unsafe {
            let cstr = iree_string_builder_buffer(self.iree_string_builder_ptr);
            CStr::from_ptr(cstr).to_string_lossy().into_owned()
        }
    }
}
