use super::pkcs7::{build_pkcs7, SPC_INDIRECT_DATA_OBJID, SPC_SIPINFO_OBJID};
use crate::Signer;
use anyhow::Result;
use byteorder::{BigEndian, ReadBytesExt};
use rasn::prelude::*;
use rasn_cms::pkcs7_compat::{EncapsulatedContentInfo, SignedData};
use rasn_cms::{ContentInfo, CONTENT_SIGNED_DATA};
use std::fs::File;
use std::io::Read;
use std::path::Path;
use zip::ZipArchive;

const P7X_MAGIC: u32 = 0x504b4358;

pub fn read_p7x(path: &Path) -> Result<SignedData> {
    let f = File::open(path)?;
    let mut zip = ZipArchive::new(f)?;
    let mut r = zip.by_name("AppxSignature.p7x")?;
    let magic = r.read_u32::<BigEndian>()?;
    anyhow::ensure!(magic == P7X_MAGIC, "not a valid p7x file");
    let mut der = vec![];
    r.read_to_end(&mut der)?;
    let info = rasn::der::decode::<ContentInfo>(&der).map_err(|err| anyhow::anyhow!("{}", err))?;
    anyhow::ensure!(CONTENT_SIGNED_DATA == info.content_type);
    let data = rasn::der::decode::<SignedData>(info.content.as_bytes())
        .map_err(|err| anyhow::anyhow!("{}", err))?;
    Ok(data)
}

