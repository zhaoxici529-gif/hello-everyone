//! API Key 的本地加密存储。
//!
//! 方案：AES-256-GCM。
//! - 主密钥 32 字节，优先托管在操作系统凭据管理器（Windows 凭据管理器 / macOS 钥匙串），
//!   取不到时退化成应用数据目录下的密钥文件；
//! - 每个 API Key 单独一个随机 nonce，密文格式 `v1:<base64(nonce || ciphertext)>`；
//! - 数据库里只存密文，明文只在内存里短暂存在。

use std::fs;
use std::path::Path;

use aes_gcm::aead::{Aead, AeadCore, KeyInit, OsRng};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine as _;

const SERVICE: &str = "自媒体AI工作台";
const ACCOUNT: &str = "master-key";
const KEY_FILE: &str = "secrets.key";
const VERSION_PREFIX: &str = "v1:";
const NONCE_LEN: usize = 12;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeySource {
    /// 系统凭据管理器
    Keyring,
    /// 应用数据目录下的密钥文件（凭据管理器不可用时的退路）
    KeyFile,
}

impl KeySource {
    pub fn id(self) -> &'static str {
        match self {
            KeySource::Keyring => "keyring",
            KeySource::KeyFile => "keyFile",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            KeySource::Keyring => "系统凭据管理器",
            KeySource::KeyFile => "本地密钥文件",
        }
    }
}

pub struct Secrets {
    key: [u8; 32],
    source: KeySource,
}

impl Secrets {
    pub fn from_key(key: [u8; 32]) -> Self {
        Self {
            key,
            source: KeySource::KeyFile,
        }
    }

    pub fn source(&self) -> KeySource {
        self.source
    }

    /// 加载主密钥；不存在就新建一个。
    /// 三级退路，尽量不让「拿不到主密钥」变成打不开软件：
    /// 系统凭据管理器 → 应用数据目录密钥文件 → 本次进程内存（会提示）。
    pub fn load_or_create(data_dir: &Path) -> Self {
        match read_from_keyring() {
            Ok(Some(key)) => {
                return Self {
                    key,
                    source: KeySource::Keyring,
                }
            }
            Ok(None) => {}
            Err(error) => eprintln!("[secrets] 读取系统凭据管理器失败：{error}"),
        }

        let key: [u8; 32] = Aes256Gcm::generate_key(&mut OsRng).into();

        if write_to_keyring(&key).is_ok() {
            return Self {
                key,
                source: KeySource::Keyring,
            };
        }

        if fs::create_dir_all(data_dir)
            .and_then(|_| fs::write(data_dir.join(KEY_FILE), BASE64.encode(key)))
            .is_ok()
        {
            return Self {
                key,
                source: KeySource::KeyFile,
            };
        }

        eprintln!("[secrets] 主密钥无法持久化，本次加密结果在重启后将无法解密");
        Self {
            key,
            source: KeySource::KeyFile,
        }
    }

    /// 加密。返回可直接写进 settings.value 的字符串。
    pub fn encrypt(&self, plaintext: &str) -> Result<String, String> {
        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&self.key));
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        let ciphertext = cipher
            .encrypt(&nonce, plaintext.as_bytes())
            .map_err(|_| "加密失败".to_string())?;

        let mut packed = Vec::with_capacity(NONCE_LEN + ciphertext.len());
        packed.extend_from_slice(&nonce);
        packed.extend_from_slice(&ciphertext);

        Ok(format!("{VERSION_PREFIX}{}", BASE64.encode(packed)))
    }

    /// 解密。格式不对或密文被改过都会报错。
    pub fn decrypt(&self, stored: &str) -> Result<String, String> {
        let payload = stored
            .strip_prefix(VERSION_PREFIX)
            .ok_or_else(|| "密钥格式无法识别".to_string())?;
        let packed = BASE64
            .decode(payload)
            .map_err(|_| "密文解码失败".to_string())?;

        if packed.len() <= NONCE_LEN {
            return Err("密文长度异常".to_string());
        }
        let (nonce, ciphertext) = packed.split_at(NONCE_LEN);

        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&self.key));
        let plaintext = cipher
            .decrypt(Nonce::from_slice(nonce), ciphertext)
            .map_err(|_| "解密失败，密钥可能已失效".to_string())?;

        String::from_utf8(plaintext).map_err(|_| "解密结果不是合法文本".to_string())
    }

    /// 打码展示：sk-abcd••••wxyz
    pub fn mask(plain: &str) -> String {
        let chars: Vec<char> = plain.chars().collect();
        if chars.len() <= 10 {
            return "•".repeat(chars.len().max(4));
        }
        let head: String = chars.iter().take(6).collect();
        let tail: String = chars.iter().skip(chars.len() - 4).collect();
        format!("{head}••••{tail}")
    }
}

