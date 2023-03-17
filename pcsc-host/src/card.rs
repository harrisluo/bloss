use {
    openpgp_card::{
        algorithm::{Algo, Curve},
        card_do::{ApplicationIdentifier, UIF},
        crypto_data::{EccType, PublicKeyMaterial, Hash},
        OpenPgp,
        SmartcardError,
        OpenPgpTransaction
    },
    openpgp_card_pcsc::PcscBackend,
    pinentry::PassphraseInput,
    secrecy::ExposeSecret,
    serde::{Serialize, Deserialize},
    solana_sdk::{
        pubkey::Pubkey,
        signature::{Signature, Signer, SignerError},
    },
    std::{
        cell::RefCell,
        error,
    },
    thiserror::Error,
    uriparse::{URIReference, URIReferenceError},
};

pub struct OpenpgpCard {
    pgp: RefCell<OpenPgp>,
}

impl From<PcscBackend> for OpenpgpCard {
    fn from(backend: PcscBackend) -> Self {
        let pgp = OpenPgp::new::<PcscBackend>(backend.into());
        Self { pgp: RefCell::new(pgp) }
    }
}

impl TryFrom<&String> for OpenpgpCard {
    type Error = openpgp_card::Error;

    fn try_from(ident: &String) -> Result<Self, Self::Error> {
        if ident.len() != 32 {
            return Err(openpgp_card::Error::ParseError("OpenPGP AID must be 32-digit hex string".to_string()));
        }
        let mut ident_bytes = Vec::<u8>::new();
        for i in (0..ident.len()).step_by(2) {
            ident_bytes.push(u8::from_str_radix(&ident[i..i + 2], 16).map_err(
                |_| openpgp_card::Error::ParseError("non-hex character found in identifier".to_string())
            )?);
        }
        let aid = ApplicationIdentifier::try_from(ident_bytes.as_slice())?;
        let backend = PcscBackend::open_by_ident(aid.ident().as_str(), None)?;
        Ok(backend.into())
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct OpenpgpCardInfo {
    pub manufacturer: String,
    pub serial_number: String,
    pub aid: String,
    pub signing_algo: String,
    pub pubkey_bytes: Vec<u8>,
}

impl OpenpgpCard {
    pub fn get_info(&self) -> Result<OpenpgpCardInfo, openpgp_card::Error> {
        let mut pgp_mut = self.pgp.borrow_mut();
        let opt = &mut pgp_mut.transaction()?;
        let ard = opt.application_related_data()?;
        let aid = ard.application_id()?;

        // TODO: Handle case where there is no signing key.
        let pk_material = opt.public_key(openpgp_card::KeyType::Signing)?;
        let pubkey = get_pubkey_from_pk_material(pk_material)?;

        Ok(OpenpgpCardInfo {
            manufacturer: aid.manufacturer_name().to_string(),
            serial_number: format!("{:08x}", aid.serial()),
            aid: aid.to_string().replace(" ", ""),
            signing_algo: "ed25519".to_string(),
            pubkey_bytes: pubkey.to_bytes().to_vec(),
        })
    }

    pub fn sign_message(&self, message: &[u8], pin: &[u8], touch_confirm_callback: fn()) -> Result<Vec<u8>, openpgp_card::Error> {
        let mut pgp_mut = self.pgp.borrow_mut();
        let opt = &mut pgp_mut.transaction()?;

        opt.verify_pw1_sign(pin)?;

        // Await user touch confirmation if and only if
        //   * Card supports touch confirmation, and
        //   * Touch policy set anything other than "off".
        let ard = opt.application_related_data()?;
        if let Some(signing_uif) = ard.uif_pso_cds()? {
            if signing_uif.touch_policy().touch_required() {
                touch_confirm_callback();
            }
        }

        // Delegate message signing to card
        let hash = Hash::EdDSA(message);
        let sig = opt.signature_for_hash(hash)?;

        Ok(sig)
    }
}

fn get_pubkey_from_pk_material(pk_material: PublicKeyMaterial) -> Result<Pubkey, openpgp_card::Error> {
    let pk_bytes: [u8; 32] = match pk_material {
        PublicKeyMaterial::E(pk) => match pk.algo() {
            Algo::Ecc(ecc_attrs) => {
                if ecc_attrs.ecc_type() != EccType::EdDSA || ecc_attrs.curve() != Curve::Ed25519 {
                    return Err(openpgp_card::Error::UnsupportedAlgo(
                        format!("expected Ed25519 key, got {:?}", ecc_attrs.curve())
                    ));
                }
                pk.data().try_into().map_err(
                    |e| openpgp_card::Error::ParseError(format!("key on card is malformed: {}", e))
                )?
            },
            _ => return Err(openpgp_card::Error::UnsupportedAlgo("expected ECC key, got RSA".to_string())),
        }
        _ => return Err(openpgp_card::Error::UnsupportedAlgo("expected ECC key, got RSA".to_string())),
    };
    Ok(Pubkey::from(pk_bytes))
}
