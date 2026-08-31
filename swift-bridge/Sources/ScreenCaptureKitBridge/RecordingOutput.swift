// Recording Output APIs (macOS 15.0+)
// Stub implementation for macOS < 15.0

import AVFoundation
import Foundation
import ObjectiveC
import ScreenCaptureKit

// MARK: - Recording Output (macOS 15.0+)

// Callback type definitions for recording delegate
public typealias RecordingStartedCallback = @convention(c) (UnsafeMutableRawPointer?) -> Void
public typealias RecordingFailedCallback = @convention(c) (UnsafeMutableRawPointer?, Int32, UnsafePointer<CChar>) -> Void
public typealias RecordingFinishedCallback = @convention(c) (UnsafeMutableRawPointer?) -> Void
public typealias RecordingContextRelease = @convention(c) (UnsafeMutableRawPointer?) -> Void

@_cdecl("sc_recording_output_is_available")
public func recordingOutputIsAvailable() -> Bool {
    if #available(macOS 15.0, *) {
        return true
    }
    return false
}

#if SCREENCAPTUREKIT_HAS_MACOS15_SDK
    // Full implementation for macOS 15 SDK

    // Stable wire codes for `AVVideoCodecType` / `AVFileType`.
    //
    // `availableVideoCodecTypes` and `availableOutputFileTypes` are open lists
    // that grow with every macOS release. Mapping an unrecognised entry to a
    // sentinel used to make the `_at` accessor disagree with the `_count`
    // accessor, so Rust silently dropped entries. Every value now round-trips:
    // known ones through the tables below, unknown ones through
    // `unknownCode` so the index-addressed list still has `count` entries.
    // `SCRecordingOutputConfiguration`'s `outputURL` / `videoCodecType` /
    // `outputFileType` are declared `nonnull` and therefore import as
    // non-optional Swift values, but a freshly constructed configuration
    // leaves all three nil. Touching them directly traps at runtime, so every
    // read goes through KVC, which hands back a genuine `Optional`.
    @available(macOS 15.0, *)
    private enum RecordingConfigurationValue {
        static func outputURL(_ config: SCRecordingOutputConfiguration) -> URL? {
            config.value(forKey: "outputURL") as? URL
        }

        static func videoCodecType(_ config: SCRecordingOutputConfiguration) -> AVVideoCodecType? {
            (config.value(forKey: "videoCodecType") as? String).map(AVVideoCodecType.init(rawValue:))
        }

        static func outputFileType(_ config: SCRecordingOutputConfiguration) -> AVFileType? {
            (config.value(forKey: "outputFileType") as? String).map(AVFileType.init(rawValue:))
        }
    }

    @available(macOS 15.0, *)
    private enum RecordingWireCode {
        static let unknown: Int32 = -1

        static let videoCodecs: [(Int32, AVVideoCodecType)] = {
            var table: [(Int32, AVVideoCodecType)] = [
                (0, .h264),
                (1, .hevc),
                (2, .jpeg),
                (3, .proRes422),
                (4, .proRes4444),
                (5, .hevcWithAlpha),
                (6, .proRes422HQ),
                (7, .proRes422LT),
                (8, .proRes422Proxy),
            ]
            return table
        }()

        static let fileTypes: [(Int32, AVFileType)] = [
            (0, .mp4),
            (1, .mov),
            (2, .m4v),
            (3, .m4a),
            (4, .mobile3GPP),
        ]

        static func code(for codec: AVVideoCodecType) -> Int32 {
            videoCodecs.first { $0.1 == codec }?.0 ?? unknown
        }

        static func codec(for code: Int32) -> AVVideoCodecType? {
            videoCodecs.first { $0.0 == code }?.1
        }

        static func code(for fileType: AVFileType) -> Int32 {
            fileTypes.first { $0.1 == fileType }?.0 ?? unknown
        }

        static func fileType(for code: Int32) -> AVFileType? {
            fileTypes.first { $0.0 == code }?.1
        }
    }

    @available(macOS 15.0, *)
    private class RecordingDelegate: NSObject, SCRecordingOutputDelegate {
        var startedCallback: RecordingStartedCallback?
        var failedCallback: RecordingFailedCallback?
        var finishedCallback: RecordingFinishedCallback?
        var contextRelease: RecordingContextRelease?
        var context: UnsafeMutableRawPointer?
        private let stateLock = NSLock()
        private var started = false
        private var terminal = false
        private var terminalHandlers: [PendingHandler] = []
        private var nextHandlerID: UInt64 = 0

        private struct PendingHandler {
            let id: UInt64
            let run: () -> Void
        }

        /// How long a terminal wait tolerates `didStartRecording` not having
        /// been delivered yet before concluding the recording never began.
        private static let startGrace = DispatchTimeInterval.milliseconds(500)

        /// Upper bound on waiting for finalization once the recording is known
        /// to have started. A start that races the stream's stop can leave
        /// ScreenCaptureKit without a terminal event to deliver, and blocking
        /// forever there is worse than reporting a removal that already
        /// succeeded natively.
        private static let terminalGrace = DispatchTimeInterval.seconds(3)

        deinit {
            contextRelease?(context)
        }

        func recordingOutputDidStartRecording(_ output: SCRecordingOutput) {
            stateLock.lock()
            if terminal {
                stateLock.unlock()
                return
            }
            started = true
            stateLock.unlock()
            ActiveRecordingDelegates.retain(self, for: output)
            if let cb = startedCallback {
                cb(context)
            }
        }

        func recordingOutput(_ output: SCRecordingOutput, didFailWithError error: Error) {
            guard completeTerminalHandlers() else { return }
            if let cb = failedCallback {
                let errorCode = extractStreamErrorCode(error)
                error.localizedDescription.withCString { cb(context, errorCode, $0) }
            }
            ActiveRecordingDelegates.release(self, for: output)
        }

        func recordingOutputDidFinishRecording(_ output: SCRecordingOutput) {
            guard completeTerminalHandlers() else { return }
            if let cb = finishedCallback {
                cb(context)
            }
            ActiveRecordingDelegates.release(self, for: output)
        }

        func afterTerminal(_ handler: @escaping () -> Void) {
            stateLock.lock()
            if terminal {
                stateLock.unlock()
                handler()
                return
            }
            let id = nextHandlerID
            nextHandlerID += 1
            terminalHandlers.append(PendingHandler(id: id, run: handler))
            let awaitingStart = !started
            stateLock.unlock()

            // An output removed before capture produced a frame never receives
            // a terminal event, so waiting on it would stall until the caller's
            // timeout. But `didStartRecording` is also merely late whenever the
            // removal races a start that already landed natively, and giving up
            // there would report completion for a movie still being finalized.
            // Waiting out a short grace period tells the two apart.
            scheduleResolve(id, after: awaitingStart ? Self.startGrace : Self.terminalGrace,
                            awaitingStart: awaitingStart)
        }

        private func scheduleResolve(
            _ id: UInt64,
            after delay: DispatchTimeInterval,
            awaitingStart: Bool
        ) {
            DispatchQueue.global().asyncAfter(deadline: .now() + delay) {
                self.resolve(id, awaitingStart: awaitingStart)
            }
        }

        private func resolve(_ id: UInt64, awaitingStart: Bool) {
            stateLock.lock()
            guard !terminal,
                let index = terminalHandlers.firstIndex(where: { $0.id == id })
            else {
                stateLock.unlock()
                return
            }
            if awaitingStart && started {
                stateLock.unlock()
                scheduleResolve(id, after: Self.terminalGrace, awaitingStart: false)
                return
            }
            let handler = terminalHandlers.remove(at: index).run
            stateLock.unlock()
            handler()
        }

        private func completeTerminalHandlers() -> Bool {
            stateLock.lock()
            if terminal {
                stateLock.unlock()
                return false
            }
            terminal = true
            let handlers = terminalHandlers
            terminalHandlers.removeAll()
            stateLock.unlock()
            for handler in handlers {
                handler.run()
            }
            return true
        }
    }

    // `SCRecordingOutput` does not retain its delegate. Associate it strongly
    // with the output so it follows the output's real ARC lifetime, including
    // clones and ScreenCaptureKit's own finalisation retain.
    @available(macOS 15.0, *)
    private enum RecordingDelegateAssociation {
        nonisolated(unsafe) static var key: UInt8 = 0

        static func retain(_ delegate: RecordingDelegate, for output: SCRecordingOutput) {
            objc_setAssociatedObject(
                output,
                &key,
                delegate,
                .OBJC_ASSOCIATION_RETAIN_NONATOMIC
            )
        }

        static func afterTerminal(
            for output: SCRecordingOutput,
            _ handler: @escaping () -> Void
        ) {
            let delegate =
                objc_getAssociatedObject(output, &key) as? RecordingDelegate
                    ?? ActiveRecordingDelegates.delegate(for: output)
            delegate?.afterTerminal(handler) ?? handler()
        }
    }

    @available(macOS 15.0, *)
    func afterRecordingOutputTerminal(
        _ output: SCRecordingOutput,
        _ handler: @escaping () -> Void
    ) {
        RecordingDelegateAssociation.afterTerminal(for: output, handler)
    }

    /// A removed output may be deallocated before ScreenCaptureKit finishes
    /// the movie, but its delegate must survive to receive the terminal event.
    /// Only recordings that actually started enter this table.
    ///
    /// The entry retains the output as well as the delegate. The key is the
    /// output's address, so letting the output deallocate while its entry
    /// lives would leave a dangling key: the allocator could hand the same
    /// address to a new `SCRecordingOutput`, and inserting that one would drop
    /// the last reference to the old delegate — running its `deinit`, and so
    /// releasing the Rust-side context — while ScreenCaptureKit may still
    /// message it.
    @available(macOS 15.0, *)
    private enum ActiveRecordingDelegates {
        private struct Entry {
            let output: SCRecordingOutput
            let delegate: RecordingDelegate
        }

        private static let lock = NSLock()
        private nonisolated(unsafe) static var delegates: [ObjectIdentifier: Entry] = [:]

        static func retain(_ delegate: RecordingDelegate, for output: SCRecordingOutput) {
            lock.lock()
            delegates[ObjectIdentifier(output)] = Entry(output: output, delegate: delegate)
            lock.unlock()
        }

        static func release(_ delegate: RecordingDelegate, for output: SCRecordingOutput) {
            lock.lock()
            let key = ObjectIdentifier(output)
            if delegates[key]?.delegate === delegate {
                delegates.removeValue(forKey: key)
            }
            lock.unlock()
        }

        static func delegate(for output: SCRecordingOutput) -> RecordingDelegate? {
            lock.lock()
            defer { lock.unlock() }
            return delegates[ObjectIdentifier(output)]?.delegate
        }
    }

    @available(macOS 15.0, *)
    @_cdecl("sc_recording_output_configuration_create")
    public func createRecordingOutputConfiguration() -> OpaquePointer {
        let config = SCRecordingOutputConfiguration()
        let box = Box(config)
        return retain(box)
    }

    @available(macOS 15.0, *)
    @_cdecl("sc_recording_output_configuration_set_output_url")
    public func setRecordingOutputURL(_ config: OpaquePointer, _ path: UnsafePointer<CChar>) {
        let box: Box<SCRecordingOutputConfiguration> = unretained(config)
        let pathString = String(cString: path)
        box.value.outputURL = URL(fileURLWithPath: pathString)
    }

    @available(macOS 15.0, *)
    @_cdecl("sc_recording_output_configuration_get_output_url")
    public func getRecordingOutputURL(_ config: OpaquePointer, _ buffer: UnsafeMutablePointer<CChar>, _ bufferSize: Int) -> Bool {
        let box: Box<SCRecordingOutputConfiguration> = unretained(config)
        guard let url = RecordingConfigurationValue.outputURL(box.value) else { return false }
        return writeCString(url.path, into: buffer, bufferSize: bufferSize)
    }

    @available(macOS 15.0, *)
    @_cdecl("sc_recording_output_configuration_get_output_path_owned")
    public func getRecordingOutputPathOwned(
        _ config: OpaquePointer
    ) -> UnsafeMutablePointer<CChar>? {
        let box: Box<SCRecordingOutputConfiguration> = unretained(config)
        guard let url = RecordingConfigurationValue.outputURL(box.value) else { return nil }
        return url.withUnsafeFileSystemRepresentation { path -> UnsafeMutablePointer<CChar>? in
            guard let path else { return nil }
            return strdup(path)
        }
    }

    @available(macOS 15.0, *)
    @_cdecl("sc_recording_output_configuration_set_video_codec")
    public func setRecordingOutputVideoCodec(_ config: OpaquePointer, _ codec: Int32) {
        let box: Box<SCRecordingOutputConfiguration> = unretained(config)
        guard let value = RecordingWireCode.codec(for: codec) else { return }
        box.value.videoCodecType = value
    }

    @available(macOS 15.0, *)
    @_cdecl("sc_recording_output_configuration_set_video_codec_identifier")
    public func setRecordingOutputVideoCodecIdentifier(
        _ config: OpaquePointer,
        _ identifier: UnsafePointer<CChar>
    ) {
        let box: Box<SCRecordingOutputConfiguration> = unretained(config)
        box.value.videoCodecType = AVVideoCodecType(rawValue: String(cString: identifier))
    }

    @available(macOS 15.0, *)
    @_cdecl("sc_recording_output_configuration_get_video_codec_identifier_owned")
    public func getRecordingOutputVideoCodecIdentifierOwned(
        _ config: OpaquePointer
    ) -> UnsafeMutablePointer<CChar>? {
        let box: Box<SCRecordingOutputConfiguration> = unretained(config)
        let codec = RecordingConfigurationValue.videoCodecType(box.value) ?? .h264
        return strdup(codec.rawValue)
    }

    @available(macOS 15.0, *)
    @_cdecl("sc_recording_output_configuration_get_video_codec")
    public func getRecordingOutputVideoCodec(_ config: OpaquePointer) -> Int32 {
        let box: Box<SCRecordingOutputConfiguration> = unretained(config)
        guard let codec = RecordingConfigurationValue.videoCodecType(box.value) else {
            return RecordingWireCode.code(for: .h264)
        }
        return RecordingWireCode.code(for: codec)
    }

    @available(macOS 15.0, *)
    @_cdecl("sc_recording_output_configuration_set_output_file_type")
    public func setRecordingOutputFileType(_ config: OpaquePointer, _ fileType: Int32) {
        let box: Box<SCRecordingOutputConfiguration> = unretained(config)
        guard let value = RecordingWireCode.fileType(for: fileType) else { return }
        box.value.outputFileType = value
    }

    @available(macOS 15.0, *)
    @_cdecl("sc_recording_output_configuration_set_output_file_type_identifier")
    public func setRecordingOutputFileTypeIdentifier(
        _ config: OpaquePointer,
        _ identifier: UnsafePointer<CChar>
    ) {
        let box: Box<SCRecordingOutputConfiguration> = unretained(config)
        box.value.outputFileType = AVFileType(rawValue: String(cString: identifier))
    }

    @available(macOS 15.0, *)
    @_cdecl("sc_recording_output_configuration_get_output_file_type_identifier_owned")
    public func getRecordingOutputFileTypeIdentifierOwned(
        _ config: OpaquePointer
    ) -> UnsafeMutablePointer<CChar>? {
        let box: Box<SCRecordingOutputConfiguration> = unretained(config)
        let fileType = RecordingConfigurationValue.outputFileType(box.value) ?? .mp4
        return strdup(fileType.rawValue)
    }

    @available(macOS 15.0, *)
    @_cdecl("sc_recording_output_configuration_get_output_file_type")
    public func getRecordingOutputFileType(_ config: OpaquePointer) -> Int32 {
        let box: Box<SCRecordingOutputConfiguration> = unretained(config)
        guard let fileType = RecordingConfigurationValue.outputFileType(box.value) else {
            return RecordingWireCode.code(for: .mp4)
        }
        return RecordingWireCode.code(for: fileType)
    }

    @available(macOS 15.0, *)
    @_cdecl("sc_recording_output_configuration_get_available_video_codecs_count")
    public func getRecordingOutputAvailableVideoCodecsCount(_ config: OpaquePointer) -> Int {
        let box: Box<SCRecordingOutputConfiguration> = unretained(config)
        return box.value.availableVideoCodecTypes.count
    }

    @available(macOS 15.0, *)
    @_cdecl("sc_recording_output_configuration_get_available_video_codec_at")
    public func getRecordingOutputAvailableVideoCodecAt(_ config: OpaquePointer, _ index: Int) -> Int32 {
        let box: Box<SCRecordingOutputConfiguration> = unretained(config)
        guard index >= 0, index < box.value.availableVideoCodecTypes.count else {
            return RecordingWireCode.unknown
        }
        return RecordingWireCode.code(for: box.value.availableVideoCodecTypes[index])
    }

    @available(macOS 15.0, *)
    @_cdecl("sc_recording_output_configuration_get_available_video_codec_identifier_at_owned")
    public func getRecordingOutputAvailableVideoCodecIdentifierAtOwned(
        _ config: OpaquePointer,
        _ index: Int
    ) -> UnsafeMutablePointer<CChar>? {
        let box: Box<SCRecordingOutputConfiguration> = unretained(config)
        guard index >= 0, index < box.value.availableVideoCodecTypes.count else { return nil }
        return strdup(box.value.availableVideoCodecTypes[index].rawValue)
    }

    @available(macOS 15.0, *)
    @_cdecl("sc_recording_output_configuration_get_available_output_file_types_count")
    public func getRecordingOutputAvailableFileTypesCount(_ config: OpaquePointer) -> Int {
        let box: Box<SCRecordingOutputConfiguration> = unretained(config)
        return box.value.availableOutputFileTypes.count
    }

    @available(macOS 15.0, *)
    @_cdecl("sc_recording_output_configuration_get_available_output_file_type_at")
    public func getRecordingOutputAvailableFileTypeAt(_ config: OpaquePointer, _ index: Int) -> Int32 {
        let box: Box<SCRecordingOutputConfiguration> = unretained(config)
        guard index >= 0, index < box.value.availableOutputFileTypes.count else {
            return RecordingWireCode.unknown
        }
        return RecordingWireCode.code(for: box.value.availableOutputFileTypes[index])
    }

    @available(macOS 15.0, *)
    @_cdecl("sc_recording_output_configuration_get_available_output_file_type_identifier_at_owned")
    public func getRecordingOutputAvailableFileTypeIdentifierAtOwned(
        _ config: OpaquePointer,
        _ index: Int
    ) -> UnsafeMutablePointer<CChar>? {
        let box: Box<SCRecordingOutputConfiguration> = unretained(config)
        guard index >= 0, index < box.value.availableOutputFileTypes.count else { return nil }
        return strdup(box.value.availableOutputFileTypes[index].rawValue)
    }

    @available(macOS 15.0, *)
    @_cdecl("sc_recording_output_configuration_retain")
    public func retainRecordingOutputConfiguration(_ config: OpaquePointer) -> OpaquePointer {
        let box: Box<SCRecordingOutputConfiguration> = unretained(config)
        return retain(box)
    }

    /// Produce an independent `SCRecordingOutputConfiguration` with the same
    /// settings.
    ///
    /// `SCRecordingOutputConfiguration` is a mutable class, so retaining the
    /// same instance for Rust's `Clone` makes every clone an alias: mutating
    /// one handle mutates all of them, and two threads configuring separate
    /// "clones" race on the same non-atomic properties.
    @available(macOS 15.0, *)
    @_cdecl("sc_recording_output_configuration_copy")
    public func copyRecordingOutputConfiguration(_ config: OpaquePointer) -> OpaquePointer {
        let box: Box<SCRecordingOutputConfiguration> = unretained(config)
        let source = box.value

        let copy: SCRecordingOutputConfiguration
        if let copyable = source as? NSCopying,
           let copied = copyable.copy(with: nil) as? SCRecordingOutputConfiguration
        {
            copy = copied
        } else {
            copy = SCRecordingOutputConfiguration()
            if let url = RecordingConfigurationValue.outputURL(source) {
                copy.outputURL = url
            }
            if let codec = RecordingConfigurationValue.videoCodecType(source) {
                copy.videoCodecType = codec
            }
            if let fileType = RecordingConfigurationValue.outputFileType(source) {
                copy.outputFileType = fileType
            }
        }

        return retain(Box(copy))
    }

    @available(macOS 15.0, *)
    @_cdecl("sc_recording_output_configuration_release")
    public func releaseRecordingOutputConfiguration(_ config: OpaquePointer) {
        release(config)
    }

    @available(macOS 15.0, *)
    @_cdecl("sc_recording_output_create")
    public func createRecordingOutput(_ config: OpaquePointer) -> OpaquePointer? {
        let box: Box<SCRecordingOutputConfiguration> = unretained(config)
        let delegate = RecordingDelegate()
        let output = SCRecordingOutput(configuration: box.value, delegate: delegate)

        RecordingDelegateAssociation.retain(delegate, for: output)

        return retain(output)
    }

    @available(macOS 15.0, *)
    @_cdecl("sc_recording_output_create_with_delegate")
    public func createRecordingOutputWithDelegate(
        _ config: OpaquePointer,
        _ startedCallback: RecordingStartedCallback?,
        _ failedCallback: RecordingFailedCallback?,
        _ finishedCallback: RecordingFinishedCallback?,
        _ contextRelease: RecordingContextRelease?,
        _ context: UnsafeMutableRawPointer?
    ) -> OpaquePointer? {
        let box: Box<SCRecordingOutputConfiguration> = unretained(config)
        let delegate = RecordingDelegate()
        delegate.startedCallback = startedCallback
        delegate.failedCallback = failedCallback
        delegate.finishedCallback = finishedCallback
        delegate.contextRelease = contextRelease
        delegate.context = context

        let output = SCRecordingOutput(configuration: box.value, delegate: delegate)

        RecordingDelegateAssociation.retain(delegate, for: output)

        return retain(output)
    }

    @available(macOS 15.0, *)
    @_cdecl("sc_recording_output_get_recorded_duration")
    public func getRecordingOutputRecordedDuration(_ output: OpaquePointer, _ value: UnsafeMutablePointer<Int64>, _ timescale: UnsafeMutablePointer<Int32>) {
        let o: SCRecordingOutput = unretained(output)
        let duration = o.recordedDuration
        value.pointee = duration.value
        timescale.pointee = duration.timescale
    }

    @available(macOS 15.0, *)
    @_cdecl("sc_recording_output_get_recorded_file_size")
    public func getRecordingOutputRecordedFileSize(_ output: OpaquePointer) -> Int64 {
        let o: SCRecordingOutput = unretained(output)
        return Int64(o.recordedFileSize)
    }

    @available(macOS 15.0, *)
    @_cdecl("sc_recording_output_wait_until_terminal")
    public func waitUntilRecordingOutputTerminal(
        _ output: OpaquePointer,
        _ context: UnsafeMutableRawPointer?,
        _ callback: @escaping @convention(c) (
            UnsafeMutableRawPointer?,
            Bool,
            UnsafePointer<CChar>?
        ) -> Void
    ) {
        let o: SCRecordingOutput = unretained(output)
        afterRecordingOutputTerminal(o) {
            callback(context, true, nil)
        }
    }

    @available(macOS 15.0, *)
    @_cdecl("sc_recording_output_retain")
    public func retainRecordingOutput(_ output: OpaquePointer) -> OpaquePointer {
        let o: SCRecordingOutput = unretained(output)
        return retain(o)
    }

    @available(macOS 15.0, *)
    @_cdecl("sc_recording_output_release")
    public func releaseRecordingOutput(_ output: OpaquePointer) {
        release(output)
    }

