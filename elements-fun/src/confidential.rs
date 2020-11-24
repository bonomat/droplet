// Rust Elements Library
// Written in 2018 by
//   Andrew Poelstra <apoelstra@blockstream.com>
//
// To the extent possible under law, the author(s) have dedicated all
// copyright and related and neighboring rights to this software to
// the public domain worldwide. This software is distributed without
// any warranty.
//
// You should have received a copy of the CC0 Public Domain Dedication
// along with this software.
// If not, see <http://creativecommons.org/publicdomain/zero/1.0/>.
//

//! # Confidential Commitments
//!
//! Structures representing Pedersen commitments of various types
//!

use crate::{
    encode::{Decodable, Encodable},
    AssetId,
};
use bitcoin::secp256k1::{
    rand::{CryptoRng, Rng, RngCore},
    CommitmentSecrets, Error, Generator, PedersenCommitment, PublicKey, Secp256k1, SecretKey,
    Signing,
};
use std::io;

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(crate = "serde_crate")
)]
pub struct AssetGenerator(pub(crate) Generator);

impl AssetGenerator {
    pub fn new<C: Signing>(secp: &Secp256k1<C>, asset: AssetId, bf: AssetBlindingFactor) -> Self {
        Self(secp.blind(asset.into_tag(), bf.into_inner()))
    }

    pub fn encoded_length(&self) -> usize {
        33
    }
}

impl Encodable for AssetGenerator {
    fn consensus_encode<W: io::Write>(&self, mut e: W) -> Result<usize, crate::encode::Error> {
        e.write_all(&self.0.serialize())?;

        Ok(33)
    }
}

impl Decodable for AssetGenerator {
    fn consensus_decode<D: io::BufRead>(d: D) -> Result<Self, crate::encode::Error> {
        let bytes = <[u8; 33]>::consensus_decode(d)?;

        Ok(Self(Generator::from_slice(&bytes)?))
    }
}

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(crate = "serde_crate")
)]
pub struct ValueCommitment(pub(crate) PedersenCommitment);

impl ValueCommitment {
    pub fn new<C: Signing>(
        secp: &Secp256k1<C>,
        value: u64,
        asset: AssetGenerator,
        bf: ValueBlindingFactor,
    ) -> Self {
        Self(secp.commit(value, bf.0, asset.0))
    }

    pub fn encoded_length(&self) -> usize {
        33
    }
}

impl Encodable for ValueCommitment {
    fn consensus_encode<W: io::Write>(&self, mut e: W) -> Result<usize, crate::encode::Error> {
        e.write_all(&self.0.serialize())?;

        Ok(33)
    }
}

impl Decodable for ValueCommitment {
    fn consensus_decode<D: io::BufRead>(d: D) -> Result<Self, crate::encode::Error> {
        let bytes = <[u8; 33]>::consensus_decode(d)?;

        Ok(Self(PedersenCommitment::from_slice(&bytes)?))
    }
}

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(crate = "serde_crate")
)]
pub struct Nonce(pub(crate) PublicKey);

impl Nonce {
    pub fn new<R: RngCore + CryptoRng, C: Signing>(
        rng: &mut R,
        secp: &Secp256k1<C>,
    ) -> (Self, SecretKey) {
        let secret_key = SecretKey::new(rng);
        let public_key = PublicKey::from_secret_key(&secp, &secret_key);

        (Self(public_key), secret_key)
    }

    pub fn encoded_length(&self) -> usize {
        33
    }
}

impl From<PublicKey> for Nonce {
    fn from(public_key: PublicKey) -> Self {
        Nonce(public_key)
    }
}

impl Encodable for Nonce {
    fn consensus_encode<W: io::Write>(&self, mut e: W) -> Result<usize, crate::encode::Error> {
        e.write_all(&self.0.serialize())?;

        Ok(33)
    }
}

impl Decodable for Nonce {
    fn consensus_decode<D: io::BufRead>(d: D) -> Result<Self, crate::encode::Error> {
        let bytes = <[u8; 33]>::consensus_decode(d)?;

        Ok(Self(PublicKey::from_slice(&bytes)?))
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct ValueBlindingFactor(pub(crate) SecretKey);

impl ValueBlindingFactor {
    pub fn random<R: Rng>(rng: &mut R) -> Self {
        Self(SecretKey::new(rng))
    }

    pub fn last<C: Signing>(
        secp: &Secp256k1<C>,
        value: u64,
        abf: AssetBlindingFactor,
        inputs: &[(u64, AssetBlindingFactor, ValueBlindingFactor)],
        outputs: &[(u64, AssetBlindingFactor, ValueBlindingFactor)],
    ) -> Self {
        let set_a = inputs
            .iter()
            .copied()
            .map(|(value, abf, vbf)| CommitmentSecrets {
                value,
                value_blinding_factor: vbf.0,
                generator_blinding_factor: abf.into_inner(),
            })
            .collect::<Vec<_>>();
        let set_b = outputs
            .iter()
            .copied()
            .map(|(value, abf, vbf)| CommitmentSecrets {
                value,
                value_blinding_factor: vbf.0,
                generator_blinding_factor: abf.into_inner(),
            })
            .collect::<Vec<_>>();

        Self(secp.compute_adaptive_blinding_factor(value, abf.0, &set_a, &set_b))
    }
}

// impl FromHex for ValueBlindingFactor {
//     type Error = FromHexError;

//     fn from_hex<T: AsRef<[u8]>>(hex: T) -> Result<Self, Self::Error> {
//         Ok(Self(FromHex::from_hex(hex)?))
//     }
// }

// impl From<[u8; 32]> for ValueBlindingFactor {
//     fn from(bytes: [u8; 32]) -> Self {
//         Self(bytes)
//     }
// }

#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct AssetBlindingFactor(pub(crate) SecretKey);

impl AssetBlindingFactor {
    pub fn new<R: Rng>(rng: &mut R) -> Self {
        Self(SecretKey::new(rng))
    }

    pub fn from_slice(bytes: &[u8]) -> Result<Self, Error> {
        Ok(Self(SecretKey::from_slice(bytes)?))
    }

    pub fn into_inner(self) -> SecretKey {
        self.0
    }
}
