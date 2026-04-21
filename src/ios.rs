// Copyright 2026 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use accesskit_ios::{Adapter, CGPoint, QueuedEvents, SubclassingAdapter};
use std::ffi::{c_char, c_void};

use crate::{
    box_from_ptr, debug_repr_from_ptr, mut_from_ptr, tree_update_factory,
    tree_update_factory_userdata, ActionHandlerCallback, ActivationHandlerCallback, BoxCastPtr,
    CastPtr, DeactivationHandlerCallback, FfiActionHandler, FfiActivationHandler,
    FfiDeactivationHandler,
};

pub struct ios_queued_events {
    _private: [u8; 0],
}

impl CastPtr for ios_queued_events {
    type RustType = QueuedEvents;
}

impl BoxCastPtr for ios_queued_events {}

impl ios_queued_events {
    /// Memory is also freed when calling this function.
    #[no_mangle]
    pub extern "C" fn accesskit_ios_queued_events_raise(events: *mut ios_queued_events) {
        let events = box_from_ptr(events);
        events.raise();
    }
}

pub struct ios_adapter {
    _private: [u8; 0],
}

impl CastPtr for ios_adapter {
    type RustType = Adapter;
}

impl BoxCastPtr for ios_adapter {}

impl ios_adapter {
    /// This function must be called on the main thread.
    /// All handlers will always be called on the main thread.
    ///
    /// # Safety
    ///
    /// `view` must be a valid, unreleased pointer to a `UIView`.
    #[no_mangle]
    pub unsafe extern "C" fn accesskit_ios_adapter_new(
        view: *mut c_void,
        activation_handler: ActivationHandlerCallback,
        activation_handler_userdata: *mut c_void,
        action_handler: ActionHandlerCallback,
        action_handler_userdata: *mut c_void,
        deactivation_handler: DeactivationHandlerCallback,
        deactivation_handler_userdata: *mut c_void,
    ) -> *mut ios_adapter {
        let activation_handler =
            FfiActivationHandler::new(activation_handler, activation_handler_userdata);
        let action_handler = FfiActionHandler::new(action_handler, action_handler_userdata);
        let deactivation_handler =
            FfiDeactivationHandler::new(deactivation_handler, deactivation_handler_userdata);
        let adapter = Adapter::new(
            view,
            activation_handler,
            action_handler,
            deactivation_handler,
        );
        BoxCastPtr::to_mut_ptr(adapter)
    }

    #[no_mangle]
    pub extern "C" fn accesskit_ios_adapter_free(adapter: *mut ios_adapter) {
        drop(box_from_ptr(adapter));
    }

    /// You must call `accesskit_ios_queued_events_raise` on the returned pointer. It can be null if the adapter is not active.
    #[no_mangle]
    pub extern "C" fn accesskit_ios_adapter_update_if_active(
        adapter: *mut ios_adapter,
        update_factory: tree_update_factory,
        update_factory_userdata: *mut c_void,
    ) -> *mut ios_queued_events {
        let update_factory = update_factory.unwrap();
        let update_factory_userdata = tree_update_factory_userdata(update_factory_userdata);
        let adapter = mut_from_ptr(adapter);
        let events =
            adapter.update_if_active(|| *box_from_ptr(update_factory(update_factory_userdata)));
        BoxCastPtr::to_nullable_mut_ptr(events)
    }

    /// Call this when the host view has just appeared on screen. If an
    /// assistive technology is running, this proactively builds the
    /// accessibility tree.
    ///
    /// You must call `accesskit_ios_queued_events_raise` on the returned pointer. It can be null if the adapter is not active.
    #[no_mangle]
    pub extern "C" fn accesskit_ios_adapter_view_did_appear(
        adapter: *mut ios_adapter,
    ) -> *mut ios_queued_events {
        let adapter = mut_from_ptr(adapter);
        let events = adapter.view_did_appear();
        BoxCastPtr::to_nullable_mut_ptr(events)
    }

    /// Returns whether the view itself is an accessibility element.
    /// This corresponds to `isAccessibilityElement`.
    #[no_mangle]
    pub extern "C" fn accesskit_ios_adapter_is_accessibility_element(
        adapter: *mut ios_adapter,
    ) -> bool {
        let adapter = mut_from_ptr(adapter);
        adapter.is_accessibility_element()
    }

    /// Returns a pointer to an `NSArray` of accessibility elements
    /// contained in the view. Ownership of the pointer is not transferred.
    /// This corresponds to `accessibilityElements`.
    #[no_mangle]
    pub extern "C" fn accesskit_ios_adapter_accessibility_elements(
        adapter: *mut ios_adapter,
    ) -> *mut c_void {
        let adapter = mut_from_ptr(adapter);
        adapter.accessibility_elements() as *mut _
    }

