use std::{io::{BufRead, Read, Write}, num::TryFromIntError};

use byteorder::{ReadBytesExt, NativeEndian, WriteBytesExt};
use pcsc_host::card;
use serde::{Serialize, Deserialize};
use serde_json::Value;
use solana_sdk::{pubkey::Pubkey, signature::Signature};
use thiserror::Error;

use {
    openpgp_card_pcsc::PcscBackend,
    pcsc_host::card::{
        Locator,
        OpenpgpCard,
    },
    solana_sdk::signer::Signer,
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
    SignData {
        aid: String,
        digest: Vec<u8>,
    },
}

#[derive(Serialize, Deserialize, Debug)]
struct PgpSignerInfo {
    pub aid: String,
    pub pk: Pubkey,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug)]
enum PcscHostResponse {
    ListCards(Vec<PgpSignerInfo>),
    SignData(Signature),
}

impl PcscHostRequest {
    fn handle(&self) -> Result<PcscHostResponse, Box<dyn Error>> {
        let response = match &self.command {
            PcscHostCommand::ListCards => {
                eprintln!("LIST CARDS");
                PcscHostResponse::ListCards(list_cards()?)
            },
            PcscHostCommand::SignData { aid, digest } => {
                eprintln!("SIGN DATA");
                eprintln!("{:?}", &self.command);
                //PcscHostResponse::SignData(Signature::new(&[7u8; 64]))

                let uri_string = format!("pgpcard://{}", aid);
                let uri = uriparse::URIReference::try_from(uri_string.as_str())?;
                let card = OpenpgpCard::try_from(
                    &Locator::try_from(&uri)?
                )?;
                let signature = card.sign_message(&digest.as_slice());
                PcscHostResponse::SignData(signature)
            },
        };
        eprintln!("DONE HANDLING");
        Ok(response)
    }
}

fn list_cards() -> Result<Vec<PgpSignerInfo>, Box<dyn Error>> {
    let card_results = PcscBackend::cards(None);
    let backends = match card_results {
        Ok(b) => b,
        Err(openpgp_card::Error::Smartcard(openpgp_card::SmartcardError::NoReaderFoundError)) => Vec::new(),
        _ => card_results?,
    };
    let mut cards = Vec::<PgpSignerInfo>::new();
    for backend in backends {
        let card = OpenpgpCard::from(backend);
        let (name, aid ) = card.get_name_and_aid()?;
        cards.push(PgpSignerInfo { name, aid, pk: card.pubkey() });
    }
    Ok(cards)
}

// fn demo() -> Result<(), Box<dyn Error>> {
//     let big_yubi_uri = uriparse::URIReference::try_from("pgpcard://D2760001240103040006223637060000")?;
//     let big_yubi = OpenpgpCard::try_from(
//         &Locator::try_from(&big_yubi_uri)?
//     )?;
//     println!("Pubkey: {}", big_yubi.pubkey());

//     Ok(())
// }

fn read_header() -> Result<u32, io::Error> {
    let header = io::stdin().read_u32::<NativeEndian>()?;
    eprintln!("READ HEADER");
    Ok(header)
}

#[derive(Debug, Error)]
pub enum ReadRequestError {
    #[error("end of input reached")]
    EndOfInput,
    #[error(transparent)]
    IoError(#[from] io::Error),
    #[error(transparent)]
    TryFromIntError(#[from] TryFromIntError),
    #[error(transparent)]
    SerdeError(#[from] serde_json::Error),
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
        let response = request.handle()?;
        write_response(&response)?;
        eprintln!("END CMD");
    }
}
