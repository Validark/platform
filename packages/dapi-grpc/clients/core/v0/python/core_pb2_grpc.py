# Generated by the gRPC Python protocol compiler plugin. DO NOT EDIT!
"""Client and server classes corresponding to protobuf-defined services."""
import grpc

import core_pb2 as core__pb2


class CoreStub(object):
    """Missing associated documentation comment in .proto file."""

    def __init__(self, channel):
        """Constructor.

        Args:
            channel: A grpc.Channel.
        """
        self.getBlockchainStatus = channel.unary_unary(
                '/org.dash.platform.dapi.v0.Core/getBlockchainStatus',
                request_serializer=core__pb2.GetBlockchainStatusRequest.SerializeToString,
                response_deserializer=core__pb2.GetBlockchainStatusResponse.FromString,
                )
        self.getMasternodeStatus = channel.unary_unary(
                '/org.dash.platform.dapi.v0.Core/getMasternodeStatus',
                request_serializer=core__pb2.GetMasternodeStatusRequest.SerializeToString,
                response_deserializer=core__pb2.GetMasternodeStatusResponse.FromString,
                )
        self.getBlock = channel.unary_unary(
                '/org.dash.platform.dapi.v0.Core/getBlock',
                request_serializer=core__pb2.GetBlockRequest.SerializeToString,
                response_deserializer=core__pb2.GetBlockResponse.FromString,
                )
        self.broadcastTransaction = channel.unary_unary(
                '/org.dash.platform.dapi.v0.Core/broadcastTransaction',
                request_serializer=core__pb2.BroadcastTransactionRequest.SerializeToString,
                response_deserializer=core__pb2.BroadcastTransactionResponse.FromString,
                )
        self.getTransaction = channel.unary_unary(
                '/org.dash.platform.dapi.v0.Core/getTransaction',
                request_serializer=core__pb2.GetTransactionRequest.SerializeToString,
                response_deserializer=core__pb2.GetTransactionResponse.FromString,
                )
        self.getEstimatedTransactionFee = channel.unary_unary(
                '/org.dash.platform.dapi.v0.Core/getEstimatedTransactionFee',
                request_serializer=core__pb2.GetEstimatedTransactionFeeRequest.SerializeToString,
                response_deserializer=core__pb2.GetEstimatedTransactionFeeResponse.FromString,
                )
        self.subscribeToBlockHeadersWithChainLocks = channel.unary_stream(
                '/org.dash.platform.dapi.v0.Core/subscribeToBlockHeadersWithChainLocks',
                request_serializer=core__pb2.BlockHeadersWithChainLocksRequest.SerializeToString,
                response_deserializer=core__pb2.BlockHeadersWithChainLocksResponse.FromString,
                )
        self.subscribeToTransactionsWithProofs = channel.unary_stream(
                '/org.dash.platform.dapi.v0.Core/subscribeToTransactionsWithProofs',
                request_serializer=core__pb2.TransactionsWithProofsRequest.SerializeToString,
                response_deserializer=core__pb2.TransactionsWithProofsResponse.FromString,
                )


class CoreServicer(object):
    """Missing associated documentation comment in .proto file."""

    def getBlockchainStatus(self, request, context):
        """Missing associated documentation comment in .proto file."""
        context.set_code(grpc.StatusCode.UNIMPLEMENTED)
        context.set_details('Method not implemented!')
        raise NotImplementedError('Method not implemented!')

    def getMasternodeStatus(self, request, context):
        """Missing associated documentation comment in .proto file."""
        context.set_code(grpc.StatusCode.UNIMPLEMENTED)
        context.set_details('Method not implemented!')
        raise NotImplementedError('Method not implemented!')

    def getBlock(self, request, context):
        """Missing associated documentation comment in .proto file."""
        context.set_code(grpc.StatusCode.UNIMPLEMENTED)
        context.set_details('Method not implemented!')
        raise NotImplementedError('Method not implemented!')

    def broadcastTransaction(self, request, context):
        """Missing associated documentation comment in .proto file."""
        context.set_code(grpc.StatusCode.UNIMPLEMENTED)
        context.set_details('Method not implemented!')
        raise NotImplementedError('Method not implemented!')

    def getTransaction(self, request, context):
        """Missing associated documentation comment in .proto file."""
        context.set_code(grpc.StatusCode.UNIMPLEMENTED)
        context.set_details('Method not implemented!')
        raise NotImplementedError('Method not implemented!')

    def getEstimatedTransactionFee(self, request, context):
        """Missing associated documentation comment in .proto file."""
        context.set_code(grpc.StatusCode.UNIMPLEMENTED)
        context.set_details('Method not implemented!')
        raise NotImplementedError('Method not implemented!')

    def subscribeToBlockHeadersWithChainLocks(self, request, context):
        """Missing associated documentation comment in .proto file."""
        context.set_code(grpc.StatusCode.UNIMPLEMENTED)
        context.set_details('Method not implemented!')
        raise NotImplementedError('Method not implemented!')

    def subscribeToTransactionsWithProofs(self, request, context):
        """Missing associated documentation comment in .proto file."""
        context.set_code(grpc.StatusCode.UNIMPLEMENTED)
        context.set_details('Method not implemented!')
        raise NotImplementedError('Method not implemented!')


