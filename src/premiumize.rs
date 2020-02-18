extern crate reqwest;

use serde::{Serialize, Deserialize};
use std::path::Path;
use std::fs::File;
use std::fs::create_dir;
use reqwest::blocking::Client;
use std::io::{ErrorKind, Read, Write};

pub fn copy<R: ?Sized, W: ?Sized>(reader: &mut R, writer: &mut W) -> std::io::Result<u64>
where
    R: Read,
    W: Write,
{
    let mut buf : [u8; 128*1024] = [0;128*1024];

    let mut written = 0;
    loop {
        let len = match reader.read(&mut buf) {
            Ok(0) => return Ok(written),
            Ok(len) => len,
            Err(ref e) if e.kind() == ErrorKind::Interrupted => continue,
            Err(e) => return Err(e),
        };
        writer.write_all(&buf[..len])?;
        written += len as u64;
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Folder
{
    id: String,
    name: String,
    #[serde(rename(deserialize = "type"))]
    type_: String,
    #[serde(default)]
    link: String,
    //#[serde(default)]
    //stream_link: String,
    #[serde(default)]
    transcode_status: String,
    #[serde(default)]
    created_at: u64,
    #[serde(default)]
    size: u64,
    #[serde(default)]
    mime_type: String,
    #[serde(default)]
    ip: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Response
{
    content: Vec<Folder>,
    #[serde(default)]
    name: String,
    #[serde(default)]
    parent_id: String,
    #[serde(default)]
    status: String,
    #[serde(default)]
    folder_id: String
}

type Result<T> = std::result::Result<T, PremiumizeError>;
#[derive(Debug, Clone)]
pub struct PremiumizeError;

impl std::convert::From<reqwest::Error> for PremiumizeError {
    fn from(_e: reqwest::Error) -> Self {
        Self{}
    }
}
impl std::convert::From<std::io::Error> for PremiumizeError {
    fn from(_e: std::io::Error) -> Self {
        Self{}
    }
}

pub struct Premiumize
{
    pub customer_id: String,
    pub key: String,
    pub client: Client
}

impl Premiumize
{
    fn list(&self, folder_id: Option<&str>) -> std::result::Result<Response, reqwest::Error>
    {
        let api = "https://www.premiumize.me/api/";
        let mut url = api.to_owned() + "folder/list" + "?customer_id=" + self.customer_id.as_str() + "&pin=" + self.key.as_str();

        match folder_id
        {
            Some(x) => url = url + "&id=" + x,
            None => {}
        }

        let resp: String = self.client.get(url.as_str()).send()?.text()?;

        let deserialized: Response = serde_json::from_str(resp.as_str()).unwrap();
        
        Ok(deserialized)
    }
    
    pub fn download(&self, folder_id: Option<&str>, local_dir: &str) -> Result<()>
    {
        match folder_id
        {
            Some(x) => println!("download {} {}", x, local_dir),
            None => println!("download None {}", local_dir)
        }
        
        let response = self.list(folder_id).unwrap();

        for item in response.content
        {
            let local = Path::new(local_dir).join(item.name.as_str());
            let path = local.to_str().unwrap();
            if item.type_ == "folder"
            {
                println!("folder {}({}) -> {}", item.name, item.id, path);
                create_dir(&local).ok();
                self.download(Some(item.id.as_str()), path)?;
            }
            else if item.type_ == "file" && !local.exists()
            {
                println!("downloading {}", item.link.as_str());
                let mut resp = self.client.get(item.link.as_str()).send()?;
                let mut file = File::create(path)?;
                let _bytes = copy(&mut resp, &mut file)?;
            }
        }
        Ok(())
    }
}
