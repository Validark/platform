// Generated by the protocol buffer compiler.  DO NOT EDIT!
// source: platform.proto

// This CPP symbol can be defined to use imports that match up to the framework
// imports needed when using CocoaPods.
#if !defined(GPB_USE_PROTOBUF_FRAMEWORK_IMPORTS)
 #define GPB_USE_PROTOBUF_FRAMEWORK_IMPORTS 0
#endif

#if GPB_USE_PROTOBUF_FRAMEWORK_IMPORTS
 #import <Protobuf/GPBProtocolBuffers.h>
#else
 #import "GPBProtocolBuffers.h"
#endif

#if GOOGLE_PROTOBUF_OBJC_VERSION < 30004
#error This file was generated by a newer version of protoc which is incompatible with your Protocol Buffer library sources.
#endif
#if 30004 < GOOGLE_PROTOBUF_OBJC_MIN_SUPPORTED_VERSION
#error This file was generated by an older version of protoc which is incompatible with your Protocol Buffer library sources.
#endif

// @@protoc_insertion_point(imports)

#pragma clang diagnostic push
#pragma clang diagnostic ignored "-Wdeprecated-declarations"

CF_EXTERN_C_BEGIN

@class AllKeys;
@class ConsensusParamsBlock;
@class ConsensusParamsEvidence;
@class GPBUInt32Value;
@class GPBUInt64Value;
@class GetDataContractsResponse_DataContractEntry;
@class GetDataContractsResponse_DataContractValue;
@class GetDataContractsResponse_DataContracts;
@class GetIdentitiesKeysResponse_PublicKey;
@class GetIdentitiesKeysResponse_PublicKeyEntries;
@class GetIdentitiesKeysResponse_PublicKeyEntry;
@class GetIdentityKeysResponse_Keys;
@class KeyRequestType;
@class Proof;
@class ResponseMetadata;
@class SearchKey;
@class SecurityLevelMap;
@class SpecificKeys;
@class StateTransitionBroadcastError;

NS_ASSUME_NONNULL_BEGIN

#pragma mark - Enum SecurityLevelMap_KeyKindRequestType

typedef GPB_ENUM(SecurityLevelMap_KeyKindRequestType) {
  /**
   * Value used if any message's field encounters a value that is not defined
   * by this enum. The message will also have C functions to get/set the rawValue
   * of the field.
   **/
  SecurityLevelMap_KeyKindRequestType_GPBUnrecognizedEnumeratorValue = kGPBUnrecognizedEnumeratorValue,
  SecurityLevelMap_KeyKindRequestType_CurrentKeyOfKindRequest = 0,
  SecurityLevelMap_KeyKindRequestType_AllKeysOfKindRequest = 1,
};

GPBEnumDescriptor *SecurityLevelMap_KeyKindRequestType_EnumDescriptor(void);

/**
 * Checks to see if the given value is defined by the enum or was not known at
 * the time this source was generated.
 **/
BOOL SecurityLevelMap_KeyKindRequestType_IsValidValue(int32_t value);

#pragma mark - Enum GetIdentitiesKeysRequest_SecurityLevelMap_KeyKindRequestType

typedef GPB_ENUM(GetIdentitiesKeysRequest_SecurityLevelMap_KeyKindRequestType) {
  /**
   * Value used if any message's field encounters a value that is not defined
   * by this enum. The message will also have C functions to get/set the rawValue
   * of the field.
   **/
  GetIdentitiesKeysRequest_SecurityLevelMap_KeyKindRequestType_GPBUnrecognizedEnumeratorValue = kGPBUnrecognizedEnumeratorValue,
  GetIdentitiesKeysRequest_SecurityLevelMap_KeyKindRequestType_CurrentKeyOfKindRequest = 0,
};

GPBEnumDescriptor *GetIdentitiesKeysRequest_SecurityLevelMap_KeyKindRequestType_EnumDescriptor(void);

/**
 * Checks to see if the given value is defined by the enum or was not known at
 * the time this source was generated.
 **/
BOOL GetIdentitiesKeysRequest_SecurityLevelMap_KeyKindRequestType_IsValidValue(int32_t value);

#pragma mark - PlatformRoot

/**
 * Exposes the extension registry for this file.
 *
 * The base class provides:
 * @code
 *   + (GPBExtensionRegistry *)extensionRegistry;
 * @endcode
 * which is a @c GPBExtensionRegistry that includes all the extensions defined by
 * this file and all files that it depends on.
 **/
GPB_FINAL @interface PlatformRoot : GPBRootObject
@end

#pragma mark - Proof

typedef GPB_ENUM(Proof_FieldNumber) {
  Proof_FieldNumber_GrovedbProof = 1,
  Proof_FieldNumber_QuorumHash = 2,
  Proof_FieldNumber_Signature = 3,
  Proof_FieldNumber_Round = 4,
};

GPB_FINAL @interface Proof : GPBMessage

@property(nonatomic, readwrite, copy, null_resettable) NSData *grovedbProof;

@property(nonatomic, readwrite, copy, null_resettable) NSData *quorumHash;

@property(nonatomic, readwrite, copy, null_resettable) NSData *signature;

