
use std::convert::From;
use crate::{DB,Error};
use async_trait::async_trait;
use json::JsonValue;
use crate::parser;
use std::collections::HashMap;

mod query;

pub use query::Query;

use firestore_grpc::v1::{
    CreateDocumentRequest,GetDocumentRequest,
    UpdateDocumentRequest, Document,
    DeleteDocumentRequest,DocumentMask,
    BatchWriteRequest,Write,
    write::Operation,
    BatchGetDocumentsRequest,
    batch_get_documents_response::Result as BatchResultEnum
};

#[async_trait]
pub trait ApiV1{
    async fn create_document(
        &mut self,
        parent:&str,
        collection:&str,
        document_id:&str,
        document:&JsonValue
    )->Result<(),Error>;
    async fn get_document(
        &mut self,
        parent:&str
    )->Result<Doc,Error>;
    async fn update(
        &mut self,
        parent:&str,
        doc:&JsonValue
    )->Result<(),Error>;
    async fn update_document(
        &mut self,
        document:&mut Doc,
    )->Result<(),Error>;
    async fn delete_document(
        &mut self,
        parent:&str
    )->Result<(),Error>;
    fn new_query()->Query;
    async fn run_query(
        &mut self,
        query:Query,
    )->Result<Vec<Doc>,Error>;
    async fn batch_read(
        &mut self,
        docs:Vec<String>
    )->Result<BatchRead,Error>;
    async fn batch_write(
        &mut self,
        docs:Vec<BatchWriteOpp>
    )->Result<(),Error>;
}

