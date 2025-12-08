// Metal Bridge - Create Metal textures from IOSurface

import Foundation
import IOSurface
import Metal
import QuartzCore

// MARK: - Metal Texture from IOSurface

/// Create a Metal texture from an IOSurface plane
/// Returns nil if texture creation fails
@_cdecl("metal_create_texture_from_iosurface")
public func metal_create_texture_from_iosurface(
    _ device: UnsafeMutableRawPointer,
    _ ioSurfacePtr: UnsafeMutableRawPointer,
    _ plane: Int,
    _ width: Int,
    _ height: Int,
    _ pixelFormat: UInt // MTLPixelFormat raw value
) -> UnsafeMutableRawPointer? {
    let mtlDevice = Unmanaged<MTLDevice>.fromOpaque(device).takeUnretainedValue()
    let ioSurface = Unmanaged<IOSurface>.fromOpaque(ioSurfacePtr).takeUnretainedValue()

    let descriptor = MTLTextureDescriptor.texture2DDescriptor(
        pixelFormat: MTLPixelFormat(rawValue: pixelFormat) ?? .bgra8Unorm,
        width: width,
        height: height,
        mipmapped: false
    )
    descriptor.storageMode = .shared
    descriptor.usage = .shaderRead

    guard let texture = mtlDevice.makeTexture(
        descriptor: descriptor,
        iosurface: ioSurface,
        plane: plane
    ) else {
        return nil
    }

    return Unmanaged.passRetained(texture).toOpaque()
}

/// Release a Metal texture
@_cdecl("metal_texture_release")
public func metal_texture_release(_ texture: UnsafeMutableRawPointer) {
    Unmanaged<MTLTexture>.fromOpaque(texture).release()
}

/// Retain a Metal texture
@_cdecl("metal_texture_retain")
public func metal_texture_retain(_ texture: UnsafeMutableRawPointer) -> UnsafeMutableRawPointer {
    let tex = Unmanaged<MTLTexture>.fromOpaque(texture).takeUnretainedValue()
    return Unmanaged.passRetained(tex).toOpaque()
}

/// Get the width of a Metal texture
@_cdecl("metal_texture_get_width")
public func metal_texture_get_width(_ texture: UnsafeMutableRawPointer) -> Int {
    let tex = Unmanaged<MTLTexture>.fromOpaque(texture).takeUnretainedValue()
    return tex.width
}

/// Get the height of a Metal texture
@_cdecl("metal_texture_get_height")
public func metal_texture_get_height(_ texture: UnsafeMutableRawPointer) -> Int {
    let tex = Unmanaged<MTLTexture>.fromOpaque(texture).takeUnretainedValue()
    return tex.height
}

/// Get the pixel format of a Metal texture
@_cdecl("metal_texture_get_pixel_format")
public func metal_texture_get_pixel_format(_ texture: UnsafeMutableRawPointer) -> UInt {
    let tex = Unmanaged<MTLTexture>.fromOpaque(texture).takeUnretainedValue()
    return tex.pixelFormat.rawValue
}

// MARK: - Metal Device

/// Get the system default Metal device
/// Returns nil if no Metal device is available
@_cdecl("metal_create_system_default_device")
public func metal_create_system_default_device() -> UnsafeMutableRawPointer? {
    guard let device = MTLCreateSystemDefaultDevice() else {
        return nil
    }
    return Unmanaged.passRetained(device).toOpaque()
}

/// Release a Metal device
@_cdecl("metal_device_release")
public func metal_device_release(_ device: UnsafeMutableRawPointer) {
    Unmanaged<MTLDevice>.fromOpaque(device).release()
}

/// Get the name of a Metal device
@_cdecl("metal_device_get_name")
public func metal_device_get_name(_ device: UnsafeMutableRawPointer) -> UnsafePointer<CChar>? {
    let dev = Unmanaged<MTLDevice>.fromOpaque(device).takeUnretainedValue()
    return (dev.name as NSString).utf8String
}

// MARK: - Metal Command Queue

/// Create a command queue from a device
@_cdecl("metal_device_create_command_queue")
public func metal_device_create_command_queue(_ device: UnsafeMutableRawPointer) -> UnsafeMutableRawPointer? {
    let dev = Unmanaged<MTLDevice>.fromOpaque(device).takeUnretainedValue()
    guard let queue = dev.makeCommandQueue() else {
        return nil
    }
    return Unmanaged.passRetained(queue).toOpaque()
}

