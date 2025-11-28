#include <metal_stdlib>
using namespace metal;

// ============================================================================
// Shared Vertex Structures (must match Rust #[repr(C)] structs)
// ============================================================================

// Vertex with position (float2) and color (float4)
// Total size: 24 bytes (2*4 + 4*4), aligned to 4 bytes
struct Vertex {
    packed_float2 position;  // 8 bytes
    packed_float4 color;     // 16 bytes
};

// Vertex with position (float2), texcoord (float2), and color (float4)
// Total size: 32 bytes, aligned to 4 bytes
struct TexturedVertex {
    packed_float2 position;  // 8 bytes
    packed_float2 texcoord;  // 8 bytes
    packed_float4 color;     // 16 bytes
};

// Uniforms passed to shaders
// Must match Rust struct exactly with proper alignment
struct Uniforms {
    float2 viewport_size;    // 8 bytes
    float time;              // 4 bytes
    float padding;           // 4 bytes (for 16-byte alignment)
};

// Interpolated vertex output
struct VertexOut {
    float4 position [[position]];
    float4 color;
    float2 texcoord;
};

// ============================================================================
// Vertex Shaders
// ============================================================================

// Basic colored vertex shader - transforms NDC coords and passes color
vertex VertexOut vertex_colored(
    const device Vertex* vertices [[buffer(0)]],
    constant Uniforms& uniforms [[buffer(1)]],
    uint vid [[vertex_id]]
) {
    VertexOut out;
    
    // Convert from pixel coordinates to normalized device coordinates
    float2 pos = vertices[vid].position;
    float2 ndc = (pos / uniforms.viewport_size) * 2.0 - 1.0;
    ndc.y = -ndc.y;  // Flip Y for Metal coordinate system
    
    out.position = float4(ndc, 0.0, 1.0);
    out.color = float4(vertices[vid].color);
    out.texcoord = float2(0.0);
    
    return out;
}

// Textured vertex shader for screen capture display
vertex VertexOut vertex_textured(
    const device TexturedVertex* vertices [[buffer(0)]],
    constant Uniforms& uniforms [[buffer(1)]],
    uint vid [[vertex_id]]
) {
    VertexOut out;
    
    float2 pos = vertices[vid].position;
    float2 ndc = (pos / uniforms.viewport_size) * 2.0 - 1.0;
    ndc.y = -ndc.y;
    
    out.position = float4(ndc, 0.0, 1.0);
    out.color = float4(vertices[vid].color);
    out.texcoord = vertices[vid].texcoord;
    
    return out;
}

// Fullscreen quad vertex shader (generates vertices from vertex_id)
vertex VertexOut vertex_fullscreen(
    uint vid [[vertex_id]]
) {
    VertexOut out;
    
    // Generate fullscreen triangle strip
    float2 positions[4] = {
        float2(-1.0, -1.0),
        float2( 1.0, -1.0),
        float2(-1.0,  1.0),
        float2( 1.0,  1.0)
    };
    
    float2 texcoords[4] = {
        float2(0.0, 1.0),
        float2(1.0, 1.0),
        float2(0.0, 0.0),
        float2(1.0, 0.0)
    };
    
    out.position = float4(positions[vid], 0.0, 1.0);
    out.texcoord = texcoords[vid];
    out.color = float4(1.0);
    
    return out;
}

// ============================================================================
// Fragment Shaders
// ============================================================================

// Simple solid color fragment
fragment float4 fragment_colored(VertexOut in [[stage_in]]) {
    return in.color;
}

// Textured fragment with color tint
fragment float4 fragment_textured(
    VertexOut in [[stage_in]],
    texture2d<float> tex [[texture(0)]],
    sampler samp [[sampler(0)]]
) {
    float4 texColor = tex.sample(samp, in.texcoord);
    return texColor * in.color;
}