@property(nonatomic, readwrite) uint32_t round;

@end

#pragma mark - ResponseMetadata

typedef GPB_ENUM(ResponseMetadata_FieldNumber) {
  ResponseMetadata_FieldNumber_Height = 1,
  ResponseMetadata_FieldNumber_CoreChainLockedHeight = 2,
  ResponseMetadata_FieldNumber_TimeMs = 3,
  ResponseMetadata_FieldNumber_ProtocolVersion = 4,
};

GPB_FINAL @interface ResponseMetadata : GPBMessage

@property(nonatomic, readwrite) uint64_t height;

@property(nonatomic, readwrite) uint32_t coreChainLockedHeight;

@property(nonatomic, readwrite) uint64_t timeMs;

@property(nonatomic, readwrite) uint32_t protocolVersion;

@end

#pragma mark - StateTransitionBroadcastError

typedef GPB_ENUM(StateTransitionBroadcastError_FieldNumber) {
  StateTransitionBroadcastError_FieldNumber_Code = 1,
  StateTransitionBroadcastError_FieldNumber_Message = 2,
  StateTransitionBroadcastError_FieldNumber_Data_p = 3,
};

GPB_FINAL @interface StateTransitionBroadcastError : GPBMessage

@property(nonatomic, readwrite) uint32_t code;

@property(nonatomic, readwrite, copy, null_resettable) NSString *message;

@property(nonatomic, readwrite, copy, null_resettable) NSData *data_p;

@end

#pragma mark - BroadcastStateTransitionRequest

typedef GPB_ENUM(BroadcastStateTransitionRequest_FieldNumber) {
  BroadcastStateTransitionRequest_FieldNumber_StateTransition = 1,
};

GPB_FINAL @interface BroadcastStateTransitionRequest : GPBMessage

@property(nonatomic, readwrite, copy, null_resettable) NSData *stateTransition;

@end

#pragma mark - BroadcastStateTransitionResponse

GPB_FINAL @interface BroadcastStateTransitionResponse : GPBMessage

@end

#pragma mark - GetIdentityRequest

typedef GPB_ENUM(GetIdentityRequest_FieldNumber) {
  GetIdentityRequest_FieldNumber_Id_p = 1,
  GetIdentityRequest_FieldNumber_Prove = 2,
};

GPB_FINAL @interface GetIdentityRequest : GPBMessage

@property(nonatomic, readwrite, copy, null_resettable) NSData *id_p;

@property(nonatomic, readwrite) BOOL prove;

@end

#pragma mark - GetIdentityResponse

typedef GPB_ENUM(GetIdentityResponse_FieldNumber) {
  GetIdentityResponse_FieldNumber_Identity = 1,
  GetIdentityResponse_FieldNumber_Proof = 2,
  GetIdentityResponse_FieldNumber_Metadata = 3,
};

GPB_FINAL @interface GetIdentityResponse : GPBMessage

@property(nonatomic, readwrite, copy, null_resettable) NSData *identity;

@property(nonatomic, readwrite, strong, null_resettable) Proof *proof;
/** Test to see if @c proof has been set. */
@property(nonatomic, readwrite) BOOL hasProof;

@property(nonatomic, readwrite, strong, null_resettable) ResponseMetadata *metadata;
/** Test to see if @c metadata has been set. */
@property(nonatomic, readwrite) BOOL hasMetadata;

@end

#pragma mark - GetIdentityBalanceResponse

typedef GPB_ENUM(GetIdentityBalanceResponse_FieldNumber) {
  GetIdentityBalanceResponse_FieldNumber_Balance = 1,
  GetIdentityBalanceResponse_FieldNumber_Proof = 2,
  GetIdentityBalanceResponse_FieldNumber_Metadata = 3,
};

GPB_FINAL @interface GetIdentityBalanceResponse : GPBMessage

@property(nonatomic, readwrite, strong, null_resettable) GPBUInt64Value *balance;
/** Test to see if @c balance has been set. */
@property(nonatomic, readwrite) BOOL hasBalance;

@property(nonatomic, readwrite, strong, null_resettable) Proof *proof;
/** Test to see if @c proof has been set. */
@property(nonatomic, readwrite) BOOL hasProof;

@property(nonatomic, readwrite, strong, null_resettable) ResponseMetadata *metadata;
/** Test to see if @c metadata has been set. */
@property(nonatomic, readwrite) BOOL hasMetadata;

@end

#pragma mark - GetIdentityBalanceAndRevisionResponse

typedef GPB_ENUM(GetIdentityBalanceAndRevisionResponse_FieldNumber) {
  GetIdentityBalanceAndRevisionResponse_FieldNumber_Balance = 1,
  GetIdentityBalanceAndRevisionResponse_FieldNumber_Revision = 2,
  GetIdentityBalanceAndRevisionResponse_FieldNumber_Proof = 3,
  GetIdentityBalanceAndRevisionResponse_FieldNumber_Metadata = 4,
};

GPB_FINAL @interface GetIdentityBalanceAndRevisionResponse : GPBMessage

@property(nonatomic, readwrite, strong, null_resettable) GPBUInt64Value *balance;
/** Test to see if @c balance has been set. */
@property(nonatomic, readwrite) BOOL hasBalance;