/// Release a command queue
@_cdecl("metal_command_queue_release")
public func metal_command_queue_release(_ queue: UnsafeMutableRawPointer) {
    Unmanaged<MTLCommandQueue>.fromOpaque(queue).release()
}

// MARK: - Metal Library (Shaders)

/// Create a Metal library from source code
@_cdecl("metal_device_create_library_with_source")
public func metal_device_create_library_with_source(
    _ device: UnsafeMutableRawPointer,
    _ source: UnsafePointer<CChar>,
    _ errorOut: UnsafeMutablePointer<UnsafePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    let dev = Unmanaged<MTLDevice>.fromOpaque(device).takeUnretainedValue()
    let sourceStr = String(cString: source)

    do {
        let library = try dev.makeLibrary(source: sourceStr, options: nil)
        return Unmanaged.passRetained(library).toOpaque()
    } catch {
        if let errorOut = errorOut {
            let errorStr = (error.localizedDescription as NSString).utf8String
            errorOut.pointee = errorStr
        }
        return nil
    }
}

/// Release a Metal library
@_cdecl("metal_library_release")
public func metal_library_release(_ library: UnsafeMutableRawPointer) {
    Unmanaged<MTLLibrary>.fromOpaque(library).release()
}

/// Get a function from a Metal library
@_cdecl("metal_library_get_function")
public func metal_library_get_function(
    _ library: UnsafeMutableRawPointer,
    _ name: UnsafePointer<CChar>
) -> UnsafeMutableRawPointer? {
    let lib = Unmanaged<MTLLibrary>.fromOpaque(library).takeUnretainedValue()
    let nameStr = String(cString: name)
    guard let function = lib.makeFunction(name: nameStr) else {
        return nil
    }
    return Unmanaged.passRetained(function).toOpaque()
}

/// Release a Metal function
@_cdecl("metal_function_release")
public func metal_function_release(_ function: UnsafeMutableRawPointer) {
    Unmanaged<MTLFunction>.fromOpaque(function).release()
}

// MARK: - Metal Buffer

/// Create a buffer with data
@_cdecl("metal_device_create_buffer")
public func metal_device_create_buffer(
    _ device: UnsafeMutableRawPointer,
    _ length: Int,
    _ options: UInt // MTLResourceOptions raw value
) -> UnsafeMutableRawPointer? {
    let dev = Unmanaged<MTLDevice>.fromOpaque(device).takeUnretainedValue()
    guard let buffer = dev.makeBuffer(length: length, options: MTLResourceOptions(rawValue: options)) else {
        return nil
    }
    return Unmanaged.passRetained(buffer).toOpaque()
}

/// Get buffer contents pointer
@_cdecl("metal_buffer_contents")
public func metal_buffer_contents(_ buffer: UnsafeMutableRawPointer) -> UnsafeMutableRawPointer {
    let buf = Unmanaged<MTLBuffer>.fromOpaque(buffer).takeUnretainedValue()
    return buf.contents()
}

/// Get buffer length
@_cdecl("metal_buffer_length")
public func metal_buffer_length(_ buffer: UnsafeMutableRawPointer) -> Int {
    let buf = Unmanaged<MTLBuffer>.fromOpaque(buffer).takeUnretainedValue()
    return buf.length
}

/// Notify buffer of modified range
@_cdecl("metal_buffer_did_modify_range")
public func metal_buffer_did_modify_range(_ buffer: UnsafeMutableRawPointer, _ location: Int, _ length: Int) {
    let buf = Unmanaged<MTLBuffer>.fromOpaque(buffer).takeUnretainedValue()
    buf.didModifyRange(location ..< (location + length))
}

/// Release a buffer
@_cdecl("metal_buffer_release")
public func metal_buffer_release(_ buffer: UnsafeMutableRawPointer) {
    Unmanaged<MTLBuffer>.fromOpaque(buffer).release()
}

// MARK: - Metal Layer

/// Create a CAMetalLayer
@_cdecl("metal_layer_create")
public func metal_layer_create() -> UnsafeMutableRawPointer {
    let layer = CAMetalLayer()
    return Unmanaged.passRetained(layer).toOpaque()
}

