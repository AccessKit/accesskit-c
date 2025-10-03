// Copyright 2026 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use accesskit_android::*;
use std::os::raw::c_void;

use crate::{
    box_from_ptr, mut_from_ptr, ref_from_ptr, tree_update_factory, tree_update_factory_userdata,
    ActionHandlerCallback, ActivationHandlerCallback, BoxCastPtr, CastPtr, FfiActionHandler,
    FfiActivationHandler,
};

pub struct android_platform_action {
    _private: [u8; 0],
}

impl CastPtr for android_platform_action {
    type RustType = PlatformAction;
}

impl BoxCastPtr for android_platform_action {}

impl android_platform_action {
    #[no_mangle]
    pub extern "C" fn accesskit_android_platform_action_from_java(
        env: *mut jni::sys::JNIEnv,
        action: jni::sys::jint,
        arguments: jni::sys::jobject,
    ) -> *mut android_platform_action {
        let mut env = unsafe { jni::JNIEnv::from_raw(env).unwrap() };
        let arguments = unsafe { jni::objects::JObject::from_raw(arguments) };
        let platform_action = PlatformAction::from_java(&mut env, action, &arguments);
        BoxCastPtr::to_nullable_mut_ptr(platform_action)
    }

    #[no_mangle]
    pub extern "C" fn accesskit_android_platform_action_free(action: *mut android_platform_action) {
        drop(box_from_ptr(action));
    }
}

pub struct android_queued_events {
    _private: [u8; 0],
}

impl CastPtr for android_queued_events {
    type RustType = QueuedEvents;
}

impl BoxCastPtr for android_queued_events {}

impl android_queued_events {
    /// Memory is also freed when calling this function.
    #[no_mangle]
    pub extern "C" fn accesskit_android_queued_events_raise(
        events: *mut android_queued_events,
        env: *mut jni::sys::JNIEnv,
        host: jni::sys::jobject,
    ) {
        let events = box_from_ptr(events);
        let mut env = unsafe { jni::JNIEnv::from_raw(env).unwrap() };
        let host = unsafe { jni::objects::JObject::from_raw(host) };
        events.raise(&mut env, &host);
    }
}

pub struct android_adapter {
    _private: [u8; 0],
}

impl CastPtr for android_adapter {
    type RustType = Adapter;
}

impl BoxCastPtr for android_adapter {}

impl android_adapter {
    #[no_mangle]
    pub extern "C" fn accesskit_android_adapter_new() -> *mut android_adapter {
        let adapter = Adapter::default();
        BoxCastPtr::to_mut_ptr(adapter)
    }

    #[no_mangle]
    pub extern "C" fn accesskit_android_adapter_free(adapter: *mut android_adapter) {
        drop(box_from_ptr(adapter));
    }

    /// You must call `accesskit_android_queued_events_raise` on the returned pointer. It can be null if the adapter is not active.
    #[no_mangle]
    pub extern "C" fn accesskit_android_adapter_update_if_active(
        adapter: *mut android_adapter,
        update_factory: tree_update_factory,
        update_factory_userdata: *mut c_void,
    ) -> *mut android_queued_events {
        let update_factory = update_factory.unwrap();
        let update_factory_userdata = tree_update_factory_userdata(update_factory_userdata);
        let adapter = mut_from_ptr(adapter);
        let events =
            adapter.update_if_active(|| *box_from_ptr(update_factory(update_factory_userdata)));
        BoxCastPtr::to_nullable_mut_ptr(events)
    }

    #[no_mangle]
    pub extern "C" fn accesskit_android_adapter_create_accessibility_node_info(
        adapter: *mut android_adapter,
        activation_handler: ActivationHandlerCallback,
        activation_handler_userdata: *mut c_void,
        env: *mut jni::sys::JNIEnv,
        host: jni::sys::jobject,
        virtual_view_id: jni::sys::jint,
    ) -> jni::sys::jobject {
        let adapter = mut_from_ptr(adapter);
        let mut activation_handler =
            FfiActivationHandler::new(activation_handler, activation_handler_userdata);
        let mut env = unsafe { jni::JNIEnv::from_raw(env).unwrap() };
        let host = unsafe { jni::objects::JObject::from_raw(host) };
        adapter
            .create_accessibility_node_info(
                &mut activation_handler,
                &mut env,
                &host,
                virtual_view_id,
            )
            .into_raw()
    }