@property(nonatomic, readwrite, strong, null_resettable) GPBUInt64Value *revision;
/** Test to see if @c revision has been set. */
@property(nonatomic, readwrite) BOOL hasRevision;

@property(nonatomic, readwrite, strong, null_resettable) Proof *proof;
/** Test to see if @c proof has been set. */
@property(nonatomic, readwrite) BOOL hasProof;

@property(nonatomic, readwrite, strong, null_resettable) ResponseMetadata *metadata;
/** Test to see if @c metadata has been set. */
@property(nonatomic, readwrite) BOOL hasMetadata;

@end

#pragma mark - KeyRequestType

typedef GPB_ENUM(KeyRequestType_FieldNumber) {
  KeyRequestType_FieldNumber_AllKeys = 1,
  KeyRequestType_FieldNumber_SpecificKeys = 2,
  KeyRequestType_FieldNumber_SearchKey = 3,
};

typedef GPB_ENUM(KeyRequestType_Request_OneOfCase) {
  KeyRequestType_Request_OneOfCase_GPBUnsetOneOfCase = 0,
  KeyRequestType_Request_OneOfCase_AllKeys = 1,
  KeyRequestType_Request_OneOfCase_SpecificKeys = 2,
  KeyRequestType_Request_OneOfCase_SearchKey = 3,
};

GPB_FINAL @interface KeyRequestType : GPBMessage

@property(nonatomic, readonly) KeyRequestType_Request_OneOfCase requestOneOfCase;

@property(nonatomic, readwrite, strong, null_resettable) AllKeys *allKeys;

@property(nonatomic, readwrite, strong, null_resettable) SpecificKeys *specificKeys;

@property(nonatomic, readwrite, strong, null_resettable) SearchKey *searchKey;

@end

/**
 * Clears whatever value was set for the oneof 'request'.
 **/
void KeyRequestType_ClearRequestOneOfCase(KeyRequestType *message);

#pragma mark - AllKeys

GPB_FINAL @interface AllKeys : GPBMessage

@end

#pragma mark - SpecificKeys

typedef GPB_ENUM(SpecificKeys_FieldNumber) {
  SpecificKeys_FieldNumber_KeyIdsArray = 1,
};

GPB_FINAL @interface SpecificKeys : GPBMessage

@property(nonatomic, readwrite, strong, null_resettable) GPBUInt32Array *keyIdsArray;
/** The number of items in @c keyIdsArray without causing the array to be created. */
@property(nonatomic, readonly) NSUInteger keyIdsArray_Count;

@end

#pragma mark - SearchKey

typedef GPB_ENUM(SearchKey_FieldNumber) {
  SearchKey_FieldNumber_PurposeMap = 1,
};

GPB_FINAL @interface SearchKey : GPBMessage

@property(nonatomic, readwrite, strong, null_resettable) GPBUInt32ObjectDictionary<SecurityLevelMap*> *purposeMap;
/** The number of items in @c purposeMap without causing the array to be created. */
@property(nonatomic, readonly) NSUInteger purposeMap_Count;

@end

#pragma mark - SecurityLevelMap

typedef GPB_ENUM(SecurityLevelMap_FieldNumber) {
  SecurityLevelMap_FieldNumber_SecurityLevelMap = 1,
};

GPB_FINAL @interface SecurityLevelMap : GPBMessage

// |securityLevelMap| values are |SecurityLevelMap_KeyKindRequestType|
@property(nonatomic, readwrite, strong, null_resettable) GPBUInt32EnumDictionary *securityLevelMap;
/** The number of items in @c securityLevelMap without causing the array to be created. */
@property(nonatomic, readonly) NSUInteger securityLevelMap_Count;

@end

#pragma mark - GetIdentityKeysRequest

typedef GPB_ENUM(GetIdentityKeysRequest_FieldNumber) {
  GetIdentityKeysRequest_FieldNumber_IdentityId = 1,
  GetIdentityKeysRequest_FieldNumber_RequestType = 2,
  GetIdentityKeysRequest_FieldNumber_Limit = 3,
  GetIdentityKeysRequest_FieldNumber_Offset = 4,
  GetIdentityKeysRequest_FieldNumber_Prove = 5,
};

GPB_FINAL @interface GetIdentityKeysRequest : GPBMessage

@property(nonatomic, readwrite, copy, null_resettable) NSData *identityId;

@property(nonatomic, readwrite, strong, null_resettable) KeyRequestType *requestType;
/** Test to see if @c requestType has been set. */
@property(nonatomic, readwrite) BOOL hasRequestType;

@property(nonatomic, readwrite, strong, null_resettable) GPBUInt32Value *limit;
/** Test to see if @c limit has been set. */
@property(nonatomic, readwrite) BOOL hasLimit;

@property(nonatomic, readwrite, strong, null_resettable) GPBUInt32Value *offset;
/** Test to see if @c offset has been set. */
@property(nonatomic, readwrite) BOOL hasOffset;

@property(nonatomic, readwrite) BOOL prove;

@end

#pragma mark - GetIdentityKeysResponse