/// Set the device for a Metal layer
@_cdecl("metal_layer_set_device")
public func metal_layer_set_device(_ layer: UnsafeMutableRawPointer, _ device: UnsafeMutableRawPointer) {
    let mtlLayer = Unmanaged<CAMetalLayer>.fromOpaque(layer).takeUnretainedValue()
    let dev = Unmanaged<MTLDevice>.fromOpaque(device).takeUnretainedValue()
    mtlLayer.device = dev
}

/// Set pixel format for a Metal layer
@_cdecl("metal_layer_set_pixel_format")
public func metal_layer_set_pixel_format(_ layer: UnsafeMutableRawPointer, _ format: UInt) {
    let mtlLayer = Unmanaged<CAMetalLayer>.fromOpaque(layer).takeUnretainedValue()
    mtlLayer.pixelFormat = MTLPixelFormat(rawValue: format) ?? .bgra8Unorm
}

/// Set drawable size for a Metal layer
@_cdecl("metal_layer_set_drawable_size")
public func metal_layer_set_drawable_size(_ layer: UnsafeMutableRawPointer, _ width: Double, _ height: Double) {
    let mtlLayer = Unmanaged<CAMetalLayer>.fromOpaque(layer).takeUnretainedValue()
    mtlLayer.drawableSize = CGSize(width: width, height: height)
}

/// Set presents with transaction
@_cdecl("metal_layer_set_presents_with_transaction")
public func metal_layer_set_presents_with_transaction(_ layer: UnsafeMutableRawPointer, _ value: Bool) {
    let mtlLayer = Unmanaged<CAMetalLayer>.fromOpaque(layer).takeUnretainedValue()
    mtlLayer.presentsWithTransaction = value
}

/// Get next drawable from layer
@_cdecl("metal_layer_next_drawable")
public func metal_layer_next_drawable(_ layer: UnsafeMutableRawPointer) -> UnsafeMutableRawPointer? {
    let mtlLayer = Unmanaged<CAMetalLayer>.fromOpaque(layer).takeUnretainedValue()
    guard let drawable = mtlLayer.nextDrawable() else {
        return nil
    }
    return Unmanaged.passRetained(drawable).toOpaque()
}

/// Release a Metal layer
@_cdecl("metal_layer_release")
public func metal_layer_release(_ layer: UnsafeMutableRawPointer) {
    Unmanaged<CAMetalLayer>.fromOpaque(layer).release()
}

// MARK: - Metal Drawable

/// Get texture from drawable
@_cdecl("metal_drawable_texture")
public func metal_drawable_texture(_ drawable: UnsafeMutableRawPointer) -> UnsafeMutableRawPointer {
    let drw = Unmanaged<CAMetalDrawable>.fromOpaque(drawable).takeUnretainedValue()
    return Unmanaged.passUnretained(drw.texture).toOpaque()
}

/// Present drawable
@_cdecl("metal_drawable_present")
public func metal_drawable_present(_ drawable: UnsafeMutableRawPointer) {
    let drw = Unmanaged<CAMetalDrawable>.fromOpaque(drawable).takeUnretainedValue()
    drw.present()
}

/// Release drawable
@_cdecl("metal_drawable_release")
public func metal_drawable_release(_ drawable: UnsafeMutableRawPointer) {
    Unmanaged<CAMetalDrawable>.fromOpaque(drawable).release()
}

// MARK: - Command Buffer

/// Create a command buffer from a queue
@_cdecl("metal_command_queue_command_buffer")
public func metal_command_queue_command_buffer(_ queue: UnsafeMutableRawPointer) -> UnsafeMutableRawPointer? {
    let q = Unmanaged<MTLCommandQueue>.fromOpaque(queue).takeUnretainedValue()
    guard let buffer = q.makeCommandBuffer() else {
        return nil
    }
    return Unmanaged.passRetained(buffer).toOpaque()
}

/// Present drawable in command buffer
@_cdecl("metal_command_buffer_present_drawable")
public func metal_command_buffer_present_drawable(_ cmdBuffer: UnsafeMutableRawPointer, _ drawable: UnsafeMutableRawPointer) {
    let buf = Unmanaged<MTLCommandBuffer>.fromOpaque(cmdBuffer).takeUnretainedValue()
    let drw = Unmanaged<CAMetalDrawable>.fromOpaque(drawable).takeUnretainedValue()
    buf.present(drw)
}

