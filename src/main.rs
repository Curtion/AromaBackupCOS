#![deny(warnings)]
#![allow(dead_code)]

use qcos::acl::{AclHeader, ObjectAcl};
use qcos::bucket::Bucket;
use qcos::client::Client;
use serde::Deserialize;
use std::fs;

#[derive(Deserialize, Debug)]
struct Config {
    secrect_id: String,
    secrect_key: String,
    bucket: String,
    region: String,
}

fn get_last_back() -> Option<String> {
    let text = fs::read_to_string("./backups/World/backupstore.txt")
        .expect("未找到备份文件,请把该程序放在MC实例目录下运行");
    let back_list: Vec<&str> = text.split("\r\n").filter(|x| !x.is_empty()).collect();
    match back_list.last() {
        Some(x) => Some(x.to_string()),
        None => None,
    }
}

fn up_file(path: String, name: String) {
    let toml_str = fs::read_to_string("./config.toml").expect("缺少配置文件");
    let decoded: Config = toml::from_str(&toml_str).unwrap();
    let client = Client::new(
        decoded.secrect_id.as_str(),
        decoded.secrect_key.as_str(),
        decoded.bucket.as_str(),
        decoded.region.as_str(),
    );
    let mut acl_header = AclHeader::new();
    acl_header.insert_object_x_cos_acl(ObjectAcl::PublicRead);
    let res = client.list_objects("", "", "", "/", 100);
    print!("{:?},{},{}", res, path, name);
    // let file = std::fs::File::open(path).unwrap();
    // let res = client.put_object(
    //     file,
    //     name.as_str(),
    //     mime::STAR_STAR,
    //     Some(&acl_header),
    //     false,
    // );
    // if res.error_no == ErrNo::SUCCESS {
    //     println!("备份成功:{}", name);
    // } else {
    //     println!("备份失败:{}", res.error_message);
    // }
}

fn backup(back_name: String) {
    let back_info: Vec<&str> = back_name.split("=").collect();
    let back_path = format!(
        "./backups/{}/{}/{}/{}/Backup-{}-{}-{}-{}--{}-{}.zip",
        back_info[0],
        back_info[1],
        back_info[2],
        back_info[3],
        back_info[0],
        back_info[1],
        back_info[2],
        back_info[3],
        back_info[4],
        back_info[5],
    );
    let back_name = format!(
        "Backup-{}-{}-{}-{}--{}-{}.zip",
        back_info[0], back_info[1], back_info[2], back_info[3], back_info[4], back_info[5],
    );
    up_file(back_path, back_name);
}

fn main() {
    let last_back_name = get_last_back();
    match last_back_name {
        Some(x) => backup(x),
        None => println!("当前无需备份"),
    }
}
