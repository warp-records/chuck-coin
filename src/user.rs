use k256::{
    ecdsa::{SigningKey, VerifyingKey},
    elliptic_curve::{sec1::*, PublicKey},
    Secp256k1,
};

//for debugging purposes
pub struct User {
    pub signing: SigningKey,
    pub verifying: VerifyingKey,
}

impl User {
    pub fn random() -> Self {
        use rand_core::OsRng;

        let signing_key = SigningKey::random(&mut OsRng);
        let verifying_key = VerifyingKey::from(signing_key.clone());
        Self {
            signing: signing_key,
            verifying: verifying_key,
        }
    }

    pub fn from_priv(priv_key: &str) -> User {
        let keypair = keys_from_str(priv_key);
        User {
            signing: keypair.0,
            verifying: keypair.1,
        }
    }

    pub fn try_from_priv(priv_key: &str) -> Result<User, ()> {
        let bytes = hex::decode(priv_key).map_err(|_| ())?;
        let signing_key = SigningKey::from_bytes(bytes.as_slice().into()).map_err(|_| ())?;
        let verifying_key = VerifyingKey::from(signing_key.clone());

        Ok({
            User {
                signing: signing_key,
                verifying: verifying_key,
            }
        })
    }
}

pub fn try_public_from_str(public_key: &str) -> Result<PublicKey<Secp256k1>, ()> {
    let encoded_point =
        EncodedPoint::<Secp256k1>::from_bytes(hex::decode(public_key).unwrap().as_slice()).map_err(|_| ())?;

    Ok(VerifyingKey::from_encoded_point(&encoded_point).unwrap().into())
}

pub fn keys_from_str(priv_key: &str) -> (SigningKey, VerifyingKey) {
    let signing_key =
        SigningKey::from_bytes(hex::decode(priv_key).unwrap().as_slice().into()).unwrap();
    let verifying_key = VerifyingKey::from(signing_key.clone());

    //println!("Private key: {} ", hex::encode_upper(signing_key.to_bytes()));
    //println!("Public key: {}", hex::encode_upper(verifying_key.to_encoded_point(false)));

    (signing_key, verifying_key)
}

pub fn pk_from_encoded_str(public_key: &str) -> PublicKey<Secp256k1> {
    let encoded_point =
        EncodedPoint::<Secp256k1>::from_bytes(hex::decode(public_key).unwrap().as_slice()).unwrap();
    PublicKey::<Secp256k1>::from_encoded_point(&encoded_point).unwrap()
}

pub fn create_keypair() -> (SigningKey, VerifyingKey) {
    use rand_core::OsRng;

    let signing_key = SigningKey::random(&mut OsRng);
    let verifying_key = VerifyingKey::from(signing_key.clone());
    //println!("Private key: {} ", hex::encode_upper(signing_key.to_bytes()));
    //println!("Public key: {}", hex::encode_upper(verifying_key.to_encoded_point(false).as_bytes()));
    (signing_key, verifying_key)
}

pub fn vk_from_encoded_str(public_key: &str) -> VerifyingKey {
    let encoded_point =
        EncodedPoint::<Secp256k1>::from_bytes(hex::decode(public_key).unwrap().as_slice()).unwrap();
    VerifyingKey::from_encoded_point(&encoded_point).unwrap()
}
