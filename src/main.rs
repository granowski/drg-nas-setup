use std::fs::{DirBuilder, read_to_string, set_permissions, Permissions, metadata};

extern crate yaml_rust;
extern crate libc;

use libc::chmod;
use libc::chown;

use users::get_user_by_name;

use yaml_rust::{YamlLoader, YamlEmitter};

use std::path::Path;
use std::error::Error;

fn main() {
    let configuration_defaults_path = Path::new("./configuration/defaults.yaml");

    if !configuration_defaults_path.exists() {
        // todo -> check for non-default configurations, all required options, and use those when no defaults are given...
        println!("default configurations do not exist, exiting...");
        return ;
    }

    let configurations_as_string = read_to_string(configuration_defaults_path).unwrap_or_default();
    // todo -> learn what is &* ?
    let configurations = YamlLoader::load_from_str(&*configurations_as_string).unwrap();

    let credentials_configuration = &configurations[0];
    let disk_configuration = &configurations[1];

    println!("share to map -> {}", disk_configuration["share-to-map"].as_str().unwrap_or_default());

    let local_shares_location = disk_configuration["host-shares-location"].as_str().unwrap_or_default();
    let share_to_map = disk_configuration["share-to-map"].as_str().unwrap_or_default();

    if share_to_map == String::default() {
        println!("could not read share name from configuration... exiting...");
        return ;
    }

    let mut sp = "/".to_owned();
    sp.push_str(share_to_map);

    let mut share_path = local_shares_location.to_owned();
    share_path.push_str(sp.as_str());

    let local_shares_location_path = Path::new(share_path.as_str());

    if !local_shares_location_path.exists() {
        let mut host_share_location_directory_builder = DirBuilder::new();
        host_share_location_directory_builder.recursive(true);

        let directory_result = host_share_location_directory_builder.create(local_shares_location_path);
        if directory_result.is_err() {
            // todo -> be more explicit as to why the failure occurred
            println!("could not create the location for shares, exiting... ; {}", directory_result.unwrap_err().to_string());
            return ;
        }

        let path_c_str = std::ffi::CString::new(share_path).unwrap();

        let owner_of_share_location = disk_configuration["owner"].as_str().unwrap_or_default();

        if owner_of_share_location == String::default() {
            // todo -> assume process user if none given, write warning, etc...
            println!("could not read owner username from configuration, exiting...");
            return ;
        }

        // todo -> error handling of failed user retrieval
        let owner_user = get_user_by_name(owner_of_share_location).unwrap();

        unsafe {
            // todo -> write some constants for the mode integer...
            // 448 = rwx...... = (owner has read/write/execute, but no group or others)
            chmod(path_c_str.as_ptr(), 448);
            chown(path_c_str.as_ptr(), owner_user.uid(), owner_user.primary_group_id());
        }
    }

    return ;
}
