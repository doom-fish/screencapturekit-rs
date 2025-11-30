// Screenshot Manager APIs (macOS 14.0+)

import CoreGraphics
import CoreMedia
import Foundation
import ScreenCaptureKit
import UniformTypeIdentifiers

// MARK: - Screenshot Manager (macOS 14.0+)

@available(macOS 14.0, *)
@_cdecl("sc_screenshot_manager_capture_image")
public func captureScreenshot(
    _ contentFilter: OpaquePointer,
    _ config: OpaquePointer,
    _ callback: @escaping @convention(c) (OpaquePointer?, UnsafePointer<CChar>?, UnsafeMutableRawPointer?) -> Void,
    _ userData: UnsafeMutableRawPointer?
) {
    let filter: SCContentFilter = unretained(contentFilter)
    let configuration: SCStreamConfiguration = unretained(config)

    Task {
        do {
            let image = try await SCScreenshotManager.captureImage(
                contentFilter: filter,
                configuration: configuration
            )
            callback(retain(image), nil, userData)
        } catch {
            let bridgeError = SCBridgeError.screenshotError(error.localizedDescription)
            bridgeError.description.withCString { callback(nil, $0, userData) }
        }
    }
}

@available(macOS 14.0, *)
@_cdecl("sc_screenshot_manager_capture_sample_buffer")
public func captureScreenshotSampleBuffer(
    _ contentFilter: OpaquePointer,
    _ config: OpaquePointer,
    _ callback: @escaping @convention(c) (OpaquePointer?, UnsafePointer<CChar>?, UnsafeMutableRawPointer?) -> Void,
    _ userData: UnsafeMutableRawPointer?
) {
    let filter: SCContentFilter = unretained(contentFilter)
    let configuration: SCStreamConfiguration = unretained(config)

    Task {
        do {
            let sampleBuffer = try await SCScreenshotManager.captureSampleBuffer(
                contentFilter: filter,
                configuration: configuration
            )
            let retained = Unmanaged.passRetained(sampleBuffer as AnyObject)
            callback(OpaquePointer(retained.toOpaque()), nil, userData)
        } catch {
            let bridgeError = SCBridgeError.screenshotError(error.localizedDescription)
            bridgeError.description.withCString { callback(nil, $0, userData) }
        }
    }
}

// MARK: - Capture image in rect (macOS 15.2+)

#if compiler(>=6.0)
    @_cdecl("sc_screenshot_manager_capture_image_in_rect")
    public func captureScreenshotInRect(
        _ x: Double,
        _ y: Double,
        _ width: Double,
        _ height: Double,
        _ callback: @escaping @convention(c) (OpaquePointer?, UnsafePointer<CChar>?, UnsafeMutableRawPointer?) -> Void,
        _ userData: UnsafeMutableRawPointer?
    ) {
        if #available(macOS 15.2, *) {
            let rect = CGRect(x: x, y: y, width: width, height: height)
            Task {
                do {
                    let image = try await SCScreenshotManager.captureImage(in: rect)
                    callback(retain(image), nil, userData)
                } catch {
                    let bridgeError = SCBridgeError.screenshotError(error.localizedDescription)
                    bridgeError.description.withCString { callback(nil, $0, userData) }
                }
            }
        } else {
            let bridgeError = SCBridgeError.screenshotError("captureImageInRect requires macOS 15.2+")
            bridgeError.description.withCString { callback(nil, $0, userData) }
        }
    }
#else
    @_cdecl("sc_screenshot_manager_capture_image_in_rect")
    public func captureScreenshotInRect(
        _: Double,
        _: Double,
        _: Double,
        _: Double,
        _ callback: @escaping @convention(c) (OpaquePointer?, UnsafePointer<CChar>?, UnsafeMutableRawPointer?) -> Void,
        _ userData: UnsafeMutableRawPointer?
    ) {
        let bridgeError = SCBridgeError.screenshotError("captureImageInRect requires macOS 15.2+")
        bridgeError.description.withCString { callback(nil, $0, userData) }
    }
#endif

// MARK: - SCScreenshotConfiguration (macOS 26.0+)

