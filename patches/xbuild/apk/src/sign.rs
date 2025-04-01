use anyhow::Result;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use rasn_pkix::Certificate;
use rsa::pkcs8::{DecodePublicKey, EncodePublicKey};
use rsa::{PaddingScheme, PublicKey, RsaPublicKey};
use sha2::{Digest as _, Sha256};
use std::fs::File;
use std::io::{BufReader, Cursor, Read, Seek, SeekFrom, Write};
use std::path::Path;
use xcommon::{Signer, ZipInfo};

const DEBUG_PEM: &str = include_str!("../assets/debug.pem");

const APK_SIGNING_BLOCK_MAGIC: &[u8] = b"APK Sig Block 42";
const APK_SIGNING_BLOCK_V2_ID: u32 = 0x7109871a;
const APK_SIGNING_BLOCK_V3_ID: u32 = 0xf05368c0;
const APK_SIGNING_BLOCK_V4_ID: u32 = 0x42726577;
const RSA_PKCS1V15_SHA2_256: u32 = 0x0103;
const MAX_CHUNK_SIZE: usize = 1024 * 1024;

pub fn verify(path: &Path) -> Result<Vec<Certificate>> {
    let f = File::open(path)?;
    let mut r = BufReader::new(f);
    let sblock = parse_apk_signing_block(&mut r)?;
    let mut sblockv2 = None;
    for block in &sblock.blocks {
        match block.id {
            APK_SIGNING_BLOCK_V2_ID => {
                tracing::debug!("v2 signing block");
                sblockv2 = Some(*block);
            }
            APK_SIGNING_BLOCK_V3_ID => {
                tracing::debug!("v3 signing block");
            }
            APK_SIGNING_BLOCK_V4_ID => {
                tracing::debug!("v4 signing block");
            }
            id => {
                tracing::debug!("unknown signing block 0x{:x}", id);
            }
        }
    }
    let block = if let Some(block) = sblockv2 {
        r.seek(SeekFrom::Start(block.start))?;
        ApkSignatureBlockV2::read(&mut r)?
    } else {
        anyhow::bail!("no signing block v2 found");
    };
    let zip_hash = compute_digest(&mut r, sblock.sb_start, sblock.cd_start, sblock.cde_start)?;
    let mut certificates = vec![];
    for signer in &block.signers {
        anyhow::ensure!(
            !signer.signatures.is_empty(),
            "found no signatures in v2 block"
        );
        for sig in &signer.signatures {
            anyhow::ensure!(
                sig.algorithm == RSA_PKCS1V15_SHA2_256,
                "found unsupported signature algorithm 0x{:x}",
                sig.algorithm
            );
            let pubkey = RsaPublicKey::from_public_key_der(&signer.public_key)?;
            let digest = Sha256::digest(&signer.signed_data);
            let padding = PaddingScheme::new_pkcs1v15_sign::<sha2::Sha256>();
            pubkey.verify(padding, &digest, &sig.signature)?;
        }
        let mut r = Cursor::new(&signer.signed_data[..]);
        let signed_data = SignedData::read(&mut r)?;
        anyhow::ensure!(
            !signed_data.digests.is_empty(),
            "found no digests in v2 block"
        );
        for digest in &signed_data.digests {
            anyhow::ensure!(
                digest.algorithm == RSA_PKCS1V15_SHA2_256,
                "found unsupported digest algorithm 0x{:x}",
                digest.algorithm
            );
            anyhow::ensure!(
                digest.digest == zip_hash,
                "computed hash doesn't match signed hash."
            );
        }
        for cert in &signed_data.certificates {
            let cert =
                rasn::der::decode::<Certificate>(cert).map_err(|err| anyhow::anyhow!("{}", err))?;
            certificates.push(cert);
        }
        for attr in &signed_data.additional_attributes {
            tracing::debug!("v2: additional attribute: 0x{:x} {:?}", attr.0, &attr.1);
        }
    }
    Ok(certificates)
}

