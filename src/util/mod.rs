use bevy::prelude::*;
pub mod flycam;

pub fn vary_color(rgb: Vec3, variance: f32) -> Vec3 {
    let hsv = rgb_to_hsv(rgb);

    // Add variance to the hue and value
    let mut new_hsv = hsv;
    new_hsv.x += variance;
    new_hsv.y -= variance;

    // Clamp within the [0, 1] range
    new_hsv.x = new_hsv.x.clamp(0.0, 1.0);
    new_hsv.y = new_hsv.y.clamp(0.0, 1.0);

    // Convert back to RGB
    hsv_to_rgb(new_hsv)
}

fn rgb_to_hsv(rgb: Vec3) -> Vec3 {
    let min_val = rgb.x.min(rgb.y).min(rgb.z);
    let max_val = rgb.x.max(rgb.y).max(rgb.z);
    let delta = max_val - min_val;

    let mut hue: f32;
    let saturation: f32;
    let value: f32 = max_val;

    if delta < 0.00001 {
        // No chroma, it's a shade of gray
        hue = 0.0;
        saturation = 0.0;
    } else {
        // Chromatic data
        if max_val == rgb.x {
            hue = (rgb.y - rgb.z) / delta;
        } else if max_val == rgb.y {
            hue = 2.0 + (rgb.z - rgb.x) / delta;
        } else {
            hue = 4.0 + (rgb.x - rgb.y) / delta;
        }

        hue /= 6.0; // Normalize hue to the [0, 1] range

        if hue < 0.0 {
            hue += 1.0;
        }

        saturation = delta / max_val;
    }

    Vec3 {
        x: hue,
        y: saturation,
        z: value,
    }
}

fn hsv_to_rgb(hsv: Vec3) -> Vec3 {
    let hue = hsv.x;
    let saturation = hsv.y;
    let value = hsv.z;

    let chroma = value * saturation;
    let hue_prime = hue * 6.0;
    let x = chroma * (1.0 - f32::abs(hue_prime % 2.0 - 1.0));

    let mut rgb = if hue_prime < 1.0 {
        Vec3 {
            x: chroma,
            y: x,
            z: 0.0,
        }
    } else if hue_prime < 2.0 {
        Vec3 {
            x: x,
            y: chroma,
            z: 0.0,
        }
    } else if hue_prime < 3.0 {
        Vec3 {
            x: 0.0,
            y: chroma,
            z: x,
        }
    } else if hue_prime < 4.0 {
        Vec3 {
            x: 0.0,
            y: x,
            z: chroma,
        }
    } else if hue_prime < 5.0 {
        Vec3 {
            x: x,
            y: 0.0,
            z: chroma,
        }
    } else {
        Vec3 {
            x: chroma,
            y: 0.0,
            z: x,
        }
    };

    let m = value - chroma;
    rgb.x += m;
    rgb.y += m;
    rgb.z += m;

    rgb
}

pub fn create_perspective_projection_matrix(aspect_ratio : f32, fov : f32, near : f32, far : f32) -> Mat4 {
    let tan_half_fov = f32::tan(fov * 0.5 * 3.14159265 / 180.0);
    let sx = 1.0 / (aspect_ratio * tan_half_fov);
    let sy = 1.0 / tan_half_fov;
    let sz = -(far + near) / (far - near);
    let pz = -(2.0 * far * near) / (far - near);
    return Mat4::from_cols(
        Vec4::new(sx, 0.0, 0.0, 0.0),
        Vec4::new(0.0, sy, 0.0, 0.0),
        Vec4::new(0.0, 0.0, sz, -1.0),
        Vec4::new(0.0, 0.0, pz, 0.0),
    );
}
