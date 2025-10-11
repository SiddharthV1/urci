//! SQLx support for Sol! generated BLS types

use sqlx::{Encode, Type};
use sqlx::encode::IsNull;
use sqlx::error::BoxDynError;
use sqlx::postgres::{PgArgumentBuffer, PgTypeInfo, PgValueRef, Postgres};

use crate::BLS::{G1Point, G2Point};

// Implement SQLx traits for G1Point (128 bytes)
impl Type<Postgres> for G1Point {
    fn type_info() -> PgTypeInfo {
        <Vec<u8> as Type<Postgres>>::type_info()
    }
}

impl Encode<'_, Postgres> for G1Point {
    fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> Result<IsNull, BoxDynError> {
        let mut bytes = Vec::with_capacity(128);
        bytes.extend_from_slice(&self.x_a.0);
        bytes.extend_from_slice(&self.x_b.0);
        bytes.extend_from_slice(&self.y_a.0);
        bytes.extend_from_slice(&self.y_b.0);
        <Vec<u8> as Encode<'_, Postgres>>::encode(bytes, buf)
    }
}

impl sqlx::Decode<'_, Postgres> for G1Point {
    fn decode(value: PgValueRef<'_>) -> Result<Self, BoxDynError> {
        let bytes = <Vec<u8> as sqlx::Decode<Postgres>>::decode(value)?;
        if bytes.len() != 128 {
            return Err("G1Point must be exactly 128 bytes".into());
        }
        
        let mut x_a = [0u8; 32];
        let mut x_b = [0u8; 32];
        let mut y_a = [0u8; 32];
        let mut y_b = [0u8; 32];
        
        x_a.copy_from_slice(&bytes[0..32]);
        x_b.copy_from_slice(&bytes[32..64]);
        y_a.copy_from_slice(&bytes[64..96]);
        y_b.copy_from_slice(&bytes[96..128]);
        
        Ok(G1Point {
            x_a: alloy_sol_types::private::FixedBytes(x_a),
            x_b: alloy_sol_types::private::FixedBytes(x_b),
            y_a: alloy_sol_types::private::FixedBytes(y_a),
            y_b: alloy_sol_types::private::FixedBytes(y_b),
        })
    }
}

// Implement SQLx traits for G2Point (256 bytes)
impl Type<Postgres> for G2Point {
    fn type_info() -> PgTypeInfo {
        <Vec<u8> as Type<Postgres>>::type_info()
    }
}

impl Encode<'_, Postgres> for G2Point {
    fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> Result<IsNull, BoxDynError> {
        let mut bytes = Vec::with_capacity(256);
        bytes.extend_from_slice(&self.x_c0_a.0);
        bytes.extend_from_slice(&self.x_c0_b.0);
        bytes.extend_from_slice(&self.x_c1_a.0);
        bytes.extend_from_slice(&self.x_c1_b.0);
        bytes.extend_from_slice(&self.y_c0_a.0);
        bytes.extend_from_slice(&self.y_c0_b.0);
        bytes.extend_from_slice(&self.y_c1_a.0);
        bytes.extend_from_slice(&self.y_c1_b.0);
        <Vec<u8> as Encode<'_, Postgres>>::encode(bytes, buf)
    }
}

impl sqlx::Decode<'_, Postgres> for G2Point {
    fn decode(value: PgValueRef<'_>) -> Result<Self, BoxDynError> {
        let bytes = <Vec<u8> as sqlx::Decode<Postgres>>::decode(value)?;
        if bytes.len() != 256 {
            return Err("G2Point must be exactly 256 bytes".into());
        }
        
        let mut x_c0_a = [0u8; 32];
        let mut x_c0_b = [0u8; 32];
        let mut x_c1_a = [0u8; 32];
        let mut x_c1_b = [0u8; 32];
        let mut y_c0_a = [0u8; 32];
        let mut y_c0_b = [0u8; 32];
        let mut y_c1_a = [0u8; 32];
        let mut y_c1_b = [0u8; 32];
        
        x_c0_a.copy_from_slice(&bytes[0..32]);
        x_c0_b.copy_from_slice(&bytes[32..64]);
        x_c1_a.copy_from_slice(&bytes[64..96]);
        x_c1_b.copy_from_slice(&bytes[96..128]);
        y_c0_a.copy_from_slice(&bytes[128..160]);
        y_c0_b.copy_from_slice(&bytes[160..192]);
        y_c1_a.copy_from_slice(&bytes[192..224]);
        y_c1_b.copy_from_slice(&bytes[224..256]);
        
        Ok(G2Point {
            x_c0_a: alloy_sol_types::private::FixedBytes(x_c0_a),
            x_c0_b: alloy_sol_types::private::FixedBytes(x_c0_b),
            x_c1_a: alloy_sol_types::private::FixedBytes(x_c1_a),
            x_c1_b: alloy_sol_types::private::FixedBytes(x_c1_b),
            y_c0_a: alloy_sol_types::private::FixedBytes(y_c0_a),
            y_c0_b: alloy_sol_types::private::FixedBytes(y_c0_b),
            y_c1_a: alloy_sol_types::private::FixedBytes(y_c1_a),
            y_c1_b: alloy_sol_types::private::FixedBytes(y_c1_b),
        })
    }
}