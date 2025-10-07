// Copyright 2023 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use accesskit::*;
use std::{
    ffi::{CStr, CString},
    mem,
    os::raw::{c_char, c_void},
    ptr, slice,
};

use crate::{
    box_from_ptr, debug_repr_from_ptr, mut_from_ptr, opt_struct, ref_from_ptr, BoxCastPtr, CastPtr,
};

pub struct node {
    _private: [u8; 0],
}

impl CastPtr for node {
    type RustType = Node;
}

impl BoxCastPtr for node {}

macro_rules! clearer {
    ($c_clearer:ident, $clearer:ident) => {
        impl node {
            #[no_mangle]
            pub extern "C" fn $c_clearer(node: *mut node) {
                let node = mut_from_ptr(node);
                node.$clearer()
            }
        }
    };
}

macro_rules! flag_methods {
    ($(($c_getter:ident, $getter:ident, $c_setter:ident, $setter:ident, $c_clearer:ident, $clearer:ident)),+) => {
        $(impl node {
            #[no_mangle]
            pub extern "C" fn $c_getter(node: *const node) -> bool {
                let node = ref_from_ptr(node);
                node.$getter()
            }
            #[no_mangle]
            pub extern "C" fn $c_setter(node: *mut node) {
                let node = mut_from_ptr(node);
                node.$setter()
            }
        }
        clearer! { $c_clearer, $clearer })*
    }
}

macro_rules! array_setter {
    ($c_setter:ident, $setter:ident, $ffi_type:ty, $rust_type:ty) => {
        impl node {
            /// Caller is responsible for freeing `values`.
            #[no_mangle]
            pub extern "C" fn $c_setter(node: *mut node, length: usize, values: *const $ffi_type) {
                let node = mut_from_ptr(node);
                let values = unsafe {
                    slice::from_raw_parts(values, length)
                        .iter()
                        .cloned()
                        .map(From::from)
                        .collect::<Vec<$rust_type>>()
                };
                node.$setter(values);
            }
        }
    };
}

macro_rules! property_getters {
    ($c_getter:ident, $getter:ident, *const $getter_result:tt) => {
        impl node {
            #[no_mangle]
            pub extern "C" fn $c_getter(node: *const node) -> *const $getter_result {
                let node = ref_from_ptr(node);
                match node.$getter() {
                    Some(value) => value as *const _,
                    None => ptr::null(),
                }
            }
        }
    };
    ($c_getter:ident, $getter:ident, *mut $getter_result:tt) => {
        impl node {
            /// Caller is responsible for freeing the returned value.
            #[no_mangle]
            pub extern "C" fn $c_getter(node: *const node) -> *const $getter_result {
                let node = ref_from_ptr(node);
                BoxCastPtr::to_mut_ptr(node.$getter().into())
            }
        }
    };
    ($c_getter:ident, $getter:ident, $getter_result:tt) => {
        impl node {
            #[no_mangle]
            pub extern "C" fn $c_getter(node: *const node) -> $getter_result {
                let node = ref_from_ptr(node);
                node.$getter().into()
            }
        }
    };
}

macro_rules! simple_property_methods {
    ($c_getter:ident, $getter:ident, $getter_result:tt, $c_setter:ident, $setter:ident, $setter_param:tt, $c_clearer:ident, $clearer:ident) => {
        property_getters! { $c_getter, $getter, $getter_result }
        impl node {
            #[no_mangle]
            pub extern "C" fn $c_setter(node: *mut node, value: $setter_param) {
                let node = mut_from_ptr(node);
                node.$setter(value.into());
            }
        }
        clearer! { $c_clearer, $clearer }
    };
    ($c_getter:ident, $getter:ident, *const $getter_result:tt, $c_setter:ident, $setter:ident, $setter_param:tt, $c_clearer:ident, $clearer:ident) => {
        property_getters! { $c_getter, $getter, *const $getter_result }
        impl node {
            #[no_mangle]
            pub extern "C" fn $c_setter(node: *mut node, value: $setter_param) {
                let node = mut_from_ptr(node);
                node.$setter(Box::new(value));
            }
        }
        clearer! { $c_clearer, $clearer }
    };
    ($c_getter:ident, $getter:ident, $getter_result:tt, $c_setter:ident, $setter:ident, *const $setter_param:tt, $c_clearer:ident, $clearer:ident) => {
        property_getters! { $c_getter, $getter, $getter_result }
        array_setter! { $c_setter, $setter, $setter_param, $setter_param }
        clearer! { $c_clearer, $clearer }
    };
}

macro_rules! slice_struct {
    ($struct_name:ident, $rust_type:ty, $ffi_type:ty) => {
        #[repr(C)]
        pub struct $struct_name {
            pub length: usize,
            pub values: *const $ffi_type,
        }
        impl From<&[$rust_type]> for $struct_name {
            fn from(values: &[$rust_type]) -> Self {
                Self {
                    length: values.len(),
                    values: values.as_ptr() as *const _,
                }
            }
        }
        impl From<$struct_name> for Vec<$rust_type> {
            fn from(values: $struct_name) -> Self {
                unsafe {
                    slice::from_raw_parts(values.values as *mut $rust_type, values.length).to_vec()
                }
            }
        }
    };
}

macro_rules! array_struct {
    ($struct_name:ident, $rust_type:ty, $ffi_type:ty, $c_free_fn:ident) => {
        #[repr(C)]
        pub struct $struct_name {
            pub length: usize,
            pub values: *mut $ffi_type,
        }
        impl CastPtr for $struct_name {
            type RustType = $struct_name;
        }
        impl BoxCastPtr for $struct_name {}
        impl $struct_name {
            #[no_mangle]
            pub extern "C" fn $c_free_fn(value: *mut $struct_name) {
                let array = box_from_ptr(value);
                unsafe { Vec::from_raw_parts(array.values, array.length, array.length) };
                drop(array);
            }
        }
        impl From<&[$rust_type]> for $struct_name {
            fn from(values: &[$rust_type]) -> Self {
                let length = values.len();
                let mut ffi_values = values.iter().map(From::from).collect::<Vec<$ffi_type>>();
                let array = Self {
                    length,
                    values: ffi_values.as_mut_ptr(),
                };
                mem::forget(ffi_values);
                array
            }
        }
    };
}

