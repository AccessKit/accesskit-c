// Copyright 2023 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use accesskit::{Affine, Point, Rect, Size, Vec2};

#[no_mangle]
pub const extern "C" fn accesskit_affine_identity() -> Affine {
    Affine::scale(1.0)
}

#[no_mangle]
pub const extern "C" fn accesskit_affine_flip_y() -> Affine {
    Affine::new([1.0, 0., 0., -1.0, 0., 0.])
}

#[no_mangle]
pub const extern "C" fn accesskit_affine_flip_x() -> Affine {
    Affine::new([-1.0, 0., 0., 1.0, 0., 0.])
}

#[no_mangle]
pub const extern "C" fn accesskit_affine_scale(s: f64) -> Affine {
    Affine::scale(s)
}

#[no_mangle]
pub const extern "C" fn accesskit_affine_scale_non_uniform(s_x: f64, s_y: f64) -> Affine {
    Affine::scale_non_uniform(s_x, s_y)
}

#[no_mangle]
pub extern "C" fn accesskit_affine_translate(p: Vec2) -> Affine {
    Affine::translate(p)
}

#[no_mangle]
pub extern "C" fn accesskit_affine_map_unit_square(rect: Rect) -> Affine {
    Affine::map_unit_square(rect)
}

#[no_mangle]
pub extern "C" fn accesskit_affine_determinant(affine: Affine) -> f64 {
    Affine::determinant(affine)
}

#[no_mangle]
pub extern "C" fn accesskit_affine_inverse(affine: Affine) -> Affine {
    Affine::inverse(affine)
}

#[no_mangle]
pub extern "C" fn accesskit_affine_transform_rect_bbox(affine: Affine, rect: Rect) -> Rect {
    Affine::transform_rect_bbox(affine, rect)
}

#[no_mangle]
pub extern "C" fn accesskit_affine_is_finite(affine: *const Affine) -> bool {
    if affine.is_null() {
        false
    } else {
        unsafe { Box::from_raw(affine as *mut Affine).is_finite() }
    }
}

#[no_mangle]
pub extern "C" fn accesskit_affine_is_nan(affine: *const Affine) -> bool {
    if affine.is_null() {
        false
    } else {
        unsafe { Box::from_raw(affine as *mut Affine).is_nan() }
    }
}

#[no_mangle]
pub extern "C" fn accesskit_affine_mul(a: Affine, b: Affine) -> Affine {
    a * b
}

#[no_mangle]
pub extern "C" fn accesskit_affine_transform_point(affine: Affine, point: Point) -> Point {
    affine * point
}

#[no_mangle]
pub const extern "C" fn accesskit_point_to_vec2(point: Point) -> Vec2 {
    Point::to_vec2(point)
}

#[no_mangle]
pub extern "C" fn accesskit_point_add_vec2(point: Point, vec: Vec2) -> Point {
    point + vec
}

#[no_mangle]
pub extern "C" fn accesskit_point_sub_vec2(point: Point, vec: Vec2) -> Point {
    point - vec
}

#[no_mangle]
pub extern "C" fn accesskit_point_sub_point(a: Point, b: Point) -> Vec2 {
    a - b
}

#[no_mangle]
pub const extern "C" fn accesskit_rect_new(x0: f64, y0: f64, x1: f64, y1: f64) -> Rect {
    Rect::new(x0, y0, x1, y1)
}

#[no_mangle]
pub extern "C" fn accesskit_rect_from_points(p0: Point, p1: Point) -> Rect {
    Rect::from_points(p0, p1)
}

#[no_mangle]
pub extern "C" fn accesskit_rect_from_origin_size(origin: Point, size: Size) -> Rect {
    Rect::from_origin_size(origin, size)
}

#[no_mangle]
pub extern "C" fn accesskit_rect_with_origin(rect: Rect, origin: Point) -> Rect {
    Rect::with_origin(rect, origin)
}

#[no_mangle]
pub extern "C" fn accesskit_rect_with_size(rect: Rect, size: Size) -> Rect {
    Rect::with_size(rect, size)
}