#else
    // Stub implementation for older SDKs (macOS < 15 SDK)

    @_cdecl("sc_recording_output_configuration_create")
    public func createRecordingOutputConfiguration() -> OpaquePointer? {
        nil
    }

    @_cdecl("sc_recording_output_configuration_set_output_url")
    public func setRecordingOutputURL(_: OpaquePointer?, _: UnsafePointer<CChar>) {}

    @_cdecl("sc_recording_output_configuration_get_output_url")
    public func getRecordingOutputURL(_: OpaquePointer?, _: UnsafeMutablePointer<CChar>, _: Int) -> Bool { false }

    @_cdecl("sc_recording_output_configuration_get_output_path_owned")
    public func getRecordingOutputPathOwned(_: OpaquePointer?) -> UnsafeMutablePointer<CChar>? { nil }

    @_cdecl("sc_recording_output_configuration_set_video_codec")
    public func setRecordingOutputVideoCodec(_: OpaquePointer?, _: Int32) {}

    @_cdecl("sc_recording_output_configuration_set_video_codec_identifier")
    public func setRecordingOutputVideoCodecIdentifier(_: OpaquePointer?, _: UnsafePointer<CChar>) {}

    @_cdecl("sc_recording_output_configuration_get_video_codec_identifier_owned")
    public func getRecordingOutputVideoCodecIdentifierOwned(_: OpaquePointer?) -> UnsafeMutablePointer<CChar>? { nil }

    @_cdecl("sc_recording_output_configuration_get_video_codec")
    public func getRecordingOutputVideoCodec(_: OpaquePointer?) -> Int32 { 0 }

    @_cdecl("sc_recording_output_configuration_set_output_file_type")
    public func setRecordingOutputFileType(_: OpaquePointer?, _: Int32) {}

    @_cdecl("sc_recording_output_configuration_set_output_file_type_identifier")
    public func setRecordingOutputFileTypeIdentifier(_: OpaquePointer?, _: UnsafePointer<CChar>) {}

    @_cdecl("sc_recording_output_configuration_get_output_file_type_identifier_owned")
    public func getRecordingOutputFileTypeIdentifierOwned(_: OpaquePointer?) -> UnsafeMutablePointer<CChar>? { nil }

    @_cdecl("sc_recording_output_configuration_get_output_file_type")
    public func getRecordingOutputFileType(_: OpaquePointer?) -> Int32 { 0 }

    @_cdecl("sc_recording_output_configuration_get_available_video_codecs_count")
    public func getRecordingOutputAvailableVideoCodecsCount(_: OpaquePointer?) -> Int { 0 }

    @_cdecl("sc_recording_output_configuration_get_available_video_codec_at")
    public func getRecordingOutputAvailableVideoCodecAt(_: OpaquePointer?, _: Int) -> Int32 { -1 }

    @_cdecl("sc_recording_output_configuration_get_available_video_codec_identifier_at_owned")
    public func getRecordingOutputAvailableVideoCodecIdentifierAtOwned(_: OpaquePointer?, _: Int) -> UnsafeMutablePointer<CChar>? { nil }

    @_cdecl("sc_recording_output_configuration_get_available_output_file_types_count")
    public func getRecordingOutputAvailableFileTypesCount(_: OpaquePointer?) -> Int { 0 }

    @_cdecl("sc_recording_output_configuration_get_available_output_file_type_at")
    public func getRecordingOutputAvailableFileTypeAt(_: OpaquePointer?, _: Int) -> Int32 { -1 }

    @_cdecl("sc_recording_output_configuration_get_available_output_file_type_identifier_at_owned")
    public func getRecordingOutputAvailableFileTypeIdentifierAtOwned(_: OpaquePointer?, _: Int) -> UnsafeMutablePointer<CChar>? { nil }

    @_cdecl("sc_recording_output_configuration_retain")
    public func retainRecordingOutputConfiguration(_: OpaquePointer?) -> OpaquePointer? {
        nil
    }

    @_cdecl("sc_recording_output_configuration_copy")
    public func copyRecordingOutputConfiguration(_: OpaquePointer?) -> OpaquePointer? {
        nil
    }

    @_cdecl("sc_recording_output_configuration_release")
    public func releaseRecordingOutputConfiguration(_: OpaquePointer?) {}

    @_cdecl("sc_recording_output_create")
    public func createRecordingOutput(_: OpaquePointer?) -> OpaquePointer? {
        nil
    }

    @_cdecl("sc_recording_output_create_with_delegate")
    public func createRecordingOutputWithDelegate(
        _: OpaquePointer?,
        _: RecordingStartedCallback?,
        _: RecordingFailedCallback?,
        _: RecordingFinishedCallback?,
        _: RecordingContextRelease?,
        _: UnsafeMutableRawPointer?
    ) -> OpaquePointer? {
        nil
    }

    @_cdecl("sc_recording_output_get_recorded_duration")
    public func getRecordingOutputRecordedDuration(_: OpaquePointer?, _ value: UnsafeMutablePointer<Int64>, _ timescale: UnsafeMutablePointer<Int32>) {
        value.pointee = 0
        timescale.pointee = 0
    }

    @_cdecl("sc_recording_output_get_recorded_file_size")
    public func getRecordingOutputRecordedFileSize(_: OpaquePointer?) -> Int64 { 0 }

    @_cdecl("sc_recording_output_wait_until_terminal")
    public func waitUntilRecordingOutputTerminal(
        _: OpaquePointer?,
        _ context: UnsafeMutableRawPointer?,
        _ callback: @escaping @convention(c) (
            UnsafeMutableRawPointer?,
            Bool,
            UnsafePointer<CChar>?
        ) -> Void
    ) {
        "SCRecordingOutput requires macOS 15.0 SDK or later".withCString {
            callback(context, false, $0)
        }
    }

    @_cdecl("sc_recording_output_retain")
    public func retainRecordingOutput(_: OpaquePointer?) -> OpaquePointer? {
        nil
    }

    @_cdecl("sc_recording_output_release")
    public func releaseRecordingOutput(_: OpaquePointer?) {}

#endif
