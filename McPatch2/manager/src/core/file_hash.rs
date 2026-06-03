//! 计算文件哈希相关操作

use std::io::Read;

use crc::Crc;
use crc::CRC_16_IBM_SDLC;
use crc::CRC_64_XZ;
use sha2::Digest;
use sha2::Sha256;
use tokio::io::AsyncRead;
use tokio::io::AsyncReadExt;

/// 计算文件哈希值
pub fn calculate_hash(read: &mut impl Read) -> String {
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 16 * 1024];

    loop {
        let count = read.read(&mut buffer).unwrap();

        if count == 0 {
            break;
        }

        hasher.update(&buffer[0..count]);
    }

    base16ct::lower::encode_string(&hasher.finalize())
}

/// 计算文件哈希值
pub async fn calculate_hash_async(read: &mut (impl AsyncRead + Unpin)) -> String {
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 16 * 1024];

    tokio::pin!(read);

    loop {
        let count = read.read(&mut buffer).await.unwrap();

        if count == 0 {
            break;
        }

        hasher.update(&buffer[0..count]);
    }

    base16ct::lower::encode_string(&hasher.finalize())
}

pub fn calculate_hash_for_expected(read: &mut impl Read, expected: &str) -> String {
    if is_legacy_hash(expected) {
        calculate_legacy_hash(read)
    } else {
        calculate_hash(read)
    }
}

pub fn matches_expected_hash(read: &mut impl Read, expected: &str) -> bool {
    calculate_hash_for_expected(read, expected).eq_ignore_ascii_case(expected)
}

fn calculate_legacy_hash(read: &mut impl Read) -> String {
    let crc64 = Crc::<u64>::new(&CRC_64_XZ);
    let mut crc64 = crc64.digest();

    let crc16 = Crc::<u16>::new(&CRC_16_IBM_SDLC);
    let mut crc16 = crc16.digest();

    let mut buffer = [0u8; 16 * 1024];

    loop {
        let count = read.read(&mut buffer).unwrap();

        if count == 0 {
            break;
        }

        crc64.update(&buffer[0..count]);
        crc16.update(&buffer[0..count]);
    }

    format!("{:016x}_{:04x}", &crc64.finalize(), crc16.finalize())
}

fn is_legacy_hash(expected: &str) -> bool {
    let bytes = expected.as_bytes();
    bytes.len() == 21
        && bytes[16] == b'_'
        && bytes[..16].iter().all(|b| b.is_ascii_hexdigit())
        && bytes[17..].iter().all(|b| b.is_ascii_hexdigit())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calculate_hash_returns_sha256_hex() {
        let mut reader = std::io::Cursor::new(b"mcpatch");
        assert_eq!(
            calculate_hash(&mut reader),
            "d548b12edf3314f09885c4a5e54b357d593abc72fd277cc96acac236a13181bd"
        );
    }

    #[test]
    fn calculate_hash_for_expected_keeps_legacy_crc_compatibility() {
        let mut reader = std::io::Cursor::new(b"mcpatch");
        let expected = "15d48d2db7cc7de9_5e10";
        assert_eq!(calculate_hash_for_expected(&mut reader, expected), expected);
    }

    #[test]
    fn matches_expected_hash_accepts_sha256_and_legacy_crc() {
        let sha256_expected =
            "d548b12edf3314f09885c4a5e54b357d593abc72fd277cc96acac236a13181bd";
        let legacy_expected = "15d48d2db7cc7de9_5e10";

        let mut sha256_reader = std::io::Cursor::new(b"mcpatch");
        assert!(matches_expected_hash(&mut sha256_reader, sha256_expected));

        let mut legacy_reader = std::io::Cursor::new(b"mcpatch");
        assert!(matches_expected_hash(&mut legacy_reader, legacy_expected));
    }
}
