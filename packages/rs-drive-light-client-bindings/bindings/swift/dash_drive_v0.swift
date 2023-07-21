// This file was autogenerated by some hot garbage in the `uniffi` crate.
// Trust me, you don't want to mess with it!
import Foundation

// Depending on the consumer's build setup, the low-level FFI code
// might be in a separate module, or it might be compiled inline into
// this module. This is a bit of light hackery to work with both.
#if canImport(dash_drive_v0FFI)
import dash_drive_v0FFI
#endif

fileprivate extension RustBuffer {
    // Allocate a new buffer, copying the contents of a `UInt8` array.
    init(bytes: [UInt8]) {
        let rbuf = bytes.withUnsafeBufferPointer { ptr in
            RustBuffer.from(ptr)
        }
        self.init(capacity: rbuf.capacity, len: rbuf.len, data: rbuf.data)
    }

    static func from(_ ptr: UnsafeBufferPointer<UInt8>) -> RustBuffer {
        try! rustCall { ffi_dash_drive_v0_rustbuffer_from_bytes(ForeignBytes(bufferPointer: ptr), $0) }
    }

    // Frees the buffer in place.
    // The buffer must not be used after this is called.
    func deallocate() {
        try! rustCall { ffi_dash_drive_v0_rustbuffer_free(self, $0) }
    }
}

fileprivate extension ForeignBytes {
    init(bufferPointer: UnsafeBufferPointer<UInt8>) {
        self.init(len: Int32(bufferPointer.count), data: bufferPointer.baseAddress)
    }
}

// For every type used in the interface, we provide helper methods for conveniently
// lifting and lowering that type from C-compatible data, and for reading and writing
// values of that type in a buffer.

// Helper classes/extensions that don't change.
// Someday, this will be in a library of its own.

fileprivate extension Data {
    init(rustBuffer: RustBuffer) {
        // TODO: This copies the buffer. Can we read directly from a
        // Rust buffer?
        self.init(bytes: rustBuffer.data!, count: Int(rustBuffer.len))
    }
}

// Define reader functionality.  Normally this would be defined in a class or
// struct, but we use standalone functions instead in order to make external
// types work.
//
// With external types, one swift source file needs to be able to call the read
// method on another source file's FfiConverter, but then what visibility
// should Reader have?
// - If Reader is fileprivate, then this means the read() must also
//   be fileprivate, which doesn't work with external types.
// - If Reader is internal/public, we'll get compile errors since both source
//   files will try define the same type.
//
// Instead, the read() method and these helper functions input a tuple of data

fileprivate func createReader(data: Data) -> (data: Data, offset: Data.Index) {
    (data: data, offset: 0)
}

// Reads an integer at the current offset, in big-endian order, and advances
// the offset on success. Throws if reading the integer would move the
// offset past the end of the buffer.
fileprivate func readInt<T: FixedWidthInteger>(_ reader: inout (data: Data, offset: Data.Index)) throws -> T {
    let range = reader.offset..<reader.offset + MemoryLayout<T>.size
    guard reader.data.count >= range.upperBound else {
        throw UniffiInternalError.bufferOverflow
    }
    if T.self == UInt8.self {
        let value = reader.data[reader.offset]
        reader.offset += 1
        return value as! T
    }
    var value: T = 0
    let _ = withUnsafeMutableBytes(of: &value, { reader.data.copyBytes(to: $0, from: range)})
    reader.offset = range.upperBound
    return value.bigEndian
}

// Reads an arbitrary number of bytes, to be used to read
// raw bytes, this is useful when lifting strings
fileprivate func readBytes(_ reader: inout (data: Data, offset: Data.Index), count: Int) throws -> Array<UInt8> {
    let range = reader.offset..<(reader.offset+count)
    guard reader.data.count >= range.upperBound else {
        throw UniffiInternalError.bufferOverflow
    }
    var value = [UInt8](repeating: 0, count: count)
    value.withUnsafeMutableBufferPointer({ buffer in
        reader.data.copyBytes(to: buffer, from: range)
    })
    reader.offset = range.upperBound
    return value
}

