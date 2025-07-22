mod diskutil_exec;

use {
    super::DuDevice,
    crate::*,
    diskutil_exec::*,
    lazy_regex::*,
};

pub fn mounted_du_devices() -> Result<Vec<DuDevice>, Error> {
    let lines = du_lines(&["info", "-all"])?;
    Ok(lines_to_devices(&lines))
}

fn lines_to_devices(lines: &[String]) -> Vec<DuDevice> {
    let mut devs = Vec::new();
    let mut start = 0;
    for (i, line) in lines.iter().enumerate() {
        if regex_is_match!(r"\s*\*{8,}\s*$", line) {
            if i > start + 3 {
                let dev_lines = &lines[start..i];
                if let Some(dev) = lines_to_device(dev_lines) {
                    devs.push(dev);
                } else {
                    eprintln!("Device not understood:\n{}", dev_lines.join("\n"),);
                }
            }
            start = i + 1;
        }
    }
    devs
}

fn lines_to_device(lines: &[String]) -> Option<DuDevice> {
    let mut id = None;
    let mut node = None;
    let mut file_system = None;
    let mut mount_point = None;
    let mut part_of_whole = None;
    let mut encrypted = None;
    let mut read_only = None;
    let mut removable = None;
    let mut protocol = None;
    let mut solid_state = None;
    let mut volume_total_space = None;
    let mut volume_free_space = None;
    let mut volume_used_space = None;
    let mut container_total_space = None;
    let mut container_free_space = None;
    let mut allocation_block_size = None;
    for line in lines {
        let Some((_, key, value)) = regex_captures!(r"^\s+([^\:]+):\s+(.+)$", &line) else {
            continue;
        };
        match key {
            "Device Identifier" => {
                id = Some(value.to_string());
            }
            "Device Node" => {
                node = Some(value.to_string());
            }
            "File System" | "File System Personality" => {
                if value != "None" {
                    file_system = Some(value.to_string());
                }
            }
            "Mount Point" => {
                mount_point = Some(value.to_string());
            }
            "Part of Whole" => {
                part_of_whole = Some(value.to_string());
            }
            "Protocol" => {
                protocol = Some(value.to_string());
            }
            "Encrypted" => {
                encrypted = extract_bool(value);
            }
            "Media Read-Only" => {
                read_only = extract_bool(value);
            }
            "Removable Media" => match value {
                "Removable" => {
                    removable = Some(true);
                }
                "Fixed" => {
                    removable = Some(false);
                }
                _ => {}
            },
            "Solid State" => {
                solid_state = extract_bool(value);
            }
            "Volume Total Space" => {
                volume_total_space = extract_bytes(value);
            }
            "Volume Free Space" => {
                volume_free_space = extract_bytes(value);
            }
            "Volume Used Space" => {
                volume_used_space = extract_bytes(value);
            }
            "Container Total Space" => {
                container_total_space = extract_bytes(value);
            }
            "Container Free Space" => {
                container_free_space = extract_bytes(value);
            }
            "Allocation Block Size" => {
                allocation_block_size = extract_bytes(value);
            }
            _ => {}
        }
    }
    Some(DuDevice {
        id: id?,
        node: node?,
        file_system,
        mount_point,
        part_of_whole,
        removable,
        protocol,
        solid_state,
        read_only,
        encrypted,
        volume_total_space,
        volume_free_space,
        volume_used_space,
        container_total_space,
        container_free_space,
        allocation_block_size,
    })
}

fn extract_bytes(s: &str) -> Option<u64> {
    let (_, num) = regex_captures!(r"(\d+)\sBytes", s)?;
    num.parse().ok()
}
fn extract_bool(value: &str) -> Option<bool> {
    regex_switch!(value,
        r"^Yes\b" => true,
        r"^No\b" => false,
    )
}