/// Commit command buffer
@_cdecl("metal_command_buffer_commit")
public func metal_command_buffer_commit(_ cmdBuffer: UnsafeMutableRawPointer) {
    let buf = Unmanaged<MTLCommandBuffer>.fromOpaque(cmdBuffer).takeUnretainedValue()
    buf.commit()
}

/// Wait until command buffer completes
@_cdecl("metal_command_buffer_wait_until_completed")
public func metal_command_buffer_wait_until_completed(_ cmdBuffer: UnsafeMutableRawPointer) {
    let buf = Unmanaged<MTLCommandBuffer>.fromOpaque(cmdBuffer).takeUnretainedValue()
    buf.waitUntilCompleted()
}

/// Release command buffer
@_cdecl("metal_command_buffer_release")
public func metal_command_buffer_release(_ cmdBuffer: UnsafeMutableRawPointer) {
    Unmanaged<MTLCommandBuffer>.fromOpaque(cmdBuffer).release()
}

// MARK: - Render Pass Descriptor

/// Create a render pass descriptor
@_cdecl("metal_render_pass_descriptor_create")
public func metal_render_pass_descriptor_create() -> UnsafeMutableRawPointer {
    let desc = MTLRenderPassDescriptor()
    return Unmanaged.passRetained(desc).toOpaque()
}

/// Set color attachment texture
@_cdecl("metal_render_pass_set_color_attachment_texture")
public func metal_render_pass_set_color_attachment_texture(
    _ desc: UnsafeMutableRawPointer,
    _ index: Int,
    _ texture: UnsafeMutableRawPointer
) {
    let rpd = Unmanaged<MTLRenderPassDescriptor>.fromOpaque(desc).takeUnretainedValue()
    let tex = Unmanaged<MTLTexture>.fromOpaque(texture).takeUnretainedValue()
    rpd.colorAttachments[index].texture = tex
}

/// Set color attachment load action
@_cdecl("metal_render_pass_set_color_attachment_load_action")
public func metal_render_pass_set_color_attachment_load_action(
    _ desc: UnsafeMutableRawPointer,
    _ index: Int,
    _ action: UInt // MTLLoadAction raw value
) {
    let rpd = Unmanaged<MTLRenderPassDescriptor>.fromOpaque(desc).takeUnretainedValue()
    rpd.colorAttachments[index].loadAction = MTLLoadAction(rawValue: action) ?? .clear
}

/// Set color attachment store action
@_cdecl("metal_render_pass_set_color_attachment_store_action")
public func metal_render_pass_set_color_attachment_store_action(
    _ desc: UnsafeMutableRawPointer,
    _ index: Int,
    _ action: UInt // MTLStoreAction raw value
) {
    let rpd = Unmanaged<MTLRenderPassDescriptor>.fromOpaque(desc).takeUnretainedValue()
    rpd.colorAttachments[index].storeAction = MTLStoreAction(rawValue: action) ?? .store
}

/// Set color attachment clear color
@_cdecl("metal_render_pass_set_color_attachment_clear_color")
public func metal_render_pass_set_color_attachment_clear_color(
    _ desc: UnsafeMutableRawPointer,
    _ index: Int,
    _ r: Double, _ g: Double, _ b: Double, _ a: Double
) {
    let rpd = Unmanaged<MTLRenderPassDescriptor>.fromOpaque(desc).takeUnretainedValue()
    rpd.colorAttachments[index].clearColor = MTLClearColor(red: r, green: g, blue: b, alpha: a)
}

/// Release render pass descriptor
@_cdecl("metal_render_pass_descriptor_release")
public func metal_render_pass_descriptor_release(_ desc: UnsafeMutableRawPointer) {
    Unmanaged<MTLRenderPassDescriptor>.fromOpaque(desc).release()
}

// MARK: - Vertex Descriptor

/// Create a vertex descriptor
@_cdecl("metal_vertex_descriptor_create")
public func metal_vertex_descriptor_create() -> UnsafeMutableRawPointer {
    let desc = MTLVertexDescriptor()
    return Unmanaged.passRetained(desc).toOpaque()
}