pub fn sign(path: &Path, signer: Option<Signer>) -> Result<()> {
    let signer = signer.map(Ok).unwrap_or_else(|| Signer::new(DEBUG_PEM))?;
    let apk = std::fs::read(path)?;
    let mut r = Cursor::new(&apk);
    let block = parse_apk_signing_block(&mut r)?;
    let zip_hash = compute_digest(&mut r, block.sb_start, block.cd_start, block.cde_start)?;
    let mut nblock = vec![];
    let mut w = Cursor::new(&mut nblock);
    write_apk_signing_block(&mut w, zip_hash, &signer)?;
    let mut f = File::create(path)?;
    f.write_all(&apk[..(block.sb_start as usize)])?;
    f.write_all(&nblock)?;
    let cd_start = f.stream_position()?;
    f.write_all(&apk[(block.cd_start as usize)..(block.cde_start as usize)])?;
    let cde_start = f.stream_position()?;
    f.write_all(&apk[(block.cde_start as usize)..])?;
    f.seek(SeekFrom::Start(cde_start + 16))?;
    f.write_u32::<LittleEndian>(cd_start as u32)?;
    Ok(())
}

fn compute_digest<R: Read + Seek>(
    r: &mut R,
    sb_start: u64,
    cd_start: u64,
    cde_start: u64,
) -> Result<[u8; 32]> {
    let mut chunks = vec![];
    let mut hasher = Sha256::new();
    let mut chunk = vec![0u8; MAX_CHUNK_SIZE];

    // chunk contents
    r.rewind()?;
    let mut pos = 0;
    while pos < sb_start {
        hash_chunk(&mut chunks, r, sb_start, &mut hasher, &mut chunk, &mut pos)?;
    }

    // chunk cd
    let mut pos = r.seek(SeekFrom::Start(cd_start))?;
    while pos < cde_start {
        hash_chunk(&mut chunks, r, cde_start, &mut hasher, &mut chunk, &mut pos)?;
    }

    // chunk cde
    chunk.clear();
    r.read_to_end(&mut chunk)?;
    let mut cursor = Cursor::new(&mut chunk);
    cursor.seek(SeekFrom::Start(16))?;
    cursor.write_u32::<LittleEndian>(sb_start as u32)?;
    hasher.update([0xa5]);
    assert!(chunk.len() <= MAX_CHUNK_SIZE);
    hasher.update((chunk.len() as u32).to_le_bytes());
    hasher.update(chunk);
    chunks.push(hasher.finalize_reset().into());

    // compute root
    hasher.update([0x5a]);
    hasher.update((chunks.len() as u32).to_le_bytes());
    for chunk in &chunks {
        hasher.update(chunk);
    }
    Ok(hasher.finalize().into())
}

fn hash_chunk<R: Read + Seek>(
    chunks: &mut Vec<[u8; 32]>,
    r: &mut R,
    size: u64,
    hasher: &mut Sha256,
    buffer: &mut Vec<u8>,
    pos: &mut u64,
) -> Result<()> {
    let end = std::cmp::min(*pos + MAX_CHUNK_SIZE as u64, size);
    let len = (end - *pos) as usize;
    buffer.resize(len, 0);
    r.read_exact(buffer).unwrap();
    hasher.update([0xa5]);
    hasher.update((len as u32).to_le_bytes());
    hasher.update(buffer);
    chunks.push(hasher.finalize_reset().into());
    *pos = end;
    Ok(())
}

#[derive(Debug, Default)]
struct Digest {
    pub algorithm: u32,
    pub digest: Vec<u8>,
}

impl Digest {
    fn new(hash: [u8; 32]) -> Self {
        Self {
            algorithm: RSA_PKCS1V15_SHA2_256,
            digest: hash.to_vec(),
        }
    }

    fn size(&self) -> u32 {
        self.digest.len() as u32 + 12
    }

    fn read(r: &mut impl Read) -> Result<Self> {
        let _digest_size = r.read_u32::<LittleEndian>()?;
        let algorithm = r.read_u32::<LittleEndian>()?;
        let size = r.read_u32::<LittleEndian>()?;
        let mut digest = vec![0; size as usize as _];
        r.read_exact(&mut digest)?;
        Ok(Self { algorithm, digest })
    }

    fn write(&self, w: &mut impl Write) -> Result<()> {
        w.write_u32::<LittleEndian>(self.digest.len() as u32 + 8)?;
        w.write_u32::<LittleEndian>(self.algorithm)?;
        w.write_u32::<LittleEndian>(self.digest.len() as u32)?;
        w.write_all(&self.digest)?;
        Ok(())
    }
}

#[derive(Debug, Default)]
struct SignedData {
    pub digests: Vec<Digest>,
    pub certificates: Vec<Vec<u8>>,
    pub additional_attributes: Vec<(u32, Vec<u8>)>,
}