fn read_from_keyring() -> Result<Option<[u8; 32]>, String> {
    let entry = keyring::Entry::new(SERVICE, ACCOUNT).map_err(|error| error.to_string())?;
    match entry.get_password() {
        Ok(encoded) => {
            let raw = BASE64
                .decode(encoded.trim())
                .map_err(|_| "凭据管理器里的主密钥格式不正确".to_string())?;
            let bytes: [u8; 32] = raw
                .try_into()
                .map_err(|_| "凭据管理器里的主密钥长度不正确".to_string())?;
            Ok(Some(bytes))
        }
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(error) => Err(error.to_string()),
    }
}

fn write_to_keyring(key: &[u8; 32]) -> Result<(), String> {
    let entry = keyring::Entry::new(SERVICE, ACCOUNT).map_err(|error| error.to_string())?;
    entry
        .set_password(&BASE64.encode(key))
        .map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn secrets() -> Secrets {
        Secrets::from_key([7u8; 32])
    }

    #[test]
    fn encrypt_decrypt_round_trip() {
        let secrets = secrets();
        let plain = "sk-1234567890abcdefghijklmn";
        let stored = secrets.encrypt(plain).unwrap();

        assert!(stored.starts_with("v1:"), "密文应带版本前缀");
        assert!(!stored.contains(plain), "密文里不应该出现明文");
        assert_eq!(secrets.decrypt(&stored).unwrap(), plain);
    }

    #[test]
    fn same_plaintext_gets_different_ciphertext() {
        let secrets = secrets();
        let first = secrets.encrypt("sk-abcdefghijklmnop").unwrap();
        let second = secrets.encrypt("sk-abcdefghijklmnop").unwrap();
        assert_ne!(first, second, "每次加密都应使用新的随机 nonce");
    }

    #[test]
    fn tampered_ciphertext_is_rejected() {
        let secrets = secrets();
        let stored = secrets.encrypt("sk-abcdefghijklmnop").unwrap();

        // 翻转最后一个字符，模拟密文被改动
        let mut tampered = stored.clone();
        let last = tampered.pop().unwrap();
        tampered.push(if last == 'A' { 'B' } else { 'A' });
        assert!(secrets.decrypt(&tampered).is_err());

        // 换一把主密钥也解不开
        let other = Secrets::from_key([9u8; 32]);
        assert!(other.decrypt(&stored).is_err());

        assert!(secrets.decrypt("不是密文").is_err());
    }

    #[test]
    fn mask_hides_the_middle() {
        assert_eq!(Secrets::mask("sk-1234567890abcdef"), "sk-123••••cdef");
        assert_eq!(Secrets::mask("short"), "•••••");
    }

    /// 「关掉软件重开，Key 还在」的底层保证：主密钥必须能跨进程复用。
    #[test]
    fn master_key_is_reusable_across_runs() {
        let dir = std::env::temp_dir().join(format!("media-ai-workbench-key-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();

        let first = Secrets::load_or_create(&dir);
        let encrypted = first.encrypt("sk-persist-check-123456").unwrap();

        let files: Vec<String> = std::fs::read_dir(&dir)
            .map(|entries| {
                entries
                    .filter_map(Result::ok)
                    .map(|entry| entry.file_name().to_string_lossy().into_owned())
                    .collect()
            })
            .unwrap_or_default();
        println!(
            "主密钥来源 = {}, 目录内容 = {:?}",
            first.source().label(),
            files
        );

        let second = Secrets::load_or_create(&dir);
        assert_eq!(
            second.decrypt(&encrypted).unwrap(),
            "sk-persist-check-123456",
            "重启后必须能用同一把主密钥解开之前的密文"
        );
        assert_eq!(first.source(), second.source(), "两次启动的主密钥来源应一致");

        let _ = std::fs::remove_dir_all(&dir);
    }
}
