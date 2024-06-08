
mod common;
mod test;
mod db;
mod api;
pub mod parser;

pub use common::{Error,ErrType};
pub use db::DB;
pub use api::v1::{
    ApiV1,Query,Doc,BatchRead,
    BatchWriteOpp,BatchWrite
};
pub use parser::ParserError;

#[tokio::main]
async fn main()->Result<(),Error>{

    let creds_path = "../creds/firebase_daachi.json".to_string();

    match test::_query(
        creds_path,
        "daachi-cd79a".to_string()
    ).await{
        Ok(_)=>{},
        Err(_e)=>{

            if false{
                match &_e._e{
                    ErrType::TonicStatus(t)=>{
                        let message = t.message().to_string();
                        println!("\nstatus : {}\n",message);
                        println!("\n========================");
                        let as_bytes = message.clone().as_bytes().to_vec();
                        let mut index = 0;
                        for char in message.chars(){
                            print!(" {:?} {:?}",
                            char,
                            as_bytes[index]
                        );
                            index += 1;
                        }
                        println!("\n========================\n");
    
                    },
                    _=>{}
                }
            }

            println!("!!! failed x12 init => {:?}",_e);

            return Err(_e);

        }
    }

    Ok(())

}