/// Set attribute format, offset, and buffer index
@_cdecl("metal_vertex_descriptor_set_attribute")
public func metal_vertex_descriptor_set_attribute(
    _ desc: UnsafeMutableRawPointer,
    _ index: Int,
    _ format: UInt, // MTLVertexFormat raw value
    _ offset: Int,
    _ bufferIndex: Int
) {
    let vd = Unmanaged<MTLVertexDescriptor>.fromOpaque(desc).takeUnretainedValue()
    vd.attributes[index].format = MTLVertexFormat(rawValue: format) ?? .float2
    vd.attributes[index].offset = offset
    vd.attributes[index].bufferIndex = bufferIndex
}

/// Set layout stride and step function
@_cdecl("metal_vertex_descriptor_set_layout")
public func metal_vertex_descriptor_set_layout(
    _ desc: UnsafeMutableRawPointer,
    _ bufferIndex: Int,
    _ stride: Int,
    _ stepFunction: UInt // MTLVertexStepFunction raw value
) {
    let vd = Unmanaged<MTLVertexDescriptor>.fromOpaque(desc).takeUnretainedValue()
    vd.layouts[bufferIndex].stride = stride
    vd.layouts[bufferIndex].stepFunction = MTLVertexStepFunction(rawValue: stepFunction) ?? .perVertex
}

/// Release vertex descriptor
@_cdecl("metal_vertex_descriptor_release")
public func metal_vertex_descriptor_release(_ desc: UnsafeMutableRawPointer) {
    Unmanaged<MTLVertexDescriptor>.fromOpaque(desc).release()
}

// MARK: - Render Pipeline

/// Create a render pipeline descriptor
@_cdecl("metal_render_pipeline_descriptor_create")
public func metal_render_pipeline_descriptor_create() -> UnsafeMutableRawPointer {
    let desc = MTLRenderPipelineDescriptor()
    return Unmanaged.passRetained(desc).toOpaque()
}

/// Set vertex function
@_cdecl("metal_render_pipeline_descriptor_set_vertex_function")
public func metal_render_pipeline_descriptor_set_vertex_function(
    _ desc: UnsafeMutableRawPointer,
    _ function: UnsafeMutableRawPointer
) {
    let rpd = Unmanaged<MTLRenderPipelineDescriptor>.fromOpaque(desc).takeUnretainedValue()
    let func_ = Unmanaged<MTLFunction>.fromOpaque(function).takeUnretainedValue()
    rpd.vertexFunction = func_
}

/// Set fragment function
@_cdecl("metal_render_pipeline_descriptor_set_fragment_function")
public func metal_render_pipeline_descriptor_set_fragment_function(
    _ desc: UnsafeMutableRawPointer,
    _ function: UnsafeMutableRawPointer
) {
    let rpd = Unmanaged<MTLRenderPipelineDescriptor>.fromOpaque(desc).takeUnretainedValue()
    let func_ = Unmanaged<MTLFunction>.fromOpaque(function).takeUnretainedValue()
    rpd.fragmentFunction = func_
}

/// Set color attachment pixel format
@_cdecl("metal_render_pipeline_descriptor_set_color_attachment_pixel_format")
public func metal_render_pipeline_descriptor_set_color_attachment_pixel_format(
    _ desc: UnsafeMutableRawPointer,
    _ index: Int,
    _ format: UInt
) {
    let rpd = Unmanaged<MTLRenderPipelineDescriptor>.fromOpaque(desc).takeUnretainedValue()
    rpd.colorAttachments[index].pixelFormat = MTLPixelFormat(rawValue: format) ?? .bgra8Unorm
}

/// Set color attachment blending enabled
@_cdecl("metal_render_pipeline_descriptor_set_blending_enabled")
public func metal_render_pipeline_descriptor_set_blending_enabled(
    _ desc: UnsafeMutableRawPointer,
    _ index: Int,
    _ enabled: Bool
) {
    let rpd = Unmanaged<MTLRenderPipelineDescriptor>.fromOpaque(desc).takeUnretainedValue()
    rpd.colorAttachments[index].isBlendingEnabled = enabled
}