    /// Returns a pointer to the accessibility element at the specified point,
    /// or null if none. Ownership of the pointer is not transferred.
    /// This corresponds to `accessibilityHitTest:`.
    #[no_mangle]
    pub extern "C" fn accesskit_ios_adapter_hit_test(
        adapter: *mut ios_adapter,
        x: f64,
        y: f64,
    ) -> *mut c_void {
        let adapter = mut_from_ptr(adapter);
        adapter.hit_test(CGPoint::new(x, y)) as *mut _
    }

    /// Caller must call `accesskit_string_free` with the return value.
    #[no_mangle]
    pub extern "C" fn accesskit_ios_adapter_debug(adapter: *const ios_adapter) -> *mut c_char {
        debug_repr_from_ptr(adapter)
    }
}

pub struct ios_subclassing_adapter {
    _private: [u8; 0],
}

impl CastPtr for ios_subclassing_adapter {
    type RustType = SubclassingAdapter;
}

impl BoxCastPtr for ios_subclassing_adapter {}

impl ios_subclassing_adapter {
    /// All handlers will always be called on the main thread.
    ///
    /// # Safety
    ///
    /// `view` must be a valid, unreleased pointer to a `UIView`.
    #[no_mangle]
    pub unsafe extern "C" fn accesskit_ios_subclassing_adapter_new(
        view: *mut c_void,
        activation_handler: ActivationHandlerCallback,
        activation_handler_userdata: *mut c_void,
        action_handler: ActionHandlerCallback,
        action_handler_userdata: *mut c_void,
        deactivation_handler: DeactivationHandlerCallback,
        deactivation_handler_userdata: *mut c_void,
    ) -> *mut ios_subclassing_adapter {
        let activation_handler =
            FfiActivationHandler::new(activation_handler, activation_handler_userdata);
        let action_handler = FfiActionHandler::new(action_handler, action_handler_userdata);
        let deactivation_handler =
            FfiDeactivationHandler::new(deactivation_handler, deactivation_handler_userdata);
        let adapter = SubclassingAdapter::new(
            view,
            activation_handler,
            action_handler,
            deactivation_handler,
        );
        BoxCastPtr::to_mut_ptr(adapter)
    }

    /// All handlers will always be called on the main thread.
    ///
    /// # Safety
    ///
    /// `window` must be a valid, unreleased pointer to a `UIWindow`.
    ///
    /// # Panics
    ///
    /// This function panics if the specified window doesn't currently have
    /// a root view controller with a view.
    #[no_mangle]
    pub unsafe extern "C" fn accesskit_ios_subclassing_adapter_for_window(
        window: *mut c_void,
        activation_handler: ActivationHandlerCallback,
        activation_handler_userdata: *mut c_void,
        action_handler: ActionHandlerCallback,
        action_handler_userdata: *mut c_void,
        deactivation_handler: DeactivationHandlerCallback,
        deactivation_handler_userdata: *mut c_void,
    ) -> *mut ios_subclassing_adapter {
        let activation_handler =
            FfiActivationHandler::new(activation_handler, activation_handler_userdata);
        let action_handler = FfiActionHandler::new(action_handler, action_handler_userdata);
        let deactivation_handler =
            FfiDeactivationHandler::new(deactivation_handler, deactivation_handler_userdata);
        let adapter = SubclassingAdapter::for_window(
            window,
            activation_handler,
            action_handler,
            deactivation_handler,
        );
        BoxCastPtr::to_mut_ptr(adapter)
    }

    #[no_mangle]
    pub extern "C" fn accesskit_ios_subclassing_adapter_free(
        adapter: *mut ios_subclassing_adapter,
    ) {
        drop(box_from_ptr(adapter));
    }

    /// You must call `accesskit_ios_queued_events_raise` on the returned pointer. It can be null if the adapter is not active.
    #[no_mangle]
    pub extern "C" fn accesskit_ios_subclassing_adapter_update_if_active(
        adapter: *mut ios_subclassing_adapter,
        update_factory: tree_update_factory,
        update_factory_userdata: *mut c_void,
    ) -> *mut ios_queued_events {
        let update_factory = update_factory.unwrap();
        let update_factory_userdata = tree_update_factory_userdata(update_factory_userdata);
        let adapter = mut_from_ptr(adapter);
        let events =
            adapter.update_if_active(|| *box_from_ptr(update_factory(update_factory_userdata)));
        BoxCastPtr::to_nullable_mut_ptr(events)
    }
}