typedef GPB_ENUM(GetIdentityKeysResponse_FieldNumber) {
  GetIdentityKeysResponse_FieldNumber_Keys = 1,
  GetIdentityKeysResponse_FieldNumber_Proof = 2,
  GetIdentityKeysResponse_FieldNumber_Metadata = 3,
};

typedef GPB_ENUM(GetIdentityKeysResponse_Result_OneOfCase) {
  GetIdentityKeysResponse_Result_OneOfCase_GPBUnsetOneOfCase = 0,
  GetIdentityKeysResponse_Result_OneOfCase_Keys = 1,
  GetIdentityKeysResponse_Result_OneOfCase_Proof = 2,
};

GPB_FINAL @interface GetIdentityKeysResponse : GPBMessage

@property(nonatomic, readonly) GetIdentityKeysResponse_Result_OneOfCase resultOneOfCase;

@property(nonatomic, readwrite, strong, null_resettable) GetIdentityKeysResponse_Keys *keys;

@property(nonatomic, readwrite, strong, null_resettable) Proof *proof;

@property(nonatomic, readwrite, strong, null_resettable) ResponseMetadata *metadata;
/** Test to see if @c metadata has been set. */
@property(nonatomic, readwrite) BOOL hasMetadata;

@end

/**
 * Clears whatever value was set for the oneof 'result'.
 **/
void GetIdentityKeysResponse_ClearResultOneOfCase(GetIdentityKeysResponse *message);

#pragma mark - GetIdentityKeysResponse_Keys

typedef GPB_ENUM(GetIdentityKeysResponse_Keys_FieldNumber) {
  GetIdentityKeysResponse_Keys_FieldNumber_KeysBytesArray = 1,
};

GPB_FINAL @interface GetIdentityKeysResponse_Keys : GPBMessage

@property(nonatomic, readwrite, strong, null_resettable) NSMutableArray<NSData*> *keysBytesArray;
/** The number of items in @c keysBytesArray without causing the array to be created. */
@property(nonatomic, readonly) NSUInteger keysBytesArray_Count;

@end

#pragma mark - GetIdentitiesKeysRequest

typedef GPB_ENUM(GetIdentitiesKeysRequest_FieldNumber) {
  GetIdentitiesKeysRequest_FieldNumber_IdentityIdsArray = 1,
  GetIdentitiesKeysRequest_FieldNumber_RequestType = 2,
  GetIdentitiesKeysRequest_FieldNumber_Limit = 3,
  GetIdentitiesKeysRequest_FieldNumber_Offset = 4,
  GetIdentitiesKeysRequest_FieldNumber_Prove = 5,
};

GPB_FINAL @interface GetIdentitiesKeysRequest : GPBMessage

@property(nonatomic, readwrite, strong, null_resettable) NSMutableArray<NSData*> *identityIdsArray;
/** The number of items in @c identityIdsArray without causing the array to be created. */
@property(nonatomic, readonly) NSUInteger identityIdsArray_Count;

@property(nonatomic, readwrite, strong, null_resettable) KeyRequestType *requestType;
/** Test to see if @c requestType has been set. */
@property(nonatomic, readwrite) BOOL hasRequestType;

@property(nonatomic, readwrite, strong, null_resettable) GPBUInt32Value *limit;
/** Test to see if @c limit has been set. */
@property(nonatomic, readwrite) BOOL hasLimit;

@property(nonatomic, readwrite, strong, null_resettable) GPBUInt32Value *offset;
/** Test to see if @c offset has been set. */
@property(nonatomic, readwrite) BOOL hasOffset;

@property(nonatomic, readwrite) BOOL prove;

@end

#pragma mark - GetIdentitiesKeysRequest_SecurityLevelMap

typedef GPB_ENUM(GetIdentitiesKeysRequest_SecurityLevelMap_FieldNumber) {
  GetIdentitiesKeysRequest_SecurityLevelMap_FieldNumber_SecurityLevelMap = 1,
};

GPB_FINAL @interface GetIdentitiesKeysRequest_SecurityLevelMap : GPBMessage

// |securityLevelMap| values are |GetIdentitiesKeysRequest_SecurityLevelMap_KeyKindRequestType|
@property(nonatomic, readwrite, strong, null_resettable) GPBUInt32EnumDictionary *securityLevelMap;
/** The number of items in @c securityLevelMap without causing the array to be created. */
@property(nonatomic, readonly) NSUInteger securityLevelMap_Count;

@end

#pragma mark - GetIdentitiesKeysResponse

typedef GPB_ENUM(GetIdentitiesKeysResponse_FieldNumber) {
  GetIdentitiesKeysResponse_FieldNumber_PublicKeys = 1,
  GetIdentitiesKeysResponse_FieldNumber_Proof = 2,
  GetIdentitiesKeysResponse_FieldNumber_Metadata = 3,
};

typedef GPB_ENUM(GetIdentitiesKeysResponse_Result_OneOfCase) {
  GetIdentitiesKeysResponse_Result_OneOfCase_GPBUnsetOneOfCase = 0,
  GetIdentitiesKeysResponse_Result_OneOfCase_PublicKeys = 1,
  GetIdentitiesKeysResponse_Result_OneOfCase_Proof = 2,
};