/// Set color attachment blend operations
@_cdecl("metal_render_pipeline_descriptor_set_blend_operations")
public func metal_render_pipeline_descriptor_set_blend_operations(
    _ desc: UnsafeMutableRawPointer,
    _ index: Int,
    _ rgbOp: UInt,
    _ alphaOp: UInt
) {
    let rpd = Unmanaged<MTLRenderPipelineDescriptor>.fromOpaque(desc).takeUnretainedValue()
    rpd.colorAttachments[index].rgbBlendOperation = MTLBlendOperation(rawValue: rgbOp) ?? .add
    rpd.colorAttachments[index].alphaBlendOperation = MTLBlendOperation(rawValue: alphaOp) ?? .add
}

/// Set color attachment blend factors
@_cdecl("metal_render_pipeline_descriptor_set_blend_factors")
public func metal_render_pipeline_descriptor_set_blend_factors(
    _ desc: UnsafeMutableRawPointer,
    _ index: Int,
    _ srcRgb: UInt,
    _ dstRgb: UInt,
    _ srcAlpha: UInt,
    _ dstAlpha: UInt
) {
    let rpd = Unmanaged<MTLRenderPipelineDescriptor>.fromOpaque(desc).takeUnretainedValue()
    rpd.colorAttachments[index].sourceRGBBlendFactor = MTLBlendFactor(rawValue: srcRgb) ?? .one
    rpd.colorAttachments[index].destinationRGBBlendFactor = MTLBlendFactor(rawValue: dstRgb) ?? .zero
    rpd.colorAttachments[index].sourceAlphaBlendFactor = MTLBlendFactor(rawValue: srcAlpha) ?? .one
    rpd.colorAttachments[index].destinationAlphaBlendFactor = MTLBlendFactor(rawValue: dstAlpha) ?? .zero
}

/// Set vertex descriptor on render pipeline descriptor
@_cdecl("metal_render_pipeline_descriptor_set_vertex_descriptor")
public func metal_render_pipeline_descriptor_set_vertex_descriptor(
    _ desc: UnsafeMutableRawPointer,
    _ vertexDescriptor: UnsafeMutableRawPointer
) {
    let rpd = Unmanaged<MTLRenderPipelineDescriptor>.fromOpaque(desc).takeUnretainedValue()
    let vd = Unmanaged<MTLVertexDescriptor>.fromOpaque(vertexDescriptor).takeUnretainedValue()
    rpd.vertexDescriptor = vd
}

/// Release render pipeline descriptor
@_cdecl("metal_render_pipeline_descriptor_release")
public func metal_render_pipeline_descriptor_release(_ desc: UnsafeMutableRawPointer) {
    Unmanaged<MTLRenderPipelineDescriptor>.fromOpaque(desc).release()
}

/// Create render pipeline state from descriptor
@_cdecl("metal_device_create_render_pipeline_state")
public func metal_device_create_render_pipeline_state(
    _ device: UnsafeMutableRawPointer,
    _ desc: UnsafeMutableRawPointer
) -> UnsafeMutableRawPointer? {
    let dev = Unmanaged<MTLDevice>.fromOpaque(device).takeUnretainedValue()
    let rpd = Unmanaged<MTLRenderPipelineDescriptor>.fromOpaque(desc).takeUnretainedValue()
    do {
        let state = try dev.makeRenderPipelineState(descriptor: rpd)
        return Unmanaged.passRetained(state).toOpaque()
    } catch {
        return nil
    }
}

/// Release render pipeline state
@_cdecl("metal_render_pipeline_state_release")
public func metal_render_pipeline_state_release(_ state: UnsafeMutableRawPointer) {
    Unmanaged<MTLRenderPipelineState>.fromOpaque(state).release()
}

// MARK: - Render Command Encoder

/// Create render command encoder
@_cdecl("metal_command_buffer_render_command_encoder")
public func metal_command_buffer_render_command_encoder(
    _ cmdBuffer: UnsafeMutableRawPointer,
    _ renderPass: UnsafeMutableRawPointer
) -> UnsafeMutableRawPointer? {
    let buf = Unmanaged<MTLCommandBuffer>.fromOpaque(cmdBuffer).takeUnretainedValue()
    let rpd = Unmanaged<MTLRenderPassDescriptor>.fromOpaque(renderPass).takeUnretainedValue()
    guard let encoder = buf.makeRenderCommandEncoder(descriptor: rpd) else {
        return nil
    }
    return Unmanaged.passRetained(encoder).toOpaque()
}

