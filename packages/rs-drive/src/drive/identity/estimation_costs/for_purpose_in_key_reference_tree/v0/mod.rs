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
use crate::drive::identity::estimation_costs::KEY_REFERENCE_SIZE;
use dpp::identity::{Purpose, SecurityLevel};
use grovedb::EstimatedSumTrees::{NoSumTrees, SomeSumTrees};
use std::collections::HashMap;

impl Drive {
    pub(super) fn add_estimation_costs_for_purpose_in_key_reference_tree_v0(
        identity_id: [u8; 32],
        estimated_costs_only_with_layer_info: &mut HashMap<KeyInfoPath, EstimatedLayerInformation>,
        purpose: Purpose,
    ) {
        let estimated_layer_count = match purpose {
            Purpose::AUTHENTICATION => ApproximateElements(4),
            Purpose::ENCRYPTION => {
                unreachable!()
            }
            Purpose::DECRYPTION => {
                unreachable!()
            }
            Purpose::WITHDRAW => ApproximateElements(1),
            Purpose::SYSTEM => ApproximateElements(1),
            Purpose::VOTING => ApproximateElements(1),
        };

        let estimated_layer_sizes = match purpose {
            Purpose::AUTHENTICATION => AllSubtrees(1, NoSumTrees, None),
            Purpose::ENCRYPTION => {
                unreachable!()
            }
            Purpose::DECRYPTION => {
                unreachable!()
            }
            Purpose::WITHDRAW => AllReference(1, KEY_REFERENCE_SIZE, None),
            Purpose::SYSTEM => AllReference(1, KEY_REFERENCE_SIZE, None),
            Purpose::VOTING => AllReference(1, KEY_REFERENCE_SIZE, None),
        };
        // we then need to insert the identity keys layer
        estimated_costs_only_with_layer_info.insert(
            KeyInfoPath::from_known_owned_path(identity_query_keys_purpose_tree_path_vec(
                identity_id.as_slice(),
                purpose,
            )),
            EstimatedLayerInformation {
                is_sum_tree: false,
                estimated_layer_count, // there are
                //We can mark these as all subtrees, because the revision will be under
                estimated_layer_sizes,
            },
        );
    }
}