GPB_FINAL @interface GetIdentitiesKeysResponse : GPBMessage

@property(nonatomic, readonly) GetIdentitiesKeysResponse_Result_OneOfCase resultOneOfCase;

@property(nonatomic, readwrite, strong, null_resettable) GetIdentitiesKeysResponse_PublicKeyEntries *publicKeys;

@property(nonatomic, readwrite, strong, null_resettable) Proof *proof;

@property(nonatomic, readwrite, strong, null_resettable) ResponseMetadata *metadata;
/** Test to see if @c metadata has been set. */
@property(nonatomic, readwrite) BOOL hasMetadata;

@end

/**
 * Clears whatever value was set for the oneof 'result'.
 **/
void GetIdentitiesKeysResponse_ClearResultOneOfCase(GetIdentitiesKeysResponse *message);

#pragma mark - GetIdentitiesKeysResponse_PublicKey

typedef GPB_ENUM(GetIdentitiesKeysResponse_PublicKey_FieldNumber) {
  GetIdentitiesKeysResponse_PublicKey_FieldNumber_Value = 1,
};

GPB_FINAL @interface GetIdentitiesKeysResponse_PublicKey : GPBMessage

@property(nonatomic, readwrite, copy, null_resettable) NSData *value;

@end

#pragma mark - GetIdentitiesKeysResponse_PublicKeyEntry

typedef GPB_ENUM(GetIdentitiesKeysResponse_PublicKeyEntry_FieldNumber) {
  GetIdentitiesKeysResponse_PublicKeyEntry_FieldNumber_Key = 1,
  GetIdentitiesKeysResponse_PublicKeyEntry_FieldNumber_Value = 2,
};

GPB_FINAL @interface GetIdentitiesKeysResponse_PublicKeyEntry : GPBMessage

@property(nonatomic, readwrite, copy, null_resettable) NSData *key;

@property(nonatomic, readwrite, strong, null_resettable) GetIdentitiesKeysResponse_PublicKey *value;
/** Test to see if @c value has been set. */
@property(nonatomic, readwrite) BOOL hasValue;

@end

#pragma mark - GetIdentitiesKeysResponse_PublicKeyEntries

typedef GPB_ENUM(GetIdentitiesKeysResponse_PublicKeyEntries_FieldNumber) {
  GetIdentitiesKeysResponse_PublicKeyEntries_FieldNumber_PublicKeyEntriesArray = 1,
};

GPB_FINAL @interface GetIdentitiesKeysResponse_PublicKeyEntries : GPBMessage

@property(nonatomic, readwrite, strong, null_resettable) NSMutableArray<GetIdentitiesKeysResponse_PublicKeyEntry*> *publicKeyEntriesArray;
/** The number of items in @c publicKeyEntriesArray without causing the array to be created. */
@property(nonatomic, readonly) NSUInteger publicKeyEntriesArray_Count;

@end

#pragma mark - GetDataContractRequest

typedef GPB_ENUM(GetDataContractRequest_FieldNumber) {
  GetDataContractRequest_FieldNumber_Id_p = 1,
  GetDataContractRequest_FieldNumber_Prove = 2,
};

GPB_FINAL @interface GetDataContractRequest : GPBMessage

@property(nonatomic, readwrite, copy, null_resettable) NSData *id_p;

@property(nonatomic, readwrite) BOOL prove;

@end

#pragma mark - GetDataContractResponse

typedef GPB_ENUM(GetDataContractResponse_FieldNumber) {
  GetDataContractResponse_FieldNumber_DataContract = 1,
  GetDataContractResponse_FieldNumber_Proof = 2,
  GetDataContractResponse_FieldNumber_Metadata = 3,
};

GPB_FINAL @interface GetDataContractResponse : GPBMessage

@property(nonatomic, readwrite, copy, null_resettable) NSData *dataContract;

@property(nonatomic, readwrite, strong, null_resettable) Proof *proof;
/** Test to see if @c proof has been set. */
@property(nonatomic, readwrite) BOOL hasProof;

@property(nonatomic, readwrite, strong, null_resettable) ResponseMetadata *metadata;
/** Test to see if @c metadata has been set. */
@property(nonatomic, readwrite) BOOL hasMetadata;

@end

#pragma mark - GetDataContractsRequest

typedef GPB_ENUM(GetDataContractsRequest_FieldNumber) {
  GetDataContractsRequest_FieldNumber_IdsArray = 1,
  GetDataContractsRequest_FieldNumber_Prove = 2,
};

GPB_FINAL @interface GetDataContractsRequest : GPBMessage

@property(nonatomic, readwrite, strong, null_resettable) NSMutableArray<NSData*> *idsArray;
/** The number of items in @c idsArray without causing the array to be created. */
@property(nonatomic, readonly) NSUInteger idsArray_Count;

@property(nonatomic, readwrite) BOOL prove;

@end

#pragma mark - GetDataContractsResponse

typedef GPB_ENUM(GetDataContractsResponse_FieldNumber) {
  GetDataContractsResponse_FieldNumber_DataContracts = 1,
  GetDataContractsResponse_FieldNumber_Proof = 2,
  GetDataContractsResponse_FieldNumber_Metadata = 3,
};