macro_rules! vec_property_methods {
    ($(($item_type:ty, $c_getter:ident, $getter:ident, *mut $getter_result:ty, $c_setter:ident, $setter:ident, $setter_param:ty, $c_pusher:ident, $pusher:ident, $c_clearer:ident, $clearer:ident)),+) => {
        $(property_getters! { $c_getter, $getter, *mut $getter_result }
        array_setter! { $c_setter, $setter, $setter_param, $item_type }
        impl node {
            #[no_mangle]
            pub extern "C" fn $c_pusher(node: *mut node, item: $setter_param) {
                let node = mut_from_ptr(node);
                node.$pusher(item.into());
            }
        }
        clearer! { $c_clearer, $clearer })*
    };
    ($(($item_type:ty, $c_getter:ident, $getter:ident, $getter_result:ty, $c_setter:ident, $setter:ident, $setter_param:ty, $c_pusher:ident, $pusher:ident, $c_clearer:ident, $clearer:ident)),+) => {
        $(property_getters! { $c_getter, $getter, $getter_result }
        array_setter! { $c_setter, $setter, $setter_param, $item_type }
        impl node {
            #[no_mangle]
            pub extern "C" fn $c_pusher(node: *mut node, item: $setter_param) {
                let node = mut_from_ptr(node);
                node.$pusher(item.into());
            }
        }
        clearer! { $c_clearer, $clearer })*
    }
}

pub type node_id = u64;

slice_struct! { node_ids, NodeId, node_id }

macro_rules! node_id_vec_property_methods {
    ($(($c_getter:ident, $getter:ident, $c_setter:ident, $setter:ident, $c_pusher:ident, $pusher:ident, $c_clearer:ident, $clearer:ident)),+) => {
        $(vec_property_methods! {
            (NodeId, $c_getter, $getter, node_ids, $c_setter, $setter, node_id, $c_pusher, $pusher, $c_clearer, $clearer)
        })*
    }
}

macro_rules! node_id_property_methods {
    ($(($c_getter:ident, $getter:ident, $c_setter:ident, $setter:ident, $c_clearer:ident, $clearer:ident)),+) => {
        opt_struct! { opt_node_id, node_id }
        $(simple_property_methods! {
            $c_getter, $getter, opt_node_id, $c_setter, $setter, node_id, $c_clearer, $clearer
        })*
    }
}

macro_rules! string_property_methods {
    ($(($c_getter:ident, $getter:ident, $c_setter:ident, $c_setter_with_length:ident, $setter:ident, $c_clearer:ident, $clearer:ident)),+) => {
        $(impl node {
            /// Caller must call `accesskit_string_free` with the return value.
            #[no_mangle]
            pub extern "C" fn $c_getter(node: *const node) -> *mut c_char {
                let node = ref_from_ptr(node);
                match node.$getter() {
                    Some(value) => CString::new(value).unwrap().into_raw(),
                    None => ptr::null_mut()
                }
            }
            /// Caller is responsible for freeing the memory pointed by `value`.
            #[no_mangle]
            pub extern "C" fn $c_setter(node: *mut node, value: *const c_char) {
                let node = mut_from_ptr(node);
                let value = unsafe { CStr::from_ptr(value) };
                node.$setter(value.to_string_lossy());
            }
            /// Caller is responsible for freeing the memory pointed by `value`.
            #[no_mangle]
            pub extern "C" fn $c_setter_with_length(node: *mut node, length: usize, value: *const c_char) {
                let node = mut_from_ptr(node);
                let value = unsafe { slice::from_raw_parts(value as *const u8, length) };
                node.$setter(String::from_utf8_lossy(value));
            }
        }
        clearer! { $c_clearer, $clearer })*
    }
}

macro_rules! f64_property_methods {
    ($(($c_getter:ident, $getter:ident, $c_setter:ident, $setter:ident, $c_clearer:ident, $clearer:ident)),+) => {
        opt_struct! { opt_double, f64 }
        $(simple_property_methods! {
            $c_getter, $getter, opt_double, $c_setter, $setter, f64, $c_clearer, $clearer
        })*
    }
}

macro_rules! usize_property_methods {
    ($(($c_getter:ident, $getter:ident, $c_setter:ident, $setter:ident, $c_clearer:ident, $clearer:ident)),+) => {
        opt_struct! { opt_index, usize }
        $(simple_property_methods! {
            $c_getter, $getter, opt_index, $c_setter, $setter, usize, $c_clearer, $clearer
        })*
    }
}

macro_rules! color_property_methods {
    ($(($c_getter:ident, $getter:ident, $c_setter:ident, $setter:ident, $c_clearer:ident, $clearer:ident)),+) => {
        opt_struct! { opt_color, u32 }
        $(simple_property_methods! {
            $c_getter, $getter, opt_color, $c_setter, $setter, u32, $c_clearer, $clearer
        })*
    }
}

macro_rules! text_decoration_property_methods {
    ($(($c_getter:ident, $getter:ident, $c_setter:ident, $setter:ident, $c_clearer:ident, $clearer:ident)),+) => {
        opt_struct! { opt_text_decoration, TextDecoration }
        $(simple_property_methods! {
            $c_getter, $getter, opt_text_decoration, $c_setter, $setter, TextDecoration, $c_clearer, $clearer
        })*
    }
}

macro_rules! opt_slice_struct {
    ($struct_name:ident, $rust_type:ty, $ffi_type:ty) => {
        #[repr(C)]
        pub struct $struct_name {
            pub has_value: bool,
            pub length: usize,
            pub values: *const $ffi_type,
        }
        impl From<Option<&[$rust_type]>> for $struct_name {
            fn from(value: Option<&[$rust_type]>) -> $struct_name {
                match value {
                    Some(value) => $struct_name {
                        has_value: true,
                        length: value.len(),
                        values: value.as_ptr() as *const $ffi_type,
                    },
                    None => $struct_name::default(),
                }
            }
        }
        impl Default for $struct_name {
            fn default() -> $struct_name {
                $struct_name {
                    has_value: false,
                    length: 0,
                    values: ptr::null(),
                }
            }
        }
    };
}

macro_rules! length_slice_property_methods {
    ($(($c_getter:ident, $getter:ident, $c_setter:ident, $setter:ident, $c_clearer:ident, $clearer:ident)),+) => {
        slice_struct! { lengths, u8, u8 }
        $(simple_property_methods! {
            $c_getter, $getter, lengths, $c_setter, $setter, *const u8, $c_clearer, $clearer
        })*
    }
}

