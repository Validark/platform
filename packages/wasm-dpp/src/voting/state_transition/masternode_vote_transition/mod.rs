mod to_object;

use wasm_bindgen::{JsError, JsValue};
use wasm_bindgen::prelude::wasm_bindgen;
use dpp::identifier::Identifier;
use dpp::identity::KeyType;
use dpp::platform_value::{BinaryData, string_encoding};
use dpp::platform_value::string_encoding::Encoding;
use dpp::serialization::PlatformSerializable;
use dpp::state_transition::masternode_vote_transition::MasternodeVoteTransition;
use dpp::state_transition::{StateTransition, StateTransitionIdentitySigned, StateTransitionLike};
use dpp::state_transition::masternode_vote_transition::accessors::MasternodeVoteTransitionAccessorsV0;
use dpp::version::PlatformVersion;
use crate::bls_adapter::{BlsAdapter, JsBlsAdapter};
use crate::buffer::Buffer;
use crate::errors::from_dpp_err;
use crate::identifier::IdentifierWrapper;
use crate::{IdentityPublicKeyWasm, with_js_error};
use crate::utils::WithJsError;

#[derive(Clone)]
#[wasm_bindgen(js_name=MasternodeVoteTransition)]
pub struct MasternodeVoteTransitionWasm(MasternodeVoteTransition);

impl From<MasternodeVoteTransition> for MasternodeVoteTransitionWasm {
    fn from(v: MasternodeVoteTransition) -> Self {
        MasternodeVoteTransitionWasm(v)
    }
}

impl From<MasternodeVoteTransitionWasm> for MasternodeVoteTransition {
    fn from(val: MasternodeVoteTransitionWasm) -> Self {
        val.0
    }
}

#[wasm_bindgen(js_class=MasternodeVoteTransition)]
impl MasternodeVoteTransitionWasm {
    #[wasm_bindgen(constructor)]
    pub fn new(platform_version: u32) -> Result<MasternodeVoteTransitionWasm, JsValue> {
        let platform_version =
            &PlatformVersion::get(platform_version).map_err(|e| JsValue::from(e.to_string()))?;

        MasternodeVoteTransition::default_versioned(platform_version)
            .map(Into::into)
            .map_err(from_dpp_err)
    }

    #[wasm_bindgen(js_name=getType)]
    pub fn get_type(&self) -> u8 {
        self.0.state_transition_type() as u8
    }

    #[wasm_bindgen(getter, js_name=proTxHash)]
    pub fn pro_tx_hash(&self) -> IdentifierWrapper {
        self.get_pro_tx_hash()
    }

    #[wasm_bindgen(js_name=getIdentityId)]
    pub fn get_pro_tx_hash(&self) -> IdentifierWrapper {
        self.0.pro_tx_hash().into()
    }
    
    #[wasm_bindgen(js_name=setIdentityId)]
    pub fn set_pro_tx_hash(&mut self, pro_tx_hash: &IdentifierWrapper) {
        self.0.set_pro_tx_hash(pro_tx_hash.into());
    }
    #[wasm_bindgen(js_name=toObject)]
    pub fn to_object(&self, options: JsValue) -> Result<JsValue, JsValue> {
        let opts: self::to_object::ToObjectOptions = if options.is_object() {
            with_js_error!(serde_wasm_bindgen::from_value(options))?
        } else {
            Default::default()
        };

        let object = self::to_object::to_object_struct(&self.0, opts);
        let js_object = js_sys::Object::new();

        js_sys::Reflect::set(
            &js_object,
            &"type".to_owned().into(),
            &object.transition_type.into(),
        )?;

        let version = match self.0 {
            MasternodeVoteTransition::V0(_) => "0",
        };

        js_sys::Reflect::set(&js_object, &"$version".to_owned().into(), &version.into())?;

        if let Some(signature) = object.signature {
            let signature_value: JsValue = if signature.is_empty() {
                JsValue::undefined()
            } else {
                Buffer::from_bytes(signature.as_slice()).into()
            };

            js_sys::Reflect::set(&js_object, &"signature".to_owned().into(), &signature_value)?;

            if let Some(signature_public_key_id) = object.signature_public_key_id {
                js_sys::Reflect::set(
                    &js_object,
                    &"signaturePublicKeyId".to_owned().into(),
                    &JsValue::from(signature_public_key_id),
                )?;
            } else {
                js_sys::Reflect::set(
                    &js_object,
                    &"signaturePublicKeyId".to_owned().into(),
                    &JsValue::undefined(),
                )?;
            }
        }

        js_sys::Reflect::set(
            &js_object,
            &"proTxHash".to_owned().into(),
            &Buffer::from_bytes(object.pro_tx_hash.to_buffer().as_slice()),
        )?;

        //todo: reflect vote

        Ok(js_object.into())
    }

    #[wasm_bindgen(js_name=toBuffer)]
    pub fn to_buffer(&self) -> Result<Buffer, JsValue> {
        let bytes = PlatformSerializable::serialize_to_bytes(
            &StateTransition::MasternodeVote(self.0.clone()),
        )
            .with_js_error()?;
        Ok(Buffer::from_bytes(&bytes))
    }