pub fn p7x(signer: &Signer, digests: &Digests) -> Vec<u8> {
    let payload = Payload::encode(digests);
    let encap_content_info = EncapsulatedContentInfo {
        content_type: SPC_INDIRECT_DATA_OBJID.into(),
        content: Any::new(payload),
    };
    let signed_data = build_pkcs7(signer, encap_content_info);
    let content_info = ContentInfo {
        content_type: CONTENT_SIGNED_DATA.into(),
        content: Any::new(rasn::der::encode(&signed_data).unwrap()),
    };
    let mut p7x = vec![];
    p7x.extend_from_slice(&P7X_MAGIC.to_be_bytes());
    p7x.extend(rasn::der::encode(&content_info).unwrap());
    p7x
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Digests {
    /// Zip contents hash.
    pub axpc: [u8; 32],
    /// Zip central directory hash.
    pub axcd: [u8; 32],
    /// Content types hash (uncompressed).
    pub axct: [u8; 32],
    /// Block map hash (uncompressed).
    pub axbm: [u8; 32],
    /// Content integrity hash (uncompressed, optional).
    pub axci: [u8; 32],
}

#[derive(AsnType, Clone, Debug, Eq, Encode, PartialEq)]
#[rasn(tag(context, 0))]
struct Payload {
    indirect_data: SpcIndirectData,
}

impl Payload {
    pub fn encode(digests: &Digests) -> Vec<u8> {
        let indirect_data = SpcIndirectData::new(digests);
        rasn::der::encode(&Self { indirect_data }).unwrap()
    }
}

#[derive(AsnType, Clone, Debug, Eq, Encode, PartialEq)]
struct SpcIndirectData {
    sip_info: SpcSipInfo,
    content: SpcIndirectDataContent,
}

impl SpcIndirectData {
    pub fn new(digests: &Digests) -> Self {
        let mut payload = Vec::with_capacity(184);
        payload.extend_from_slice(b"APPX");
        payload.extend_from_slice(b"AXPC");
        payload.extend_from_slice(&digests.axpc);
        payload.extend_from_slice(b"AXCD");
        payload.extend_from_slice(&digests.axcd);
        payload.extend_from_slice(b"AXCT");
        payload.extend_from_slice(&digests.axct);
        payload.extend_from_slice(b"AXBM");
        payload.extend_from_slice(&digests.axbm);
        payload.extend_from_slice(b"AXCI");
        payload.extend_from_slice(&digests.axci);
        Self {
            sip_info: Default::default(),
            content: SpcIndirectDataContent::new(payload),
        }
    }
}

#[derive(AsnType, Clone, Debug, Eq, Encode, PartialEq)]
struct SpcIndirectDataContent {
    oid: [Open; 2],
    payload: OctetString,
}

impl SpcIndirectDataContent {
    pub fn new(payload: Vec<u8>) -> Self {
        Self {
            oid: [
                Open::ObjectIdentifier(Oid::JOINT_ISO_ITU_T_COUNTRY_US_ORGANIZATION_GOV_CSOR_NIST_ALGORITHMS_HASH_SHA256.into()),
                Open::Null,
            ],
            payload: OctetString::from(payload),
        }
    }
}

#[derive(AsnType, Clone, Debug, Eq, Encode, PartialEq)]
struct SpcSipInfo {
    oid: ObjectIdentifier,
    data: SpcSipInfoContent,
}

impl Default for SpcSipInfo {
    fn default() -> Self {
        Self {
            oid: SPC_SIPINFO_OBJID.into(),
            data: Default::default(),
        }
    }
}

#[derive(AsnType, Clone, Debug, Eq, Encode, PartialEq)]
struct SpcSipInfoContent {
    i1: u32,
    s1: OctetString,
    i2: u32,
    i3: u32,
    i4: u32,
    i5: u32,
    i6: u32,
}

impl Default for SpcSipInfoContent {
    fn default() -> Self {
        const SPC_SIPINFO_MAGIC_INT: u32 = 0x0101_0000;
        const SPC_SIPINFO_MAGIC: [u8; 16] = [
            0x4b, 0xdf, 0xc5, 0x0a, 0x07, 0xce, 0xe2, 0x4d, 0xb7, 0x6e, 0x23, 0xc8, 0x39, 0xa0,
            0x9f, 0xd1,
        ];
        Self {
            i1: SPC_SIPINFO_MAGIC_INT,
            s1: OctetString::from(SPC_SIPINFO_MAGIC.to_vec()),
            i2: 0,
            i3: 0,
            i4: 0,
            i5: 0,
            i6: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rsa::pkcs8::DecodePrivateKey;
    use rsa::{PaddingScheme, RsaPrivateKey};
    use sha2::{Digest, Sha256};

    const HASHES: Digests = Digests {
        axpc: [
            37, 112, 168, 185, 134, 72, 152, 136, 166, 55, 137, 233, 153, 167, 81, 229, 137, 229,
            158, 190, 214, 181, 211, 85, 93, 172, 161, 115, 74, 99, 165, 156,
        ],
        axcd: [
            29, 87, 205, 69, 139, 92, 201, 89, 248, 232, 221, 244, 67, 120, 231, 192, 229, 135,
            200, 178, 242, 207, 83, 145, 88, 83, 238, 30, 255, 54, 226, 31,
        ],
        axct: [
            188, 251, 66, 139, 217, 90, 175, 33, 93, 159, 193, 116, 124, 19, 113, 188, 195, 138,
            75, 212, 185, 133, 87, 115, 195, 93, 4, 189, 198, 152, 59, 190,
        ],
        axbm: [
            228, 142, 202, 253, 204, 232, 223, 220, 131, 162, 12, 252, 106, 74, 3, 180, 190, 71,
            230, 173, 146, 218, 209, 13, 101, 4, 43, 186, 70, 46, 196, 194,
        ],
        axci: [
            175, 56, 231, 224, 95, 58, 68, 216, 201, 155, 33, 50, 3, 124, 19, 157, 149, 107, 194,
            174, 170, 108, 34, 110, 128, 107, 240, 29, 11, 129, 67, 233,
        ],
    };

    #[test]
    fn sp_indirect_data() {
        let orig_indirect_data = [
            160, 130, 1, 8, 48, 130, 1, 4, // hash rest
            48, 53, // oid 1.3.6.1.4.1.311.2.1.30
            6, 10, 43, 6, 1, 4, 1, 130, 55, 2, 1, 30, // start sequence
            48, 39, // integer
            2, 4, 1, 1, 0, 0, // octet string tag
            4, 16, // octet string payload
            75, 223, 197, 10, 7, 206, 226, 77, 183, 110, 35, 200, 57, 160, 159, 209,
            // int 0
            2, 1, 0, // int 0
            2, 1, 0, // int 0
            2, 1, 0, // int 0
            2, 1, 0, // int 0
            2, 1, 0, // start sequence
            48, 129, 202, // start sequence
            48, 13, // oid 2.16.840.1.101.3.4.2.1
            6, 9, 96, 134, 72, 1, 101, 3, 4, 2, 1, // null
            5, 0, // octet string tag
            4, 129, 184, // octet string bytes
            65, 80, 80, 88, // signature
            65, 88, 80, 67, // axpc signature
            37, 112, 168, 185, 134, 72, 152, 136, // axpc hash
            166, 55, 137, 233, 153, 167, 81, 229, 137, 229, 158, 190, 214, 181, 211, 85, 93, 172,
            161, 115, 74, 99, 165, 156, // end axpc hash
            65, 88, 67, 68, // axcd signature
            29, 87, 205, 69, 139, 92, 201, 89, // axcd hash
            248, 232, 221, 244, 67, 120, 231, 192, 229, 135, 200, 178, 242, 207, 83, 145, 88, 83,
            238, 30, 255, 54, 226, 31, // end axcd hash
            65, 88, 67, 84, // axct signature
            188, 251, 66, 139, 217, 90, 175, 33, // axct hash
            93, 159, 193, 116, 124, 19, 113, 188, 195, 138, 75, 212, 185, 133, 87, 115, 195, 93, 4,
            189, 198, 152, 59, 190, // end axct hash
            65, 88, 66, 77, // axbm signature
            228, 142, 202, 253, 204, 232, 223, 220, // axbm hash
            131, 162, 12, 252, 106, 74, 3, 180, 190, 71, 230, 173, 146, 218, 209, 13, 101, 4, 43,
            186, 70, 46, 196, 194, // end axbm hash
            65, 88, 67, 73, // axci signature
            175, 56, 231, 224, 95, 58, 68, 216, // axci hash
            201, 155, 33, 50, 3, 124, 19, 157, 149, 107, 194, 174, 170, 108, 34, 110, 128, 107,
            240, 29, 11, 129, 67, 233, // end axci hash
        ];
        let indirect_data = Payload::encode(&HASHES);
        let (rem, res) = der_parser::parse_der(&indirect_data).unwrap();
        assert!(rem.is_empty());
        println!("{:#?}", res);
        assert_eq!(indirect_data, orig_indirect_data);
    }

    #[test]
    #[ignore]
    fn payload_digest_and_sign() {
        let payload = Payload::encode(&HASHES);
        let encap_content_info = EncapsulatedContentInfo {
            content_type: SPC_INDIRECT_DATA_OBJID.into(),
            content: Any::new(payload),
        };
        let digest = Sha256::digest(&encap_content_info.content.as_bytes()[8..]);
        let orig_digest = [
            68, 234, 15, 167, 40, 66, 12, 133, 19, 239, 228, 168, 72, 147, 90, 139, 75, 131, 41,
            111, 247, 70, 28, 251, 130, 190, 57, 136, 200, 159, 93, 116,
        ];
        assert_eq!(&orig_digest[..], &digest[..]);
        let orig_signature = b"\x7f\x13uP\xc8m:\x99\xb6\x89u\x85y\xea\xfc\xd8Cw\x96w\x10>j\xa7Z\x8c\xa3\x1f\\\xf4\x82\\\xdf\x8eh;\x10\x16o/\"i\x89\xb9\xf1\x03\x9c\xb0)\x9f\xc4\xfe\xf1\x05\x93\xbeJ\xd2\xeb\xe3\xb1f\xb1rq\x89\xdf\x7f\xe4\xe1\n\xae\xa70\x8c|\xd3\xe6\xe6/\xad\x97\xcb1\xb6\xa0\xf9\x16z\x83R#\xe8n\r\xfdErJ\x01\xfb\xd4\xef\x05\xf9\xab\x08o\x16\xbc)C\xee\x03=$\x88>G\xa4\xba)\xbc\xf4n6\xaa\xfd\xa7e\x15\xb9,|\xd6\xf9\x9b>\xe8\x95\xf7\xc6\x08\n\t\x8a\xd5{j\x8a\xfe{,O\xf3\xd9\x8a\xc79\x9f\x80\xcd\x17k8\xf8\xb3\xc3\x96\xd8\x1a/\xa8\x14R\x14\xaf\x813\x91;>\x99\xd24\x86J\x12\x0e\x89\x0c\xb8?\xfa\xa8\x1dM\x98@vz'\xe6y\xab\xc0\xcb\xc5\xb3\xbeC'$\"\xd2\x15\xaf0\xa3\x05\xcbj\x18j\x11\xa2\xfd\xe7\xe6y\xcf\xadd\x99\xa9\xdc\xc4\xc2`\x1d\xb0\xe3\xdb\xfeC\xdc\xce\xe5@\xde;P\xfav\x8c\xff";
        let key = RsaPrivateKey::from_pkcs8_pem(crate::DEBUG_PEM).unwrap();
        let padding = PaddingScheme::new_pkcs1v15_sign::<sha2::Sha256>();
        let sig = key.sign(padding, &digest).unwrap();
        assert_eq!(sig.len(), orig_signature.len());
        assert_eq!(sig, orig_signature);
    }
}
