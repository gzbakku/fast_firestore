
use std::convert::From;
// use firestore::errors::FirestoreError;
use firestore_grpc::tonic::transport::Error as TonicError;
use firestore_grpc::tonic::metadata::errors::InvalidMetadataValue;
use firestore_grpc::tonic::Status as TonicStatus;
use crate::{ParserError,/*PathError*/};
use std::time::SystemTimeError;

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
    Static(&'static str)
}

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