#if compiler(>=6.0)
    @_cdecl("sc_screenshot_configuration_create")
    public func createScreenshotConfiguration() -> OpaquePointer? {
        if #available(macOS 26.0, *) {
            let config = SCScreenshotConfiguration()
            return retain(config)
        }
        return nil
    }

    @_cdecl("sc_screenshot_configuration_set_width")
    public func setScreenshotConfigurationWidth(_ config: OpaquePointer, _ width: Int) {
        if #available(macOS 26.0, *) {
            let c: SCScreenshotConfiguration = unretained(config)
            c.width = width
        }
    }

    @_cdecl("sc_screenshot_configuration_set_height")
    public func setScreenshotConfigurationHeight(_ config: OpaquePointer, _ height: Int) {
        if #available(macOS 26.0, *) {
            let c: SCScreenshotConfiguration = unretained(config)
            c.height = height
        }
    }

    @_cdecl("sc_screenshot_configuration_set_shows_cursor")
    public func setScreenshotConfigurationShowsCursor(_ config: OpaquePointer, _ showsCursor: Bool) {
        if #available(macOS 26.0, *) {
            let c: SCScreenshotConfiguration = unretained(config)
            c.showsCursor = showsCursor
        }
    }

    @_cdecl("sc_screenshot_configuration_set_source_rect")
    public func setScreenshotConfigurationSourceRect(_ config: OpaquePointer, _ x: Double, _ y: Double, _ width: Double, _ height: Double) {
        if #available(macOS 26.0, *) {
            let c: SCScreenshotConfiguration = unretained(config)
            c.sourceRect = CGRect(x: x, y: y, width: width, height: height)
        }
    }

    @_cdecl("sc_screenshot_configuration_set_destination_rect")
    public func setScreenshotConfigurationDestinationRect(_ config: OpaquePointer, _ x: Double, _ y: Double, _ width: Double, _ height: Double) {
        if #available(macOS 26.0, *) {
            let c: SCScreenshotConfiguration = unretained(config)
            c.destinationRect = CGRect(x: x, y: y, width: width, height: height)
        }
    }

    @_cdecl("sc_screenshot_configuration_set_ignore_shadows")
    public func setScreenshotConfigurationIgnoreShadows(_ config: OpaquePointer, _ ignoreShadows: Bool) {
        if #available(macOS 26.0, *) {
            let c: SCScreenshotConfiguration = unretained(config)
            c.ignoreShadows = ignoreShadows
        }
    }

    @_cdecl("sc_screenshot_configuration_set_ignore_clipping")
    public func setScreenshotConfigurationIgnoreClipping(_ config: OpaquePointer, _ ignoreClipping: Bool) {
        if #available(macOS 26.0, *) {
            let c: SCScreenshotConfiguration = unretained(config)
            c.ignoreClipping = ignoreClipping
        }
    }

    @_cdecl("sc_screenshot_configuration_set_include_child_windows")
    public func setScreenshotConfigurationIncludeChildWindows(_ config: OpaquePointer, _ includeChildWindows: Bool) {
        if #available(macOS 26.0, *) {
            let c: SCScreenshotConfiguration = unretained(config)
            c.includeChildWindows = includeChildWindows
        }
    }

    @_cdecl("sc_screenshot_configuration_set_display_intent")
    public func setScreenshotConfigurationDisplayIntent(_ config: OpaquePointer, _ displayIntent: Int32) {
        if #available(macOS 26.0, *) {
            let c: SCScreenshotConfiguration = unretained(config)
            switch displayIntent {
            case 0: c.displayIntent = .canonical
            case 1: c.displayIntent = .local
            default: break
            }
        }
    }

    @_cdecl("sc_screenshot_configuration_set_dynamic_range")
    public func setScreenshotConfigurationDynamicRange(_ config: OpaquePointer, _ dynamicRange: Int32) {
        if #available(macOS 26.0, *) {
            let c: SCScreenshotConfiguration = unretained(config)
            switch dynamicRange {
            case 0: c.dynamicRange = .sdr
            case 1: c.dynamicRange = .hdr
            case 2: c.dynamicRange = .bothSDRAndHDR
            default: break
            }
        }
    }

    @_cdecl("sc_screenshot_configuration_set_file_url")
    public func setScreenshotConfigurationFileURL(_ config: OpaquePointer, _ path: UnsafePointer<CChar>) {
        if #available(macOS 26.0, *) {
            let c: SCScreenshotConfiguration = unretained(config)
            let pathString = String(cString: path)
            c.fileURL = URL(fileURLWithPath: pathString)
        }
    }

    @_cdecl("sc_screenshot_configuration_release")
    public func releaseScreenshotConfiguration(_ config: OpaquePointer) {
        release(config)
    }

    // MARK: - Content Type Support (macOS 26.0+)

    /// Set the content type (output format) using UTType identifier
    @_cdecl("sc_screenshot_configuration_set_content_type")
    public func setScreenshotConfigurationContentType(_ config: OpaquePointer, _ identifier: UnsafePointer<CChar>) {
        if #available(macOS 26.0, *) {
            let c: SCScreenshotConfiguration = unretained(config)
            let typeIdentifier = String(cString: identifier)
            if let utType = UTType(typeIdentifier) {
                // UTTypeReference is bridged from UTType
                c.contentType = utType as UTTypeReference
            }
        }
    }

    /// Get the content type (output format) as UTType identifier
    @_cdecl("sc_screenshot_configuration_get_content_type")
    public func getScreenshotConfigurationContentType(_ config: OpaquePointer, _ buffer: UnsafeMutablePointer<CChar>, _ bufferSize: Int) -> Bool {
        if #available(macOS 26.0, *) {
            let c: SCScreenshotConfiguration = unretained(config)
            // UTTypeReference is type-aliased to UTType
            let utType = c.contentType as UTType
            let identifier = utType.identifier
            return identifier.withCString { src in
                strlcpy(buffer, src, bufferSize)
                return true
            }
        }
        return false
    }

    /// Get the number of supported content types
    @_cdecl("sc_screenshot_configuration_get_supported_content_types_count")
    public func getScreenshotConfigurationSupportedContentTypesCount() -> Int {
        if #available(macOS 26.0, *) {
            return SCScreenshotConfiguration.supportedContentTypes.count
        }
        return 0
    }

    /// Get a supported content type at index as UTType identifier
    @_cdecl("sc_screenshot_configuration_get_supported_content_type_at")
    public func getScreenshotConfigurationSupportedContentTypeAt(_ index: Int, _ buffer: UnsafeMutablePointer<CChar>, _ bufferSize: Int) -> Bool {
        if #available(macOS 26.0, *) {
            let types = SCScreenshotConfiguration.supportedContentTypes
            guard index >= 0, index < types.count else { return false }
            let identifier = types[index].identifier
            return identifier.withCString { src in
                strlcpy(buffer, src, bufferSize)
                return true
            }
        }
        return false
    }

    // MARK: - SCScreenshotOutput (macOS 26.0+)

    @_cdecl("sc_screenshot_output_get_sdr_image")
    public func getScreenshotOutputSDRImage(_ output: OpaquePointer) -> OpaquePointer? {
        if #available(macOS 26.0, *) {
            let o: SCScreenshotOutput = unretained(output)
            if let image = o.sdrImage {
                return retain(image)
            }
        }
        return nil
    }

    @_cdecl("sc_screenshot_output_get_hdr_image")
    public func getScreenshotOutputHDRImage(_ output: OpaquePointer) -> OpaquePointer? {
        if #available(macOS 26.0, *) {
            let o: SCScreenshotOutput = unretained(output)
            if let image = o.hdrImage {
                return retain(image)
            }
        }
        return nil
    }

    @_cdecl("sc_screenshot_output_get_file_url")
    public func getScreenshotOutputFileURL(_ output: OpaquePointer, _ buffer: UnsafeMutablePointer<CChar>, _ bufferSize: Int) -> Bool {
        if #available(macOS 26.0, *) {
            let o: SCScreenshotOutput = unretained(output)
            if let url = o.fileURL, let pathString = url.path as String?, let cString = pathString.cString(using: .utf8) {
                strncpy(buffer, cString, bufferSize - 1)
                buffer[bufferSize - 1] = 0
                return true
            }
        }
        return false
    }

    @_cdecl("sc_screenshot_output_release")
    public func releaseScreenshotOutput(_ output: OpaquePointer) {
        release(output)
    }

    // MARK: - New Screenshot Capture API (macOS 26.0+)

    @_cdecl("sc_screenshot_manager_capture_screenshot")
    public func captureScreenshotWithConfiguration(
        _ contentFilter: OpaquePointer,
        _ config: OpaquePointer,
        _ callback: @escaping @convention(c) (OpaquePointer?, UnsafePointer<CChar>?, UnsafeMutableRawPointer?) -> Void,
        _ userData: UnsafeMutableRawPointer?
    ) {
        if #available(macOS 26.0, *) {
            let filter: SCContentFilter = unretained(contentFilter)
            let configuration: SCScreenshotConfiguration = unretained(config)

            Task {
                do {
                    let output = try await SCScreenshotManager.captureScreenshot(
                        contentFilter: filter,
                        configuration: configuration
                    )
                    callback(retain(output), nil, userData)
                } catch {
                    let bridgeError = SCBridgeError.screenshotError(error.localizedDescription)
                    bridgeError.description.withCString { callback(nil, $0, userData) }
                }
            }
        } else {
            let bridgeError = SCBridgeError.screenshotError("captureScreenshot requires macOS 26.0+")
            bridgeError.description.withCString { callback(nil, $0, userData) }
        }
    }

    @_cdecl("sc_screenshot_manager_capture_screenshot_in_rect")
    public func captureScreenshotInRectWithConfiguration(
        _ x: Double,
        _ y: Double,
        _ width: Double,
        _ height: Double,
        _ config: OpaquePointer,
        _ callback: @escaping @convention(c) (OpaquePointer?, UnsafePointer<CChar>?, UnsafeMutableRawPointer?) -> Void,
        _ userData: UnsafeMutableRawPointer?
    ) {
        if #available(macOS 26.0, *) {
            let rect = CGRect(x: x, y: y, width: width, height: height)
            let configuration: SCScreenshotConfiguration = unretained(config)

            Task {
                do {
                    let output = try await SCScreenshotManager.captureScreenshot(
                        rect: rect,
                        configuration: configuration
                    )
                    callback(retain(output), nil, userData)
                } catch {
                    let bridgeError = SCBridgeError.screenshotError(error.localizedDescription)
                    bridgeError.description.withCString { callback(nil, $0, userData) }
                }
            }
        } else {
            let bridgeError = SCBridgeError.screenshotError("captureScreenshotInRect requires macOS 26.0+")
            bridgeError.description.withCString { callback(nil, $0, userData) }
        }
    }
