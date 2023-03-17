
use std::collections::HashMap;
use json::{number::Number,object::Object,object,JsonValue};
use firestore_grpc::v1::{
    // prost_types::Timestamp,
    Document, Value,value::ValueType,ArrayValue,MapValue
};
// use firestore_grpc::prost_types;
use prost_types::Timestamp;
use std::time::{SystemTime, UNIX_EPOCH,Duration};
use crate::{Error};

fn unix_epoch_time()->Result<Duration,Error>{
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(n)=>{return Ok(n);},
        Err(_e)=>{return Err(_e.into());}
    }
}

pub fn timestamp()->Result<Timestamp,Error>{
    let time = unix_epoch_time()?;
    let count = time.as_nanos() as i64;
    let secs:i64 = count/1000000000;
    let nano = count - (secs*1000000000);
    let build = Timestamp{
        seconds:secs,
        nanos:nano as i32
    };
    return Ok(build);
}

#[derive(Debug)]
pub enum ParserError{
    InvalidJsonObject,None
}

#[allow(non_snake_case)]
pub fn DocToJson(v:&Document)->JsonValue{
    let mut hold = Object::new();
    for (key,value) in v.fields.iter(){
        hold.insert(key,DocValueToJsonValue(value));
    }
    return JsonValue::Object(hold);
}

#[allow(non_snake_case)]
pub fn JsonToDoc(v:&JsonValue)->Result<Document,Error>{
    match v{
        JsonValue::Object(_)=>{
            match JsonValueToDocValue(v).value_type{
                Some(k)=>{
                    match k{
                        ValueType::MapValue(m)=>{
                            let build = Document{
                                name:String::new(),
                                fields:m.fields,
                                create_time:Some(timestamp()?),
                                update_time:None
                            };
                            return Ok(build);
                        },
                        _=>{}
                    }
                }
                None=>{}
            }
        },
        _=>{}
    }
    return Err(ParserError::InvalidJsonObject.into());
}

#[allow(non_snake_case)]
pub fn DocValueToJsonValue(o:&Value)->JsonValue{

    let h;
    match &o.value_type{
        Some(k)=>{
            h = k;
        },
        None=>{
            return JsonValue::Null;
        }
    }

    match h{
        ValueType::NullValue(_)=>{
            return JsonValue::Null;
        },
        ValueType::BooleanValue(v)=>{
            return JsonValue::Boolean(v.clone());
        },
        ValueType::IntegerValue(v)=>{
            return JsonValue::Number(v.clone().into());
        },
        ValueType::DoubleValue(v)=>{
            return JsonValue::Number(v.clone().into());
        },
        ValueType::TimestampValue(v)=>{
            let s:Number = v.seconds.clone().into();
            let n:Number = v.nanos.clone().into();
            return object!{"seconds":s,"nanos":n};
        },
        ValueType::StringValue(v)=>{
            return JsonValue::String(v.clone().into());
        },
        ValueType::BytesValue(v)=>{
            let mut collect = vec![];
            for i in v{
                let u:Number = i.clone().into();
                collect.push(JsonValue::Number(u));
            }
            return JsonValue::Array(collect);
        },
        ValueType::ReferenceValue(v)=>{
            return JsonValue::String(v.clone().into());
        },
        ValueType::GeoPointValue(v)=>{
            let lat:Number = v.latitude.clone().into();
            let long:Number = v.longitude.clone().into();
            return object!{"latitude":lat,"longitude":long};
        },
        ValueType::ArrayValue(v)=>{
            let mut collect = vec![];
            for i in &v.values{
                collect.push(DocValueToJsonValue(i));
            }
            return JsonValue::Array(collect);
        },
        ValueType::MapValue(v)=>{
            let mut collect = Object::new();
            for (key,value) in v.fields.iter(){
                collect.insert(key,DocValueToJsonValue(value));
            }
            return JsonValue::Object(collect);
        },
    }

}

#[allow(non_snake_case)]
pub fn JsonValueToDocValue(v:&JsonValue)->Value{
    match v{
        JsonValue::Null=>{
            return buildValue(ValueType::NullValue(0));
        },
        JsonValue::Short(v)=>{
            let h = v.as_str();
            return buildValue(ValueType::StringValue(h.to_string()));
        },
        JsonValue::String(v)=>{
            return buildValue(ValueType::StringValue(v.clone()));
        },
        JsonValue::Number(n)=>{
            let (positive, mantissa, exponent) = n.as_parts();
            if exponent != 0{
                let y:f64 = n.clone().into();
                return buildValue(ValueType::DoubleValue(y));
            } else {
                let y:u64 = mantissa.clone().into();
                let mut z = y as i64;
                if !positive{z = z * -1;}
                // let z = y as i64;
                return buildValue(ValueType::IntegerValue(z));
            }
        },
        JsonValue::Boolean(v)=>{
            return buildValue(ValueType::BooleanValue(v.clone()));
        },
        JsonValue::Object(v)=>{
            let mut collect = HashMap::new();
            for (key,value) in v.iter(){
                collect.insert(key.to_string(),JsonValueToDocValue(value));
            }
            return buildValue(ValueType::MapValue(MapValue{
                fields:collect
            }));
        },
        JsonValue::Array(v)=>{
            let mut collect = vec![];
            for i in v{
                collect.push(JsonValueToDocValue(i));
            }
            return buildValue(ValueType::ArrayValue(ArrayValue{
                values:collect
            }));
        }
    }
}

#[allow(non_snake_case)]
fn buildValue(v:ValueType)->Value{
    Value{
        value_type:Some(v)
    }
}