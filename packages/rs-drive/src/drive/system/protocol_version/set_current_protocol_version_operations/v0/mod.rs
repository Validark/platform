use grovedb::{Element, TransactionArg};
use integer_encoding::VarInt;
use dpp::util::deserializer::ProtocolVersion;
use dpp::version::drive_versions::DriveVersion;
use crate::drive::Drive;
use crate::drive::grove_operations::BatchInsertApplyType;
use crate::drive::object_size_info::PathKeyElementInfo;
use crate::drive::system::misc_path;
use crate::drive::system::misc_tree_constants::{NEXT_PROTOCOL_VERSION_STORAGE_KEY, PROTOCOL_VERSION_STORAGE_KEY};
use crate::error::Error;
use crate::fee::op::LowLevelDriveOperation;


impl Drive {
    /// Sets the current protocol version
    pub(super) fn set_current_protocol_version_operations_v0(
        &self,
        protocol_version: ProtocolVersion,
        transaction: TransactionArg,
        drive_operations: &mut Vec<LowLevelDriveOperation>,
        drive_version: &DriveVersion,
    ) -> Result<(), Error> {
        self.batch_insert_if_changed_value(
            PathKeyElementInfo::PathFixedSizeKeyRefElement((
                misc_path(),
                PROTOCOL_VERSION_STORAGE_KEY,
                Element::new_item(protocol_version.encode_var_vec()),
            )),
            BatchInsertApplyType::StatefulBatchInsert,
            transaction,
            drive_operations,
            drive_version,
        )?;
        Ok(())
    }
}
