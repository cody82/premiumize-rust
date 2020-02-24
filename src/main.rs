mod premiumize;
use premiumize::Premiumize;
extern crate clap;
use clap::{Arg, App, SubCommand};

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
                                .required(true))
                            .arg(Arg::with_name("api-key")
                                    .short("k")
                                    .long("api-key")
                                    .help("Sets the api key")
                                    .takes_value(true)
                                    .required(true))
                          .subcommand(SubCommand::with_name("delete")
                                      .about("deletes a folder")
                                      .arg(Arg::with_name("folder")
                                          .index(1)
                                          .required(true)
                                          .help("Folder to delete")))
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
                          .get_matches();
    
    let p = Premiumize::new(matches.value_of("customer-id").unwrap(), matches.value_of("api-key").unwrap());

    if let Some(matches) = matches.subcommand_matches("download") {
        if matches.is_present("folder") {
            let id = p.id(matches.value_of("folder").unwrap())?;
            p.download(Some(id.as_str()), matches.value_of("dest").unwrap())?;
        }
        else {
            p.download(None, matches.value_of("dest").unwrap())?;
        }
    } else if let Some(matches) = matches.subcommand_matches("delete") {
    } else {
    }
    
    Ok(())
}