def add_CoreServicer_to_server(servicer, server):
    rpc_method_handlers = {
            'getBlockchainStatus': grpc.unary_unary_rpc_method_handler(
                    servicer.getBlockchainStatus,
                    request_deserializer=core__pb2.GetBlockchainStatusRequest.FromString,
                    response_serializer=core__pb2.GetBlockchainStatusResponse.SerializeToString,
            ),
            'getMasternodeStatus': grpc.unary_unary_rpc_method_handler(
                    servicer.getMasternodeStatus,
                    request_deserializer=core__pb2.GetMasternodeStatusRequest.FromString,
                    response_serializer=core__pb2.GetMasternodeStatusResponse.SerializeToString,
            ),
            'getBlock': grpc.unary_unary_rpc_method_handler(
                    servicer.getBlock,
                    request_deserializer=core__pb2.GetBlockRequest.FromString,
                    response_serializer=core__pb2.GetBlockResponse.SerializeToString,
            ),
            'broadcastTransaction': grpc.unary_unary_rpc_method_handler(
                    servicer.broadcastTransaction,
                    request_deserializer=core__pb2.BroadcastTransactionRequest.FromString,
                    response_serializer=core__pb2.BroadcastTransactionResponse.SerializeToString,
            ),
            'getTransaction': grpc.unary_unary_rpc_method_handler(
                    servicer.getTransaction,
                    request_deserializer=core__pb2.GetTransactionRequest.FromString,
                    response_serializer=core__pb2.GetTransactionResponse.SerializeToString,
            ),
            'getEstimatedTransactionFee': grpc.unary_unary_rpc_method_handler(
                    servicer.getEstimatedTransactionFee,
                    request_deserializer=core__pb2.GetEstimatedTransactionFeeRequest.FromString,
                    response_serializer=core__pb2.GetEstimatedTransactionFeeResponse.SerializeToString,
            ),
            'subscribeToBlockHeadersWithChainLocks': grpc.unary_stream_rpc_method_handler(
                    servicer.subscribeToBlockHeadersWithChainLocks,
                    request_deserializer=core__pb2.BlockHeadersWithChainLocksRequest.FromString,
                    response_serializer=core__pb2.BlockHeadersWithChainLocksResponse.SerializeToString,
            ),
            'subscribeToTransactionsWithProofs': grpc.unary_stream_rpc_method_handler(
                    servicer.subscribeToTransactionsWithProofs,
                    request_deserializer=core__pb2.TransactionsWithProofsRequest.FromString,
                    response_serializer=core__pb2.TransactionsWithProofsResponse.SerializeToString,
            ),
    }
    generic_handler = grpc.method_handlers_generic_handler(
            'org.dash.platform.dapi.v0.Core', rpc_method_handlers)
    server.add_generic_rpc_handlers((generic_handler,))


 # This class is part of an EXPERIMENTAL API.
