use aes::Aes256;
use aes::cipher::{BlockEncrypt, KeyInit};
use ghash::universal_hash::{KeyInit as GHashInit, UniversalHash};
use ghash::{GHash, Key};
use subtle::ConstantTimeEq;
use zeroize::Zeroize;

use super::error::BackupError;

/// 12 バイト nonce の AES-256-GCM。タグは暗号文の末尾 16 バイト。
/// AAD は初期化時にブロック境界までパディングしてから、暗号文を別枠で吸う。
pub struct GcmSession {
  aes: Aes256,
  ghash: GHash,
  counter: [u8; 16],
  keystream: [u8; 16],
  keystream_off: usize,
  tag_mask: [u8; 16],
  pending: [u8; 16],
  pending_len: usize,
  aad_len: u64,
  ct_len: u64,
}

impl Drop for GcmSession {
  fn drop(&mut self) {
    self.counter.zeroize();
    self.keystream.zeroize();
    self.tag_mask.zeroize();
    self.pending.zeroize();
  }
}

impl GcmSession {
  pub fn new(key: &[u8; 32], nonce: &[u8; 12], aad: &[u8]) -> Self {
    let aes = Aes256::new(aes::cipher::generic_array::GenericArray::from_slice(key));
    let h = encrypt_block(&aes, &[0u8; 16]);
    let mut ghash = <GHash as GHashInit>::new(Key::from_slice(&h));
    absorb_padded(&mut ghash, aad);

    let mut j0 = [0u8; 16];
    j0[..12].copy_from_slice(nonce);
    j0[15] = 1;
    let tag_mask = encrypt_block(&aes, &j0);
    let mut counter = j0;
    increment_be32(&mut counter);

    Self {
      aes,
      ghash,
      counter,
      keystream: [0u8; 16],
      keystream_off: 16,
      tag_mask,
      pending: [0u8; 16],
      pending_len: 0,
      aad_len: aad.len() as u64,
      ct_len: 0,
    }
  }

  pub fn encrypt(&mut self, buf: &mut [u8]) {
    self.apply_keystream(buf);
    self.absorb_ciphertext(buf);
  }

  pub fn decrypt(&mut self, buf: &mut [u8]) {
    self.absorb_ciphertext(buf);
    self.apply_keystream(buf);
  }

  pub fn finish(mut self) -> [u8; 16] {
    if self.pending_len > 0 {
      let mut block = self.pending;
      for byte in block.iter_mut().skip(self.pending_len) {
        *byte = 0;
      }
      absorb_block(&mut self.ghash, &block);
      self.pending_len = 0;
    }

    let mut length_block = [0u8; 16];
    length_block[..8].copy_from_slice(&(self.aad_len * 8).to_be_bytes());
    length_block[8..].copy_from_slice(&(self.ct_len * 8).to_be_bytes());
    absorb_block(&mut self.ghash, &length_block);

    let ghash = std::mem::replace(&mut self.ghash, GHash::new(Key::from_slice(&[0u8; 16])));
    let mut tag = [0u8; 16];
    tag.copy_from_slice(ghash.finalize().as_slice());
    for (byte, mask) in tag.iter_mut().zip(self.tag_mask) {
      *byte ^= mask;
    }
    tag
  }

  fn apply_keystream(&mut self, buf: &mut [u8]) {
    let mut index = 0;
    while index < buf.len() {
      if self.keystream_off == 16 {
        self.keystream = encrypt_block(&self.aes, &self.counter);
        increment_be32(&mut self.counter);
        self.keystream_off = 0;
      }
      let take = (buf.len() - index).min(16 - self.keystream_off);
      for offset in 0..take {
        buf[index + offset] ^= self.keystream[self.keystream_off + offset];
      }
      self.keystream_off += take;
      index += take;
    }
  }