impl SignedData {
    fn new(hash: [u8; 32], signer: &Signer) -> Result<Self> {
        Ok(Self {
            digests: vec![Digest::new(hash)],
            certificates: vec![
                rasn::der::encode(signer.cert()).map_err(|err| anyhow::anyhow!("{}", err))?
            ],
            additional_attributes: vec![],
        })
    }

    fn read(r: &mut impl Read) -> Result<Self> {
        let mut signed_data = SignedData::default();
        let mut remaining_digests_size = r.read_u32::<LittleEndian>()?;
        while remaining_digests_size > 0 {
            let digest = Digest::read(r)?;
            remaining_digests_size -= digest.size();
            signed_data.digests.push(digest);
        }
        let mut remaining_certificates_size = r.read_u32::<LittleEndian>()?;
        while remaining_certificates_size > 0 {
            let length = r.read_u32::<LittleEndian>()?;
            let mut cert = vec![0; length as usize];
            r.read_exact(&mut cert)?;
            signed_data.certificates.push(cert);
            remaining_certificates_size -= length + 4;
        }
        let mut remaining_additional_attributes_size = r.read_u32::<LittleEndian>()?;
        while remaining_additional_attributes_size > 0 {
            let length = r.read_u32::<LittleEndian>()?;
            let id = r.read_u32::<LittleEndian>()?;
            let mut value = vec![0; length as usize - 4];
            r.read_exact(&mut value)?;
            signed_data.additional_attributes.push((id, value));
            remaining_additional_attributes_size -= length + 4;
        }
        Ok(signed_data)
    }

    fn write(&self, w: &mut impl Write) -> Result<()> {
        w.write_u32::<LittleEndian>(self.digests.iter().map(|d| d.size()).sum())?;
        for digest in &self.digests {
            digest.write(w)?;
        }
        w.write_u32::<LittleEndian>(self.certificates.iter().map(|c| c.len() as u32 + 4).sum())?;
        for cert in &self.certificates {
            w.write_u32::<LittleEndian>(cert.len() as u32)?;
            w.write_all(cert)?;
        }
        w.write_u32::<LittleEndian>(
            self.additional_attributes
                .iter()
                .map(|(_, v)| v.len() as u32 + 8)
                .sum(),
        )?;
        for (id, value) in &self.additional_attributes {
            w.write_u32::<LittleEndian>(value.len() as u32 + 4)?;
            w.write_u32::<LittleEndian>(*id)?;
            w.write_all(value)?;
        }
        Ok(())
    }
}

#[derive(Debug)]
struct ApkSignatureBlockV2 {
    pub signers: Vec<ApkSigner>,
}

#[derive(Debug)]
struct ApkSigner {
    pub signed_data: Vec<u8>,
    pub signatures: Vec<ApkSignature>,
    pub public_key: Vec<u8>,
}

#[derive(Debug)]
struct ApkSignature {
    pub algorithm: u32,
    pub signature: Vec<u8>,
}

impl ApkSignatureBlockV2 {
    fn new(hash: [u8; 32], signer: &Signer) -> Result<Self> {
        let mut signed_data = vec![];
        SignedData::new(hash, signer)?.write(&mut signed_data)?;
        let signature = signer.sign(&signed_data);
        Ok(Self {
            signers: vec![ApkSigner {
                signed_data,
                signatures: vec![ApkSignature {
                    algorithm: RSA_PKCS1V15_SHA2_256,
                    signature,
                }],
                public_key: signer.pubkey().to_public_key_der()?.as_ref().to_vec(),
            }],
        })
    }

    fn read(r: &mut impl Read) -> Result<Self> {
        let mut signers = vec![];
        let mut remaining_size = r.read_u32::<LittleEndian>()? as u64;
        while remaining_size > 0 {
            let signer_size = r.read_u32::<LittleEndian>()?;

            let signed_data_size = r.read_u32::<LittleEndian>()?;
            let mut signed_data = vec![0; signed_data_size as _];
            r.read_exact(&mut signed_data)?;

            let mut signatures = vec![];
            let mut remaining_signature_size = r.read_u32::<LittleEndian>()?;
            while remaining_signature_size > 0 {
                let signature_size = r.read_u32::<LittleEndian>()?;
                let algorithm = r.read_u32::<LittleEndian>()?;
                let size = r.read_u32::<LittleEndian>()?;
                let mut signature = vec![0; size as usize];
                r.read_exact(&mut signature)?;
                signatures.push(ApkSignature {
                    algorithm,
                    signature,
                });
                remaining_signature_size -= signature_size + 4;
            }

            let public_key_size = r.read_u32::<LittleEndian>()?;
            let mut public_key = vec![0; public_key_size as _];
            r.read_exact(&mut public_key)?;

            signers.push(ApkSigner {
                signed_data,
                signatures,
                public_key,
            });
            remaining_size -= signer_size as u64 + 4;
        }
        Ok(ApkSignatureBlockV2 { signers })
    }