class Core(object):
    """Missing associated documentation comment in .proto file."""

    @staticmethod
    def getBlockchainStatus(request,
            target,
            options=(),
            channel_credentials=None,
            call_credentials=None,
            insecure=False,
            compression=None,
            wait_for_ready=None,
            timeout=None,
            metadata=None):
        return grpc.experimental.unary_unary(request, target, '/org.dash.platform.dapi.v0.Core/getBlockchainStatus',
            core__pb2.GetBlockchainStatusRequest.SerializeToString,
            core__pb2.GetBlockchainStatusResponse.FromString,
            options, channel_credentials,
            insecure, call_credentials, compression, wait_for_ready, timeout, metadata)

    @staticmethod
    def getMasternodeStatus(request,
            target,
            options=(),
            channel_credentials=None,
            call_credentials=None,
            insecure=False,
            compression=None,
            wait_for_ready=None,
            timeout=None,
            metadata=None):
        return grpc.experimental.unary_unary(request, target, '/org.dash.platform.dapi.v0.Core/getMasternodeStatus',
            core__pb2.GetMasternodeStatusRequest.SerializeToString,
            core__pb2.GetMasternodeStatusResponse.FromString,
            options, channel_credentials,
            insecure, call_credentials, compression, wait_for_ready, timeout, metadata)

    @staticmethod
    def getBlock(request,
            target,
            options=(),
            channel_credentials=None,
            call_credentials=None,
            insecure=False,
            compression=None,
            wait_for_ready=None,
            timeout=None,
            metadata=None):
        return grpc.experimental.unary_unary(request, target, '/org.dash.platform.dapi.v0.Core/getBlock',
            core__pb2.GetBlockRequest.SerializeToString,
            core__pb2.GetBlockResponse.FromString,
            options, channel_credentials,
            insecure, call_credentials, compression, wait_for_ready, timeout, metadata)

    @staticmethod
    def broadcastTransaction(request,
            target,
            options=(),
            channel_credentials=None,
            call_credentials=None,
            insecure=False,
            compression=None,
            wait_for_ready=None,
            timeout=None,
            metadata=None):
        return grpc.experimental.unary_unary(request, target, '/org.dash.platform.dapi.v0.Core/broadcastTransaction',
            core__pb2.BroadcastTransactionRequest.SerializeToString,
            core__pb2.BroadcastTransactionResponse.FromString,
            options, channel_credentials,
            insecure, call_credentials, compression, wait_for_ready, timeout, metadata)

    @staticmethod
    def getTransaction(request,
            target,
            options=(),
            channel_credentials=None,
            call_credentials=None,
            insecure=False,
            compression=None,
            wait_for_ready=None,
            timeout=None,
            metadata=None):
        return grpc.experimental.unary_unary(request, target, '/org.dash.platform.dapi.v0.Core/getTransaction',
            core__pb2.GetTransactionRequest.SerializeToString,
            core__pb2.GetTransactionResponse.FromString,
            options, channel_credentials,
            insecure, call_credentials, compression, wait_for_ready, timeout, metadata)

    @staticmethod
    def getEstimatedTransactionFee(request,
            target,
            options=(),
            channel_credentials=None,
            call_credentials=None,
            insecure=False,
            compression=None,
            wait_for_ready=None,
            timeout=None,
            metadata=None):
        return grpc.experimental.unary_unary(request, target, '/org.dash.platform.dapi.v0.Core/getEstimatedTransactionFee',
            core__pb2.GetEstimatedTransactionFeeRequest.SerializeToString,
            core__pb2.GetEstimatedTransactionFeeResponse.FromString,
            options, channel_credentials,
            insecure, call_credentials, compression, wait_for_ready, timeout, metadata)

    @staticmethod
    def subscribeToBlockHeadersWithChainLocks(request,
            target,
            options=(),
            channel_credentials=None,
            call_credentials=None,
            insecure=False,
            compression=None,
            wait_for_ready=None,
            timeout=None,
            metadata=None):
        return grpc.experimental.unary_stream(request, target, '/org.dash.platform.dapi.v0.Core/subscribeToBlockHeadersWithChainLocks',
            core__pb2.BlockHeadersWithChainLocksRequest.SerializeToString,
            core__pb2.BlockHeadersWithChainLocksResponse.FromString,
            options, channel_credentials,
            insecure, call_credentials, compression, wait_for_ready, timeout, metadata)

    @staticmethod
    def subscribeToTransactionsWithProofs(request,
            target,
            options=(),
            channel_credentials=None,
            call_credentials=None,
            insecure=False,
            compression=None,
            wait_for_ready=None,
            timeout=None,
            metadata=None):
        return grpc.experimental.unary_stream(request, target, '/org.dash.platform.dapi.v0.Core/subscribeToTransactionsWithProofs',
            core__pb2.TransactionsWithProofsRequest.SerializeToString,
            core__pb2.TransactionsWithProofsResponse.FromString,
            options, channel_credentials,
            insecure, call_credentials, compression, wait_for_ready, timeout, metadata)