macro_rules! coord_slice_property_methods {
    ($(($c_getter:ident, $getter:ident, $c_setter:ident, $setter:ident, $c_clearer:ident, $clearer:ident)),+) => {
        opt_slice_struct! { opt_coords, f32, f32 }
        $(simple_property_methods! {
            $c_getter, $getter, opt_coords, $c_setter, $setter, *const f32, $c_clearer, $clearer
        })*
    }
}

macro_rules! bool_property_methods {
    ($(($c_getter:ident, $getter:ident, $c_setter:ident, $setter:ident, $c_clearer:ident, $clearer:ident)),+) => {
        opt_struct! { opt_bool, bool }
        $(simple_property_methods! {
            $c_getter, $getter, opt_bool, $c_setter, $setter, bool, $c_clearer, $clearer
        })*
    }
}

macro_rules! unique_enum_property_methods {
    ($(($opt_struct_name:ident, $prop_type:ty, $c_getter:ident, $getter:ident, $c_setter:ident, $setter:ident, $c_clearer:ident, $clearer:ident)),+) => {
        $(opt_struct! { $opt_struct_name, $prop_type }
        simple_property_methods! {
            $c_getter, $getter, $opt_struct_name, $c_setter, $setter, $prop_type, $c_clearer, $clearer
        })*
    }
}

property_getters! { accesskit_node_role, role, Role }
impl node {
    #[no_mangle]
    pub extern "C" fn accesskit_node_set_role(node: *mut node, value: Role) {
        let node = mut_from_ptr(node);
        node.set_role(value);
    }
}

impl node {
    #[no_mangle]
    pub extern "C" fn accesskit_node_supports_action(node: *const node, action: Action) -> bool {
        let node = ref_from_ptr(node);
        node.supports_action(action)
    }

    #[no_mangle]
    pub extern "C" fn accesskit_node_add_action(node: *mut node, action: Action) {
        let node = mut_from_ptr(node);
        node.add_action(action);
    }

    #[no_mangle]
    pub extern "C" fn accesskit_node_remove_action(node: *mut node, action: Action) {
        let node = mut_from_ptr(node);
        node.remove_action(action);
    }

    #[no_mangle]
    pub extern "C" fn accesskit_node_clear_actions(node: *mut node) {
        let node = mut_from_ptr(node);
        node.clear_actions();
    }

    /// Return whether the specified action is in the set supported on this node's
    /// direct children in the filtered tree.
    #[no_mangle]
    pub extern "C" fn accesskit_node_child_supports_action(
        node: *const node,
        action: Action,
    ) -> bool {
        let node = ref_from_ptr(node);
        node.child_supports_action(action)
    }

    /// Add the specified action to the set supported on this node's direct
    /// children in the filtered tree.
    #[no_mangle]
    pub extern "C" fn accesskit_node_add_child_action(node: *mut node, action: Action) {
        let node = mut_from_ptr(node);
        node.add_child_action(action);
    }

    /// Remove the specified action from the set supported on this node's direct
    /// children in the filtered tree.
    #[no_mangle]
    pub extern "C" fn accesskit_node_remove_child_action(node: *mut node, action: Action) {
        let node = mut_from_ptr(node);
        node.remove_child_action(action);
    }

    /// Clear the set of actions supported on this node's direct children in the
    /// filtered tree.
    #[no_mangle]
    pub extern "C" fn accesskit_node_clear_child_actions(node: *mut node) {
        let node = mut_from_ptr(node);
        node.clear_child_actions();
    }
}

flag_methods! {
    (accesskit_node_is_hidden, is_hidden, accesskit_node_set_hidden, set_hidden, accesskit_node_clear_hidden, clear_hidden),
    (accesskit_node_is_multiselectable, is_multiselectable, accesskit_node_set_multiselectable, set_multiselectable, accesskit_node_clear_multiselectable, clear_multiselectable),
    (accesskit_node_is_required, is_required, accesskit_node_set_required, set_required, accesskit_node_clear_required, clear_required),
    (accesskit_node_is_visited, is_visited, accesskit_node_set_visited, set_visited, accesskit_node_clear_visited, clear_visited),
    (accesskit_node_is_busy, is_busy, accesskit_node_set_busy, set_busy, accesskit_node_clear_busy, clear_busy),
    (accesskit_node_is_live_atomic, is_live_atomic, accesskit_node_set_live_atomic, set_live_atomic, accesskit_node_clear_live_atomic, clear_live_atomic),
    (accesskit_node_is_modal, is_modal, accesskit_node_set_modal, set_modal, accesskit_node_clear_modal, clear_modal),
    (accesskit_node_is_touch_transparent, is_touch_transparent, accesskit_node_set_touch_transparent, set_touch_transparent, accesskit_node_clear_touch_transparent, clear_touch_transparent),
    (accesskit_node_is_read_only, is_read_only, accesskit_node_set_read_only, set_read_only, accesskit_node_clear_read_only, clear_read_only),
    (accesskit_node_is_disabled, is_disabled, accesskit_node_set_disabled, set_disabled, accesskit_node_clear_disabled, clear_disabled),
    (accesskit_node_is_bold, is_bold, accesskit_node_set_bold, set_bold, accesskit_node_clear_bold, clear_bold),
    (accesskit_node_is_italic, is_italic, accesskit_node_set_italic, set_italic, accesskit_node_clear_italic, clear_italic),
    (accesskit_node_clips_children, clips_children, accesskit_node_set_clips_children, set_clips_children, accesskit_node_clear_clips_children, clear_clips_children),
    (accesskit_node_is_line_breaking_object, is_line_breaking_object, accesskit_node_set_is_line_breaking_object, set_is_line_breaking_object, accesskit_node_clear_is_line_breaking_object, clear_is_line_breaking_object),
    (accesskit_node_is_page_breaking_object, is_page_breaking_object, accesskit_node_set_is_page_breaking_object, set_is_page_breaking_object, accesskit_node_clear_is_page_breaking_object, clear_is_page_breaking_object),
    (accesskit_node_is_spelling_error, is_spelling_error, accesskit_node_set_is_spelling_error, set_is_spelling_error, accesskit_node_clear_is_spelling_error, clear_is_spelling_error),
    (accesskit_node_is_grammar_error, is_grammar_error, accesskit_node_set_is_grammar_error, set_is_grammar_error, accesskit_node_clear_is_grammar_error, clear_is_grammar_error),
    (accesskit_node_is_search_match, is_search_match, accesskit_node_set_is_search_match, set_is_search_match, accesskit_node_clear_is_search_match, clear_is_search_match),
    (accesskit_node_is_suggestion, is_suggestion, accesskit_node_set_is_suggestion, set_is_suggestion, accesskit_node_clear_is_suggestion, clear_is_suggestion)
}

