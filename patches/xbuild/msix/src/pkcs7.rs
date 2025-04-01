use crate::Signer;
use rasn::prelude::*;
use rasn_cms::pkcs7_compat::{EncapsulatedContentInfo, SignedData};
use rasn_cms::{AlgorithmIdentifier, IssuerAndSerialNumber, SignerIdentifier, SignerInfo};
use rasn_pkix::Attribute;
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;

pub const SPC_INDIRECT_DATA_OBJID: ConstOid = ConstOid(&[1, 3, 6, 1, 4, 1, 311, 2, 1, 4]);
pub const SPC_SP_OPUS_INFO_OBJID: ConstOid = ConstOid(&[1, 3, 6, 1, 4, 1, 311, 2, 1, 12]);
pub const SPC_SIPINFO_OBJID: ConstOid = ConstOid(&[1, 3, 6, 1, 4, 1, 311, 2, 1, 30]);

#[allow(clippy::mutable_key_type)]
pub fn build_pkcs7(signer: &Signer, encap_content_info: EncapsulatedContentInfo) -> SignedData {
    let digest = Sha256::digest(&encap_content_info.content.as_bytes()[8..]);
    let signature = signer.sign(&encap_content_info.content.as_bytes()[8..]);
    let cert = signer.cert();

    let digest_algorithm = AlgorithmIdentifier {
        algorithm:
            Oid::JOINT_ISO_ITU_T_COUNTRY_US_ORGANIZATION_GOV_CSOR_NIST_ALGORITHMS_HASH_SHA256.into(),
        parameters: Some(Any::new(vec![5, 0])),
    };
    let signer_info = SignerInfo {
        version: 1.into(),
        sid: SignerIdentifier::IssuerAndSerialNumber(IssuerAndSerialNumber {
            issuer: cert.tbs_certificate.issuer.clone(),
            serial_number: cert.tbs_certificate.serial_number.clone(),
        }),
        digest_algorithm: digest_algorithm.clone(),
        signed_attrs: Some({
            let mut signed_attrs = SetOf::default();
            signed_attrs.insert(Attribute {
                r#type: Oid::ISO_MEMBER_BODY_US_RSADSI_PKCS9_CONTENT_TYPE.into(),
                values: {
                    let oid = ObjectIdentifier::from(SPC_INDIRECT_DATA_OBJID);
                    let mut content_type = BTreeSet::default();
                    content_type.insert(Any::new(rasn::der::encode(&oid).unwrap()));
                    content_type
                },
            });
            signed_attrs.insert(Attribute {
                r#type: Oid::ISO_MEMBER_BODY_US_RSADSI_PKCS9_MESSAGE_DIGEST.into(),
                values: {
                    let digest = OctetString::from(digest.to_vec());
                    let mut digests = BTreeSet::default();
                    digests.insert(Any::new(rasn::der::encode(&digest).unwrap()));
                    digests
                },
            });
            signed_attrs.insert(Attribute {
                r#type: SPC_SP_OPUS_INFO_OBJID.into(),
                values: Default::default(),
            });
            signed_attrs
        }),
        signature_algorithm: AlgorithmIdentifier {
            algorithm: Oid::ISO_MEMBER_BODY_US_RSADSI_PKCS1.into(),
            parameters: Some(Any::new(vec![5, 0])),
        },
        signature: OctetString::from(signature.to_vec()),
        unsigned_attrs: Some({
            // TODO: 1.3.6.1.4.1.311.3.3.1 timestamp? optional?
            SetOf::default()
        }),
    };
    SignedData {
        version: 1.into(),
        digest_algorithms: {
            let mut digest_algorithms = SetOf::default();
            digest_algorithms.insert(digest_algorithm);
            digest_algorithms
        },
        encap_content_info,
        certificates: Some(SetOf::default()),
        crls: None,
        signer_infos: {
            let mut signer_infos = SetOf::default();
            signer_infos.insert(signer_info);
            signer_infos
        },
    }
}