// Reads a float at the current offset.
fileprivate func readFloat(_ reader: inout (data: Data, offset: Data.Index)) throws -> Float {
    return Float(bitPattern: try readInt(&reader))
}

// Reads a float at the current offset.
fileprivate func readDouble(_ reader: inout (data: Data, offset: Data.Index)) throws -> Double {
    return Double(bitPattern: try readInt(&reader))
}

// Indicates if the offset has reached the end of the buffer.
fileprivate func hasRemaining(_ reader: (data: Data, offset: Data.Index)) -> Bool {
    return reader.offset < reader.data.count
}

// Define writer functionality.  Normally this would be defined in a class or
// struct, but we use standalone functions instead in order to make external
// types work.  See the above discussion on Readers for details.

fileprivate func createWriter() -> [UInt8] {
    return []
}

fileprivate func writeBytes<S>(_ writer: inout [UInt8], _ byteArr: S) where S: Sequence, S.Element == UInt8 {
    writer.append(contentsOf: byteArr)
}

// Writes an integer in big-endian order.
//
// Warning: make sure what you are trying to write
// is in the correct type!
fileprivate func writeInt<T: FixedWidthInteger>(_ writer: inout [UInt8], _ value: T) {
    var value = value.bigEndian
    withUnsafeBytes(of: &value) { writer.append(contentsOf: $0) }
}

fileprivate func writeFloat(_ writer: inout [UInt8], _ value: Float) {
    writeInt(&writer, value.bitPattern)
}

fileprivate func writeDouble(_ writer: inout [UInt8], _ value: Double) {
    writeInt(&writer, value.bitPattern)
}

// Protocol for types that transfer other types across the FFI. This is
// analogous go the Rust trait of the same name.
fileprivate protocol FfiConverter {
    associatedtype FfiType
    associatedtype SwiftType

    static func lift(_ value: FfiType) throws -> SwiftType
    static func lower(_ value: SwiftType) -> FfiType
    static func read(from buf: inout (data: Data, offset: Data.Index)) throws -> SwiftType
    static func write(_ value: SwiftType, into buf: inout [UInt8])
}

// Types conforming to `Primitive` pass themselves directly over the FFI.
fileprivate protocol FfiConverterPrimitive: FfiConverter where FfiType == SwiftType { }

extension FfiConverterPrimitive {
    public static func lift(_ value: FfiType) throws -> SwiftType {
        return value
    }

    public static func lower(_ value: SwiftType) -> FfiType {
        return value
    }
}

// Types conforming to `FfiConverterRustBuffer` lift and lower into a `RustBuffer`.
// Used for complex types where it's hard to write a custom lift/lower.
fileprivate protocol FfiConverterRustBuffer: FfiConverter where FfiType == RustBuffer {}

extension FfiConverterRustBuffer {
    public static func lift(_ buf: RustBuffer) throws -> SwiftType {
        var reader = createReader(data: Data(rustBuffer: buf))
        let value = try read(from: &reader)
        if hasRemaining(reader) {
            throw UniffiInternalError.incompleteData
        }
        buf.deallocate()
        return value
    }

    public static func lower(_ value: SwiftType) -> RustBuffer {
          var writer = createWriter()
          write(value, into: &writer)
          return RustBuffer(bytes: writer)
    }
}
// An error type for FFI errors. These errors occur at the UniFFI level, not
// the library level.
fileprivate enum UniffiInternalError: LocalizedError {
    case bufferOverflow
    case incompleteData
    case unexpectedOptionalTag
    case unexpectedEnumCase
    case unexpectedNullPointer
    case unexpectedRustCallStatusCode
    case unexpectedRustCallError
    case unexpectedStaleHandle
    case rustPanic(_ message: String)

    public var errorDescription: String? {
        switch self {
        case .bufferOverflow: return "Reading the requested value would read past the end of the buffer"
        case .incompleteData: return "The buffer still has data after lifting its containing value"
        case .unexpectedOptionalTag: return "Unexpected optional tag; should be 0 or 1"
        case .unexpectedEnumCase: return "Raw enum value doesn't match any cases"
        case .unexpectedNullPointer: return "Raw pointer value was null"
        case .unexpectedRustCallStatusCode: return "Unexpected RustCallStatus code"
        case .unexpectedRustCallError: return "CALL_ERROR but no errorClass specified"
        case .unexpectedStaleHandle: return "The object in the handle map has been dropped already"
        case let .rustPanic(message): return message
        }
    }
}