typedef GPB_ENUM(GetDataContractsResponse_Result_OneOfCase) {
  GetDataContractsResponse_Result_OneOfCase_GPBUnsetOneOfCase = 0,
  GetDataContractsResponse_Result_OneOfCase_DataContracts = 1,
  GetDataContractsResponse_Result_OneOfCase_Proof = 2,
};

GPB_FINAL @interface GetDataContractsResponse : GPBMessage

@property(nonatomic, readonly) GetDataContractsResponse_Result_OneOfCase resultOneOfCase;

@property(nonatomic, readwrite, strong, null_resettable) GetDataContractsResponse_DataContracts *dataContracts;

@property(nonatomic, readwrite, strong, null_resettable) Proof *proof;

@property(nonatomic, readwrite, strong, null_resettable) ResponseMetadata *metadata;
/** Test to see if @c metadata has been set. */
@property(nonatomic, readwrite) BOOL hasMetadata;

@end

/**
 * Clears whatever value was set for the oneof 'result'.
 **/
void GetDataContractsResponse_ClearResultOneOfCase(GetDataContractsResponse *message);

#pragma mark - GetDataContractsResponse_DataContractValue

typedef GPB_ENUM(GetDataContractsResponse_DataContractValue_FieldNumber) {
  GetDataContractsResponse_DataContractValue_FieldNumber_Value = 1,
};

GPB_FINAL @interface GetDataContractsResponse_DataContractValue : GPBMessage

@property(nonatomic, readwrite, copy, null_resettable) NSData *value;

@end

#pragma mark - GetDataContractsResponse_DataContractEntry

typedef GPB_ENUM(GetDataContractsResponse_DataContractEntry_FieldNumber) {
  GetDataContractsResponse_DataContractEntry_FieldNumber_Key = 1,
  GetDataContractsResponse_DataContractEntry_FieldNumber_Value = 2,
};

GPB_FINAL @interface GetDataContractsResponse_DataContractEntry : GPBMessage

@property(nonatomic, readwrite, copy, null_resettable) NSData *key;

@property(nonatomic, readwrite, strong, null_resettable) GetDataContractsResponse_DataContractValue *value;
/** Test to see if @c value has been set. */
@property(nonatomic, readwrite) BOOL hasValue;

@end

#pragma mark - GetDataContractsResponse_DataContracts

typedef GPB_ENUM(GetDataContractsResponse_DataContracts_FieldNumber) {
  GetDataContractsResponse_DataContracts_FieldNumber_DataContractEntriesArray = 1,
};

GPB_FINAL @interface GetDataContractsResponse_DataContracts : GPBMessage

@property(nonatomic, readwrite, strong, null_resettable) NSMutableArray<GetDataContractsResponse_DataContractEntry*> *dataContractEntriesArray;
/** The number of items in @c dataContractEntriesArray without causing the array to be created. */
@property(nonatomic, readonly) NSUInteger dataContractEntriesArray_Count;

@end

#pragma mark - GetDocumentsRequest

typedef GPB_ENUM(GetDocumentsRequest_FieldNumber) {
  GetDocumentsRequest_FieldNumber_DataContractId = 1,
  GetDocumentsRequest_FieldNumber_DocumentType = 2,
  GetDocumentsRequest_FieldNumber_Where = 3,
  GetDocumentsRequest_FieldNumber_OrderBy = 4,
  GetDocumentsRequest_FieldNumber_Limit = 5,
  GetDocumentsRequest_FieldNumber_StartAfter = 6,
  GetDocumentsRequest_FieldNumber_StartAt = 7,
  GetDocumentsRequest_FieldNumber_Prove = 8,
};

typedef GPB_ENUM(GetDocumentsRequest_Start_OneOfCase) {
  GetDocumentsRequest_Start_OneOfCase_GPBUnsetOneOfCase = 0,
  GetDocumentsRequest_Start_OneOfCase_StartAfter = 6,
  GetDocumentsRequest_Start_OneOfCase_StartAt = 7,
};

GPB_FINAL @interface GetDocumentsRequest : GPBMessage

@property(nonatomic, readwrite, copy, null_resettable) NSData *dataContractId;

@property(nonatomic, readwrite, copy, null_resettable) NSString *documentType;

@property(nonatomic, readwrite, copy, null_resettable) NSData *where;

@property(nonatomic, readwrite, copy, null_resettable) NSData *orderBy;

@property(nonatomic, readwrite) uint32_t limit;

@property(nonatomic, readonly) GetDocumentsRequest_Start_OneOfCase startOneOfCase;

@property(nonatomic, readwrite, copy, null_resettable) NSData *startAfter;

@property(nonatomic, readwrite, copy, null_resettable) NSData *startAt;

@property(nonatomic, readwrite) BOOL prove;

@end

/**
 * Clears whatever value was set for the oneof 'start'.
 **/
void GetDocumentsRequest_ClearStartOneOfCase(GetDocumentsRequest *message);

#pragma mark - GetDocumentsResponse

