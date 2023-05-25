use std::{fmt::Display, io::{Read, self}};

#[derive(PartialEq, Eq, Clone, Debug)]
#[repr(u8)]
pub enum Message {
    /// Gets the socket status message
    GetStatus = 0,
    /// Sets the socket status ID
    SetID(u8) = 1,
    /// Go to next status ID
    NextID = 2,

    /// Reply, contains status
    ReStatus(Vec<u8>) = 128,
    /// Reply, error: no id exists
    ReErrNoID = 129,
    /// Client sent server's response
    ReErrIdiot = 130,
    /// Unknown message format
    ReErrUnkwn = 131,
}

impl Message {
    pub fn read(v: &mut impl Read) -> io::Result<Option<Self>> {
        let mut d = [255];
        v.read_exact(&mut d)?;
        match d[0] {
            0 => Ok(Some(Self::GetStatus)),
            1 => {
                let mut id = [0];
                v.read_exact(&mut id)?;
                Ok(Some(Self::SetID(id[0])))
            },
            2 => Ok(Some(Self::NextID)),
            128 => {
                let mut len = [0];
                v.read_exact(&mut len)?;
                let mut buff = vec![0; len[0] as usize];
                v.read_exact(&mut buff)?;
                Ok(Some(Self::ReStatus(buff)))
            },
            129 => Ok(Some(Self::ReErrNoID)),
            130 => Ok(Some(Self::ReErrIdiot)),
            131 => Ok(Some(Self::ReErrUnkwn)),

            _ => Ok(None),
        }
    }

    pub fn as_raw_bytes(&self) -> Vec<u8> {
        match self {
            Self::GetStatus => vec![0],
            Self::SetID(i) => vec![1, *i],
            Self::NextID => vec![2],

            Self::ReStatus(b) => {
                let mut v = Vec::with_capacity(b.len() + 2);
                v.push(128);
                v.push(b.len() as u8);
                b.iter().copied().for_each(|b| v.push(b));
                v
            }
            Self::ReErrNoID => vec![129],
            Self::ReErrIdiot => vec![130],
            Self::ReErrUnkwn => vec![131],
        }
    }
}

impl Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::GetStatus => write!(f, "get status"),
            Self::SetID(i) => write!(f, "set id to `{i}`"),
            Self::NextID => write!(f, "set id to next"),
            Self::ReStatus(s) => write!(f, "status: {s:?}"),
            Self::ReErrNoID => write!(f, "err: no such id"),
            Self::ReErrIdiot => write!(f, "err: invalid message received"),
            Self::ReErrUnkwn => write!(f, "err: unknown message format"),
        }
    }
}