    #[wasm_bindgen(js_name=toJSON)]
    pub fn to_json(&self) -> Result<JsValue, JsValue> {
        let object = self::to_object::to_object_struct(&self.0, Default::default());
        let js_object = js_sys::Object::new();

        js_sys::Reflect::set(
            &js_object,
            &"type".to_owned().into(),
            &object.transition_type.into(),
        )?;

        let version = match self.0 {
            MasternodeVoteTransition::V0(_) => "0",
        };

        js_sys::Reflect::set(&js_object, &"$version".to_owned().into(), &version.into())?;

        if let Some(signature) = object.signature {
            let signature_value: JsValue = if signature.is_empty() {
                JsValue::undefined()
            } else {
                string_encoding::encode(signature.as_slice(), Encoding::Base64).into()
            };

            js_sys::Reflect::set(&js_object, &"signature".to_owned().into(), &signature_value)?;

            if let Some(signature_public_key_id) = object.signature_public_key_id {
                js_sys::Reflect::set(
                    &js_object,
                    &"signaturePublicKeyId".to_owned().into(),
                    &signature_public_key_id.into(),
                )?;
            } else {
                js_sys::Reflect::set(
                    &js_object,
                    &"signaturePublicKeyId".to_owned().into(),
                    &JsValue::undefined(),
                )?;
            }
        }

        let pro_tx_hash = object.pro_tx_hash.to_string(Encoding::Base58);

        js_sys::Reflect::set(
            &js_object,
            &"proTxHash".to_owned().into(),
            &pro_tx_hash.into(),
        )?;

        // todo: reflect vote

        Ok(js_object.into())
    }

    #[wasm_bindgen(js_name=getModifiedDataIds)]
    pub fn modified_data_ids(&self) -> Vec<JsValue> {
        let ids = self.0.modified_data_ids();

        ids.into_iter()
            .map(|id| <IdentifierWrapper as std::convert::From<Identifier>>::from(id).into())
            .collect()
    }

    #[wasm_bindgen(js_name=isDataContractStateTransition)]
    pub fn is_data_contract_state_transition(&self) -> bool {
        self.0.is_data_contract_state_transition()
    }

    #[wasm_bindgen(js_name=isDocumentStateTransition)]
    pub fn is_document_state_transition(&self) -> bool {
        self.0.is_document_state_transition()
    }

    #[wasm_bindgen(js_name=isIdentityStateTransition)]
    pub fn is_identity_state_transition(&self) -> bool {
        self.0.is_identity_state_transition()
    }

    #[wasm_bindgen(js_name=isVotingStateTransition)]
    pub fn is_voting_state_transition(&self) -> bool {
        self.0.is_voting_state_transition()
    }

    #[wasm_bindgen(js_name=signByPrivateKey)]
    pub fn sign_by_private_key(
        &mut self,
        private_key: Vec<u8>,
        key_type: u8,
        bls: Option<JsBlsAdapter>,
    ) -> Result<(), JsValue> {
        let key_type = key_type
            .try_into()
            .map_err(|e: anyhow::Error| e.to_string())?;

        if bls.is_none() && key_type == KeyType::BLS12_381 {
            return Err(JsError::new(
                format!("BLS adapter is required for BLS key type '{}'", key_type).as_str(),
            )
                .into());
        }

        let bls_adapter = if let Some(adapter) = bls {
            BlsAdapter(adapter)
        } else {
            BlsAdapter(JsValue::undefined().into())
        };

        // TODO: not the best approach because it involves cloning the transition
        // Probably it worth to return `sign_by_private_key` per state transition
        let mut wrapper = StateTransition::MasternodeVote(self.0.clone());
        wrapper
            .sign_by_private_key(private_key.as_slice(), key_type, &bls_adapter)
            .with_js_error()?;

        self.0.set_signature(wrapper.signature().to_owned());

        Ok(())
    }

    #[wasm_bindgen(js_name=getSignature)]
    pub fn get_signature(&self) -> Buffer {
        Buffer::from_bytes(self.0.signature().as_slice())
    }

    #[wasm_bindgen(js_name=setSignature)]
    pub fn set_signature(&mut self, signature: Option<Vec<u8>>) {
        self.0
            .set_signature(BinaryData::new(signature.unwrap_or_default()))
    }

    #[wasm_bindgen]
    pub fn sign(
        &mut self,
        identity_public_key: &IdentityPublicKeyWasm,
        private_key: Vec<u8>,
        bls: JsBlsAdapter,
    ) -> Result<(), JsValue> {
        let bls_adapter = BlsAdapter(bls);

        // TODO: come up with a better way to set signature to the binding.
        let mut state_transition = StateTransition::MasternodeVote(self.0.clone());
        state_transition
            .sign(
                &identity_public_key.to_owned().into(),
                &private_key,
                &bls_adapter,
            )
            .with_js_error()?;

        let signature = state_transition.signature().to_owned();
        let signature_public_key_id = state_transition.signature_public_key_id().unwrap_or(0);

        self.0.set_signature(signature);
        self.0.set_signature_public_key_id(signature_public_key_id);

        Ok(())
    }
}