typedef GPB_ENUM(GetDocumentsResponse_FieldNumber) {
  GetDocumentsResponse_FieldNumber_DocumentsArray = 1,
  GetDocumentsResponse_FieldNumber_Proof = 2,
  GetDocumentsResponse_FieldNumber_Metadata = 3,
};

GPB_FINAL @interface GetDocumentsResponse : GPBMessage

@property(nonatomic, readwrite, strong, null_resettable) NSMutableArray<NSData*> *documentsArray;
/** The number of items in @c documentsArray without causing the array to be created. */
@property(nonatomic, readonly) NSUInteger documentsArray_Count;

@property(nonatomic, readwrite, strong, null_resettable) Proof *proof;
/** Test to see if @c proof has been set. */
@property(nonatomic, readwrite) BOOL hasProof;

@property(nonatomic, readwrite, strong, null_resettable) ResponseMetadata *metadata;
/** Test to see if @c metadata has been set. */
@property(nonatomic, readwrite) BOOL hasMetadata;

@end

#pragma mark - GetIdentitiesByPublicKeyHashesRequest

typedef GPB_ENUM(GetIdentitiesByPublicKeyHashesRequest_FieldNumber) {
  GetIdentitiesByPublicKeyHashesRequest_FieldNumber_PublicKeyHashesArray = 1,
  GetIdentitiesByPublicKeyHashesRequest_FieldNumber_Prove = 2,
};

GPB_FINAL @interface GetIdentitiesByPublicKeyHashesRequest : GPBMessage

@property(nonatomic, readwrite, strong, null_resettable) NSMutableArray<NSData*> *publicKeyHashesArray;
/** The number of items in @c publicKeyHashesArray without causing the array to be created. */
@property(nonatomic, readonly) NSUInteger publicKeyHashesArray_Count;

@property(nonatomic, readwrite) BOOL prove;

@end

#pragma mark - GetIdentitiesByPublicKeyHashesResponse

typedef GPB_ENUM(GetIdentitiesByPublicKeyHashesResponse_FieldNumber) {
  GetIdentitiesByPublicKeyHashesResponse_FieldNumber_IdentitiesArray = 1,
  GetIdentitiesByPublicKeyHashesResponse_FieldNumber_Proof = 2,
  GetIdentitiesByPublicKeyHashesResponse_FieldNumber_Metadata = 3,
};

GPB_FINAL @interface GetIdentitiesByPublicKeyHashesResponse : GPBMessage

@property(nonatomic, readwrite, strong, null_resettable) NSMutableArray<NSData*> *identitiesArray;
/** The number of items in @c identitiesArray without causing the array to be created. */
@property(nonatomic, readonly) NSUInteger identitiesArray_Count;

@property(nonatomic, readwrite, strong, null_resettable) Proof *proof;
/** Test to see if @c proof has been set. */
@property(nonatomic, readwrite) BOOL hasProof;

@property(nonatomic, readwrite, strong, null_resettable) ResponseMetadata *metadata;
/** Test to see if @c metadata has been set. */
@property(nonatomic, readwrite) BOOL hasMetadata;

@end

#pragma mark - GetIdentityByPublicKeyHashesRequest

typedef GPB_ENUM(GetIdentityByPublicKeyHashesRequest_FieldNumber) {
  GetIdentityByPublicKeyHashesRequest_FieldNumber_PublicKeyHash = 1,
  GetIdentityByPublicKeyHashesRequest_FieldNumber_Prove = 2,
};

GPB_FINAL @interface GetIdentityByPublicKeyHashesRequest : GPBMessage

@property(nonatomic, readwrite, copy, null_resettable) NSData *publicKeyHash;

@property(nonatomic, readwrite) BOOL prove;

@end

#pragma mark - GetIdentityByPublicKeyHashesResponse

typedef GPB_ENUM(GetIdentityByPublicKeyHashesResponse_FieldNumber) {
  GetIdentityByPublicKeyHashesResponse_FieldNumber_Identity = 1,
  GetIdentityByPublicKeyHashesResponse_FieldNumber_Proof = 2,
  GetIdentityByPublicKeyHashesResponse_FieldNumber_Metadata = 3,
};

typedef GPB_ENUM(GetIdentityByPublicKeyHashesResponse_Result_OneOfCase) {
  GetIdentityByPublicKeyHashesResponse_Result_OneOfCase_GPBUnsetOneOfCase = 0,
  GetIdentityByPublicKeyHashesResponse_Result_OneOfCase_Identity = 1,
  GetIdentityByPublicKeyHashesResponse_Result_OneOfCase_Proof = 2,
};

GPB_FINAL @interface GetIdentityByPublicKeyHashesResponse : GPBMessage

@property(nonatomic, readonly) GetIdentityByPublicKeyHashesResponse_Result_OneOfCase resultOneOfCase;

@property(nonatomic, readwrite, copy, null_resettable) NSData *identity;

@property(nonatomic, readwrite, strong, null_resettable) Proof *proof;

@property(nonatomic, readwrite, strong, null_resettable) ResponseMetadata *metadata;
/** Test to see if @c metadata has been set. */
@property(nonatomic, readwrite) BOOL hasMetadata;

@end

/**
 * Clears whatever value was set for the oneof 'result'.
 **/
