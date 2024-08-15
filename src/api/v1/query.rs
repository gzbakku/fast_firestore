
use crate::{parser,DB,Error,Doc};
use json::JsonValue;
// use firestore_grpc::v1::Value;
use firestore_grpc::google::firestore::v1::{
    RunQueryRequest,
    StructuredQuery,Value,Cursor,
    run_query_request::QueryType,
    run_query_response::ContinuationSelector,
    structured_query::{
        CollectionSelector,Order,FieldReference,
        Filter,CompositeFilter,FieldFilter,
        filter::FilterType
    },
    // value::ValueType,
    // ArrayValue
};

// ValueType

///op == "<" less then
///op == "<=" less then equal
///op == ">" more then
///op == ">=" more then equal 
///op == "==" equal
///op == "!=" not equal
///op == "[]=" array contains
///op == "in" in array
///op == "[]<=" array contains any
///op == "not_in" not_in
pub fn parse_op_to_int(op:&'static str)->i32{
    if op == "<"{return 1;} else
    if op == "<="{return 2;} else
    if op == ">"{return 3;} else
    if op == ">="{return 4;} else
    if op == "=="{return 5;} else
    if op == "!="{return 6;} else
    if op == "[]="{return 7;} else//array contains
    if op == "in"{return 8;} else//in
    if op == "[]<="{return 9;} else//array contains any
    if op == "not_in"{return 10;} else//not_in
    {return 0;}
}