node_id_vec_property_methods! {
    (accesskit_node_children, children, accesskit_node_set_children, set_children, accesskit_node_push_child, push_child, accesskit_node_clear_children, clear_children),
    (accesskit_node_controls, controls, accesskit_node_set_controls, set_controls, accesskit_node_push_controlled, push_controlled, accesskit_node_clear_controls, clear_controls),
    (accesskit_node_details, details, accesskit_node_set_details, set_details, accesskit_node_push_detail, push_detail, accesskit_node_clear_details, clear_details),
    (accesskit_node_described_by, described_by, accesskit_node_set_described_by, set_described_by, accesskit_node_push_described_by, push_described_by, accesskit_node_clear_described_by, clear_described_by),
    (accesskit_node_flow_to, flow_to, accesskit_node_set_flow_to, set_flow_to, accesskit_node_push_flow_to, push_flow_to, accesskit_node_clear_flow_to, clear_flow_to),
    (accesskit_node_labelled_by, labelled_by, accesskit_node_set_labelled_by, set_labelled_by, accesskit_node_push_labelled_by, push_labelled_by, accesskit_node_clear_labelled_by, clear_labelled_by),
    (accesskit_node_owns, owns, accesskit_node_set_owns, set_owns, accesskit_node_push_owned, push_owned, accesskit_node_clear_owns, clear_owns),
    (accesskit_node_radio_group, radio_group, accesskit_node_set_radio_group, set_radio_group, accesskit_node_push_to_radio_group, push_to_radio_group, accesskit_node_clear_radio_group, clear_radio_group)
}

node_id_property_methods! {
    (accesskit_node_active_descendant, active_descendant, accesskit_node_set_active_descendant, set_active_descendant, accesskit_node_clear_active_descendant, clear_active_descendant),
    (accesskit_node_error_message, error_message, accesskit_node_set_error_message, set_error_message, accesskit_node_clear_error_message, clear_error_message),
    (accesskit_node_in_page_link_target, in_page_link_target, accesskit_node_set_in_page_link_target, set_in_page_link_target, accesskit_node_clear_in_page_link_target, clear_in_page_link_target),
    (accesskit_node_member_of, member_of, accesskit_node_set_member_of, set_member_of, accesskit_node_clear_member_of, clear_member_of),
    (accesskit_node_next_on_line, next_on_line, accesskit_node_set_next_on_line, set_next_on_line, accesskit_node_clear_next_on_line, clear_next_on_line),
    (accesskit_node_previous_on_line, previous_on_line, accesskit_node_set_previous_on_line, set_previous_on_line, accesskit_node_clear_previous_on_line, clear_previous_on_line),
    (accesskit_node_popup_for, popup_for, accesskit_node_set_popup_for, set_popup_for, accesskit_node_clear_popup_for, clear_popup_for)
}

/// Only call this function with a string that originated from AccessKit.
#[no_mangle]
pub extern "C" fn accesskit_string_free(string: *mut c_char) {
    assert!(!string.is_null());
    drop(unsafe { CString::from_raw(string) });
}

string_property_methods! {
    (accesskit_node_label, label, accesskit_node_set_label, accesskit_node_set_label_with_length, set_label, accesskit_node_clear_label, clear_label),
    (accesskit_node_description, description, accesskit_node_set_description, accesskit_node_set_description_with_length, set_description, accesskit_node_clear_description, clear_description),
    (accesskit_node_value, value, accesskit_node_set_value, accesskit_node_set_value_with_length, set_value, accesskit_node_clear_value, clear_value),
    (accesskit_node_access_key, access_key, accesskit_node_set_access_key, accesskit_node_set_access_key_with_length, set_access_key, accesskit_node_clear_access_key, clear_access_key),
    (accesskit_node_author_id, author_id, accesskit_node_set_author_id, accesskit_node_set_author_id_with_length, set_author_id, accesskit_node_clear_author_id, clear_author_id),
    (accesskit_node_class_name, class_name, accesskit_node_set_class_name, accesskit_node_set_class_name_with_length, set_class_name, accesskit_node_clear_class_name, clear_class_name),
    (accesskit_node_font_family, font_family, accesskit_node_set_font_family, accesskit_node_set_font_family_with_length, set_font_family, accesskit_node_clear_font_family, clear_font_family),
    (accesskit_node_html_tag, html_tag, accesskit_node_set_html_tag, accesskit_node_set_html_tag_with_length, set_html_tag, accesskit_node_clear_html_tag, clear_html_tag),
    (accesskit_node_inner_html, inner_html, accesskit_node_set_inner_html, accesskit_node_set_inner_html_with_length, set_inner_html, accesskit_node_clear_inner_html, clear_inner_html),
    (accesskit_node_keyboard_shortcut, keyboard_shortcut, accesskit_node_set_keyboard_shortcut, accesskit_node_set_keyboard_shortcut_with_length, set_keyboard_shortcut, accesskit_node_clear_keyboard_shortcut, clear_keyboard_shortcut),
    (accesskit_node_language, language, accesskit_node_set_language, accesskit_node_set_language_with_length, set_language, accesskit_node_clear_language, clear_language),
    (accesskit_node_placeholder, placeholder, accesskit_node_set_placeholder, accesskit_node_set_placeholder_with_length, set_placeholder, accesskit_node_clear_placeholder, clear_placeholder),
    (accesskit_node_role_description, role_description, accesskit_node_set_role_description, accesskit_node_set_role_description_with_length, set_role_description, accesskit_node_clear_role_description, clear_role_description),
    (accesskit_node_state_description, state_description, accesskit_node_set_state_description, accesskit_node_set_state_description_with_length, set_state_description, accesskit_node_clear_state_description, clear_state_description),
    (accesskit_node_tooltip, tooltip, accesskit_node_set_tooltip, accesskit_node_set_tooltip_with_length, set_tooltip, accesskit_node_clear_tooltip, clear_tooltip),
    (accesskit_node_url, url, accesskit_node_set_url, accesskit_node_set_url_with_length, set_url, accesskit_node_clear_url, clear_url),
    (accesskit_node_row_index_text, row_index_text, accesskit_node_set_row_index_text, accesskit_node_set_row_index_text_with_length, set_row_index_text, accesskit_node_clear_row_index_text, clear_row_index_text),
    (accesskit_node_column_index_text, column_index_text, accesskit_node_set_column_index_text, accesskit_node_set_column_index_text_with_length, set_column_index_text, accesskit_node_clear_column_index_text, clear_column_index_text)
}