fileprivate let CALL_SUCCESS: Int8 = 0
fileprivate let CALL_ERROR: Int8 = 1
fileprivate let CALL_PANIC: Int8 = 2

fileprivate extension RustCallStatus {
    init() {
        self.init(
            code: CALL_SUCCESS,
            errorBuf: RustBuffer.init(
                capacity: 0,
                len: 0,
                data: nil
            )
        )
    }
}

private func rustCall<T>(_ callback: (UnsafeMutablePointer<RustCallStatus>) -> T) throws -> T {
    try makeRustCall(callback, errorHandler: nil)
}

private func rustCallWithError<T>(
    _ errorHandler: @escaping (RustBuffer) throws -> Error,
    _ callback: (UnsafeMutablePointer<RustCallStatus>) -> T) throws -> T {
    try makeRustCall(callback, errorHandler: errorHandler)
}

private func makeRustCall<T>(
    _ callback: (UnsafeMutablePointer<RustCallStatus>) -> T,
    errorHandler: ((RustBuffer) throws -> Error)?
) throws -> T {
    uniffiEnsureInitialized()
    var callStatus = RustCallStatus.init()
    let returnedVal = callback(&callStatus)
    try uniffiCheckCallStatus(callStatus: callStatus, errorHandler: errorHandler)
    return returnedVal
}

private func uniffiCheckCallStatus(
    callStatus: RustCallStatus,
    errorHandler: ((RustBuffer) throws -> Error)?
) throws {
    switch callStatus.code {
        case CALL_SUCCESS:
            return

        case CALL_ERROR:
            if let errorHandler = errorHandler {
                throw try errorHandler(callStatus.errorBuf)
            } else {
                callStatus.errorBuf.deallocate()
                throw UniffiInternalError.unexpectedRustCallError
            }

        case CALL_PANIC:
            // When the rust code sees a panic, it tries to construct a RustBuffer
            // with the message.  But if that code panics, then it just sends back
            // an empty buffer.
            if callStatus.errorBuf.len > 0 {
                throw UniffiInternalError.rustPanic(try FfiConverterString.lift(callStatus.errorBuf))
            } else {
                callStatus.errorBuf.deallocate()
                throw UniffiInternalError.rustPanic("Rust panic")
            }

        default:
            throw UniffiInternalError.unexpectedRustCallStatusCode
    }
}

// Public interface members begin here.


fileprivate struct FfiConverterUInt8: FfiConverterPrimitive {
    typealias FfiType = UInt8
    typealias SwiftType = UInt8

    public static func read(from buf: inout (data: Data, offset: Data.Index)) throws -> UInt8 {
        return try lift(readInt(&buf))
    }

    public static func write(_ value: UInt8, into buf: inout [UInt8]) {
        writeInt(&buf, lower(value))
    }
}

fileprivate struct FfiConverterString: FfiConverter {
    typealias SwiftType = String
    typealias FfiType = RustBuffer

    public static func lift(_ value: RustBuffer) throws -> String {
        defer {
            value.deallocate()
        }
        if value.data == nil {
            return String()
        }
        let bytes = UnsafeBufferPointer<UInt8>(start: value.data!, count: Int(value.len))
        return String(bytes: bytes, encoding: String.Encoding.utf8)!
    }

    public static func lower(_ value: String) -> RustBuffer {
        return value.utf8CString.withUnsafeBufferPointer { ptr in
            // The swift string gives us int8_t, we want uint8_t.
            ptr.withMemoryRebound(to: UInt8.self) { ptr in
                // The swift string gives us a trailing null byte, we don't want it.
                let buf = UnsafeBufferPointer(rebasing: ptr.prefix(upTo: ptr.count - 1))
                return RustBuffer.from(buf)
            }
        }
    }

    public static func read(from buf: inout (data: Data, offset: Data.Index)) throws -> String {
        let len: Int32 = try readInt(&buf)
        return String(bytes: try readBytes(&buf, count: Int(len)), encoding: String.Encoding.utf8)!
    }

    public static func write(_ value: String, into buf: inout [UInt8]) {
        let len = Int32(value.utf8.count)
        writeInt(&buf, len)
        writeBytes(&buf, value.utf8)
    }
}