#else
    // Stubs for older compilers
    @_cdecl("sc_screenshot_configuration_create")
    public func createScreenshotConfiguration() -> OpaquePointer? { nil }

    @_cdecl("sc_screenshot_configuration_set_width")
    public func setScreenshotConfigurationWidth(_: OpaquePointer, _: Int) {}

    @_cdecl("sc_screenshot_configuration_set_height")
    public func setScreenshotConfigurationHeight(_: OpaquePointer, _: Int) {}

    @_cdecl("sc_screenshot_configuration_set_shows_cursor")
    public func setScreenshotConfigurationShowsCursor(_: OpaquePointer, _: Bool) {}

    @_cdecl("sc_screenshot_configuration_set_source_rect")
    public func setScreenshotConfigurationSourceRect(_: OpaquePointer, _: Double, _: Double, _: Double, _: Double) {}

    @_cdecl("sc_screenshot_configuration_set_destination_rect")
    public func setScreenshotConfigurationDestinationRect(_: OpaquePointer, _: Double, _: Double, _: Double, _: Double) {}

    @_cdecl("sc_screenshot_configuration_set_ignore_shadows")
    public func setScreenshotConfigurationIgnoreShadows(_: OpaquePointer, _: Bool) {}

    @_cdecl("sc_screenshot_configuration_set_ignore_clipping")
    public func setScreenshotConfigurationIgnoreClipping(_: OpaquePointer, _: Bool) {}

    @_cdecl("sc_screenshot_configuration_set_include_child_windows")
    public func setScreenshotConfigurationIncludeChildWindows(_: OpaquePointer, _: Bool) {}

    @_cdecl("sc_screenshot_configuration_set_display_intent")
    public func setScreenshotConfigurationDisplayIntent(_: OpaquePointer, _: Int32) {}

    @_cdecl("sc_screenshot_configuration_set_dynamic_range")
    public func setScreenshotConfigurationDynamicRange(_: OpaquePointer, _: Int32) {}

    @_cdecl("sc_screenshot_configuration_set_file_url")
    public func setScreenshotConfigurationFileURL(_: OpaquePointer, _: UnsafePointer<CChar>) {}

    @_cdecl("sc_screenshot_configuration_release")
    public func releaseScreenshotConfiguration(_: OpaquePointer) {}

    @_cdecl("sc_screenshot_output_get_sdr_image")
    public func getScreenshotOutputSDRImage(_: OpaquePointer) -> OpaquePointer? { nil }

    @_cdecl("sc_screenshot_output_get_hdr_image")
    public func getScreenshotOutputHDRImage(_: OpaquePointer) -> OpaquePointer? { nil }

    @_cdecl("sc_screenshot_output_get_file_url")
    public func getScreenshotOutputFileURL(_: OpaquePointer, _: UnsafeMutablePointer<CChar>, _: Int) -> Bool { false }

    @_cdecl("sc_screenshot_output_release")
    public func releaseScreenshotOutput(_: OpaquePointer) {}

    @_cdecl("sc_screenshot_manager_capture_screenshot")
    public func captureScreenshotWithConfiguration(
        _: OpaquePointer,
        _: OpaquePointer,
        _ callback: @escaping @convention(c) (OpaquePointer?, UnsafePointer<CChar>?, UnsafeMutableRawPointer?) -> Void,
        _ userData: UnsafeMutableRawPointer?
    ) {
        let bridgeError = SCBridgeError.screenshotError("captureScreenshot requires macOS 26.0+")
        bridgeError.description.withCString { callback(nil, $0, userData) }
    }

    @_cdecl("sc_screenshot_manager_capture_screenshot_in_rect")
    public func captureScreenshotInRectWithConfiguration(
        _: Double,
        _: Double,
        _: Double,
        _: Double,
        _: OpaquePointer,
        _ callback: @escaping @convention(c) (OpaquePointer?, UnsafePointer<CChar>?, UnsafeMutableRawPointer?) -> Void,
        _ userData: UnsafeMutableRawPointer?
    ) {
        let bridgeError = SCBridgeError.screenshotError("captureScreenshotInRect requires macOS 26.0+")
        bridgeError.description.withCString { callback(nil, $0, userData) }
    }
#endif