f64_property_methods! {
    (accesskit_node_scroll_x, scroll_x, accesskit_node_set_scroll_x, set_scroll_x, accesskit_node_clear_scroll_x, clear_scroll_x),
    (accesskit_node_scroll_x_min, scroll_x_min, accesskit_node_set_scroll_x_min, set_scroll_x_min, accesskit_node_clear_scroll_x_min, clear_scroll_x_min),
    (accesskit_node_scroll_x_max, scroll_x_max, accesskit_node_set_scroll_x_max, set_scroll_x_max, accesskit_node_clear_scroll_x_max, clear_scroll_x_max),
    (accesskit_node_scroll_y, scroll_y, accesskit_node_set_scroll_y, set_scroll_y, accesskit_node_clear_scroll_y, clear_scroll_y),
    (accesskit_node_scroll_y_min, scroll_y_min, accesskit_node_set_scroll_y_min, set_scroll_y_min, accesskit_node_clear_scroll_y_min, clear_scroll_y_min),
    (accesskit_node_scroll_y_max, scroll_y_max, accesskit_node_set_scroll_y_max, set_scroll_y_max, accesskit_node_clear_scroll_y_max, clear_scroll_y_max),
    (accesskit_node_numeric_value, numeric_value, accesskit_node_set_numeric_value, set_numeric_value, accesskit_node_clear_numeric_value, clear_numeric_value),
    (accesskit_node_min_numeric_value, min_numeric_value, accesskit_node_set_min_numeric_value, set_min_numeric_value, accesskit_node_clear_min_numeric_value, clear_min_numeric_value),
    (accesskit_node_max_numeric_value, max_numeric_value, accesskit_node_set_max_numeric_value, set_max_numeric_value, accesskit_node_clear_max_numeric_value, clear_max_numeric_value),
    (accesskit_node_numeric_value_step, numeric_value_step, accesskit_node_set_numeric_value_step, set_numeric_value_step, accesskit_node_clear_numeric_value_step, clear_numeric_value_step),
    (accesskit_node_numeric_value_jump, numeric_value_jump, accesskit_node_set_numeric_value_jump, set_numeric_value_jump, accesskit_node_clear_numeric_value_jump, clear_numeric_value_jump),
    (accesskit_node_font_size, font_size, accesskit_node_set_font_size, set_font_size, accesskit_node_clear_font_size, clear_font_size),
    (accesskit_node_font_weight, font_weight, accesskit_node_set_font_weight, set_font_weight, accesskit_node_clear_font_weight, clear_font_weight)
}

usize_property_methods! {
    (accesskit_node_row_count, row_count, accesskit_node_set_row_count, set_row_count, accesskit_node_clear_row_count, clear_row_count),
    (accesskit_node_column_count, column_count, accesskit_node_set_column_count, set_column_count, accesskit_node_clear_column_count, clear_column_count),
    (accesskit_node_row_index, row_index, accesskit_node_set_row_index, set_row_index, accesskit_node_clear_row_index, clear_row_index),
    (accesskit_node_column_index, column_index, accesskit_node_set_column_index, set_column_index, accesskit_node_clear_column_index, clear_column_index),
    (accesskit_node_row_span, row_span, accesskit_node_set_row_span, set_row_span, accesskit_node_clear_row_span, clear_row_span),
    (accesskit_node_column_span, column_span, accesskit_node_set_column_span, set_column_span, accesskit_node_clear_column_span, clear_column_span),
    (accesskit_node_level, level, accesskit_node_set_level, set_level, accesskit_node_clear_level, clear_level),
    (accesskit_node_size_of_set, size_of_set, accesskit_node_set_size_of_set, set_size_of_set, accesskit_node_clear_size_of_set, clear_size_of_set),
    (accesskit_node_position_in_set, position_in_set, accesskit_node_set_position_in_set, set_position_in_set, accesskit_node_clear_position_in_set, clear_position_in_set)
}

color_property_methods! {
    (accesskit_node_color_value, color_value, accesskit_node_set_color_value, set_color_value, accesskit_node_clear_color_value, clear_color_value),
    (accesskit_node_background_color, background_color, accesskit_node_set_background_color, set_background_color, accesskit_node_clear_background_color, clear_background_color),
    (accesskit_node_foreground_color, foreground_color, accesskit_node_set_foreground_color, set_foreground_color, accesskit_node_clear_foreground_color, clear_foreground_color)
}

text_decoration_property_methods! {
    (accesskit_node_overline, overline, accesskit_node_set_overline, set_overline, accesskit_node_clear_overline, clear_overline),
    (accesskit_node_strikethrough, strikethrough, accesskit_node_set_strikethrough, set_strikethrough, accesskit_node_clear_strikethrough, clear_strikethrough),
    (accesskit_node_underline, underline, accesskit_node_set_underline, set_underline, accesskit_node_clear_underline, clear_underline)
}

length_slice_property_methods! {
    (accesskit_node_character_lengths, character_lengths, accesskit_node_set_character_lengths, set_character_lengths, accesskit_node_clear_character_lengths, clear_character_lengths),
    (accesskit_node_word_lengths, word_lengths, accesskit_node_set_word_lengths, set_word_lengths, accesskit_node_clear_word_lengths, clear_word_lengths)
}

coord_slice_property_methods! {
    (accesskit_node_character_positions, character_positions, accesskit_node_set_character_positions, set_character_positions, accesskit_node_clear_character_positions, clear_character_positions),
    (accesskit_node_character_widths, character_widths, accesskit_node_set_character_widths, set_character_widths, accesskit_node_clear_character_widths, clear_character_widths)
}

