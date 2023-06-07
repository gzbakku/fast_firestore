use firestore_grpc::tonic::{
    metadata::{MetadataValue,Ascii},
    transport::{Channel},
    Request
};
use firestore_grpc::v1::{
    firestore_client::FirestoreClient
};
// use firestore_grpc::tonic::client;
// use firestore_grpc::toni
use crate::{Error};
// use crate::{Path};

const URL: &'static str = "https://firestore.googleapis.com";
// const DOMAIN: &'static str = "firestore.googleapis.com";


use std::{time::Instant};

#[derive(Debug,Clone)]
pub struct DB{
    pub project_id:String,
    pub path:String,
    pub time:Instant,
    pub token:MetadataValue<Ascii>,
    pub channel:FirestoreClient<Channel>
}

impl DB{
    pub async fn get_token(&mut self)->Result<MetadataValue<Ascii>,Error>{
        self.check_time().await?;
        return Ok(self.token.clone());
    }
    pub async fn check_time(&mut self)->Result<(),Error>{
        if self.time.elapsed().as_secs() > 1500{
            self.token = generate_token(&self.path).await?;
            self.time = Instant::now();
        }
        return Ok(());
    }
    pub async fn connect(
        creds_file_location:String,
        project_id:String
    )->Result<DB,Error>{
        return Ok(DB{
            project_id:project_id,
            path:creds_file_location.clone(),
            time:Instant::now(),
            token:generate_token(&creds_file_location).await?,
            channel:FirestoreClient::connect(URL).await?
        });
    }
    pub async fn request<T>(&mut self,t:T)->Result<Request<T>,Error>{
        self.check_time().await?;
        let mut request = Request::new(t);
        let metadata = request.metadata_mut();
        metadata.insert("authorization", self.token.clone());
        return Ok(request);
    }
}

pub async fn generate_token(path:&str)->Result<MetadataValue<Ascii>,Error>{
    let bearer_token = format!("Bearer {}", get_token(path).await?);
    let header_value = MetadataValue::from_str(&bearer_token)?;
    return Ok(header_value);
}

pub async fn get_token(path:&str)->Result<String,Error>{
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