public enum Error {

    
    
    case NotInitialized
    case AlreadyInitialized
    case DriveError(`error`: String)
    case ProtocolError(`error`: String)
    case EmptyResponse
    case EmptyResponseMetadata
    case EmptyResponseProof
    case DocumentMissingInProof
    case RequestDecodeError(`error`: String)
    case ResponseDecodeError(`error`: String)
    case DataEncodingError(`error`: String)
    case SignDigestFailed(`error`: String)
    case SignatureVerificationError(`error`: String)
    case InvalidQuorum(`error`: String)
    case InvalidSignatureFormat(`error`: String)
    case InvalidPublicKey(`error`: String)
    case InvalidSignature(`error`: String)
    case UnexpectedCallbackError(`error`: String, `reason`: String)

    fileprivate static func uniffiErrorHandler(_ error: RustBuffer) throws -> Error {
        return try FfiConverterTypeError.lift(error)
    }
}


public struct FfiConverterTypeError: FfiConverterRustBuffer {
    typealias SwiftType = Error

    public static func read(from buf: inout (data: Data, offset: Data.Index)) throws -> Error {
        let variant: Int32 = try readInt(&buf)
        switch variant {

        

        
        case 1: return .NotInitialized
        case 2: return .AlreadyInitialized
        case 3: return .DriveError(
            `error`: try FfiConverterString.read(from: &buf)
            )
        case 4: return .ProtocolError(
            `error`: try FfiConverterString.read(from: &buf)
            )
        case 5: return .EmptyResponse
        case 6: return .EmptyResponseMetadata
        case 7: return .EmptyResponseProof
        case 8: return .DocumentMissingInProof
        case 9: return .RequestDecodeError(
            `error`: try FfiConverterString.read(from: &buf)
            )
        case 10: return .ResponseDecodeError(
            `error`: try FfiConverterString.read(from: &buf)
            )
        case 11: return .DataEncodingError(
            `error`: try FfiConverterString.read(from: &buf)
            )
        case 12: return .SignDigestFailed(
            `error`: try FfiConverterString.read(from: &buf)
            )
        case 13: return .SignatureVerificationError(
            `error`: try FfiConverterString.read(from: &buf)
            )
        case 14: return .InvalidQuorum(
            `error`: try FfiConverterString.read(from: &buf)
            )
        case 15: return .InvalidSignatureFormat(
            `error`: try FfiConverterString.read(from: &buf)
            )
        case 16: return .InvalidPublicKey(
            `error`: try FfiConverterString.read(from: &buf)
            )
        case 17: return .InvalidSignature(
            `error`: try FfiConverterString.read(from: &buf)
            )
        case 18: return .UnexpectedCallbackError(
            `error`: try FfiConverterString.read(from: &buf), 
            `reason`: try FfiConverterString.read(from: &buf)
            )

         default: throw UniffiInternalError.unexpectedEnumCase
        }
    }

