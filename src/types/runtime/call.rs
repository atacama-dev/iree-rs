use std::fmt::Error;

use iree_sys::{
    helper::IREE_CHECK_OK,
    iree::runtime::api::{
        iree_hal_buffer_view_t, iree_hal_element_type_t, iree_runtime_call_deinitialize,
        iree_runtime_call_flags_t, iree_runtime_call_initialize_by_name,
        iree_runtime_call_inputs_push_back_buffer_view, iree_runtime_call_invoke,
        iree_runtime_call_outputs_pop_front_buffer_view, iree_runtime_call_t, iree_string_view_t,
        iree_vm_invoke, iree_vm_list_create, iree_vm_list_push_ref_retain, iree_vm_list_push_value,
        iree_vm_list_t, iree_vm_ref_t, iree_vm_ref_type_t, iree_vm_type_def_t, iree_vm_value_t,
        iree_vm_value_type_e_IREE_VM_VALUE_TYPE_NONE, IREE_VM_REF_TYPE_TAG_BITS,
    },
};

use crate::{
    err::IreeError,
    types::{allocator::IreeAllocator, hal_buffer::IreeHalBufferView, status::IreeStatus},
};

use super::session::IreeRuntimeSession;

pub struct VmList<T: Type> {
    pub(crate) vm_list_ptr: *mut iree_vm_list_t,
    _marker: core::marker::PhantomData<T>,
}

/// Trait for types that can be used as VM references.
pub trait ToRef: Sized {
    fn to_ref(&self) -> Result<Ref<Self>, Error>;
    fn to_ref_type() -> iree_vm_ref_type_t;
}

/// VM Ref type, used for passing reference to things like HAL buffers. Ref is a reference counted type.
pub struct Ref<T: ToRef> {
    pub(crate) ctx: iree_vm_ref_t,
    pub(crate) _marker: core::marker::PhantomData<T>,
}

impl<T: ToRef> Type for Ref<T> {
    fn to_raw() -> iree_vm_type_def_t {
        let mut out = iree_vm_type_def_t::default();
        out.set_value_type_bits(iree_vm_value_type_e_IREE_VM_VALUE_TYPE_NONE.0 as usize);
        out.set_ref_type_bits(T::to_ref_type() >> IREE_VM_REF_TYPE_TAG_BITS as usize);
        out
    }
}

pub trait Type {
    fn to_raw() -> iree_vm_type_def_t;
}

impl<T: Type> VmList<T> {
    pub fn create(initial_capacity: usize, allocator: IreeAllocator) -> Self {
        let mut vm_list_ptr: std::mem::MaybeUninit<*mut iree_vm_list_t> =
            std::mem::MaybeUninit::<*mut iree_vm_list_t>::uninit();
        unsafe {
            iree_vm_list_create(
                T::to_raw(),
                initial_capacity,
                allocator.allocator,
                vm_list_ptr.as_mut_ptr(),
            );
        };
        VmList::<T> {
            vm_list_ptr: unsafe { vm_list_ptr.assume_init() },
            _marker: core::marker::PhantomData,
        }
    }

    pub fn push_value(&mut self, value: *mut iree_vm_value_t) {
        unsafe {
            iree_vm_list_push_value(self.vm_list_ptr, value);
        }
    }

    pub fn push_ref_retain(&mut self, ref_value: *mut iree_vm_ref_t) {
        unsafe {
            iree_vm_list_push_ref_retain(self.vm_list_ptr, ref_value);
        }
    }

    pub fn push_buffer(&mut self, buffer_view: &IreeHalBufferView) {
        unsafe {
            self.push_ref_retain(buffer_view.buffer_view_ptr as *mut iree_vm_ref_t);
        }
    }
}

pub struct IreeRuntimeCall {
    pub(crate) call: iree_runtime_call_t,
}
impl IreeRuntimeCall {
    pub fn initialize_by_name(
        session: &IreeRuntimeSession,
        full_name: &String,
    ) -> Result<Self, IreeError> {
        let mut call = iree_runtime_call_t::default();

        unsafe {
            let status = iree_runtime_call_initialize_by_name(
                session.session_ptr,
                iree_string_view_t {
                    data: full_name.as_ptr() as *const i8,
                    size: full_name.len(),
                },
                &mut call,
            );
            if !IREE_CHECK_OK(status) {
                return Err(IreeError::from_status(
                    IreeStatus { status },
                    &IreeAllocator::system_allocator(),
                ));
            }
        }

        Ok(Self { call })
    }

    pub fn inputs_push_back_buffer_view(
        &mut self,
        buffer_view: &IreeHalBufferView,
    ) -> Result<(), IreeError> {
        unsafe {
            let status = iree_runtime_call_inputs_push_back_buffer_view(
                &mut self.call,
                buffer_view.buffer_view_ptr,
            );
            if !IREE_CHECK_OK(status) {
                return Err(IreeError::from_status(
                    IreeStatus { status },
                    &IreeAllocator::system_allocator(),
                ));
            }
            Ok(())
        }
    }

    pub fn outputs_pop_front_buffer_view(&mut self) -> Result<IreeHalBufferView, IreeError> {
        let mut ret = std::mem::MaybeUninit::<*mut iree_hal_buffer_view_t>::uninit();
        unsafe {
            let status =
                iree_runtime_call_outputs_pop_front_buffer_view(&mut self.call, ret.as_mut_ptr());

            if !IREE_CHECK_OK(status) {
                return Err(IreeError::from_status(
                    IreeStatus { status },
                    &IreeAllocator::system_allocator(),
                ));
            }

            Ok(IreeHalBufferView {
                buffer_view_ptr: ret.assume_init(),
            })
        }
    }

    pub fn invoke(&mut self, flags: iree_runtime_call_flags_t) -> Result<(), IreeError> {
        unsafe {
            let status = iree_runtime_call_invoke(&mut self.call, flags);
            if !IREE_CHECK_OK(status) {
                return Err(IreeError::from_status(
                    IreeStatus { status },
                    &IreeAllocator::system_allocator(),
                ));
            }
        }
        Ok(())
    }
}

impl Drop for IreeRuntimeCall {
    fn drop(&mut self) {
        unsafe {
            iree_runtime_call_deinitialize(&mut self.call);
        }
    }
}
