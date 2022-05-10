use qcos::acl::{AclHeader, ObjectAcl};
use qcos::bucket::Bucket;
use qcos::client::Client;
use qcos::objects::{mime, Objects};
use qcos::request::ErrNo;
use serde::Deserialize;
use std::{fs, str};
use xmltree::Element;

#[derive(Deserialize, Debug)]
struct Config {
    secrect_id: String,
    secrect_key: String,
    bucket: String,
    region: String,
}

#[derive(Debug)]
struct CloudList {
    name: String,
}

fn str_to_name(str: &String) -> String {
    // txt项转成压缩包名
    let back_info: Vec<&str> = str.split("=").collect();
    format!(
        "Backup-{}-{}-{}-{}--{}-{}.zip",
        back_info[0], back_info[1], back_info[2], back_info[3], back_info[4], back_info[5],
    )
}

pub fn name_to_str(str: &String) -> String {
    // 压缩包名转成txt项
    let back_info: Vec<&str> = str.split("-").filter(|x| !x.is_empty()).collect();
    let last: Vec<&str> = back_info.last().unwrap().split(".").collect();
    format!(
        "{}={}={}={}={}={}",
        back_info[1], back_info[2], back_info[3], back_info[4], back_info[5], last[0]
    )
}

fn name_and_path(str: &String) -> String {
    // 根据txt项转成路径
    let back_info: Vec<&str> = str.split("=").collect();
    format!(
        "./backups/{}/{}/{}/{}/{}",
        back_info[0],
        back_info[1],
        back_info[2],
        back_info[3],
        str_to_name(&str),
    )
}

fn get_last_back() -> Option<String> {
    // 获取最新的备份
    let text = fs::read_to_string("./backups/World/backupstore.txt")
        .expect("未找到备份文件,请把该程序放在MC实例目录下运行");
    let back_list: Vec<&str> = text.split("\r\n").filter(|x| !x.is_empty()).collect();
    match back_list.last() {
        Some(x) => Some(x.to_string()),
        None => None,
    }
}

fn get_local_back_list() -> Vec<String> {
    //获取本地备份列表
    let text = fs::read_to_string("./backups/World/backupstore.txt")
        .expect("未找到备份文件,请把该程序放在MC实例目录下运行");
    let back_list: Vec<String> = text
        .split("\r\n")
        .filter(|x| !x.is_empty())
        .map(|x| x.to_string())
        .collect();
    back_list
}

fn get_cloud_back_list(client: &Client) -> Vec<CloudList> {
    let mut cloud_list: Vec<CloudList> = Vec::new();
    let res = client.list_objects("", "", "", "/", 100);
    match String::from_utf8(res.result.to_vec()) {
        Ok(x) => {
            let names_element = Element::parse(x.as_bytes()).unwrap();
            for item in names_element.children {
                if item.as_element().unwrap().name == "Contents" {
                    let key = item
                        .as_element()
                        .unwrap()
                        .get_child("Key")
                        .unwrap()
                        .get_text()
                        .unwrap();
                    cloud_list.push(CloudList {
                        name: key.to_string(),
                    });
                }
            }
        }
        Err(x) => println!("{:?}", x),
    };
    cloud_list
}

fn del_cloud_object(client: &Client) {
    let cloud_list = get_cloud_back_list(&client);
    let local_list = get_local_back_list()
        .iter()
        .map(|x| str_to_name(&x))
        .collect::<Vec<String>>();
    for item in cloud_list {
        if !local_list.contains(&item.name) {
            client.delete_object(&item.name);
            println!("删除成功:{}", &item.name);
        }
    }
}

fn upfile(path: String, client: &Client, name: String, acl_header: AclHeader) {
    let file = std::fs::File::open(path);
    match file {
        Ok(file) => {
            let res = client.put_object(
                file,
                name.as_str(),
                mime::STAR_STAR,
                Some(&acl_header),
                false,
            );
            if res.error_no == ErrNo::SUCCESS {
                println!("备份成功:{}", name);
                del_cloud_object(&client);
            } else {
                println!("备份失败:{}", res.error_message);
            }
        }
        Err(x) => println!("{:?}", x),
    }
}

fn backup(str: String) {
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
    let cloud_list = get_cloud_back_list(&client);
    let back_path = name_and_path(&str);
    let back_name = str_to_name(&str);
    let res = cloud_list
        .into_iter()
        .filter(|x| x.name == back_name)
        .collect::<Vec<CloudList>>();
    if res.len() > 0 {
        println!("已存在该备份");
    } else {
        upfile(back_path, &client, back_name, acl_header);
    }
}

fn main() {
    let last_back_name = get_last_back();
    match last_back_name {
        Some(str) => backup(str),
        None => println!("当前无需备份"),
    }
}