    public static func write(_ value: Error, into buf: inout [UInt8]) {
        switch value {

        

        
        
        case .NotInitialized:
            writeInt(&buf, Int32(1))
        
        
        case .AlreadyInitialized:
            writeInt(&buf, Int32(2))
        
        
        case let .DriveError(`error`):
            writeInt(&buf, Int32(3))
            FfiConverterString.write(`error`, into: &buf)
            
        
        case let .ProtocolError(`error`):
            writeInt(&buf, Int32(4))
            FfiConverterString.write(`error`, into: &buf)
            
        
        case .EmptyResponse:
            writeInt(&buf, Int32(5))
        
        
        case .EmptyResponseMetadata:
            writeInt(&buf, Int32(6))
        
        
        case .EmptyResponseProof:
            writeInt(&buf, Int32(7))
        
        
        case .DocumentMissingInProof:
            writeInt(&buf, Int32(8))
        
        
        case let .RequestDecodeError(`error`):
            writeInt(&buf, Int32(9))
            FfiConverterString.write(`error`, into: &buf)
            
        
        case let .ResponseDecodeError(`error`):
            writeInt(&buf, Int32(10))
            FfiConverterString.write(`error`, into: &buf)
            
        
        case let .DataEncodingError(`error`):
            writeInt(&buf, Int32(11))
            FfiConverterString.write(`error`, into: &buf)
            
        
        case let .SignDigestFailed(`error`):
            writeInt(&buf, Int32(12))
            FfiConverterString.write(`error`, into: &buf)
            
        
        case let .SignatureVerificationError(`error`):
            writeInt(&buf, Int32(13))
            FfiConverterString.write(`error`, into: &buf)
            
        
        case let .InvalidQuorum(`error`):
            writeInt(&buf, Int32(14))
            FfiConverterString.write(`error`, into: &buf)
            
        
        case let .InvalidSignatureFormat(`error`):
            writeInt(&buf, Int32(15))
            FfiConverterString.write(`error`, into: &buf)
            
        
        case let .InvalidPublicKey(`error`):
            writeInt(&buf, Int32(16))
            FfiConverterString.write(`error`, into: &buf)
            
        
        case let .InvalidSignature(`error`):
            writeInt(&buf, Int32(17))
            FfiConverterString.write(`error`, into: &buf)
            
        
        case let .UnexpectedCallbackError(`error`,`reason`):
            writeInt(&buf, Int32(18))
            FfiConverterString.write(`error`, into: &buf)
            FfiConverterString.write(`reason`, into: &buf)
            
        }
    }
}


extension Error: Equatable, Hashable {}

extension Error: Error { }

fileprivate extension NSLock {
    func withLock<T>(f: () throws -> T) rethrows -> T {
        self.lock()
        defer { self.unlock() }
        return try f()
    }
}

fileprivate typealias UniFFICallbackHandle = UInt64
fileprivate class UniFFICallbackHandleMap<T> {
    private var leftMap: [UniFFICallbackHandle: T] = [:]
    private var counter: [UniFFICallbackHandle: UInt64] = [:]
    private var rightMap: [ObjectIdentifier: UniFFICallbackHandle] = [:]

    private let lock = NSLock()
    private var currentHandle: UniFFICallbackHandle = 0
    private let stride: UniFFICallbackHandle = 1

    func insert(obj: T) -> UniFFICallbackHandle {
        lock.withLock {
            let id = ObjectIdentifier(obj as AnyObject)
            let handle = rightMap[id] ?? {
                currentHandle += stride
                let handle = currentHandle
                leftMap[handle] = obj
                rightMap[id] = handle
                return handle
            }()
            counter[handle] = (counter[handle] ?? 0) + 1
            return handle
        }
    }

    func get(handle: UniFFICallbackHandle) -> T? {
        lock.withLock {
            leftMap[handle]
        }
    }

    func delete(handle: UniFFICallbackHandle) {
        remove(handle: handle)
    }

    @discardableResult
    func remove(handle: UniFFICallbackHandle) -> T? {
        lock.withLock {
            defer { counter[handle] = (counter[handle] ?? 1) - 1 }
            guard counter[handle] == 1 else { return leftMap[handle] }
            let obj = leftMap.removeValue(forKey: handle)
            if let obj = obj {
                rightMap.removeValue(forKey: ObjectIdentifier(obj as AnyObject))
            }
            return obj
        }
    }
}

// Magic number for the Rust proxy to call using the same mechanism as every other method,
// to free the callback once it's dropped by Rust.
private let IDX_CALLBACK_FREE: Int32 = 0
// Callback return codes
private let UNIFFI_CALLBACK_SUCCESS: Int32 = 0
private let UNIFFI_CALLBACK_ERROR: Int32 = 1
private let UNIFFI_CALLBACK_UNEXPECTED_ERROR: Int32 = 2

// Declaration and FfiConverters for QuorumInfoProvider Callback Interface

public protocol QuorumInfoProvider : AnyObject {
    func `getQuorumPublicKey`(`quorumType`: UInt32, `quorumHash`: [UInt8]) throws -> [UInt8]
    
}

