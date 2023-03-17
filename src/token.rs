
use crate::Error;

pub async fn init(path:&str)->Result<String,Error>{
    match gcp_access_token::generator::init(
        path.to_string(),
        "https://www.googleapis.com/auth/cloud-platform".to_string()
    ).await{
        Ok(v)=>{
            match v["access_token"].as_str(){
                Some(s)=>{
                    return Ok(s.to_string());
                },
                None=>{
                    return Err(().into());
                },
            }
        },
        Err(_e)=>{
            return Err(().into());
        }
    }
}