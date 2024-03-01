use std::mem::size_of;

use lambdaworks_math::{
    errors::DeserializationError,
    traits::{AsBytes, Deserializable},
};

pub fn serialize_value<T: AsBytes>(cm: &T) -> Vec<u8> {
    let serialization = cm.as_bytes();
    let element_size = serialization.len() as u32;

    let mut bytes = Vec::new();

    bytes.extend_from_slice(&serialize_length(element_size));
    bytes.extend_from_slice(&serialization);

    bytes
}

pub fn deserialize_value<T: Deserializable>(
    bytes: &[u8],
) -> Result<(T, usize), DeserializationError> {
    let mut offset = 0;

    let (value_size, read) = deserialize_length(&bytes[offset..])?;
    offset += read;

    let value_bytes = &bytes[offset..]
        .get(..value_size)
        .ok_or(DeserializationError::InvalidAmountOfBytes)?;

    offset += value_bytes.len();

    let value = T::deserialize(value_bytes)?;

    Ok((value, offset))
}

pub fn serialize_length(length: u32) -> Vec<u8> {
    length.to_be_bytes().to_vec()
}

fn deserialize_length(bytes: &[u8]) -> Result<(usize, usize), DeserializationError> {
    let length_bytes = bytes
        .get(..size_of::<u32>())
        .ok_or(DeserializationError::InvalidAmountOfBytes)?
        .try_into()
        .map_err(|_| DeserializationError::InvalidAmountOfBytes)?;

    let length = u32::from_be_bytes(length_bytes) as usize;

    Ok((length, size_of::<u32>()))
}

pub fn serialize_vec<T: AsBytes>(values: &[T]) -> Vec<u8> {
    let mut bytes = Vec::new();

    bytes.extend_from_slice(&serialize_length(values.len() as u32));

    values.iter().fold(bytes, |mut bytes, value| {
        bytes.extend_from_slice(&serialize_value(value));
        bytes
    })
}

pub fn deserialize_vec<T: Deserializable>(
    bytes: &[u8],
) -> Result<(Vec<T>, usize), DeserializationError> {
    let mut offset = 0;

    let (length, read) = deserialize_length(&bytes[offset..])?;
    offset += read;

    dbg!(offset);

    let mut values = Vec::new();

    for _ in 0..length {
        let (value, read) = deserialize_value(&bytes[offset..])?;
        offset += read;

        dbg!(offset);

        values.push(value);
    }

    Ok((values, offset))
}

#[cfg(test)]
mod tests {
    use lambdaworks_math::{cyclic_group::IsGroup, elliptic_curve::traits::IsEllipticCurve};

    use crate::common::Curve;

    use super::*;

    #[test]
    fn can_serialize_value() {
        let value = Curve::generator().operate_with_self(7usize);

        let serialized = serialize_value(&value);

        let (deserialized, _) = deserialize_value(&serialized).unwrap();

        assert_eq!(value, deserialized);
    }

    #[test]
    fn can_serialize_vec() {
        let value = vec![
            Curve::generator().operate_with_self(7usize),
            Curve::generator().operate_with_self(3usize),
        ];

        let serialized = serialize_vec(&value);

        let (deserialized, _) = deserialize_vec(&serialized).unwrap();

        assert_eq!(value, deserialized);
    }
}
