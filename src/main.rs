mod premiumize;
use premiumize::Premiumize;
use premiumize::PremiumizeError;
extern crate clap;
use clap::{Arg, App, SubCommand};
use std::fs;
use std::env;
use serde::{Serialize, Deserialize};


#[derive(Serialize, Deserialize, Debug)]
pub struct Config
{
    #[serde(default)]
    pub customer_id: String,
    #[serde(default)]
    pub api_key: String
}

fn main() -> Result<(), premiumize::PremiumizeError> {
    let matches = App::new("Premiumize downloader")
                          .version("1.0")
                          .author("cody")
                          .about("Downloads files from premiumize.me")
                            .arg(Arg::with_name("customer-id")
                                .short("i")
                                .long("customer-id")
                                .help("Sets the customer id")
                                .takes_value(true)
                                .required(false))
                            .arg(Arg::with_name("api-key")
                                    .short("k")
                                    .long("api-key")
                                    .help("Sets the api key")
                                    .takes_value(true)
                                    .required(false))
                          .subcommand(SubCommand::with_name("delete")
                                      .about("deletes a folder")
                                      .arg(Arg::with_name("folder")
                                          .index(1)
                                          .required(true)
                                          .help("Folder to delete")))
                            .subcommand(SubCommand::with_name("clear")
                                        .about("deletes all files/folders in a folder")
                                        .arg(Arg::with_name("folder")
                                            .index(1)
                                            .required(true)
                                            .help("Folder to clear")))
                            .subcommand(SubCommand::with_name("mkdir")
                                        .about("create a folder")
                                        .arg(Arg::with_name("folder")
                                            .index(1)
                                            .required(true)
                                            .help("Folder to create")))
                            .subcommand(SubCommand::with_name("id")
                                        .about("get id")
                                        .arg(Arg::with_name("folder")
                                            .index(1)
                                            .required(true)
                                            .help("path")))
                            .subcommand(SubCommand::with_name("list")
                                        .about("list folder")
                                        .arg(Arg::with_name("folder")
                                            .index(1)
                                            .required(true)
                                            .help("path")))
                        .subcommand(SubCommand::with_name("download")
                                    .about("downloads a folder")
                                    .arg(Arg::with_name("dest")
                                        .index(1)
                                        .required(true)
                                        .takes_value(true)
                                        .help("Download destination path"))
                                    .arg(Arg::with_name("folder")
                                        .index(2)
                                        .required(false)
                                        .takes_value(true)
                                        .help("Folder to download"))
                                    )
                        .subcommand(SubCommand::with_name("transfer-url")
                                    .about("transfers an url")
                                    .arg(Arg::with_name("folder")
                                        .index(1)
                                        .required(true)
                                        .takes_value(true)
                                        .help("Destination folder"))
                                    .arg(Arg::with_name("url")
                                        .index(2)
                                        .required(true)
                                        .takes_value(true)
                                        .help("Source url"))
                                    )
                                    .subcommand(SubCommand::with_name("transfer-file")
                                                .about("transfers a file")
                                                .arg(Arg::with_name("folder")
                                                    .index(1)
                                                    .required(true)
                                                    .takes_value(true)
                                                    .help("Destination folder"))
                                                .arg(Arg::with_name("files")
                                                    .index(2)
                                                    .required(true)
                                                    .takes_value(true)
                                                    .max_values(100000)
                                                    .help("Source files(s)"))
                                                )
                          .get_matches();
    
    let mut home = match env::home_dir() {
        Some(path) => path,
        None => return Err(PremiumizeError{})
    };

    home.push(".premiumize-rust.json");
    //println!("{}", home.display());

    let config : Option<Config> = match fs::read_to_string(home) {
        Ok(x) => Some(serde_json::from_str(x.as_str()).unwrap()),
        Err(x) => None
    };

    let mut customer_id : Option<String> = None;
    let mut api_key : Option<String> = None;

    if config.is_some() {
        let c = &config.unwrap();
        customer_id = Some(c.customer_id.clone());
        api_key = Some(c.api_key.clone());
    }
    

    if matches.is_present("customer-id") {
        customer_id = Some(String::from(matches.value_of("customer-id").unwrap()));
    }
    if matches.is_present("api-key") {
        api_key = Some(String::from(matches.value_of("api-key").unwrap()));
    }

    if customer_id.is_none() || api_key.is_none() {
        println!("Customer ID or API key not set.");
        return Err(PremiumizeError{});
    }

    let p = Premiumize::new(customer_id.unwrap().as_str(), api_key.unwrap().as_str());

    if let Some(matches) = matches.subcommand_matches("download") {
        if matches.is_present("folder") {
            let id = p.id(matches.value_of("folder").unwrap())?;
            p.download(Some(id.as_str()), matches.value_of("dest").unwrap())?;
        }
        else {
            p.download(None, matches.value_of("dest").unwrap())?;
        }
    } else if let Some(matches) = matches.subcommand_matches("delete") {
        p.del(matches.value_of("folder").unwrap())?;
    } else if let Some(matches) = matches.subcommand_matches("clear") {
        p.clear(matches.value_of("folder").unwrap())?;
    } else if let Some(matches) = matches.subcommand_matches("id") {
        let id = p.id(matches.value_of("folder").unwrap())?;
        println!("{}", id.as_str());
    } else if let Some(matches) = matches.subcommand_matches("list") {
        let id = p.id(matches.value_of("folder").unwrap())?;
        let list = p.list(Some(id.as_str()))?;
        for f in list.content {
            println!("{}", f.name);
        }
    } else if let Some(matches) = matches.subcommand_matches("transfer-url") {
        let folder = matches.value_of("folder").unwrap();
        let url = matches.value_of("url").unwrap();
        p.create_transfer_url(url, folder)?;
    } else if let Some(matches) = matches.subcommand_matches("transfer-file") {
        let folder = matches.value_of("folder").unwrap();
        let files = matches.values_of("files").unwrap();
        for file in files {
            p.create_transfer_file(file, folder)?;
        }
    } else if let Some(matches) = matches.subcommand_matches("mkdir") {
        let f = matches.value_of("folder").unwrap();
        p.mkdir2(f)?;
    } else {
    }
    
    Ok(())
}