macro_rules! rect_getter_methods {
    ($(($c_getter:ident, $getter:ident, $getter_result:ty, $default_value:expr)),+) => {
        $(#[no_mangle]
        pub extern "C" fn $c_getter(rect: *const Rect) -> $getter_result {
            if rect.is_null() {
                $default_value
            } else {
                unsafe { Box::from_raw(rect as *mut Rect).$getter() }
            }
        })*
    }
}

rect_getter_methods! {
    (accesskit_rect_width, width, f64, 0.),
    (accesskit_rect_height, height, f64, 0.),
    (accesskit_rect_min_x, min_x, f64, 0.),
    (accesskit_rect_max_x, max_x, f64, 0.),
    (accesskit_rect_min_y, min_y, f64, 0.),
    (accesskit_rect_max_y, max_y, f64, 0.),
    (accesskit_rect_origin, origin, Point, Point::ZERO),
    (accesskit_rect_size, size, Size, Size::ZERO),
    (accesskit_rect_abs, abs, Rect, Rect::ZERO),
    (accesskit_rect_area, area, f64, 0.),
    (accesskit_rect_is_empty, is_empty, bool, true)
}

#[no_mangle]
pub extern "C" fn accesskit_rect_contains(rect: *const Rect, point: Point) -> bool {
    if rect.is_null() {
        false
    } else {
        unsafe { Box::from_raw(rect as *mut Rect).contains(point) }
    }
}

#[no_mangle]
pub extern "C" fn accesskit_rect_union(rect: *const Rect, other: Rect) -> Rect {
    if rect.is_null() {
        Rect::ZERO
    } else {
        unsafe { Box::from_raw(rect as *mut Rect).union(other) }
    }
}

#[no_mangle]
pub extern "C" fn accesskit_rect_union_pt(rect: *const Rect, pt: Point) -> Rect {
    if rect.is_null() {
        Rect::ZERO
    } else {
        unsafe { Box::from_raw(rect as *mut Rect).union_pt(pt) }
    }
}

#[no_mangle]
pub extern "C" fn accesskit_rect_intersect(rect: *const Rect, other: Rect) -> Rect {
    if rect.is_null() {
        Rect::ZERO
    } else {
        unsafe { Box::from_raw(rect as *mut Rect).intersect(other) }
    }
}

#[no_mangle]
pub extern "C" fn accesskit_rect_translate(rect: Rect, translation: Vec2) -> Rect {
    rect + translation
}

#[no_mangle]
pub const extern "C" fn accesskit_size_to_vec2(size: Size) -> Vec2 {
    Size::to_vec2(size)
}

#[no_mangle]
pub extern "C" fn accesskit_size_scale(size: Size, scalar: f64) -> Size {
    size * scalar
}

#[no_mangle]
pub extern "C" fn accesskit_size_add(a: Size, b: Size) -> Size {
    a + b
}

#[no_mangle]
pub extern "C" fn accesskit_size_sub(a: Size, b: Size) -> Size {
    a - b
}

#[no_mangle]
pub const extern "C" fn accesskit_vec2_to_point(vec2: Vec2) -> Point {
    Vec2::to_point(vec2)
}

#[no_mangle]
pub const extern "C" fn accesskit_vec2_to_size(vec2: Vec2) -> Size {
    Vec2::to_size(vec2)
}

#[no_mangle]
pub extern "C" fn accesskit_vec2_add(a: Vec2, b: Vec2) -> Vec2 {
    a + b
}

#[no_mangle]
pub extern "C" fn accesskit_vec2_sub(a: Vec2, b: Vec2) -> Vec2 {
    a - b
}

#[no_mangle]
pub extern "C" fn accesskit_vec2_scale(vec: Vec2, scalar: f64) -> Vec2 {
    vec * scalar
}

#[no_mangle]
pub extern "C" fn accesskit_vec2_neg(vec: Vec2) -> Vec2 {
    -vec
}