    #[no_mangle]
    pub extern "C" fn accesskit_android_adapter_find_focus(
        adapter: *mut android_adapter,
        activation_handler: ActivationHandlerCallback,
        activation_handler_userdata: *mut c_void,
        env: *mut jni::sys::JNIEnv,
        host: jni::sys::jobject,
        focus_type: jni::sys::jint,
    ) -> jni::sys::jobject {
        let adapter = mut_from_ptr(adapter);
        let mut activation_handler =
            FfiActivationHandler::new(activation_handler, activation_handler_userdata);
        let mut env = unsafe { jni::JNIEnv::from_raw(env).unwrap() };
        let host = unsafe { jni::objects::JObject::from_raw(host) };
        adapter
            .find_focus(&mut activation_handler, &mut env, &host, focus_type)
            .into_raw()
    }

    /// You must call `accesskit_android_queued_events_raise` on the returned pointer. It can be null if the adapter is not active.
    #[no_mangle]
    pub extern "C" fn accesskit_android_adapter_perform_action(
        adapter: *mut android_adapter,
        action_handler: ActionHandlerCallback,
        action_handler_userdata: *mut c_void,
        virtual_view_id: jni::sys::jint,
        action: *const android_platform_action,
    ) -> *mut android_queued_events {
        let adapter = mut_from_ptr(adapter);
        let mut action_handler = FfiActionHandler::new(action_handler, action_handler_userdata);
        let action = ref_from_ptr(action);
        let events = adapter.perform_action(&mut action_handler, virtual_view_id, action);
        BoxCastPtr::to_nullable_mut_ptr(events)
    }

    /// You must call `accesskit_android_queued_events_raise` on the returned pointer. It can be null if the adapter is not active.
    #[no_mangle]
    pub extern "C" fn accesskit_android_adapter_on_hover_event(
        adapter: *mut android_adapter,
        activation_handler: ActivationHandlerCallback,
        activation_handler_userdata: *mut c_void,
        action: jni::sys::jint,
        x: jni::sys::jfloat,
        y: jni::sys::jfloat,
    ) -> *mut android_queued_events {
        let adapter = mut_from_ptr(adapter);
        let mut activation_handler =
            FfiActivationHandler::new(activation_handler, activation_handler_userdata);
        let events = adapter.on_hover_event(&mut activation_handler, action, x, y);
        BoxCastPtr::to_nullable_mut_ptr(events)
    }
}

pub struct android_injecting_adapter {
    _private: [u8; 0],
}

impl CastPtr for android_injecting_adapter {
    type RustType = InjectingAdapter;
}

impl BoxCastPtr for android_injecting_adapter {}

impl android_injecting_adapter {
    #[no_mangle]
    pub extern "C" fn accesskit_android_injecting_adapter_new(
        env: *mut jni::sys::JNIEnv,
        host: jni::sys::jobject,
        activation_handler: ActivationHandlerCallback,
        activation_handler_userdata: *mut c_void,
        action_handler: ActionHandlerCallback,
        action_handler_userdata: *mut c_void,
    ) -> *mut android_injecting_adapter {
        let mut env = unsafe { jni::JNIEnv::from_raw(env).unwrap() };
        let host = unsafe { jni::objects::JObject::from_raw(host) };
        let activation_handler =
            FfiActivationHandler::new(activation_handler, activation_handler_userdata);
        let action_handler = FfiActionHandler::new(action_handler, action_handler_userdata);
        let adapter = InjectingAdapter::new(&mut env, &host, activation_handler, action_handler);
        BoxCastPtr::to_mut_ptr(adapter)
    }

    #[no_mangle]
    pub extern "C" fn accesskit_android_injecting_adapter_free(
        adapter: *mut android_injecting_adapter,
    ) {
        drop(box_from_ptr(adapter));
    }

    #[no_mangle]
    pub extern "C" fn accesskit_android_injecting_adapter_update_if_active(
        adapter: *mut android_injecting_adapter,
        update_factory: tree_update_factory,
        update_factory_userdata: *mut c_void,
    ) {
        let update_factory = update_factory.unwrap();
        let update_factory_userdata = tree_update_factory_userdata(update_factory_userdata);
        let adapter = mut_from_ptr(adapter);
        adapter.update_if_active(|| *box_from_ptr(update_factory(update_factory_userdata)));
    }
}