#[async_trait]
impl ApiV1 for DB{
    /// # Arguments
    /// 
    /// * `parent` - leave it empty if top level collection, if not its the parent of where the collection exists ie for a document "/c:users/d:akku/c:books/d:rust" the parent would be "/c:users/d:akku"
    /// 
    /// * `collection_id` - collection_id is the collection name string
    /// 
    /// * `document_id` - document id is the document unique name
    /// 
    /// * `document` - document is a JsonValue from json crate
    async fn create_document(
        &mut self,
        
        parent:&str,
        collection_id:&str,
        document_id:&str,
        document:&JsonValue
    )->Result<(),Error>{
        let req = self.request(CreateDocumentRequest {
            parent:format!(
                "projects/{}/databases/(default)/documents{}",
                self.project_id,parent
            ),
            collection_id:collection_id.into(),
            document_id:document_id.into(),
            document:Some(parser::JsonToDoc(&document)?),
            mask: None,
        }).await?;
        self.channel.create_document(req).await?;
        return Ok(());
    }
    async fn get_document(
        &mut self,
        parent:&str
    )->Result<Doc,Error>{
        let req = self.request(GetDocumentRequest {
            name:format!(
                "projects/{}/databases/(default)/documents{}",
                self.project_id,parent
            ),
            mask: None,
            consistency_selector:None
        }).await?;
        let res = self.channel.get_document(req).await?;
        let raw_doc = res.into_inner();
        let parsed = parser::DocToJson(&raw_doc);
        return Ok(Doc{
            document:raw_doc,
            json:parsed
        });
    }
    async fn update(
        &mut self,
        parent:&str,
        doc:&JsonValue
    )->Result<(),Error>{
        let mut updated = vec![];
        for (key,_) in doc.entries(){
            updated.push(key.to_string());
        }
        let mut create_doc = parser::JsonToDoc(doc)?;
        create_doc.name = format!(
            "projects/{}/databases/(default)/documents{}",
            self.project_id,parent
        );
        let req = self.request(UpdateDocumentRequest {
            document:Some(create_doc),
            update_mask:Some(DocumentMask{
                field_paths:updated
            }),
            mask:None,
            current_document:None
        }).await?;
        self.channel.update_document(req).await?;
        return Ok(());
    }
    async fn update_document(
        &mut self,
        document:&mut Doc,
    )->Result<(),Error>{
        document.update(self).await?;
        return Ok(());
    }
    async fn delete_document(
        &mut self,
        parent:&str,
    )->Result<(),Error>{
        let req = self.request(DeleteDocumentRequest {
            name:format!(
                "projects/{}/databases/(default)/documents{}",
                self.project_id,parent
            ),
            current_document:None
        }).await?;
        self.channel.delete_document(req).await?;
        return Ok(());
    }
    fn new_query()->Query{Query::new()}
    async fn run_query(&mut self,query:Query)->Result<Vec<Doc>,Error>{
        let res = query.run(self).await?;
        Ok(res)
    }
    async fn batch_read(
        &mut self,
        docs:Vec<String>
    )->Result<BatchRead,Error>{

        let req = self.request(BatchGetDocumentsRequest {
            database:format!(
                "projects/{}/databases/(default)",
                self.project_id
            ),
            documents: docs.iter().map(|v|{
                format!(
                    "projects/{}/databases/(default)/documents{}",
                    self.project_id,v
                )
            }).collect(),
            mask:None,
            consistency_selector:None
        }).await?;

        let mut res = self.channel.batch_get_documents(req).await?.into_inner();

        let mut collect:Vec<Doc> = vec![];
        let mut missing:Vec<String> = vec![];

        loop{
            match res.message().await{
                Ok(o)=>{
                    match o{
                        Some(r)=>{
                            match r.result{
                                Some(d)=>{
                                    match d{
                                        BatchResultEnum::Found(l)=>{collect.push(l.into())},
                                        BatchResultEnum::Missing(m)=>{missing.push(m);}
                                    }
                                },
                                None=>{}
                            }
                        },
                        None=>{
                            break;
                        }
                    }
                },
                Err(_e)=>{
                    return Err(_e.into());
                }
            }
        }

        return Ok(BatchRead{
            docs:collect,
            missing:missing
        });

    }
    async fn batch_write(
        &mut self,
        docs:Vec<BatchWriteOpp>
    )->Result<(),Error>{

        let mut collect = vec![];
        for item in docs{
            match item{
                BatchWriteOpp::Update(w)=>{
                    let keys:Vec<String> = w.doc.entries().map(|(key,_)|{
                        key.to_string()
                    }).collect();
                    let mut doc = parser::JsonToDoc(&w.doc)?;
                    doc.name = format!(
                        "projects/{}/databases/(default)/documents{}",
                        self.project_id,w.path
                    );
                    collect.push(Write{
                        update_mask:Some(DocumentMask{
                            field_paths:keys
                        }),
                        update_transforms:vec![],
                        current_document:None,
                        operation:Some(Operation::Update(doc))
                    });
                },
                BatchWriteOpp::Delete(s)=>{
                    collect.push(Write{
                        update_mask:None,
                        update_transforms:vec![],
                        current_document:None,
                        operation:Some(Operation::Delete(format!(
                            "projects/{}/databases/(default)/documents{}",
                            self.project_id,s
                        )))
                    });
                }
            }
        }

        let req = self.request(BatchWriteRequest{
            database:format!(
                "projects/{}/databases/(default)",
                self.project_id
            ),
            writes:collect,
            labels:HashMap::new()
        }).await?;

        self.channel.batch_write(req).await?;

        return Ok(());

    }
}

#[derive(Debug,Clone)]
pub struct BatchWrite{
    pub path:String,
    pub doc:JsonValue
}

#[derive(Debug,Clone)]
pub enum BatchWriteOpp{
    Update(BatchWrite),Delete(String)
}

#[derive(Debug,Clone)]
pub struct BatchRead{
    pub docs:Vec<Doc>,
    pub missing:Vec<String>
}

#[derive(Debug,Clone)]
pub struct Doc{
    pub document:Document,
    pub json:JsonValue
}

impl Doc{
    pub async fn update(&mut self,db:&mut DB)->Result<(),Error>{
        let updated_doc = parser::JsonToDoc(&self.json)?;
        // let mut updated = self.document.clone();
        self.document.fields = updated_doc.fields;
        let req = db.request(UpdateDocumentRequest {
            document:Some(self.document.clone()),
            update_mask:None,
            mask:None,
            current_document:None
        }).await?;
        db.channel.update_document(req).await?;
        return Ok(());
        // return Err(().into());
    }
}

impl From<Document> for Doc{
    fn from(v:Document)->Doc{
        let parsed = parser::DocToJson(&v);
        Doc{
            document:v,
            json:parsed
        }
    }
}