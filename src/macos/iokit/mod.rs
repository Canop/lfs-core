mod dev_mount_info;
mod properties;

use {
    crate::*,
    dev_mount_info::DevMountInfo,
    io_kit_sys::{
        IOIteratorNext,
        IOObjectRelease,
        IORegistryEntryGetParentEntry,
        IOServiceGetMatchingServices,
        IOServiceMatching,
        kIOMasterPortDefault,
        keys::kIOServicePlane,
        types::{
            io_iterator_t,
            io_object_t,
        },
    },
    lazy_regex::*,
    libc::KERN_SUCCESS,
    properties::Properties,
    std::os::raw::c_char,
};

/// Data coming from IOKit and related to a mounted device
#[derive(Debug)]
pub struct Device {
    id: String,   // eg "disk3s3s1"
    node: String, // eg "/dev/disk3s3s1"
    removable: Option<bool>,
    read_only: Option<bool>,
    crypted: Option<bool>,
    rotational: Option<bool>,
    uuid: Option<String>,
    part_uuid: Option<String>,
    content: Option<String>, // eg "Windows_FAT_16"
}

/// Read all the mount points and load information on them
pub fn read_mounts(_options: &ReadOptions) -> Result<Vec<Mount>, Error> {
    let devs = mounted_devices()?;
    let dmis = DevMountInfo::get_all();
    let mut mounts = Vec::new();
    for dmi in dmis {
        let mut info = dmi.to_mount_info();
        let dev = devs.iter().find(|dev| dev.node == dmi.device);
        if info.fs_type == "FAT" {
            if let Some(dev) = dev.as_ref() {
                if let Some(content) = dev.content.as_ref() {
                    if let Some((_, v)) = regex_captures!(r"^\w+_FAT_(\d\d)$", content) {
                        info.fs_type = format!("FAT{v}");
                    }
                }
            }
        }
        let disk = dev.as_ref().map(|dev| Disk {
            name: dev.id.clone(),
            rotational: dev.rotational,
            removable: dev.removable,
            read_only: dev.read_only,
            ram: false,
            image: false,
            lvm: false,
            crypted: dev.crypted.unwrap_or_default(),
        });
        let mount = Mount {
            info,
            fs_label: None, // TODO
            disk,
            stats: Ok(dmi.stats.clone()),
            uuid: dev.as_ref().and_then(|d| d.uuid.clone()),
            part_uuid: dev.as_ref().and_then(|d| d.part_uuid.clone()),
        };
        mounts.push(mount);
    }
    Ok(mounts)
}

pub fn mounted_devices() -> Result<Vec<Device>, Error> {
    let mut devs = Vec::new();
    unsafe {
        let dict = IOServiceMatching(c"IOMedia".as_ptr() as *const c_char);
        if dict.is_null() {
            return Err(Error::ServiceCallFailed {
                service: "IOServiceMatching/IOMedia",
            });
        }
        let mut iterator: io_iterator_t = 0;
        let result = IOServiceGetMatchingServices(kIOMasterPortDefault, dict, &mut iterator);
        if result != KERN_SUCCESS {
            return Err(Error::ServiceCallFailed {
                service: "IOServiceGetMatchingServices",
            });
        }
        let mut media_service: io_object_t;
        while {
            media_service = IOIteratorNext(iterator);
            media_service != 0
        } {
            let dev = service_to_device(media_service)?;
            devs.push(dev);
            IOObjectRelease(media_service);
        }
        IOObjectRelease(iterator);
    }
    //dbg!(dmis);
    Ok(devs)
}
unsafe fn service_to_device(
    media_service: io_object_t, // service from the IOMedia layer
) -> Result<Device, Error> {
    let mut current_service = media_service;
    let mut parent: io_object_t = 0;
    loop {
        let result = IORegistryEntryGetParentEntry(current_service, kIOServicePlane, &mut parent);
        if result != KERN_SUCCESS {
            break;
        }
        let props = Properties::new(parent)?;
        if props.has("Device Characteristics") || props.has("Solid State") {
            // this is the "physical" layer
            let media_props = Properties::new(media_service)?;
            let device = props_to_device(media_props, props)?;
            IOObjectRelease(current_service);
            return Ok(device);
        }
        if current_service != media_service {
            IOObjectRelease(current_service);
        }
        current_service = parent;
    }
    Err(Error::DeviceLayerNotFound)
}
fn props_to_device(
    media_props: Properties,
    bs_props: Properties, // block storage layer
) -> Result<Device, Error> {
    let id = media_props.get_mandatory_string("BSD Name")?;
    let node = format!("/dev/{id}");
    let removable = media_props.get_bool("Removable");
    let crypted = media_props.get_bool("CoreStorage Encrypted"); // TODO check this
    let read_only = media_props.get_bool("Writable").map(|b| !b);

    let medium_type = bs_props.get_sub_string("Device Characteristics", "Medium Type");
    let rotational = medium_type.map(|v| !v.contains("Solid"));

    let uuid = media_props.get_string("UUID");
    let part_uuid = None; // TODO
    let content = media_props.get_string("Content");

    Ok(Device {
        id,
        node,
        removable,
        crypted,
        rotational,
        read_only,
        uuid,
        part_uuid,
        content,
    })
}

#[test]
fn test_smb() {
    let mountinfos = get_all_dev_mount_infos();
    println!("MountInfos: {:#?}", mountinfos);
    todo!();
}
