//! (TODO) Hardware AES calculator (AES)
use core::marker::PhantomData;
use crate::pac::AES;

pub struct Aes<MODE, KLEN> {
    aes: AES,
    _mode: PhantomData<MODE>,
    _klen: PhantomData<KLEN>,
}

pub struct Ecb;

pub struct Cbc;

pub struct Gcm;

pub struct K128;

pub struct K192;

pub struct K256;

#[allow(unused)] // todo: remove
impl<MODE, KLEN> Aes<MODE, KLEN> {
    pub fn ecb128(aes: AES) -> Aes<Ecb, K128> {
        todo!()
    }

    pub fn ecb192(aes: AES) -> Aes<Ecb, K192> {
        todo!()
    }

    pub fn ecb256(aes: AES) -> Aes<Ecb, K256> {
        todo!()
    }

    pub fn cbc128(aes: AES) -> Aes<Cbc, K128> {
        todo!()
    }

    pub fn cbc192(aes: AES) -> Aes<Cbc, K192> {
        todo!()
    }

    pub fn cbc256(aes: AES) -> Aes<Cbc, K256> {
        todo!()
    }

    pub fn gcm128(aes: AES) -> Aes<Gcm, K128> {
        todo!()
    }

    pub fn gcm192(aes: AES) -> Aes<Gcm, K192> {
        todo!()
    }

    pub fn gcm256(aes: AES) -> Aes<Gcm, K256> {
        todo!()
    }
}

impl<MODE, KLEN> Aes<MODE, KLEN> {
    // todo: clock
    pub fn free(self) -> AES {
        self.aes
    }
}

#[allow(unused)] // todo: remove
impl<MODE, KLEN> Aes<MODE, KLEN> {
    // entrypt block in-place
    pub fn encrypt_block(&self, block: &mut [u8], key: &[u8]) {
        todo!()
    }
    // decrypt block in-place
    pub fn decrypt_block(&self, block: &mut [u8], key: &[u8]) {
        todo!()
    }
}
