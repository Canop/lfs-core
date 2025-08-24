use {
    crate::Error,
    core_foundation::{
        base::*,
        boolean::*,
        dictionary::{
            CFDictionary,
            CFDictionaryGetValue,
            CFDictionaryRef,
            CFMutableDictionaryRef,
        },
        number::{
            CFNumber,
            *,
        },
        string::{
            CFString,
            *,
        },
    },
    io_kit_sys::{
        types::*,
        *,
    },
    libc::KERN_SUCCESS,
    std::{
        ffi::c_void,
        mem,
    },
};

pub struct Properties {
    dict: CFDictionary<CFString, CFType>,
}

impl Properties {
    pub unsafe fn new(service: io_object_t) -> Result<Self, Error> {
        let mut dict = mem::MaybeUninit::<CFMutableDictionaryRef>::uninit();
        let result =
            IORegistryEntryCreateCFProperties(service, dict.as_mut_ptr(), kCFAllocatorDefault, 0);
        if result != KERN_SUCCESS {
            return Err(Error::ServiceCallFailed {
                service: "IORegistryEntryCreateCFProperties",
            });
        }
        let dict = CFDictionary::wrap_under_create_rule(dict.assume_init());
        //dict.show();
        Ok(Self { dict })
    }
    pub fn has(
        &self,
        key: &'static str,
    ) -> bool {
        let key = CFString::from_static_string(key);
        self.dict.contains_key(&key)
    }
    pub fn get_mandatory_string(
        &self,
        key: &'static str,
    ) -> Result<String, Error> {
        self.get_string(key).ok_or(Error::MissingValue { key })
    }
    #[allow(dead_code)]
    pub fn get_mandatory_u64(
        &self,
        key: &'static str,
    ) -> Result<u64, Error> {
        self.get_u64(key).ok_or(Error::MissingValue { key })
    }
    pub fn get_mandatory_u32(
        &self,
        key: &'static str,
    ) -> Result<u32, Error> {
        self.get_u32(key).ok_or(Error::MissingValue { key })
    }
    pub fn get_sub_string(
        &self,
        dict_key: &'static str,
        prop_key: &'static str,
    ) -> Option<String> {
        let dict_key = CFString::from_static_string(dict_key);
        let dict = self.dict.find(&dict_key)?;
        let dict = dict.as_CFTypeRef() as CFDictionaryRef;
        let prop_key = CFString::from_static_string(prop_key);
        let value =
            unsafe { CFDictionaryGetValue(dict, prop_key.as_concrete_TypeRef() as *const c_void) };
        if value.is_null() {
            return None;
        }
        let value = unsafe { CFString::wrap_under_get_rule(value as CFStringRef) };
        Some(value.to_string())
    }
    pub fn get_string(
        &self,
        key: &'static str,
    ) -> Option<String> {
        let key = CFString::from_static_string(key);
        self.dict
            .find(&key)
            .and_then(|value_ref| {
                unsafe {
                    debug_assert!(value_ref.type_of() == CFStringGetTypeID());
                }
                value_ref.downcast::<CFString>()
            })
            .map(|cf_string| cf_string.to_string())
    }
    pub fn get_u64(
        &self,
        key: &'static str,
    ) -> Option<u64> {
        let key = CFString::from_static_string(key);
        self.dict
            .find(&key)
            .and_then(|value_ref| {
                unsafe {
                    debug_assert!(value_ref.type_of() == CFNumberGetTypeID());
                }
                value_ref.downcast::<CFNumber>()
            })
            .and_then(|cf_value| cf_value.to_i64())
            .and_then(|v| v.try_into().ok())
    }
    pub fn get_u32(
        &self,
        key: &'static str,
    ) -> Option<u32> {
        self.get_u64(key).map(|cf_value| cf_value as u32)
    }
    pub fn get_bool(
        &self,
        key: &'static str,
    ) -> Option<bool> {
        let key = CFString::from_static_string(key);
        self.dict
            .find(&key)
            .and_then(|value_ref| {
                unsafe {
                    debug_assert!(value_ref.type_of() == CFBooleanGetTypeID());
                }
                value_ref.downcast::<CFBoolean>()
            })
            .map(|cf_value| cf_value.into())
    }
}