// The ForeignCallback that is passed to Rust.
fileprivate let foreignCallbackCallbackInterfaceQuorumInfoProvider : ForeignCallback =
    { (handle: UniFFICallbackHandle, method: Int32, argsData: UnsafePointer<UInt8>, argsLen: Int32, out_buf: UnsafeMutablePointer<RustBuffer>) -> Int32 in
    

    func `invokeGetQuorumPublicKey`(_ swiftCallbackInterface: QuorumInfoProvider, _ argsData: UnsafePointer<UInt8>, _ argsLen: Int32, _ out_buf: UnsafeMutablePointer<RustBuffer>) throws -> Int32 {
        var reader = createReader(data: Data(bytes: argsData, count: Int(argsLen)))
        func makeCall() throws -> Int32 {
            let result = try swiftCallbackInterface.`getQuorumPublicKey`(
                    `quorumType`:  try FfiConverterUInt32.read(from: &reader), 
                    `quorumHash`:  try FfiConverterSequenceUInt8.read(from: &reader)
                    )
            var writer = [UInt8]()
            FfiConverterSequenceUInt8.write(result, into: &writer)
            out_buf.pointee = RustBuffer(bytes: writer)
            return UNIFFI_CALLBACK_SUCCESS
        }
        do {
            return try makeCall()
        } catch let error as Error {
            out_buf.pointee = FfiConverterTypeError.lower(error)
            return UNIFFI_CALLBACK_ERROR
        }
    }


    switch method {
        case IDX_CALLBACK_FREE:
            FfiConverterCallbackInterfaceQuorumInfoProvider.drop(handle: handle)
            // Sucessful return
            // See docs of ForeignCallback in `uniffi_core/src/ffi/foreigncallbacks.rs`
            return UNIFFI_CALLBACK_SUCCESS
        case 1:
            let cb: QuorumInfoProvider
            do {
                cb = try FfiConverterCallbackInterfaceQuorumInfoProvider.lift(handle)
            } catch {
                out_buf.pointee = FfiConverterString.lower("QuorumInfoProvider: Invalid handle")
                return UNIFFI_CALLBACK_UNEXPECTED_ERROR
            }
            do {
                return try `invokeGetQuorumPublicKey`(cb, argsData, argsLen, out_buf)
            } catch let error {
                out_buf.pointee = FfiConverterString.lower(String(describing: error))
                return UNIFFI_CALLBACK_UNEXPECTED_ERROR
            }
        
        // This should never happen, because an out of bounds method index won't
        // ever be used. Once we can catch errors, we should return an InternalError.
        // https://github.com/mozilla/uniffi-rs/issues/351
        default:
            // An unexpected error happened.
            // See docs of ForeignCallback in `uniffi_core/src/ffi/foreigncallbacks.rs`
            return UNIFFI_CALLBACK_UNEXPECTED_ERROR
    }
}

// FfiConverter protocol for callback interfaces
fileprivate struct FfiConverterCallbackInterfaceQuorumInfoProvider {
    private static let initCallbackOnce: () = {
        // Swift ensures this initializer code will once run once, even when accessed by multiple threads.
        try! rustCall { (err: UnsafeMutablePointer<RustCallStatus>) in
            uniffi_dash_drive_v0_fn_init_callback_quoruminfoprovider(foreignCallbackCallbackInterfaceQuorumInfoProvider, err)
        }
    }()

    private static func ensureCallbackinitialized() {
        _ = initCallbackOnce
    }

    static func drop(handle: UniFFICallbackHandle) {
        handleMap.remove(handle: handle)
    }

    private static var handleMap = UniFFICallbackHandleMap<QuorumInfoProvider>()
}

extension FfiConverterCallbackInterfaceQuorumInfoProvider : FfiConverter {
    typealias SwiftType = QuorumInfoProvider
    // We can use Handle as the FfiType because it's a typealias to UInt64
    typealias FfiType = UniFFICallbackHandle

    public static func lift(_ handle: UniFFICallbackHandle) throws -> SwiftType {
        ensureCallbackinitialized();
        guard let callback = handleMap.get(handle: handle) else {
            throw UniffiInternalError.unexpectedStaleHandle
        }
        return callback
    }