/// Set render pipeline state
@_cdecl("metal_render_encoder_set_pipeline_state")
public func metal_render_encoder_set_pipeline_state(
    _ encoder: UnsafeMutableRawPointer,
    _ state: UnsafeMutableRawPointer
) {
    let enc = Unmanaged<MTLRenderCommandEncoder>.fromOpaque(encoder).takeUnretainedValue()
    let pso = Unmanaged<MTLRenderPipelineState>.fromOpaque(state).takeUnretainedValue()
    enc.setRenderPipelineState(pso)
}

/// Set vertex buffer
@_cdecl("metal_render_encoder_set_vertex_buffer")
public func metal_render_encoder_set_vertex_buffer(
    _ encoder: UnsafeMutableRawPointer,
    _ buffer: UnsafeMutableRawPointer,
    _ offset: Int,
    _ index: Int
) {
    let enc = Unmanaged<MTLRenderCommandEncoder>.fromOpaque(encoder).takeUnretainedValue()
    let buf = Unmanaged<MTLBuffer>.fromOpaque(buffer).takeUnretainedValue()
    enc.setVertexBuffer(buf, offset: offset, index: index)
}

/// Set fragment buffer
@_cdecl("metal_render_encoder_set_fragment_buffer")
public func metal_render_encoder_set_fragment_buffer(
    _ encoder: UnsafeMutableRawPointer,
    _ buffer: UnsafeMutableRawPointer,
    _ offset: Int,
    _ index: Int
) {
    let enc = Unmanaged<MTLRenderCommandEncoder>.fromOpaque(encoder).takeUnretainedValue()
    let buf = Unmanaged<MTLBuffer>.fromOpaque(buffer).takeUnretainedValue()
    enc.setFragmentBuffer(buf, offset: offset, index: index)
}

/// Set fragment texture
@_cdecl("metal_render_encoder_set_fragment_texture")
public func metal_render_encoder_set_fragment_texture(
    _ encoder: UnsafeMutableRawPointer,
    _ texture: UnsafeMutableRawPointer,
    _ index: Int
) {
    let enc = Unmanaged<MTLRenderCommandEncoder>.fromOpaque(encoder).takeUnretainedValue()
    let tex = Unmanaged<MTLTexture>.fromOpaque(texture).takeUnretainedValue()
    enc.setFragmentTexture(tex, index: index)
}

/// Draw primitives
@_cdecl("metal_render_encoder_draw_primitives")
public func metal_render_encoder_draw_primitives(
    _ encoder: UnsafeMutableRawPointer,
    _ primitiveType: UInt, // MTLPrimitiveType raw value
    _ vertexStart: Int,
    _ vertexCount: Int
) {
    let enc = Unmanaged<MTLRenderCommandEncoder>.fromOpaque(encoder).takeUnretainedValue()
    enc.drawPrimitives(type: MTLPrimitiveType(rawValue: primitiveType) ?? .triangle, vertexStart: vertexStart, vertexCount: vertexCount)
}

/// End encoding
@_cdecl("metal_render_encoder_end_encoding")
public func metal_render_encoder_end_encoding(_ encoder: UnsafeMutableRawPointer) {
    let enc = Unmanaged<MTLRenderCommandEncoder>.fromOpaque(encoder).takeUnretainedValue()
    enc.endEncoding()
}

/// Release render command encoder
@_cdecl("metal_render_encoder_release")
public func metal_render_encoder_release(_ encoder: UnsafeMutableRawPointer) {
    Unmanaged<MTLRenderCommandEncoder>.fromOpaque(encoder).release()
}

// MARK: - NSView Helpers

import AppKit

/// Set wantsLayer on a view (YES)
@_cdecl("nsview_set_wants_layer")
public func nsview_set_wants_layer(_ view: UnsafeMutableRawPointer) {
    let nsView = Unmanaged<NSView>.fromOpaque(view).takeUnretainedValue()
    nsView.wantsLayer = true
}

/// Set the layer on a view
@_cdecl("nsview_set_layer")
public func nsview_set_layer(_ view: UnsafeMutableRawPointer, _ layer: UnsafeMutableRawPointer) {
    let nsView = Unmanaged<NSView>.fromOpaque(view).takeUnretainedValue()
    let caLayer = Unmanaged<CAMetalLayer>.fromOpaque(layer).takeUnretainedValue()
    nsView.layer = caLayer
}
