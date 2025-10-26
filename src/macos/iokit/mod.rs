mod properties;

use {
    crate::*,
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
    libc::{
        KERN_SUCCESS,
        MNT_NOWAIT,
        getfsstat,
        statfs,
    },
    properties::Properties,
    std::{
        os::raw::c_char,
        path::PathBuf,
    },
};

/// Data coming from IOKit and related to a mounted device
#[derive(Debug)]
pub struct Device {
    id: String,   // eg "disk3s3s1"
    node: String, // eg "/dev/disk3s3s1"
    bsd_major: u32,
    bsd_minor: u32,
    removable: Option<bool>,
    read_only: Option<bool>,
    crypted: Option<bool>,
    rotational: Option<bool>,
    uuid: Option<String>,
    part_uuid: Option<String>,
    content: Option<String>, // eg "Windows_FAT_16"
}

#[derive(Debug)]
struct DevMountInfo {
    device: String,
    mount_point: String,
    fs_type: String,
    stats: Stats,
    options: Vec<MountOption>,
}

/// Read all the mount points and load information on them
pub fn read_mounts(_options: &ReadOptions) -> Result<Vec<Mount>, Error> {
    let devs = mounted_devices()?;
    let dmis = get_all_dev_mount_infos();
    let mut mounts = Vec::new();
    for dev in devs {
        let Some(dmi) = dmis.iter().find(|dmi| dmi.device == dev.node) else {
            continue;
        };
        let mount_point = PathBuf::from(&dmi.mount_point);
        let mut fs_type = dmi.fs_type.clone();
        if fs_type == "FAT" {
            if let Some(content) = dev.content.as_ref() {
                if let Some((_, v)) = regex_captures!(r"^\w+_FAT_(\d\d)$", content) {
                    fs_type = format!("FAT{v}");
                }
            }
        }
        let info = MountInfo {
            id: None,
            parent: None,
            dev: DeviceId {
                major: dev.bsd_major,
                minor: dev.bsd_minor,
            },
            root: mount_point.clone(),
            mount_point,
            options: dmi.options.clone(),
            fs: dev.node.clone(),
            fs_type,
            bound: false, // FIXME
        };
        let disk = Disk {
            name: dev.id.clone(),
            rotational: dev.rotational,
            removable: dev.removable,
            read_only: dev.read_only,
            ram: false,
            image: false,
            lvm: false,
            crypted: dev.crypted.unwrap_or_default(),
        };
        let mount = Mount {
            info,
            fs_label: None, // TODO
            disk: Some(disk),
            stats: Ok(dmi.stats.clone()),
            uuid: dev.uuid.clone(),
            part_uuid: dev.part_uuid.clone(),
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
    let bsd_major = media_props.get_mandatory_u32("BSD Major")?;
    let bsd_minor = media_props.get_mandatory_u32("BSD Minor")?;
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
        bsd_major,
        bsd_minor,
        removable,
        crypted,
        rotational,
        read_only,
        uuid,
        part_uuid,
        content,
    })
}

fn get_all_dev_mount_infos() -> Vec<DevMountInfo> {
    unsafe {
        // First call to get the number of filesystems
        let count = getfsstat(std::ptr::null_mut(), 0, MNT_NOWAIT);
        if count <= 0 {
            return Vec::new();
        }

        // Allocate buffer
        let mut buf: Vec<statfs> = Vec::with_capacity(count as usize);
        let buf_size = (count as usize) * std::mem::size_of::<statfs>();

        // Second call to get the data
        let actual_count = getfsstat(buf.as_mut_ptr(), buf_size as i32, MNT_NOWAIT);
        if actual_count <= 0 {
            return Vec::new();
        }
        buf.set_len(actual_count as usize);

        buf.into_iter()
            .filter_map(|stat| {
                let device = std::ffi::CStr::from_ptr(stat.f_mntfromname.as_ptr())
                    .to_str()
                    .ok()?;
                let mount_point = std::ffi::CStr::from_ptr(stat.f_mntonname.as_ptr())
                    .to_str()
                    .ok()?;
                let fs_type = std::ffi::CStr::from_ptr(stat.f_fstypename.as_ptr())
                    .to_str()
                    .ok()?;
                let stats = Stats {
                    bsize: stat.f_bsize as u64,
                    blocks: stat.f_blocks,
                    bfree: stat.f_bfree,
                    bavail: stat.f_bavail,
                    bused: stat.f_blocks - stat.f_bavail,
                    inodes: None,
                };
                let fs_type = match fs_type {
                    "apfs" => "APFS",
                    "exfat" => "ExFAT",
                    "ftp" => "FTP",
                    "hfs" => "HFS+",
                    "msdos" if stats.bsize * stats.blocks > 2_147_484_648 => "FAT32",
                    "msdos" => "FAT", // will be detemined using device.content
                    "nfs" => "NFS",
                    "ntfs" => "NTFS",
                    "udf" => "UDF",
                    "ufs" => "UFS",
                    "xfs" => "XHS",
                    "zfs" => "ZFS",
                    v => v, // other ones unchanged
                };
                // we'll try to build a "mount options" array consistent with the semantics of linux
                // Constants are defined in https://github.com/apple/darwin-xnu/blob/main/bsd/sys/mount.h
                // I'm not sure how stable those flag values are
                let flags: u32 = stat.f_flags;
                let mut options = Vec::new();
                if flags & 1 == 0 {
                    // MNT_READ_ONLY = 1
                    options.push(MountOption::new("rw", None));
                }
                if flags & 2 != 0 {
                    // MNT_SYNCHRONOUS = 2
                    options.push(MountOption::new("synchronous", None));
                }
                if flags & 4 != 0 {
                    // MNT_NOEXEC = 4
                    options.push(MountOption::new("noexec", None));
                }
                if flags & 8 != 0 {
                    // MNT_NOSUID = 8
                    options.push(MountOption::new("nosuid", None));
                }
                if flags & 16 != 0 {
                    // MNT_NODEV = 16
                    options.push(MountOption::new("nodev", None));
                }
                if flags & 32 != 0 {
                    // MNT_UNION = 32
                    options.push(MountOption::new("union", None));
                }
                if flags & 64 != 0 {
                    // MNT_ASYNC = 64
                    options.push(MountOption::new("async", None));
                }
                if flags & 128 != 0 {
                    // MNT_CPROTECT = 128
                    options.push(MountOption::new("cprotect", None));
                }
                if flags & 512 != 0 {
                    // MNT_REMOVABLE = 512
                    options.push(MountOption::new("removable", None));
                }

                // Following ones don't seem correct
                // if flags & 0x00100000 != 0 { // MNT_DONTBROWSE  = 0x00100000
                //     options.push(MountOption::new("dontbrowse", None));
                // }
                // if flags & 0x10000000 != 0 { // MNT_NOATIME = 0x10000000
                //     options.push(MountOption::new("noatime", None));
                // }

                Some(DevMountInfo {
                    device: device.to_string(),
                    mount_point: mount_point.to_string(),
                    fs_type: fs_type.to_string(),
                    stats,
                    options,
                })
            })
            .collect()
    }
}
