// Vertex shader
struct FeatureUniform {
    grayscale: u32,
    // sepia: u32,
    invert: u32,
    gamma: u32,
    blur: u32,
    radius: u32,
    width: u32,
    height: u32,
    sharpen: u32,
    sharpen_factor: u32,
    edge_detect: u32,
    transform: mat4x4<f32>,
};

struct DrawUniform {
    crosshair: u32,
    circle_center_x: f32,
    circle_center_y: f32,
    circle_radius: f32,
}


@group(1) @binding(0)
var<uniform> feature_uniform: FeatureUniform;


@group(2) @binding(0)
var<uniform> draw_uniform: DrawUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}

fn transform_matrix() -> mat3x3<f32> {
    return mat3x3<f32>(
        feature_uniform.transform[0].xyz,
        feature_uniform.transform[1].xyz,
        feature_uniform.transform[2].xyz,
    );
}

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;

    var transformed = transform_matrix() * vec3<f32>(model.position.x, model.position.y, 1.0);

    out.clip_position = vec4<f32>(transformed.xy, model.position.z, 1.0);
    out.tex_coords = model.tex_coords;

    return out;
}

// Fragment shader

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

const PI: f32 = 22.0 / 7.0;

fn gaussian(offset: vec2<f32>) -> f32 {
    var GAUSSIAN_SIGMA: f32 = f32(feature_uniform.radius) * 0.25;
    return 1.0 / (2.0 * PI * GAUSSIAN_SIGMA * GAUSSIAN_SIGMA) * exp(-((offset.x * offset.x + offset.y * offset.y) / (2.0 * GAUSSIAN_SIGMA * GAUSSIAN_SIGMA)));
}

fn gaussian_blur(tex_coords: vec2<f32>, radius: f32, viewport_resolution: vec2<f32>) -> vec4<f32> {
    var acc = 0.0;
    var color = vec4<f32>();
    var weight = 0.0;

    for (var x = -1.0 * radius / 2.0; x < radius / 2.0; x = x + 1) {
        for (var y = -1.0 * radius / 2.0; y < radius / 2.0; y = y + 1) {
            var offset = vec2(x, y);
            weight = gaussian(offset);
            color = color + (textureSample(t_diffuse, s_diffuse, tex_coords + viewport_resolution * offset) * weight);
            acc = acc + weight;
        }
    }

    return color / acc;
}

fn box_blur(tex_coords: vec2<f32>, viewport_resolution: vec2<f32>) -> vec4<f32> {
    var acc_color = vec4<f32>(0.0);

    var ct = 0.0;

    for (var dx = -1.0; dx <= 1.0; dx = dx + 1.0) {
        for (var dy = -1.0; dy <= 1.0; dy = dy + 1.0) {
            acc_color += textureSample(t_diffuse, s_diffuse, tex_coords + viewport_resolution * vec2<f32>(dx, dy));
            ct = ct + 1.0;
        }
    }

    return acc_color / ct;
}

fn sharpen(tex_coords: vec2<f32>, sharpen_factor: f32, viewport_resolution: vec2<f32>) -> vec4<f32> {
    var center = textureSample(t_diffuse, s_diffuse, tex_coords);

    var up = textureSample(t_diffuse, s_diffuse, tex_coords + (vec2(0, 1) * viewport_resolution));
    var left = textureSample(t_diffuse, s_diffuse, tex_coords + (vec2(-1, 0) * viewport_resolution));
    var right = textureSample(t_diffuse, s_diffuse, tex_coords + (vec2(1, 0) * viewport_resolution));
    var down = textureSample(t_diffuse, s_diffuse, tex_coords + (vec2(0, -1) * viewport_resolution));

    return (1.0 + 4.0 * sharpen_factor) * center - sharpen_factor * (up + left + right + down);
}


fn intensity(color: vec4<f32>) -> f32 {
    let squared = color.rgb * color.rgb;
    return sqrt(squared.r + squared.g + squared.b);
}

fn detect_edge(tex_coords: vec2<f32>, viewport_resolution: vec2<f32>) -> vec4<f32> {
    var x = viewport_resolution.x;
    var y = viewport_resolution.y;

    var top_left = intensity(textureSample(t_diffuse, s_diffuse, tex_coords + vec2(-x, y)));
    var left = intensity(textureSample(t_diffuse, s_diffuse, tex_coords + vec2(-x, 0)));
    var bottom_left = intensity(textureSample(t_diffuse, s_diffuse, tex_coords + vec2(-x, -y)));
    var top = intensity(textureSample(t_diffuse, s_diffuse, tex_coords + vec2(0, y)));
    var bottom = intensity(textureSample(t_diffuse, s_diffuse, tex_coords + vec2(0, -y)));
    var top_right = intensity(textureSample(t_diffuse, s_diffuse, tex_coords + vec2(x, y)));
    var right = intensity(textureSample(t_diffuse, s_diffuse, tex_coords + vec2(x, 0)));
    var bottom_right = intensity(textureSample(t_diffuse, s_diffuse, tex_coords + vec2(x, -y)));

    var gx = top_left + 2.0 * left + bottom_left - top_right - 2.0 * right - bottom_right;
    var gy = -left - 2.0 * top - top_right + bottom_left + 2.0 * bottom + bottom_right;

    var color = sqrt((gx * gx) + (gy * gy));

    return vec4(color, color, color, 1.0);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var pixels = textureSample(t_diffuse, s_diffuse, in.tex_coords);
    var viewport_resolution = 1.0 / vec2(f32(feature_uniform.width), f32(feature_uniform.height));

    if feature_uniform.edge_detect == 1u {
        pixels = detect_edge(in.tex_coords, viewport_resolution);
    }

    if feature_uniform.sharpen == 1u {
        pixels = sharpen(in.tex_coords, f32(feature_uniform.sharpen_factor), viewport_resolution);
    }

    if feature_uniform.blur == 1u {
        pixels = gaussian_blur(in.tex_coords, f32(feature_uniform.radius), viewport_resolution);
    }

    if feature_uniform.gamma != 0u {
        // todo! modify the pixels to account for gamma
        // see https://www.w3.org/TR/2003/REC-PNG-20031110/#13Decoder-gamma-handling
    }

    if feature_uniform.grayscale == 1u {
        var y = (pixels.r * 0.29891 + pixels.g * 0.58661 + pixels.b * 0.11448);
        pixels = vec4(y, y, y, 1.0);
    }

//    if feature_uniform.sepia == 1u {
//        pixels = vec4<f32>(
//            0.393 * pixels.r + 0.769 * pixels.g + 0.189 * pixels.b,
//            0.349 * pixels.r + 0.686 * pixels.g + 0.168 * pixels.b,
//            0.272 * pixels.r + 0.534 * pixels.g + 0.131 * pixels.b,
//            0.30 * pixels.r + 0.59 * pixels.g + 0.11 * pixels.b
//        );
//    }

    if feature_uniform.invert == 1u {
        pixels = vec4<f32>(
            1.0 - pixels.r,
            1.0 - pixels.g,
            1.0 - pixels.b,
            pixels.a
        );
    }

    if draw_uniform.crosshair == 1u {
        var pixel_coords = in.tex_coords * vec2(f32(feature_uniform.width), f32(feature_uniform.height));
        var mouse_pixel = vec2(draw_uniform.circle_center_x, draw_uniform.circle_center_y);

        if distance(pixel_coords, mouse_pixel) < draw_uniform.circle_radius {
            pixels.r = 0.0;
            pixels.g = 1.0;
            pixels.b = 1.0;
        }
    }

    return pixels;
}
