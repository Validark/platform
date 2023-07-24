use crate::drive::defaults::{AVERAGE_BALANCE_SIZE, DEFAULT_HASH_SIZE_U8};

use crate::drive::{identity_tree_path, Drive};

use grovedb::batch::KeyInfoPath;
use grovedb::EstimatedLayerCount::{ApproximateElements, EstimatedLevel, PotentiallyAtMaxElements};
use grovedb::EstimatedLayerInformation;
use grovedb::EstimatedLayerSizes::{AllItems, AllReference, AllSubtrees, Mix};

use crate::drive::identity::{
    identity_key_tree_path_vec, identity_path_vec, identity_query_keys_purpose_tree_path_vec,
    identity_query_keys_security_level_tree_path_vec, identity_query_keys_tree_path_vec,
};

use crate::drive::balances::balance_path_vec;
use dpp::identity::{Purpose, SecurityLevel};
use grovedb::EstimatedSumTrees::{NoSumTrees, SomeSumTrees};
use std::collections::HashMap;

impl Drive {
    pub(super) fn add_estimation_costs_for_negative_credit_v0(
        identity_id: [u8; 32],
        estimated_costs_only_with_layer_info: &mut HashMap<KeyInfoPath, EstimatedLayerInformation>,
    ) {
        // we then need to insert the root identity layer
        estimated_costs_only_with_layer_info.insert(
            KeyInfoPath::from_known_path(identity_tree_path()),
            EstimatedLayerInformation {
                is_sum_tree: false,
                estimated_layer_count: PotentiallyAtMaxElements,
                estimated_layer_sizes: AllSubtrees(DEFAULT_HASH_SIZE_U8, NoSumTrees, None),
            },
        );

        // In this layer we have
        //                   Keys
        //              /         \
        //DataContract Info         Revision
        //       /                   /
        //  Negative Credit      Query Keys

        // we then need to insert the identity layer for fee refunds
        estimated_costs_only_with_layer_info.insert(
            KeyInfoPath::from_known_owned_path(identity_path_vec(identity_id.as_slice())),
            EstimatedLayerInformation {
                is_sum_tree: false,
                estimated_layer_count: EstimatedLevel(2, false),
                //We can mark these as all subtrees, because the revision will be under
                estimated_layer_sizes: Mix {
                    subtrees_size: Some((1, NoSumTrees, None, 2)),
                    items_size: Some((1, 8, None, 1)),
                    references_size: None,
                },
            },
        );
    }
}