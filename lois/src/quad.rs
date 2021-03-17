use crate::commons::{Point, Rect, ViewSize};

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Quad {
    transform: [[f32; 4]; 4],
    src_rect: [f32; 4],
}

impl Quad {
    pub fn new(
        target_size: ViewSize,
        texture_size: ViewSize,
        src_rect: Option<Rect>,
        dest_rect: Option<Rect>,
        rotation_center: Option<Point>,
        rotation_angle: f32,
    ) -> Self {
        use glam::*;

        let ViewSize { width: texture_width, height: texture_height } = texture_size;

        let src_rect = src_rect
            .and_then(|r| Some(r.normalized(texture_width as f32, texture_height as f32)))
            .unwrap_or([0.0, 0.0, 1.0, 1.0]);

        let dest_rect = dest_rect.unwrap_or(target_size.into());

        let rotation_center: [f32; 2] = rotation_center
            .unwrap_or(Point::new(
                dest_rect.x as f32 + dest_rect.w as f32 / 2.0,
                dest_rect.y as f32 + dest_rect.h as f32 / 2.0,
            ))
            .into();

        let dest_rect: [f32; 4] = dest_rect.into();

        let transform = {
            let [x, y, w, h] = dest_rect;
            let [rx, ry] = rotation_center;
            let (dx, dy) = (rx - x, ry - y);
            let position = Mat4::from_translation(Vec3::new(x, y, 0.0));
            let scale = Mat4::from_scale(Vec3::new(w, h, 0.0));
            let rotation = Mat4::from_translation(Vec3::new(dx, dy, 0.0))
                * Mat4::from_rotation_z(rotation_angle)
                * Mat4::from_translation(Vec3::new(-dx, -dy, 0.0));
            position * rotation * scale
        };

        Quad {
            src_rect,
            transform: transform.to_cols_array_2d(),
        }
    }
}
