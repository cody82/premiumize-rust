extern crate reqwest;

use serde::{Serialize, Deserialize};
use std::path::Path;
use std::fs::File;
use std::fs::create_dir;
use reqwest::blocking::Client;
use reqwest::blocking::Body;
use std::io::{ErrorKind, Read, Write};
use indicatif::ProgressBar;
use indicatif::ProgressStyle;
use reqwest::blocking::multipart;
use std::io::BufReader;
use std::error::Error;

pub fn copy2<R: ?Sized, W: ?Sized>(reader: &mut R, writer: &mut W, bar: &ProgressBar) -> std::io::Result<u64>
where
    R: Read,
    W: Write,
{
    let mut buf : [u8; 512*1024] = [0;512*1024];

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
        bar.inc(len as u64);
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Folder
{
    pub id: String,
    pub name: String,
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
    pub content: Vec<Folder>,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    parent_id: String,
    #[serde(default)]
    status: String,
    #[serde(default)]
    folder_id: String
}

type Result<T> = std::result::Result<T, PremiumizeError>;

#[derive(Debug, Clone)]
pub struct PremiumizeError
{
    pub message: String
}

impl PremiumizeError{
    pub fn new() -> Self {
        Self {
            message: "?".to_string()
        } 
    }
}

impl std::convert::From<reqwest::Error> for PremiumizeError {
    fn from(e: reqwest::Error) -> Self {
        Self {
            message: format!("reqwest::Error: {}", e)
        }
    }
}
impl std::convert::From<std::io::Error> for PremiumizeError {
    fn from(e: std::io::Error) -> Self {
        Self {
            message: e.description().to_string()
        }
    }
}

pub struct Premiumize
{
    pub customer_id: String,
    pub key: String,
    pub client: Client
}

const API: &'static str = "https://www.premiumize.me/api/";

impl Premiumize
{
    pub fn new(customer_id: &str, api_key: &str) -> Premiumize {
        Premiumize {
            customer_id: String::from(customer_id),
            key: String::from(api_key),
            client: Client::new()
        }
    }

    pub fn id(&self, path: &str) -> Result<String> {
        let mut list = self.list(None)?;

        for part in path.split("/") {
            if part.len() > 0 {
                match list.content.iter().find(|&x| x.name == part) {
                    Some(x) => {
                        list = self.list(Some(x.id.as_str()))?;
                    }
                    None => return Err(PremiumizeError{message: "Can not get id for path.".to_string()})
                }
            }
        }
        Ok(list.folder_id)
    }

    pub fn del(&self, name: &str) -> Result<()> {
        let id = self.id(name)?;
        self.del_id(id.as_str())
    }

    pub fn del_id(&self, id: &str) -> Result<()> {
        let url = API.to_owned() + "folder/delete" + "?customer_id=" + self.customer_id.as_str() + "&pin=" + self.key.as_str() + "&id=" + id;
        let resp: String = self.client.get(url.as_str()).send()?.text()?;

        Ok(())
    }
    
    pub fn del_file_id(&self, id: &str) -> Result<()> {
        let url = API.to_owned() + "item/delete" + "?customer_id=" + self.customer_id.as_str() + "&pin=" + self.key.as_str() + "&id=" + id;
        let resp: String = self.client.get(url.as_str()).send()?.text()?;

        Ok(())
    }

    pub fn clear(&self, folder: &str) -> Result<()> {
        let id = self.id(folder)?;
        let response = self.list(Some(id.as_str()))?;
        
        for item in response.content
        {
            if item.type_ == "folder"
            {
                self.del_id(item.id.as_str())?;
            }
            else if item.type_ == "file"
            {
                self.del_file_id(item.id.as_str())?;
            }
        }
        Ok(())
    }

    pub fn mkdir2(&self, fullname: &str) -> Result<()> {
        let parts : Vec<&str> = fullname.split("/").collect();
        let name = match parts.last() {
            Some(x) => x,
            None => return Err(PremiumizeError{message: "Invalid path.".to_string()})
        };

        let path = parts.iter().take(parts.len() - 1).fold("".to_string(), |a,b| a + "/" + b);
        self.mkdir(path.as_str(), name)
    }
    
    pub fn create_transfer_url(&self, url: &str, target_dir: &str) -> Result<()> {
        let folder_id = self.id(target_dir)?;
        let requrl = API.to_owned() + "transfer/create" + "?customer_id=" + self.customer_id.as_str() + "&pin=" + self.key.as_str();// + "&src=" + url + "&folder_id=" + folder_id.as_str();

        let params = [("src", url), ("folder_id", folder_id.as_str())];
        let resp = self.client.post(requrl.as_str()).form(&params).send()?;

        if resp.status().is_success() {
        }
        else {
            return Err(PremiumizeError{message: "Request failed.".to_string()});
        }
        Ok(())
    }

    pub fn create_transfer_file(&self, filepath: &str, target_dir: &str) -> Result<()> {
        let folder_id = self.id(target_dir)?;
        let requrl = API.to_owned() + "transfer/create" + "?customer_id=" + self.customer_id.as_str() + "&pin=" + self.key.as_str() + "&folder_id=" + folder_id.as_str();
       
        //let folder = folder_id.as_str();

        let form = multipart::Form::new()
            //.text("folder_id", folder)
            .file("file", filepath)?;

        let resp = self.client.post(requrl.as_str()).multipart(form).send()?;

        if resp.status().is_success() {
        }
        else {
            return Err(PremiumizeError{message: "Request failed.".to_string()});
        }
        Ok(())
    }

    pub fn mkdir(&self, path: &str, name: &str) -> Result<()> {
        let parent_id = self.id(path)?;
        let url = API.to_owned() + "folder/create" + "?customer_id=" + self.customer_id.as_str() + "&pin=" + self.key.as_str() + "&parent_id=" + parent_id.as_str() + "&name=" + name;
        let resp: String = self.client.get(url.as_str()).send()?.text()?;

        Ok(())
    }

    pub fn list(&self, folder_id: Option<&str>) -> std::result::Result<Response, reqwest::Error>
    {
        let mut url = API.to_owned() + "folder/list" + "?customer_id=" + self.customer_id.as_str() + "&pin=" + self.key.as_str();

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
                println!("file {}", item.link.as_str());
                
                let bar = ProgressBar::new(item.size);
                bar.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.green/grey}] {bytes}/{total_bytes} ({eta})"));
        //.progress_chars("#>-"));

                let mut resp = self.client.get(item.link.as_str()).send()?;
                if resp.status().is_success() {
                    let mut file = File::create(path)?;
                    let _bytes = copy2(&mut resp, &mut file, &bar)?;
                }
                else {
                    return Err(PremiumizeError{message: "Download failed.".to_string()});
                }
                bar.finish();
            }
        }
        Ok(())
    }
}