    fn write(&self, w: &mut impl Write) -> Result<()> {
        let mut buffer = vec![];
        for signer in &self.signers {
            let mut signer_buffer = vec![];
            signer_buffer.write_u32::<LittleEndian>(signer.signed_data.len() as u32)?;
            signer_buffer.write_all(&signer.signed_data)?;
            let mut sig_buffer = vec![];
            for sig in &signer.signatures {
                sig_buffer.write_u32::<LittleEndian>(sig.signature.len() as u32 + 8)?;
                sig_buffer.write_u32::<LittleEndian>(sig.algorithm)?;
                sig_buffer.write_u32::<LittleEndian>(sig.signature.len() as u32)?;
                sig_buffer.write_all(&sig.signature)?;
            }
            signer_buffer.write_u32::<LittleEndian>(sig_buffer.len() as u32)?;
            signer_buffer.write_all(&sig_buffer)?;
            signer_buffer.write_u32::<LittleEndian>(signer.public_key.len() as u32)?;
            signer_buffer.write_all(&signer.public_key)?;
            buffer.write_u32::<LittleEndian>(signer_buffer.len() as u32)?;
            buffer.write_all(&signer_buffer)?;
        }
        w.write_u32::<LittleEndian>(buffer.len() as u32)?;
        w.write_all(&buffer)?;
        Ok(())
    }
}

#[derive(Debug, Default)]
struct ApkSignatureBlock {
    pub blocks: Vec<ApkOpaqueBlock>,
    pub sb_start: u64,
    pub cd_start: u64,
    pub cde_start: u64,
}

#[derive(Clone, Copy, Debug)]
struct ApkOpaqueBlock {
    pub id: u32,
    pub start: u64,
}

fn write_apk_signing_block<W: Write + Seek>(
    w: &mut W,
    hash: [u8; 32],
    signer: &Signer,
) -> Result<()> {
    let mut buf = vec![];
    ApkSignatureBlockV2::new(hash, signer)?.write(&mut buf)?;
    let size = buf.len() as u64 + 36;
    w.write_u64::<LittleEndian>(size)?;
    w.write_u64::<LittleEndian>(buf.len() as u64 + 4)?;
    w.write_u32::<LittleEndian>(APK_SIGNING_BLOCK_V2_ID)?;
    w.write_all(&buf)?;
    w.write_u64::<LittleEndian>(size)?;
    w.write_all(APK_SIGNING_BLOCK_MAGIC)?;
    Ok(())
}

fn parse_apk_signing_block<R: Read + Seek>(r: &mut R) -> Result<ApkSignatureBlock> {
    let info = ZipInfo::new(r)?;
    let mut block = ApkSignatureBlock {
        cde_start: info.cde_start,
        cd_start: info.cd_start,
        ..Default::default()
    };
    r.seek(SeekFrom::Start(block.cd_start - 16 - 8))?;
    let mut remaining_size = r.read_u64::<LittleEndian>()?;
    let mut magic = [0; 16];
    r.read_exact(&mut magic)?;
    if magic != APK_SIGNING_BLOCK_MAGIC {
        block.sb_start = block.cd_start;
        return Ok(block);
    }
    let mut pos = r.seek(SeekFrom::Current(-(remaining_size as i64)))?;
    block.sb_start = pos - 8;
    while remaining_size > 24 {
        let length = r.read_u64::<LittleEndian>()?;
        let id = r.read_u32::<LittleEndian>()?;
        block.blocks.push(ApkOpaqueBlock {
            id,
            start: pos + 8 + 4,
        });
        pos = r.seek(SeekFrom::Start(pos + length + 8))?;
        remaining_size -= length + 8;
    }
    Ok(block)
}
