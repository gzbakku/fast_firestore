
use std::convert::From;
// use firestore::errors::FirestoreError;
use firestore_grpc::tonic::transport::Error as TonicError;
use firestore_grpc::tonic::metadata::errors::InvalidMetadataValue;
use firestore_grpc::tonic::Status as TonicStatus;
use crate::{ParserError,/*PathError*/};
use std::time::SystemTimeError;
use std::fmt::Debug;

#[derive(Debug)]
pub enum ErrType{
    None,
    NotFound,
    // PathError(PathError),
    SystemTimeError(SystemTimeError),
    ParserError(ParserError),
    TonicStatus(TonicStatus),
    TonicError(TonicError),
    InvalidMetadataValue(InvalidMetadataValue),
    FailedTokenGenerate,
    Static(&'static str),
    String(String)
}

#[allow(dead_code)]
impl ErrType{
    pub fn not_found(&self)->bool{
        match self{
            ErrType::NotFound=>{true},
            _=>{false}
        }
    }
}

//--------------------------------
//result traits
//--------------------------------

#[allow(dead_code)]
pub trait ResultCheckAnyToError<T,E>
where E:Debug{
    fn to_error(self)->Error;
    fn on_error_any(self,e:&'static str)->Result<T,Error>;
}

impl<T,E> ResultCheckAnyToError<T,E> for Result<T,E>
where E:Debug{
    fn to_error(self)->Error{
        match self{
            Ok(_)=>{return ().into();},
            Err(e)=>{
                return format!("{:?}",e).into();
            }
        }
    }
    fn on_error_any(self,e:&'static str)->Result<T,Error>{
        match self{
            Ok(_v)=>{Ok(_v)},
            Err(_e)=>{
                Err(format!("{:?} => {e}",_e).into())
            }
        }
    }
}

#[allow(dead_code)]

pub trait ResultCheckCrate<T>{
    fn on_error(self,tag:&'static str)->Result<T,Error>;
}

impl<T> ResultCheckCrate<T> for Result<T,Error>{
    fn on_error(self,tag:&'static str)->Result<T,Error>{
        match self{
            Ok(v)=>{Ok(v)},
            Err(_e)=>{
                Err(format!("{:?} => {tag}",_e).into())
            }
        }
    }
}

//--------------------------------
//result end
//--------------------------------

#[derive(Debug)]
pub struct Error{
    pub _e:ErrType
}

impl From<()> for Error{
    fn from(_:())->Error{
        Error{
            _e:ErrType::None
        }
    }
}

impl From<String> for Error{
    fn from(v:String)->Error{
        Error{
            _e:ErrType::String(v)
        }
    }
}

impl From<&'static str> for Error{
    fn from(v:&'static str)->Error{
        Error{
            _e:ErrType::Static(v)
        }
    }
}

// impl From<PathError> for Error{
//     fn from(v:PathError)->Error{
//         Error{
//             _e:ErrType::PathError(v)
//         }
//     }
// }

impl From<SystemTimeError> for Error{
    fn from(v:SystemTimeError)->Error{
        Error{
            _e:ErrType::SystemTimeError(v)
        }
    }
}

impl From<ParserError> for Error{
    fn from(v:ParserError)->Error{
        Error{
            _e:ErrType::ParserError(v)
        }
    }
}

impl From<ErrType> for Error{
    fn from(v:ErrType)->Error{
        Error{
            _e:v
        }
    }
}

impl From<TonicStatus> for Error{
    fn from(v:TonicStatus)->Error{
        Error{
            _e:ErrType::TonicStatus(v)
        }
    }
}

impl From<TonicError> for Error{
    fn from(v:TonicError)->Error{
        Error{
            _e:ErrType::TonicError(v)
        }
    }
}

impl From<InvalidMetadataValue> for Error{
    fn from(v:InvalidMetadataValue)->Error{
        Error{
            _e:ErrType::InvalidMetadataValue(v)
        }
    }
}