bool_property_methods! {
    (accesskit_node_is_expanded, is_expanded, accesskit_node_set_expanded, set_expanded, accesskit_node_clear_expanded, clear_expanded),
    (accesskit_node_is_selected, is_selected, accesskit_node_set_selected, set_selected, accesskit_node_clear_selected, clear_selected)
}

unique_enum_property_methods! {
    (opt_Invalid, Invalid, accesskit_node_invalid, invalid, accesskit_node_set_invalid, set_invalid, accesskit_node_clear_invalid, clear_invalid),
    (opt_Toggled, Toggled, accesskit_node_toggled, toggled, accesskit_node_set_toggled, set_toggled, accesskit_node_clear_toggled, clear_toggled),
    (opt_Live, Live, accesskit_node_live, live, accesskit_node_set_live, set_live, accesskit_node_clear_live, clear_live),
    (opt_TextDirection, TextDirection, accesskit_node_text_direction, text_direction, accesskit_node_set_text_direction, set_text_direction, accesskit_node_clear_text_direction, clear_text_direction),
    (opt_Orientation, Orientation, accesskit_node_orientation, orientation, accesskit_node_set_orientation, set_orientation, accesskit_node_clear_orientation, clear_orientation),
    (opt_SortDirection, SortDirection, accesskit_node_sort_direction, sort_direction, accesskit_node_set_sort_direction, set_sort_direction, accesskit_node_clear_sort_direction, clear_sort_direction),
    (opt_AriaCurrent, AriaCurrent, accesskit_node_aria_current, aria_current, accesskit_node_set_aria_current, set_aria_current, accesskit_node_clear_aria_current, clear_aria_current),
    (opt_AutoComplete, AutoComplete, accesskit_node_auto_complete, auto_complete, accesskit_node_set_auto_complete, set_auto_complete, accesskit_node_clear_auto_complete, clear_auto_complete),
    (opt_HasPopup, HasPopup, accesskit_node_has_popup, has_popup, accesskit_node_set_has_popup, set_has_popup, accesskit_node_clear_has_popup, clear_has_popup),
    (opt_ListStyle, ListStyle, accesskit_node_list_style, list_style, accesskit_node_set_list_style, set_list_style, accesskit_node_clear_list_style, clear_list_style),
    (opt_TextAlign, TextAlign, accesskit_node_text_align, text_align, accesskit_node_set_text_align, set_text_align, accesskit_node_clear_text_align, clear_text_align),
    (opt_VerticalOffset, VerticalOffset, accesskit_node_vertical_offset, vertical_offset, accesskit_node_set_vertical_offset, set_vertical_offset, accesskit_node_clear_vertical_offset, clear_vertical_offset)
}

simple_property_methods! {
    accesskit_node_transform, transform, *const Affine, accesskit_node_set_transform, set_transform, Affine, accesskit_node_clear_transform, clear_transform
}
opt_struct! { opt_rect, Rect }
simple_property_methods! {
    accesskit_node_bounds, bounds, opt_rect, accesskit_node_set_bounds, set_bounds, Rect, accesskit_node_clear_bounds, clear_bounds
}

#[repr(C)]
pub struct text_position {
    pub node: node_id,
    pub character_index: usize,
}

impl From<text_position> for TextPosition {
    fn from(position: text_position) -> Self {
        Self {
            node: position.node.into(),
            character_index: position.character_index,
        }
    }
}

impl From<TextPosition> for text_position {
    fn from(position: TextPosition) -> Self {
        Self {
            node: position.node.into(),
            character_index: position.character_index,
        }
    }
}

#[repr(C)]
pub struct text_selection {
    pub anchor: text_position,
    pub focus: text_position,
}

impl From<text_selection> for TextSelection {
    fn from(selection: text_selection) -> Self {
        Self {
            anchor: selection.anchor.into(),
            focus: selection.focus.into(),
        }
    }
}

impl From<TextSelection> for text_selection {
    fn from(selection: TextSelection) -> Self {
        Self {
            anchor: selection.anchor.into(),
            focus: selection.focus.into(),
        }
    }
}

impl From<&TextSelection> for text_selection {
    fn from(selection: &TextSelection) -> Self {
        Self {
            anchor: selection.anchor.into(),
            focus: selection.focus.into(),
        }
    }
}

opt_struct! { opt_text_selection, text_selection }
property_getters! { accesskit_node_text_selection, text_selection, opt_text_selection }
impl node {
    #[no_mangle]
    pub extern "C" fn accesskit_node_set_text_selection(node: *mut node, value: text_selection) {
        let node = mut_from_ptr(node);
        node.set_text_selection(Box::new(value.into()));
    }
}
clearer! { accesskit_node_clear_text_selection, clear_text_selection }

/// Use `accesskit_custom_action_new` or 
/// `accesskit_custom_action_new_with_length` to create this struct. Do not 
/// reallocate `description`.
///
/// When you get this struct, you are responsible for freeing `description`.
#[derive(Clone)]
#[repr(C)]
pub struct custom_action {
    pub id: i32,
    pub description: *mut c_char,
}

impl custom_action {
    #[no_mangle]
    pub extern "C" fn accesskit_custom_action_new(
        id: i32,
        description: *const c_char,
    ) -> custom_action {
        let description = CString::new(String::from(
            unsafe { CStr::from_ptr(description) }.to_string_lossy(),
        ))
        .unwrap();
        Self {
            id,
            description: description.into_raw(),
        }
    }
    
    /// The string must not contain null bytes.
    #[no_mangle]
    pub extern "C" fn accesskit_custom_action_new_with_length(
        id: i32,
        length: usize,
        description: *const c_char,
    ) -> custom_action {
        let description = CString::new(String::from_utf8_lossy(
            unsafe { slice::from_raw_parts(description as *const u8, length) }
        ).into_owned())
        .unwrap();
        Self {
            id,
            description: description.into_raw(),
        }
    }
}

impl Drop for custom_action {
    fn drop(&mut self) {
        accesskit_string_free(self.description);
    }
}

impl From<custom_action> for CustomAction {
    fn from(action: custom_action) -> Self {
        Self {
            id: action.id,
            description: unsafe { CStr::from_ptr(action.description).to_string_lossy().into() },
        }
    }
}

impl From<&custom_action> for CustomAction {
    fn from(action: &custom_action) -> Self {
        Self {
            id: action.id,
            description: unsafe { CStr::from_ptr(action.description).to_string_lossy().into() },
        }
    }
}

