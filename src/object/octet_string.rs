//! Octet String Value Object Type Implementation
//!
//! This module implements the Octet String Value object type as defined in ASHRAE 135

use crate::object::{
    BacnetObject, ObjectError, ObjectIdentifier, ObjectType, PropertyIdentifier, PropertyValue,
    Result,
};

#[cfg(not(feature = "std"))]
use alloc::{string::String, vec::Vec};

// limit vec size so we can use MAX_ADPU 1024 and not worry about segmenting
const MAX_OCTET_STRING_SIZE: usize = 900;
struct BoundedVec {
    inner: Vec<u8>,
}

#[derive(Debug)]
pub enum BoundedVecError {
    OversizeData { len: usize, max_len: usize },
}

impl BoundedVec {
    pub fn new(data: Vec<u8>) -> std::result::Result<Self, BoundedVecError> {
        if data.len() > MAX_OCTET_STRING_SIZE {
            Err(BoundedVecError::OversizeData {
                len: data.len(),
                max_len: MAX_OCTET_STRING_SIZE,
            })
        } else {
            Ok(Self { inner: data })
        }
    }

    // if need for slice instead of vec
    // pub fn as_slice(&self) -> &[u8] {
    //     &self.inner
    // }

    pub fn len(&self) -> usize {
        self.inner.len()
    }
}

/// Octet String Value object
#[derive(Debug, Clone)]
pub struct OctetString {
    /// Object identifier R
    pub identifier: ObjectIdentifier,
    /// Object name R
    pub object_name: String,
    /// Description O
    pub description: String,
    /// Present value R (required to be writeable when out_of_service is true)
    pub present_value: Vec<u8>,
    /// Status flags R
    pub status_flags: u8,
}

impl OctetString {
    /// create a new Octet String Value object
    pub fn new(instance: u32, object_name: String) -> Self {
        Self {
            identifier: ObjectIdentifier::new(ObjectType::OctetString, instance),
            object_name,
            description: String::new(),
            present_value: Vec::new(),
            status_flags: 0,
        }
    }

    /// set the present value
    pub fn set_present_value(
        &mut self,
        value: Vec<u8>,
    ) -> std::result::Result<(), BoundedVecError> {
        let bounded_value = BoundedVec::new(value)?;
        self.present_value = bounded_value.inner;
        Ok(())
    }

    pub fn get_status_flags(&self) -> (bool, bool, bool, bool) {
        (
            (self.status_flags & 0x08) != 0, // in_alarm
            (self.status_flags & 0x04) != 0, // fault
            (self.status_flags & 0x02) != 0, // overridden
            (self.status_flags & 0x01) != 0, // out_of_service
        )
    }

    pub fn set_status_flags(
        &mut self,
        in_alarm: bool,
        fault: bool,
        overridden: bool,
        out_of_service: bool,
    ) {
        self.status_flags = 0;
        if in_alarm {
            self.status_flags |= 0x08;
        }
        if fault {
            self.status_flags |= 0x04;
        }
        if overridden {
            self.status_flags |= 0x02;
        }
        if out_of_service {
            self.status_flags |= 0x01;
        }
    }
}

impl BacnetObject for OctetString {
    fn identifier(&self) -> ObjectIdentifier {
        self.identifier
    }

    fn get_property(&self, property: PropertyIdentifier) -> Result<PropertyValue> {
        match property {
            PropertyIdentifier::ObjectIdentifier => {
                Ok(PropertyValue::ObjectIdentifier(self.identifier))
            }
            PropertyIdentifier::ObjectName => {
                Ok(PropertyValue::CharacterString(self.object_name.clone()))
            }
            PropertyIdentifier::ObjectType => {
                Ok(PropertyValue::Enumerated(ObjectType::OctetString as u32))
            }
            PropertyIdentifier::PresentValue => {
                Ok(PropertyValue::OctetString(self.present_value.clone()))
            }
            _ => Err(ObjectError::UnknownProperty),
        }
    }

    fn set_property(&mut self, property: PropertyIdentifier, value: PropertyValue) -> Result<()> {
        match property {
            PropertyIdentifier::ObjectName => {
                if let PropertyValue::CharacterString(name) = value {
                    self.object_name = name;
                    Ok(())
                } else {
                    Err(ObjectError::InvalidPropertyType)
                }
            }
            _ => Err(ObjectError::PropertyNotWritable),
        }
    }

    fn is_property_writable(&self, property: PropertyIdentifier) -> bool {
        matches!(property, PropertyIdentifier::ObjectName)
    }

    fn property_list(&self) -> Vec<PropertyIdentifier> {
        vec![
            PropertyIdentifier::ObjectIdentifier,
            PropertyIdentifier::ObjectName,
            PropertyIdentifier::ObjectType,
            PropertyIdentifier::PresentValue,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_octet_string_creation() {
        let octet_string = OctetString::new(1, "test".to_string());
        assert_eq!(
            octet_string.identifier(),
            ObjectIdentifier::new(ObjectType::OctetString, 1)
        );
        assert_eq!(octet_string.object_name, "test");
        assert_eq!(octet_string.present_value, Vec::new());
        assert_eq!(octet_string.status_flags, 0);
    }

    #[test]
    fn test_octet_string_operations() {
        // make object
        let mut octet_string = OctetString::new(1, "test".to_string());

        // set a dummy value
        let data = vec![1, 2, 3, 4];
        octet_string.set_present_value(data.clone()).unwrap();

        // retrieve that dummy value
        assert_eq!(octet_string.present_value, data.clone());
    }

    #[test]
    fn test_octet_string_oversize() {
        let mut octet_string = OctetString::new(1, "test".to_string());
        let data = vec![1; MAX_OCTET_STRING_SIZE + 1];
        assert!(octet_string.set_present_value(data.clone()).is_err());
    }
}
