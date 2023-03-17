use std::{io::{Read, Write}, num::TryFromIntError};

use byteorder::{ReadBytesExt, NativeEndian, WriteBytesExt};
use pcsc_host::card::{OpenpgpCardInfo};
use serde::{Serialize, Deserialize};
use thiserror::Error;

use {
    openpgp_card::{
        Error as OpenpgpCardError,
        SmartcardError as SmartcardError,
        StatusBytes as CardStatusBytes,
    },
    openpgp_card_pcsc::PcscBackend,
    pcsc_host::card::{
        OpenpgpCard,
    },
    std::{
        error::Error,
        io,
    },
};

#[derive(Serialize, Deserialize, Debug)]
struct PcscHostRequest {
    command: PcscHostCommand,
}

#[derive(Serialize, Deserialize, Debug)]
enum PcscHostCommand {
    ListCards,
    SignMessage {
        aid: String,
        message: Vec<u8>,
        pin: Vec<u8>,
    },
}

#[derive(Serialize, Deserialize, Debug)]
enum PcscHostResponse {
    Ok(ResponseData),
    Error(PcscHostError),
}

#[derive(Serialize, Deserialize, Debug)]
enum ResponseData {
    ListCards(Vec<OpenpgpCardInfo>),
    SignMessage(Vec<u8>),
    AwaitTouch,
}

#[derive(Serialize, Deserialize, Clone, Debug, Error, PartialEq, Eq)]
enum PcscHostError {
    #[error("internal OpenPGP error: {0}")]
    InternalError(String),
    #[error("error parsing AID: {0}")]
    AIDParseError(String),
    #[error("could not find card with AID {0}")]
    CardNotFound(String),
    #[error("invalid PIN")]
    InvalidPin,
    #[error("touch confirmation timed out")]
    TouchConfirmationTimeout,
}

impl PcscHostRequest {
    fn handle(&self) -> PcscHostResponse {
        match &self.command {
            PcscHostCommand::ListCards => {
                eprintln!("LIST CARDS");
                match list_cards() {
                    Ok(cards) => PcscHostResponse::Ok(ResponseData::ListCards(cards)),
                    Err(e) => PcscHostResponse::Error(e),
                }
            },
            PcscHostCommand::SignMessage { aid, message, pin } => {
                eprintln!("SIGN DATA");
                match sign_message(aid, message, pin) {
                    Ok(signature) => PcscHostResponse::Ok(ResponseData::SignMessage(signature)),
                    Err(e) => PcscHostResponse::Error(e),
                }
            },
        }
    }
}

fn write_touch_notification() {
    eprintln!("Awaiting touch confirmation...");
    let response = PcscHostResponse::Ok(ResponseData::AwaitTouch);
    write_response(&response).unwrap();
}

fn list_cards() -> Result<Vec<OpenpgpCardInfo>, PcscHostError> {
    let card_results = PcscBackend::cards(None);
    let backends = match card_results {
        Ok(b) => b,
        Err(OpenpgpCardError::Smartcard(SmartcardError::NoReaderFoundError)) => Vec::new(),
        Err(e) => return Err(PcscHostError::InternalError(e.to_string())),
    };
    let mut cards = Vec::<OpenpgpCardInfo>::new();
    for backend in backends {
        let card = OpenpgpCard::from(backend);
        cards.push(card.get_info().map_err(|e| PcscHostError::InternalError(e.to_string()))?);
    }
    Ok(cards)
}

fn sign_message(aid: &String, message: &Vec<u8>, pin: &Vec<u8>) -> Result<Vec<u8>, PcscHostError> {
    let card = OpenpgpCard::try_from(aid).map_err(
        |e| match e {
            OpenpgpCardError::ParseError(desc) => PcscHostError::AIDParseError(desc),
            OpenpgpCardError::Smartcard(SmartcardError::CardNotFound(_)) =>
                PcscHostError::CardNotFound(aid.to_string()),
            _ => PcscHostError::InternalError(e.to_string()),
        }
    )?;
    let signature = card.sign_message(
        &message.as_slice(),
        pin.as_slice(),
        write_touch_notification
    ).map_err(
        |e| match e {
            OpenpgpCardError::CardStatus(CardStatusBytes::IncorrectParametersCommandDataField) => {
                PcscHostError::InvalidPin
            },
            OpenpgpCardError::CardStatus(CardStatusBytes::SecurityRelatedIssues) => {
                PcscHostError::TouchConfirmationTimeout
            },
            _ => PcscHostError::InternalError(e.to_string()),
        }
    )?;
    Ok(signature)
}

#[derive(Debug, Error)]
enum ReadRequestError {
    #[error("end of input reached")]
    EndOfInput,
    #[error(transparent)]
    IoError(#[from] io::Error),
    #[error(transparent)]
    TryFromIntError(#[from] TryFromIntError),
    #[error(transparent)]
    SerdeError(#[from] serde_json::Error),
}

fn read_header() -> Result<u32, io::Error> {
    let header = io::stdin().read_u32::<NativeEndian>()?;
    eprintln!("READ HEADER");
    Ok(header)
}

fn read_request() -> Result<PcscHostRequest, ReadRequestError> {
    let msg_len = read_header().map_err(|_| ReadRequestError::EndOfInput)?;
    let mut buf = vec![0u8; msg_len.try_into()?];
    io::stdin().read_exact(&mut buf)?;
    let v: PcscHostRequest = serde_json::from_slice(buf.as_slice())?;
    eprintln!("READ REQUEST");
    eprintln!("{:?}", v);
    Ok(v)
}

fn write_header(msg_len: u32) -> Result<(), io::Error> {
    io::stdout().write_u32::<NativeEndian>(msg_len)?;
    eprintln!("WRITE HEADER");
    Ok(())
}

fn write_response(resp: &PcscHostResponse) -> Result<(), Box<dyn Error>> {
    let resp_string = serde_json::to_string(&resp)?;
    let resp_bytes = resp_string.as_bytes();
    let msg_len = resp_bytes.len();
    write_header(msg_len as u32)?;

    let bytes_written = io::stdout().write(&resp_bytes)?;
    assert_eq!(bytes_written, msg_len);

    io::stdout().flush()?;
    eprintln!("WRITE RESPONSE");
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    //println!("{}", serde_json::to_string(&PcscHostRequest{ command: PcscHostCommand::ListCards })?);
    eprintln!("START NATIVE HOST");
    loop {
        eprintln!("BEGIN CMD");
        let request = match read_request() {
            Ok(req) => req,
            Err(ReadRequestError::EndOfInput) => {
                eprintln!("TERMINATE NATIVE HOST");
                return Ok(());
            },
            Err(e) => return Err(Box::new(e))
        };
        eprintln!("START HANDLING");
        let response = request.handle();
        eprintln!("DONE HANDLING");
        write_response(&response)?;
        eprintln!("END CMD");
    }
}