impl From<&CustomAction> for custom_action {
    fn from(action: &CustomAction) -> Self {
        Self {
            id: action.id,
            description: CString::new(&*action.description).unwrap().into_raw(),
        }
    }
}

array_struct! { custom_actions, CustomAction, custom_action, accesskit_custom_actions_free }

vec_property_methods! {
    (CustomAction, accesskit_node_custom_actions, custom_actions, *mut custom_actions, accesskit_node_set_custom_actions, set_custom_actions, custom_action, accesskit_node_push_custom_action, push_custom_action, accesskit_node_clear_custom_actions, clear_custom_actions)
}

impl node {
    #[no_mangle]
    pub extern "C" fn accesskit_node_new(role: Role) -> *mut node {
        let node = Node::new(role);
        BoxCastPtr::to_mut_ptr(node)
    }

    #[no_mangle]
    pub extern "C" fn accesskit_node_free(node: *mut node) {
        drop(box_from_ptr(node));
    }

    /// Caller must call `accesskit_string_free` with the return value.
    #[no_mangle]
    pub extern "C" fn accesskit_node_debug(node: *const node) -> *mut c_char {
        debug_repr_from_ptr(node)
    }
}

pub struct tree {
    _private: [u8; 0],
}

impl CastPtr for tree {
    type RustType = Tree;
}

impl BoxCastPtr for tree {}

impl tree {
    #[no_mangle]
    pub extern "C" fn accesskit_tree_new(root: node_id) -> *mut tree {
        let tree = Tree::new(root.into());
        BoxCastPtr::to_mut_ptr(tree)
    }

    #[no_mangle]
    pub extern "C" fn accesskit_tree_free(tree: *mut tree) {
        drop(box_from_ptr(tree));
    }

    /// Caller must call `accesskit_string_free` with the return value.
    #[no_mangle]
    pub extern "C" fn accesskit_tree_get_toolkit_name(tree: *const tree) -> *mut c_char {
        let tree = ref_from_ptr(tree);
        match tree.toolkit_name.as_ref() {
            Some(value) => CString::new(value.clone()).unwrap().into_raw(),
            None => ptr::null_mut(),
        }
    }

    #[no_mangle]
    pub extern "C" fn accesskit_tree_set_toolkit_name(
        tree: *mut tree,
        toolkit_name: *const c_char,
    ) {
        let tree = mut_from_ptr(tree);
        tree.toolkit_name = Some(String::from(
            unsafe { CStr::from_ptr(toolkit_name) }.to_string_lossy(),
        ));
    }
    
    #[no_mangle]
    pub extern "C" fn accesskit_tree_set_toolkit_name_with_length(
        tree: *mut tree,
        length: usize,
        toolkit_name: *const c_char,
    ) {
        let tree = mut_from_ptr(tree);
        tree.toolkit_name = Some(String::from_utf8_lossy(
            unsafe { slice::from_raw_parts(toolkit_name as *const u8, length) }
        ).into_owned())
    }

    #[no_mangle]
    pub extern "C" fn accesskit_tree_clear_toolkit_name(tree: *mut tree) {
        let tree = mut_from_ptr(tree);
        tree.toolkit_name = None;
    }

    /// Caller must call `accesskit_string_free` with the return value.
    #[no_mangle]
    pub extern "C" fn accesskit_tree_get_toolkit_version(tree: *const tree) -> *mut c_char {
        let tree = ref_from_ptr(tree);
        match tree.toolkit_version.as_ref() {
            Some(value) => CString::new(value.clone()).unwrap().into_raw(),
            None => ptr::null_mut(),
        }
    }

    #[no_mangle]
    pub extern "C" fn accesskit_tree_set_toolkit_version(
        tree: *mut tree,
        toolkit_version: *const c_char,
    ) {
        let tree = mut_from_ptr(tree);
        tree.toolkit_version = Some(String::from(
            unsafe { CStr::from_ptr(toolkit_version) }.to_string_lossy(),
        ));
    }
    
    #[no_mangle]
    pub extern "C" fn accesskit_tree_set_toolkit_version_with_length(
        tree: *mut tree,
        length: usize,
        toolkit_version: *const c_char,
    ) {
        let tree = mut_from_ptr(tree);
        tree.toolkit_version = Some(String::from_utf8_lossy(
            unsafe { slice::from_raw_parts(toolkit_version as *const u8, length) }
        ).into_owned())
    }

    #[no_mangle]
    pub extern "C" fn accesskit_tree_clear_toolkit_version(tree: *mut tree) {
        let tree = mut_from_ptr(tree);
        tree.toolkit_version = None;
    }

    /// Caller must call `accesskit_string_free` with the return value.
    #[no_mangle]
    pub extern "C" fn accesskit_tree_debug(tree: *const tree) -> *mut c_char {
        debug_repr_from_ptr(tree)
    }
}

pub struct tree_update {
    _private: [u8; 0],
}

impl CastPtr for tree_update {
    type RustType = TreeUpdate;
}

impl BoxCastPtr for tree_update {}

impl tree_update {
    #[no_mangle]
    pub extern "C" fn accesskit_tree_update_with_focus(focus: node_id) -> *mut tree_update {
        let update = TreeUpdate {
            nodes: vec![],
            tree: None,
            focus: focus.into(),
        };
        BoxCastPtr::to_mut_ptr(update)
    }

    #[no_mangle]
    pub extern "C" fn accesskit_tree_update_with_capacity_and_focus(
        capacity: usize,
        focus: node_id,
    ) -> *mut tree_update {
        let update = TreeUpdate {
            nodes: Vec::with_capacity(capacity),
            tree: None,
            focus: focus.into(),
        };
        BoxCastPtr::to_mut_ptr(update)
    }

    #[no_mangle]
    pub extern "C" fn accesskit_tree_update_free(update: *mut tree_update) {
        drop(box_from_ptr(update));
    }

    /// Appends the provided node to the tree update's list of nodes.
    /// Takes ownership of `node`.
    #[no_mangle]
    pub extern "C" fn accesskit_tree_update_push_node(
        update: *mut tree_update,
        id: node_id,
        node: *mut node,
    ) {
        let update = mut_from_ptr(update);
        let node = box_from_ptr(node);
        update.nodes.push((id.into(), *node));
    }