    public static func read(from buf: inout (data: Data, offset: Data.Index)) throws -> SwiftType {
        ensureCallbackinitialized();
        let handle: UniFFICallbackHandle = try readInt(&buf)
        return try lift(handle)
    }

    public static func lower(_ v: SwiftType) -> UniFFICallbackHandle {
        ensureCallbackinitialized();
        return handleMap.insert(obj: v)
    }

    public static func write(_ v: SwiftType, into buf: inout [UInt8]) {
        ensureCallbackinitialized();
        writeInt(&buf, lower(v))
    }
}

fileprivate struct FfiConverterSequenceUInt8: FfiConverterRustBuffer {
    typealias SwiftType = [UInt8]

    public static func write(_ value: [UInt8], into buf: inout [UInt8]) {
        let len = Int32(value.count)
        writeInt(&buf, len)
        for item in value {
            FfiConverterUInt8.write(item, into: &buf)
        }
    }

    public static func read(from buf: inout (data: Data, offset: Data.Index)) throws -> [UInt8] {
        let len: Int32 = try readInt(&buf)
        var seq = [UInt8]()
        seq.reserveCapacity(Int(len))
        for _ in 0 ..< len {
            seq.append(try FfiConverterUInt8.read(from: &buf))
        }
        return seq
    }
}

public func `identityByPubkeysProofJson`(`request`: [UInt8], `response`: [UInt8], `callback`: QuorumInfoProvider) throws -> [UInt8] {
    return try  FfiConverterSequenceUInt8.lift(
        try rustCallWithError(FfiConverterTypeError.lift) {
    uniffi_drive_light_client_fn_func_identity_by_pubkeys_proof_json(
        FfiConverterSequenceUInt8.lower(`request`),
        FfiConverterSequenceUInt8.lower(`response`),
        FfiConverterCallbackInterfaceQuorumInfoProvider.lower(`callback`),$0)
}
    )
}

public func `identityProofJson`(`request`: [UInt8], `response`: [UInt8], `callback`: QuorumInfoProvider) throws -> [UInt8] {
    return try  FfiConverterSequenceUInt8.lift(
        try rustCallWithError(FfiConverterTypeError.lift) {
    uniffi_drive_light_client_fn_func_identity_proof_json(
        FfiConverterSequenceUInt8.lower(`request`),
        FfiConverterSequenceUInt8.lower(`response`),
        FfiConverterCallbackInterfaceQuorumInfoProvider.lower(`callback`),$0)
}
    )
}

public func `version`()  -> String {
    return try!  FfiConverterString.lift(
        try! rustCall() {
    uniffi_drive_light_client_fn_func_version($0)
}
    )
}

private enum InitializationResult {
    case ok
    case contractVersionMismatch
    case apiChecksumMismatch
}
// Use a global variables to perform the versioning checks. Swift ensures that
// the code inside is only computed once.
private var initializationResult: InitializationResult {
    // Get the bindings contract version from our ComponentInterface
    let bindings_contract_version = 22
    // Get the scaffolding contract version by calling the into the dylib
    let scaffolding_contract_version = ffi_dash_drive_v0_uniffi_contract_version()
    if bindings_contract_version != scaffolding_contract_version {
        return InitializationResult.contractVersionMismatch
    }
    if (uniffi_drive_light_client_checksum_func_identity_by_pubkeys_proof_json() != 24692) {
        return InitializationResult.apiChecksumMismatch
    }
    if (uniffi_drive_light_client_checksum_func_identity_proof_json() != 10710) {
        return InitializationResult.apiChecksumMismatch
    }
    if (uniffi_drive_light_client_checksum_func_version() != 33802) {
        return InitializationResult.apiChecksumMismatch
    }
    if (uniffi_drive_light_client_checksum_method_quoruminfoprovider_get_quorum_public_key() != 49931) {
        return InitializationResult.apiChecksumMismatch
    }

    return InitializationResult.ok
}

private func uniffiEnsureInitialized() {
    switch initializationResult {
    case .ok:
        break
    case .contractVersionMismatch:
        fatalError("UniFFI contract version mismatch: try cleaning and rebuilding your project")
    case .apiChecksumMismatch:
        fatalError("UniFFI API checksum mismatch: try cleaning and rebuilding your project")
    }
}