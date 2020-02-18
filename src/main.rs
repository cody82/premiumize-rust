mod premiumize;
use premiumize::Premiumize;
use std::env;
use reqwest::blocking::Client;

fn main() -> Result<(), premiumize::PremiumizeError> {
    let p = Premiumize{client: Client::new(),customer_id: env::args().nth(2).unwrap().to_string(), key: env::args().nth(3).unwrap().to_string()};
    p.download(None, env::args().nth(1).unwrap().as_str())?;
    Ok(())
    
}