    #[no_mangle]
    pub extern "C" fn accesskit_tree_update_set_tree(update: *mut tree_update, tree: *mut tree) {
        let update = mut_from_ptr(update);
        update.tree = Some(*box_from_ptr(tree));
    }

    #[no_mangle]
    pub extern "C" fn accesskit_tree_update_clear_tree(update: *mut tree_update) {
        let update = mut_from_ptr(update);
        update.tree = None;
    }

    #[no_mangle]
    pub extern "C" fn accesskit_tree_update_set_focus(update: *mut tree_update, focus: node_id) {
        let update = mut_from_ptr(update);
        update.focus = focus.into();
    }

    /// Caller must call `accesskit_string_free` with the return value.
    #[no_mangle]
    pub extern "C" fn accesskit_tree_update_debug(tree_update: *const tree_update) -> *mut c_char {
        debug_repr_from_ptr(tree_update)
    }
}

#[repr(C)]
pub enum action_data {
    CustomAction(i32),
    Value(*mut c_char),
    NumericValue(f64),
    ScrollUnit(ScrollUnit),
    /// Optional suggestion for `ACCESSKIT_ACTION_SCROLL_INTO_VIEW`, specifying
    /// the preferred position of the target node relative to the scrollable
    /// container's viewport.
    ScrollHint(ScrollHint),
    ScrollToPoint(Point),
    SetScrollOffset(Point),
    SetTextSelection(text_selection),
}

impl Drop for action_data {
    fn drop(&mut self) {
        if let Self::Value(value) = *self {
            accesskit_string_free(value);
        }
    }
}

opt_struct! { opt_action_data, action_data }

impl From<ActionData> for action_data {
    fn from(data: ActionData) -> Self {
        match data {
            ActionData::CustomAction(action) => Self::CustomAction(action),
            ActionData::Value(value) => Self::Value(CString::new(&*value).unwrap().into_raw()),
            ActionData::NumericValue(value) => Self::NumericValue(value),
            ActionData::ScrollUnit(value) => Self::ScrollUnit(value),
            ActionData::ScrollHint(hint) => Self::ScrollHint(hint),
            ActionData::ScrollToPoint(point) => Self::ScrollToPoint(point),
            ActionData::SetScrollOffset(offset) => Self::SetScrollOffset(offset),
            ActionData::SetTextSelection(selection) => Self::SetTextSelection(selection.into()),
        }
    }
}

#[repr(C)]
pub struct action_request {
    pub action: Action,
    pub target: node_id,
    pub data: opt_action_data,
}

impl From<ActionRequest> for action_request {
    fn from(request: ActionRequest) -> action_request {
        Self {
            action: request.action,
            target: request.target.into(),
            data: request.data.into(),
        }
    }
}

#[no_mangle]
pub extern "C" fn accesskit_action_request_free(request: *mut action_request) {
    drop(unsafe { Box::from_raw(request) });
}

type ActivationHandlerCallbackUnwrapped = extern "C" fn(userdata: *mut c_void) -> *mut tree_update;
pub type ActivationHandlerCallback =
    Option<extern "C" fn(userdata: *mut c_void) -> *mut tree_update>;

struct FfiActivationHandlerUserdata(*mut c_void);

unsafe impl Send for FfiActivationHandlerUserdata {}

pub(crate) struct FfiActivationHandler {
    callback: ActivationHandlerCallbackUnwrapped,
    userdata: FfiActivationHandlerUserdata,
}

impl FfiActivationHandler {
    pub(crate) fn new(callback: ActivationHandlerCallback, userdata: *mut c_void) -> Self {
        Self {
            callback: callback.unwrap(),
            userdata: FfiActivationHandlerUserdata(userdata),
        }
    }
}

impl ActivationHandler for FfiActivationHandler {
    fn request_initial_tree(&mut self) -> Option<TreeUpdate> {
        let result = (self.callback)(self.userdata.0);
        if result.is_null() {
            None
        } else {
            Some(*box_from_ptr(result))
        }
    }
}

type ActionHandlerCallbackUnwrapped =
    extern "C" fn(request: *mut action_request, userdata: *mut c_void);

/// Ownership of `request` is transferred to the callback. `request` must
/// be freed using `accesskit_action_request_free`.
pub type ActionHandlerCallback =
    Option<extern "C" fn(request: *mut action_request, userdata: *mut c_void)>;

struct FfiActionHandlerUserdata(*mut c_void);

unsafe impl Send for FfiActionHandlerUserdata {}

pub(crate) struct FfiActionHandler {
    callback: ActionHandlerCallbackUnwrapped,
    userdata: FfiActionHandlerUserdata,
}

impl FfiActionHandler {
    pub(crate) fn new(callback: ActionHandlerCallback, userdata: *mut c_void) -> Self {
        Self {
            callback: callback.unwrap(),
            userdata: FfiActionHandlerUserdata(userdata),
        }
    }
}

impl ActionHandler for FfiActionHandler {
    fn do_action(&mut self, request: ActionRequest) {
        let request = Box::new(action_request::from(request));
        (self.callback)(Box::into_raw(request), self.userdata.0);
    }
}

type DeactivationHandlerCallbackUnwrapped = extern "C" fn(userdata: *mut c_void);
pub type DeactivationHandlerCallback = Option<extern "C" fn(userdata: *mut c_void)>;

struct FfiDeactivationHandlerUserdata(*mut c_void);

unsafe impl Send for FfiDeactivationHandlerUserdata {}

pub(crate) struct FfiDeactivationHandler {
    callback: DeactivationHandlerCallbackUnwrapped,
    userdata: FfiDeactivationHandlerUserdata,
}

impl FfiDeactivationHandler {
    #[allow(dead_code)]
    pub(crate) fn new(callback: DeactivationHandlerCallback, userdata: *mut c_void) -> Self {
        Self {
            callback: callback.unwrap(),
            userdata: FfiDeactivationHandlerUserdata(userdata),
        }
    }
}

impl DeactivationHandler for FfiDeactivationHandler {
    fn deactivate_accessibility(&mut self) {
        (self.callback)(self.userdata.0);
    }
}

#[repr(transparent)]
pub struct tree_update_factory_userdata(pub *mut c_void);

unsafe impl Send for tree_update_factory_userdata {}

/// This function can't return a null pointer. Ownership of the returned value will be transferred to the caller.
pub type tree_update_factory =
    Option<extern "C" fn(tree_update_factory_userdata) -> *mut tree_update>;
