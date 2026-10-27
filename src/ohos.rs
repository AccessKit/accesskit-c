// Copyright 2026 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use accesskit_ohos::{Adapter, Provider};
use std::{
    ffi::{c_char, c_void},
    ptr,
};

use crate::{
    ActionHandlerCallback, ActivationHandlerCallback, BoxCastPtr, CastPtr, FfiActionHandler,
    FfiActivationHandler, box_from_ptr, debug_repr_from_ptr, mut_from_ptr, tree_update_factory,
    tree_update_factory_userdata,
};

pub struct ohos_adapter {
    _private: [u8; 0],
}

impl CastPtr for ohos_adapter {
    type RustType = Adapter<'static>;
}

impl BoxCastPtr for ohos_adapter {}

impl ohos_adapter {
    /// Wrap an `ArkUI_AccessibilityProvider*` and register the single-instance
    /// callback table.
    ///
    /// Returns null if `provider` is null or if a single-instance callback table
    /// is already registered in this process.
    ///
    /// # Safety
    ///
    /// `provider` must be a valid `ArkUI_AccessibilityProvider*` pointer that
    /// remains alive until `accesskit_ohos_adapter_free` is called.
    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn accesskit_ohos_adapter_new(
        provider: *mut c_void,
        activation_handler: ActivationHandlerCallback,
        activation_handler_userdata: *mut c_void,
        action_handler: ActionHandlerCallback,
        action_handler_userdata: *mut c_void,
    ) -> *mut ohos_adapter {
        let Ok(provider) = (unsafe { Provider::from_raw(provider.cast()) }) else {
            return ptr::null_mut();
        };
        let activation_handler =
            FfiActivationHandler::new(activation_handler, activation_handler_userdata);
        let action_handler = FfiActionHandler::new(action_handler, action_handler_userdata);
        let adapter = Adapter::new(provider, activation_handler, action_handler).ok();
        BoxCastPtr::to_nullable_mut_ptr(adapter)
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn accesskit_ohos_adapter_free(adapter: *mut ohos_adapter) {
        drop(box_from_ptr(adapter));
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn accesskit_ohos_adapter_update_if_active(
        adapter: *mut ohos_adapter,
        update_factory: tree_update_factory,
        update_factory_userdata: *mut c_void,
    ) {
        let update_factory = update_factory.unwrap();
        let update_factory_userdata = tree_update_factory_userdata(update_factory_userdata);
        let adapter = mut_from_ptr(adapter);
        let _ = adapter.update_if_active(|| *box_from_ptr(update_factory(update_factory_userdata)));
    }

    /// Update the tree state based on whether the host XComponent or window is focused.
    #[unsafe(no_mangle)]
    pub extern "C" fn accesskit_ohos_adapter_update_window_focus_state(
        adapter: *mut ohos_adapter,
        is_focused: bool,
    ) {
        let adapter = mut_from_ptr(adapter);
        let _ = adapter.set_host_focus_state(is_focused);
    }

    /// Apply an additional provider-local pixel offset to AccessKit bounds.
    #[unsafe(no_mangle)]
    pub extern "C" fn accesskit_ohos_adapter_set_screen_offset(
        adapter: *mut ohos_adapter,
        x: i32,
        y: i32,
    ) {
        let adapter = mut_from_ptr(adapter);
        let _ = adapter.set_screen_offset(x, y);
    }

    /// Drop the retained tree so it will be requested again on the next native
    /// accessibility query.
    #[unsafe(no_mangle)]
    pub extern "C" fn accesskit_ohos_adapter_deactivate(adapter: *mut ohos_adapter) {
        let adapter = mut_from_ptr(adapter);
        let _ = adapter.deactivate();
    }

    /// Caller must call `accesskit_string_free` with the return value.
    #[unsafe(no_mangle)]
    pub extern "C" fn accesskit_ohos_adapter_debug(adapter: *const ohos_adapter) -> *mut c_char {
        debug_repr_from_ptr(adapter)
    }
}
