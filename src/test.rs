use crate::{
    Error,DB,ApiV1,
    BatchWrite,BatchWriteOpp
};
// use futures::future::join_all;
use json::object;

pub async fn init(path:String,project_id:String)->Result<(),Error>{

    let mut client = DB::connect(path,project_id).await?;

    //add
    if false{
        if false {client.create_document(
            "",
            "users", "akku", &object!{
                "name":"akku",
                "age":25,
                "score":50
            }
        ).await?;}
        if false {client.create_document(
            "",
            "users", "akku1", &object!{
                "name":"akku1",
                "age":20,
                "score":60
            }
        ).await?;}
        if false {client.create_document(
            "",
            "users", "akku2", &object!{
                "name":"akku2",
                "age":23,
                "score":70
            }
        ).await?;}
        if false{
            if false{client.create_document(
                "/users/akku",
                "books", "king", &object!{
                    "book_name":"king",
                    "price":12,
                }
            ).await?;}
            if false{client.create_document(
                "/users/akku/books/king",
                "purchase", "india", &object!{
                    "units":5000,
                }
            ).await?;}
        }
    }

    //get
    if false{
        let res = client.get_document(
            "/users/akku/books/king"
        ).await?;
        println!("get : {:?}",res.json);
    }

    //update doc
    if false{
        let mut doc = client.get_document(
            "/users/akku1",
        ).await?;
        doc.json["score"] = 65.into();
        let update = doc.update(&mut client).await?;
        println!("update : {:?}",update);
    }

    //update
    if false{
        let update = client.update(
            "/users/akku1",
            &object!{
                "score":65
            }
        ).await?;
        println!("update : {:?}",update);
    }

    //delete
    if false{
        let res = client.delete_document(
            "/users/akku2"
        ).await?;
        println!("delete : {:?}",res);
    }

    //query
    if false{
        let mut query = DB::new_query();
        // query.parent("/users");
        query.from("users",true);
        query.add_where("age",">=",23.into());
        query.limit(2);
        let res = query.run(&mut client).await?;
        println!("query res : {:?}",res);
    }

    if false{
        let res = client.batch_read(vec![
            "/users/akku".to_string(),
            "/users/akku2".to_string(),
            "/users/akku1".to_string()
        ]).await?;
        println!("batch_read res : {:?}",res);
    }

    //=======================
    if false {client.create_document(
        "",
        "users", "akku2", &object!{
            "name":"akku2",
            "age":23,
            "score":70
        }
    ).await?;}

    if false{
        let res = client.batch_write(vec![
            BatchWriteOpp::Update(BatchWrite{
                path:"/users/akku3".to_string(),
                doc:object!{
                    age:23,
                    name:"akku3",
                    score:112
                }
            }),
            BatchWriteOpp::Update(BatchWrite{
                path:"/users/akku4".to_string(),
                doc:object!{
                    age:11,
                    name:"akku4",
                    score:119
                }
            }),
            BatchWriteOpp::Delete("/users/akku2".to_string())
        ]).await?;
        println!("batch_write res : {:?}",res);
    }

    return Ok(());

}