void GetIdentityByPublicKeyHashesResponse_ClearResultOneOfCase(GetIdentityByPublicKeyHashesResponse *message);

#pragma mark - WaitForStateTransitionResultRequest

typedef GPB_ENUM(WaitForStateTransitionResultRequest_FieldNumber) {
  WaitForStateTransitionResultRequest_FieldNumber_StateTransitionHash = 1,
  WaitForStateTransitionResultRequest_FieldNumber_Prove = 2,
};

GPB_FINAL @interface WaitForStateTransitionResultRequest : GPBMessage

@property(nonatomic, readwrite, copy, null_resettable) NSData *stateTransitionHash;

@property(nonatomic, readwrite) BOOL prove;

@end

#pragma mark - WaitForStateTransitionResultResponse

typedef GPB_ENUM(WaitForStateTransitionResultResponse_FieldNumber) {
  WaitForStateTransitionResultResponse_FieldNumber_Error = 1,
  WaitForStateTransitionResultResponse_FieldNumber_Proof = 2,
  WaitForStateTransitionResultResponse_FieldNumber_Metadata = 3,
};

typedef GPB_ENUM(WaitForStateTransitionResultResponse_Responses_OneOfCase) {
  WaitForStateTransitionResultResponse_Responses_OneOfCase_GPBUnsetOneOfCase = 0,
  WaitForStateTransitionResultResponse_Responses_OneOfCase_Error = 1,
  WaitForStateTransitionResultResponse_Responses_OneOfCase_Proof = 2,
};

GPB_FINAL @interface WaitForStateTransitionResultResponse : GPBMessage

@property(nonatomic, readonly) WaitForStateTransitionResultResponse_Responses_OneOfCase responsesOneOfCase;

@property(nonatomic, readwrite, strong, null_resettable) StateTransitionBroadcastError *error;

@property(nonatomic, readwrite, strong, null_resettable) Proof *proof;

@property(nonatomic, readwrite, strong, null_resettable) ResponseMetadata *metadata;
/** Test to see if @c metadata has been set. */
@property(nonatomic, readwrite) BOOL hasMetadata;

@end

/**
 * Clears whatever value was set for the oneof 'responses'.
 **/
void WaitForStateTransitionResultResponse_ClearResponsesOneOfCase(WaitForStateTransitionResultResponse *message);

#pragma mark - ConsensusParamsBlock

typedef GPB_ENUM(ConsensusParamsBlock_FieldNumber) {
  ConsensusParamsBlock_FieldNumber_MaxBytes = 1,
  ConsensusParamsBlock_FieldNumber_MaxGas = 2,
  ConsensusParamsBlock_FieldNumber_TimeIotaMs = 3,
};

GPB_FINAL @interface ConsensusParamsBlock : GPBMessage

@property(nonatomic, readwrite, copy, null_resettable) NSString *maxBytes;

@property(nonatomic, readwrite, copy, null_resettable) NSString *maxGas;

@property(nonatomic, readwrite, copy, null_resettable) NSString *timeIotaMs;

@end

#pragma mark - ConsensusParamsEvidence

typedef GPB_ENUM(ConsensusParamsEvidence_FieldNumber) {
  ConsensusParamsEvidence_FieldNumber_MaxAgeNumBlocks = 1,
  ConsensusParamsEvidence_FieldNumber_MaxAgeDuration = 2,
  ConsensusParamsEvidence_FieldNumber_MaxBytes = 3,
};

GPB_FINAL @interface ConsensusParamsEvidence : GPBMessage

@property(nonatomic, readwrite, copy, null_resettable) NSString *maxAgeNumBlocks;

@property(nonatomic, readwrite, copy, null_resettable) NSString *maxAgeDuration;

@property(nonatomic, readwrite, copy, null_resettable) NSString *maxBytes;

@end

#pragma mark - GetConsensusParamsRequest

typedef GPB_ENUM(GetConsensusParamsRequest_FieldNumber) {
  GetConsensusParamsRequest_FieldNumber_Height = 1,
  GetConsensusParamsRequest_FieldNumber_Prove = 2,
};

GPB_FINAL @interface GetConsensusParamsRequest : GPBMessage

@property(nonatomic, readwrite) int64_t height;

@property(nonatomic, readwrite) BOOL prove;

@end

#pragma mark - GetConsensusParamsResponse

typedef GPB_ENUM(GetConsensusParamsResponse_FieldNumber) {
  GetConsensusParamsResponse_FieldNumber_Block = 1,
  GetConsensusParamsResponse_FieldNumber_Evidence = 2,
};

GPB_FINAL @interface GetConsensusParamsResponse : GPBMessage

@property(nonatomic, readwrite, strong, null_resettable) ConsensusParamsBlock *block;
/** Test to see if @c block has been set. */
@property(nonatomic, readwrite) BOOL hasBlock;

@property(nonatomic, readwrite, strong, null_resettable) ConsensusParamsEvidence *evidence;
/** Test to see if @c evidence has been set. */
@property(nonatomic, readwrite) BOOL hasEvidence;

@end

NS_ASSUME_NONNULL_END

CF_EXTERN_C_END

#pragma clang diagnostic pop

// @@protoc_insertion_point(global_scope)