#[derive(Default,Debug)]
pub struct Query{
    query_parent:&'static str,
    query_where:Vec<(&'static str,i32,Value)>,
    query_where_string:Vec<(String,i32,Value)>,
    query_orders:Vec<(&'static str,i32)>,//key,direction,
    query_from:Vec<(&'static str,bool)>,
    query_start_at:Option<(Value,bool)>,
    query_end_at:Option<(Value,bool)>,
    query_offset:Option<i32>,
    query_limit:Option<i32>,
}

#[allow(dead_code)]
pub enum Direction{
    Asc,Desc
}

#[allow(dead_code)]
impl Query{
    pub fn new()->Query{
        Query{
            query_parent:"",
            query_where:Vec::new(),
            query_where_string:Vec::new(),
            query_orders:Vec::new(),
            query_from:Vec::new(),
            query_start_at:None,
            query_end_at:None,
            query_offset:None,
            query_limit:None,
        }
    }
    /// for string input use add_where_string
    ///op == "<" less then
    ///op == "<=" less then equal
    ///op == ">" more then
    ///op == ">=" more then equal 
    ///op == "==" equal
    ///op == "!=" not equal
    ///op == "[]=" array contains
    ///op == "in" in array
    ///op == "[]<=" array contains any
    ///op == "not_in" not_in
    pub fn add_where(&mut self,key:&'static str,op:&'static str,val:JsonValue){
        self.query_where.push(
            (key,parse_op_to_int(op),parser::JsonValueToDocValue(&val))
        );
    }
    ///op == "<" less then
    ///op == "<=" less then equal
    ///op == ">" more then
    ///op == ">=" more then equal 
    ///op == "==" equal
    ///op == "!=" not equal
    ///op == "[]=" array contains
    ///op == "in" in array
    ///op == "[]<=" array contains any
    ///op == "not_in" not_in
    pub fn add_where_string(&mut self,key:String,op:&'static str,val:JsonValue){
        self.query_where_string.push(
            (key,parse_op_to_int(op),parser::JsonValueToDocValue(&val))
        );
    }
    pub fn order(&mut self,key:&'static str,direction:Direction){
        match direction{
            Direction::Asc=>{self.query_orders.push((key,1));},
            Direction::Desc=>{self.query_orders.push((key,2));}
        }
    }
    pub fn from(&mut self,key:&'static str,val:bool){
        self.query_from.push((key,val));
    }
    pub fn parent(&mut self,parent:&'static str){
        self.query_parent = parent;
    }
    pub fn start_at(&mut self,value:JsonValue,before:bool){
        self.query_start_at = Some(
            (parser::JsonValueToDocValue(&value),before)
        );
    }
    pub fn end_at(&mut self,value:JsonValue,before:bool){
        self.query_end_at = Some(
            (parser::JsonValueToDocValue(&value),before)
        );
    }
    pub fn offset(&mut self,val:i32){
        self.query_offset = Some(val);
    }
    pub fn limit(&mut self,val:i32){
        self.query_limit = Some(val);
    }
    pub fn build(self)->StructuredQuery{
        build(self)
    }
    pub async fn run(self,db:&mut DB)->Result<Vec<Doc>,Error>{

        let parent = format!(
            "projects/{}/databases/(default)/documents{}",
            db.project_id,self.query_parent
        );
        // let as_bytes = parent.clone().as_bytes().to_vec();
        // println!("parent : {} {:?} {:?} \n {:?}",parent,parent.len(),as_bytes[58],as_bytes);
        let query = self.build();

        let req = db.request(RunQueryRequest{
            parent:parent,
            explain_options:None,
            query_type:Some(QueryType::StructuredQuery(query)),
            consistency_selector:None
        }).await?;

        let mut res = db.channel.run_query(req).await?.into_inner();
        let mut collect:Vec<Doc> = vec![];
        loop{
            match res.message().await{
                Ok(opt)=>{
                    match opt{
                        Some(a)=>{
                            match a.document{
                                Some(d)=>{
                                    collect.push(d.into());
                                },
                                None=>{}
                            }
                            match a.continuation_selector{
                                Some(c)=>{
                                    match c{
                                        ContinuationSelector::Done(g)=>{
                                            if g{break;}
                                        }
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

        return Ok(collect);

    }
}

fn build(query:Query)->StructuredQuery{
    let mut build = StructuredQuery::default();
    if query.query_where.len() > 0{
        build.r#where = build_filters(
            query.query_where,
            query.query_where_string
        );
    }
    if query.query_orders.len() > 0{build.order_by = build_orders(query.query_orders);}
    // println!("order : {:#?}",build.order_by);
    if query.query_from.len() > 0{build.from = build_from(query.query_from);}
    build.start_at = build_cursor(query.query_start_at);
    build.end_at = build_cursor(query.query_end_at);
    match query.query_offset{Some(v)=>{build.offset = v;},None=>{}}
    build.limit = query.query_limit;
    return build;
}

fn build_from(mut pool:Vec<(&'static str,bool)>)->Vec<CollectionSelector>{
    let mut collect = vec![];
    loop{
        if pool.len() == 0{break;}
        let hold = pool.remove(0);
        collect.push(CollectionSelector{
            collection_id:hold.0.to_string(),
            all_descendants:hold.1
        });
    }
    return collect;
}

fn build_cursor(hold:Option<(Value,bool)>)->Option<Cursor>{
    match hold{
        Some(v)=>{
            return Some(Cursor{
                values:vec![v.0],
                before:v.1
            });
        },
        None=>{
            return None;
        }
    }
}

fn build_orders(mut pool:Vec<(&'static str,i32)>)->Vec<Order>{
    let mut collect = vec![];
    loop{
        if pool.len() == 0{break;}
        let hold = pool.remove(0);
        collect.push(Order{
            field:Some(FieldReference{
                field_path:hold.0.to_string()
            }),
            direction:hold.1
        });
    }
    return collect;
}

fn build_filters(
    mut pool_1:Vec<(&'static str,i32,Value)>,
    mut pool_2:Vec<(String,i32,Value)>
)->Option<Filter>{
    let mut collect = vec![];
    loop{
        if pool_1.len() == 0{break;}
        let hold = pool_1.remove(0);
        collect.push(build_filter(hold.0.to_string(),hold.1,hold.2));
    }
    loop{
        if pool_2.len() == 0{break;}
        let hold = pool_2.remove(0);
        collect.push(build_filter(hold.0,hold.1,hold.2));
    }
    Some(
        Filter{
            filter_type:Some(FilterType::CompositeFilter(CompositeFilter{
                op:1,
                filters:collect
            }))
        }
    )
}

fn build_filter(key:String,op:i32,value:Value)->Filter{
    // let v;
    // if op == 8{
    //     let is_array;
    //     match value{
    //         ValueType::ArrayValue(_v)=>{
    //             v = Value{
    //                 value_type:Some(ValueType::ArrayValue(
    //                     _v
    //                 ))
    //             };
    //         },
    //         _=>{
    //             v = Value{
    //                 value_type:Some(
    //                     ValueType::ArrayValue(
    //                         ArrayValue{
    //                             values:vec![value]
    //                         }
    //                     )
    //                 )
    //             };
    //         }
    //     }
    // } else {
    //     v = value;
    // }
    let v = value;
    Filter{
        filter_type:Some(FilterType::FieldFilter(FieldFilter{
            field:Some(FieldReference{
                field_path:key
            }),
            op:op,
            value:Some(v)
        }))
    }
}