// Waveform fragment shader - renders audio waveform from sample buffer
fragment float4 fragment_waveform(
    VertexOut in [[stage_in]],
    constant float* samples [[buffer(0)]],
    constant Uniforms& uniforms [[buffer(1)]]
) {
    // Waveform rendering parameters
    float2 uv = in.texcoord;
    int sample_count = 256;
    
    // Get sample index based on x position
    int idx = int(uv.x * float(sample_count - 1));
    float sample_val = samples[idx];
    
    // Center the waveform vertically
    float center = 0.5;
    float amplitude = sample_val * 0.4;  // Scale amplitude
    
    // Calculate distance from waveform line
    float wave_y = center + amplitude;
    float dist = abs(uv.y - wave_y);
    
    // Anti-aliased line rendering
    float line_width = 0.01;
    float alpha = smoothstep(line_width, 0.0, dist);
    
    // Green waveform color
    float3 wave_color = float3(0.2, 1.0, 0.3);
    
    // Add glow effect
    float glow = exp(-dist * 20.0) * 0.5;
    wave_color += float3(0.1, 0.3, 0.1) * glow;
    
    return float4(wave_color, alpha + glow * 0.5);
}

// VU Meter fragment shader
fragment float4 fragment_vu_meter(
    VertexOut in [[stage_in]],
    constant float& level [[buffer(0)]]
) {
    float2 uv = in.texcoord;
    
    // Calculate level in dB scale
    float db = (level > 0.0) ? 20.0 * log10(level) : -60.0;
    float normalized = clamp((db + 60.0) / 60.0, 0.0, 1.0);
    
    // Color gradient: green -> yellow -> red
    float3 color;
    if (uv.x < 0.6) {
        color = float3(0.2, 0.9, 0.2);  // Green
    } else if (uv.x < 0.85) {
        color = float3(0.9, 0.9, 0.2);  // Yellow
    } else {
        color = float3(0.9, 0.2, 0.2);  // Red
    }
    
    // Fill based on level
    float alpha = (uv.x < normalized) ? 1.0 : 0.1;
    
    return float4(color, alpha);
}

// Bitmap font fragment shader
// Expects font texture where each glyph is 8x8 pixels
// Arranged in 16x8 grid (128 ASCII characters)
fragment float4 fragment_bitmap_font(
    VertexOut in [[stage_in]],
    texture2d<float> font_texture [[texture(0)]],
    sampler font_sampler [[sampler(0)]]
) {
    float4 texColor = font_texture.sample(font_sampler, in.texcoord);
    
    // Use texture alpha and vertex color
    float alpha = texColor.r;  // Assuming single-channel font texture
    return float4(in.color.rgb, in.color.a * alpha);
}

// Menu background with rounded corners and shadow
fragment float4 fragment_menu_background(
    VertexOut in [[stage_in]],
    constant float4& rect_params [[buffer(0)]]  // x, y, width, height
) {
    float2 uv = in.texcoord;
    float2 size = rect_params.zw;
    float2 half_size = size * 0.5;
    float2 center = float2(0.5);
    
    // Distance from center
    float2 d = abs(uv - center) * size - (half_size - float2(8.0));  // 8px corner radius
    float dist = length(max(d, float2(0.0))) + min(max(d.x, d.y), 0.0);
    
    // Rounded rectangle
    float corner_radius = 8.0;
    float edge = smoothstep(corner_radius + 1.0, corner_radius, dist);
    
    // Background color with alpha
    float4 bg_color = float4(0.15, 0.15, 0.18, 0.92);
    
    // Subtle border
    float border = smoothstep(corner_radius - 1.0, corner_radius, dist);
    float3 border_color = float3(0.4, 0.4, 0.5);
    
    float3 final_color = mix(bg_color.rgb, border_color, border * 0.5);
    
    return float4(final_color, bg_color.a * edge);
}

// Scanline overlay effect (optional CRT-style effect)
fragment float4 fragment_scanlines(
    VertexOut in [[stage_in]],
    texture2d<float> tex [[texture(0)]],
    sampler samp [[sampler(0)]],
    constant Uniforms& uniforms [[buffer(0)]]
) {
    float4 texColor = tex.sample(samp, in.texcoord);
    
    // Scanline effect
    float scanline = sin(in.texcoord.y * uniforms.viewport_size.y * 0.5) * 0.5 + 0.5;
    scanline = pow(scanline, 0.3);
    
    return float4(texColor.rgb * (0.85 + 0.15 * scanline), texColor.a);
}