  fn absorb_ciphertext(&mut self, data: &[u8]) {
    self.ct_len += data.len() as u64;
    let mut rest = data;
    if self.pending_len > 0 {
      let take = (16 - self.pending_len).min(rest.len());
      self.pending[self.pending_len..self.pending_len + take].copy_from_slice(&rest[..take]);
      self.pending_len += take;
      rest = &rest[take..];
      if self.pending_len == 16 {
        let block = self.pending;
        absorb_block(&mut self.ghash, &block);
        self.pending_len = 0;
      }
    }
    while rest.len() >= 16 {
      let mut block = [0u8; 16];
      block.copy_from_slice(&rest[..16]);
      absorb_block(&mut self.ghash, &block);
      rest = &rest[16..];
    }
    if !rest.is_empty() {
      self.pending[..rest.len()].copy_from_slice(rest);
      self.pending_len = rest.len();
    }
  }
}

pub fn tags_equal(left: &[u8; 16], right: &[u8; 16]) -> bool {
  left.ct_eq(right).unwrap_u8() == 1
}

pub fn derive_key(password: &str, salt: &[u8; 16]) -> Result<[u8; 32], BackupError> {
  let params = argon2::Params::new(19_456, 2, 1, Some(32)).map_err(|_| BackupError::io())?;
  let argon2 = argon2::Argon2::new(argon2::Algorithm::Argon2id, argon2::Version::V0x13, params);
  let mut key = [0u8; 32];
  argon2
    .hash_password_into(password.as_bytes(), salt, &mut key)
    .map_err(|_| BackupError::io())?;
  Ok(key)
}

fn encrypt_block(aes: &Aes256, block: &[u8; 16]) -> [u8; 16] {
  let mut out = aes::cipher::generic_array::GenericArray::clone_from_slice(block);
  aes.encrypt_block(&mut out);
  let mut result = [0u8; 16];
  result.copy_from_slice(out.as_slice());
  result
}

fn absorb_padded(ghash: &mut GHash, data: &[u8]) {
  let mut offset = 0;
  while offset + 16 <= data.len() {
    let mut block = [0u8; 16];
    block.copy_from_slice(&data[offset..offset + 16]);
    absorb_block(ghash, &block);
    offset += 16;
  }
  if offset < data.len() {
    let mut block = [0u8; 16];
    block[..data.len() - offset].copy_from_slice(&data[offset..]);
    absorb_block(ghash, &block);
  }
}

fn absorb_block(ghash: &mut GHash, block: &[u8; 16]) {
  let generic = ghash::Block::clone_from_slice(block);
  ghash.update(std::slice::from_ref(&generic));
}

fn increment_be32(block: &mut [u8; 16]) {
  let mut value = u32::from_be_bytes(block[12..16].try_into().expect("4 bytes"));
  value = value.wrapping_add(1);
  block[12..16].copy_from_slice(&value.to_be_bytes());
}

#[cfg(test)]
mod tests {
  use aes_gcm::aead::Aead;
  use aes_gcm::aead::Payload;
  use aes_gcm::{Aes256Gcm, KeyInit};

  use super::GcmSession;

  #[test]
  fn streaming_aes_gcm_matches_one_shot() {
    let key = [7u8; 32];
    let nonce = [9u8; 12];
    let aad = [1u8; 48];
    let plain = b"manifest-and-db-bytes-that-cross-block-boundaries-0123456789";

    let mut session = GcmSession::new(&key, &nonce, &aad);
    let mut buffer = plain.to_vec();
    session.encrypt(&mut buffer[..20]);
    session.encrypt(&mut buffer[20..]);
    let tag = session.finish();
    buffer.extend_from_slice(&tag);

    let cipher = Aes256Gcm::new((&key).into());
    let expected = cipher
      .encrypt(
        (&nonce).into(),
        Payload {
          msg: plain,
          aad: &aad,
        },
      )
      .unwrap();
    assert_eq!(buffer, expected);

    let (body, expected_tag) = expected.split_at(expected.len() - 16);
    let mut session = GcmSession::new(&key, &nonce, &aad);
    let mut decoded = body.to_vec();
    session.decrypt(&mut decoded);
    let actual_tag = session.finish();
    assert_eq!(decoded, plain);
    assert_eq!(actual_tag, expected_tag);
  }
}
