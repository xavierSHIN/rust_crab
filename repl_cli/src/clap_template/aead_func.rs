use chacha20poly1305::{
    aead::{Aead, KeyInit},
    ChaCha20Poly1305
};
use sha2::{Sha256, Digest};
use base64::{
    engine::general_purpose::{STANDARD, URL_SAFE_NO_PAD},
    Engine as _,
};
//use rand_core::RngCore;
//use std::convert::TryInto;
use std::{fs::File, io::Read};
use anyhow::{Result, anyhow, Error};


// 示例函数，用于生成 plaintext
//fn generate_plaintext(osrng: &mut OsRng) -> Vec<u8> {
//    let mut plaintext = Vec::new();
//    osrng.fill_bytes(&mut plaintext);
//    plaintext
//}
// tod:: incorporaTE THE chachaerror into anyhow error? done


pub fn get_reader(input: &str) -> Result<Box<dyn Read>> {
    // TODO:: add func that could judge the utf encode?
    let reader: Box<dyn Read> = if input == "-" {
        Box::new(std::io::stdin())
    } else {
        Box::new(File::open(input)?)
    };
    Ok(reader)
}

fn convert_u128_to_u8_array(num: u128) -> [u8; 12] {
    //12345678 大位数在前，小位数在后
    //0x11223344 一样
    //[44,33,22,11] little endian = u128 表达方式和阿拉伯数字相反， 即LE
    //[11,22,33,44] big endian 表达方式和阿拉伯数字一样， 即BE
    let bytes: [u8; 16] = num.to_le_bytes();
    //println!("{:?}", bytes);
    let mut result: [u8; 12] = [0; 12];
    result.copy_from_slice(&bytes[..12]);
    result
}



// Function to convert a byte slice to a hex string
//fn bytes_to_hex(bytes: &[u8]) -> String {
//    bytes.iter().map(|b| format!("{:02x}", b)).collect()
//}

pub fn generate_key_sha256(key_input: &str) -> [u8; 32] {
    // Generate a random salt (e.g., 16 bytes)
    // 但是这里用一个固定&str
    //let mut salt = [0_u8; 16];
    //rand_core::OsRng.fill_bytes(&mut salt);
    let salt = "salt is salty";
    // Create a new Sha256 hasher instance
    let mut hasher = Sha256::new();
    // Update the hasher with the salt
    hasher.update(salt.as_bytes());
    // Update the hasher with the input data
    hasher.update(key_input.as_bytes());
    // Finalize the hash computation and obtain the result
    let hash_result = hasher.finalize();
    // Print the salt and hash in hexadecimal format
    hash_result.into()
    //println!("Salt: {:#?}", &salt);
    //println!("SHA-256 hash: {:#?}, len:{}", &hash_result, hash_result.len());
}


pub fn process_aead_encode(
    input:&str  
    , key: &str
    , nonce: u128
    , format: &str
    ) -> Result<String, Error> {
    let key_1 = generate_key_sha256(key);
    let nonce_1 = convert_u128_to_u8_array(nonce);
    //rand_core::OsRng.fill_bytes(&mut key_1);
    //rand_core::OsRng.fill_bytes(&mut nonce_1);
    let cipher = ChaCha20Poly1305::new(&key_1.into());
    // cipher algorithm completed
    
    let mut buf = Vec::new();
    let mut rdr = get_reader(input).unwrap();
    rdr.read_to_end(&mut buf)
        .map_err(|e| anyhow!("read to end error: {:?}", e))?;
    let ciphertext = cipher.encrypt(&nonce_1.into(), &buf[..])
                            .map_err(|e| anyhow!("encrypt error: {:?}", e))?;
    
    println!("\n");
    //let mut buf = ciphertext;
    //reader.read_to_end(&mut buf)?;
    let encoded = match format.to_lowercase().as_str() {
        "standard" => STANDARD.encode(&ciphertext),
        "urlsafe" => URL_SAFE_NO_PAD.encode(&ciphertext),
        _ => panic!("Invalid format"),
    };
    Ok(encoded)
}

pub fn process_aead_decode(
    encoded_input: &str
    , key:&str
    , nonce: u128
    , format: &str
    ) -> Result<String, Error> {

    let key_1 = generate_key_sha256(key);
    let nonce_1 = convert_u128_to_u8_array(nonce);
    let cipher = ChaCha20Poly1305::new(&key_1.into());
    
    let mut buf_2 = String::new();
    let mut rdr = get_reader(encoded_input).unwrap();
    rdr.read_to_string(&mut buf_2)
                .map_err(|e| anyhow!("read to str error: {:?}", e))?;
    let buf_2 = buf_2.trim();

    let decoded = match format.to_lowercase().as_str() {
        "standard" => STANDARD.decode(buf_2),
        "urlsafe" => URL_SAFE_NO_PAD.decode(buf_2),
        _ => panic!("Invalid format"),
    };
    //println!("{:#?}", decoded);
    // 这里decode的是cipphertext, 超过128了
    let plaintext = cipher.decrypt(&nonce_1.into(), decoded.unwrap().as_ref())
                                    .map_err(|e| anyhow!("decrypt error: {:?}", e))?;
    //println!("{:#?}, ", plaintext);
    println!("\n");
    Ok(String::from_utf8(plaintext)?)
}

//pub fn test_chahca() -> Result<(), Error> {
//    let mut key_1 = [0u8; 32];
//    let mut nonce_1 = [0u8; 12];
//    rand_core::OsRng.fill_bytes(&mut key_1);
//    rand_core::OsRng.fill_bytes(&mut nonce_1);
//    println!("nonce_1:{:?}", nonce_1);
//    //let random_u64 = OsRng.next_u64();
//    //let key = ChaCha20Poly1305::generate_key(&mut OsRng);
//    let cipher = ChaCha20Poly1305::new(&key_1.into());
//    //let nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng); // 96-bits; unique per message
//    
//    //print!("key: {:?}, nonce: {:?},", key.len(), nonce.len());
//    //key: 32, nonce: 12,
//    //ptext = generate_plaintext(osrng);
//    //println!("plaintext: {:?}", Sting::from_bytes(ptext));
//    let ciphertext = cipher.encrypt(&nonce_1.into(), b"plain_x".as_ref())
//                            .map_err(|e| anyhow!("encrypt error: {:?}", e))?;
//
//    let plaintext = cipher.decrypt(&nonce_1.into(), ciphertext.as_ref())
//                            .map_err(|e| anyhow!("decrypt error: {:?}", e))?;
//    //let lazi = plaintext.clone();
//    let o_put = String::from_utf8(plaintext)?;
//    println!("{:#?}", o_put);
//    Ok(())
//}