use iree_sys::iree::runtime::api::iree_string_view_t;
use std::ffi::CStr;

pub fn iree_string_view_to_string(iree_string_view: iree_string_view_t) -> String {
    unsafe {
        CStr::from_ptr(iree_string_view.data)
            .to_str()
            .unwrap()
            .to_string()
            .split_at(iree_string_view.size)
            .0
            .to_string()
    }
}
