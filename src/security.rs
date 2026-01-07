use super::*;

pub fn path_traversal_check(path: &str) -> bool {
    if path.contains("../") || path.contains("..\\") || path.contains("..") || path.contains("%"){
        return true;
    }
    false
}



pub fn decode_and_check_path(buffer:Request) -> Result<String, ()> { //can it get better???
    println!("\n\nDecoding and checking path: {}\n\n", 
        String::from_utf8_lossy(&buffer.header.clone()[..]), 
    ); //decode here the string and then parse it 

    // println!("the post body: {:?}", String::from_utf8_lossy(&buffer.body.clone().unwrap()[..]));

    if path_traversal_check(&String::from_utf8_lossy(&buffer.header.clone()[..])) {
        return Err(());
    } else {
        return Ok(String::from("placeholder"));
    }

    // let decoded_path = percent_decode_str(path).decode_utf8_lossy();
    // if path_traversal_check(&decoded_path.clone()) {
    //     return Err(false);
    // } 
    // Ok(decoded_path.into_owned())
    
}