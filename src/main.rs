use serde_json::Value;
use mailgun_rs::{Mailgun, EmailAddress, Message};
use dotenv::dotenv;

fn main() {

    println!("A simple scraping bot to check for updates in TTD (Tirumala Tirupati Devasthanams).\nThank you for using!");
    dotenv().ok();

    let mut email_addr = std::env::var("EMAIL_ADDRESS").expect("DEFAULT EMAIL ADDRESS MUST BE SET").to_owned();
    if let Some(arg1) = std::env::args().nth(1){
        println!("The email will be sent to {}",arg1);
        email_addr = arg1;
    }
    let mut language = match std::env::var("LANGUAGE") {
        Ok(language) => language,
        Err(_error) => "English".to_owned(),
    };
    if let Some(arg2) = std::env::args().nth(2){
        println!("The language for scraping {}",arg2);
        language = arg2;
    }
    let path= match std::env::var("FILEPATH") {
        Ok(path) =>path,
        Err(_error) => "./".to_owned(),
    };
    let domain = std::env::var("MAILGUN_DOMAIN").expect("Enter domain");
    let key = std::env::var("MAILGUN_API_KEY").expect("Enter domain");



    let filename = "updates.json";
    let filepath=&(path.to_owned()+filename);
    let agent = ureq::agent();
    let response = agent
    .get("https://tirupatibalaji.ap.gov.in/content/getLatestUpdates.json")
    .call()
    .expect("Invalid http response");

    let response_text = response
    .into_string()
    .expect("Failed to get response text");

    
    let flag:bool;
    if std::path::Path::new(filepath).exists(){
        if response_text == std::fs::read_to_string(&filepath).unwrap(){
            println!("No changes detected");
            flag=false;
        }
        else{
            flag=true;
        }

    }
    else{
        flag=true;
    }

    let language_str = language.as_str();
    if flag {
    std::fs::write(filepath, &response_text).expect("Unable to write file");

            let res_text:String= response_text.chars().skip(1).collect();
            
        
            let latest_updates = serde_json::from_str::<Value>(&res_text.replace("<img src='content/img/new.gif'>", r#""#).replace("<b>", "").replace("</b>", "")).expect("Failed to parse JSON");
            
        
            let eng_updates_elements = latest_updates[language_str]["latestUpdates"].as_array().unwrap().len();
        
            let mut indiv_result ="".to_owned();
            let mut final_result_vec:Vec<String>= Vec::new();
            
            for index in 0..eng_updates_elements{
                if let Value::String(message) =  &latest_updates[language_str]["latestUpdates"][index]["message"] { 
                    
                    if let Value::String(link_url) = &latest_updates[language_str]["latestUpdates"][index]["linkURL"] {
                        
                        if !link_url.starts_with("#") && !link_url.starts_with("/") {
                        if link_url==""{
                            indiv_result=message.to_owned();
                        }
                        else{
                        indiv_result=message.to_owned()+" => "+link_url;
                        }
                        
                    }
                        else{
                            indiv_result=message.to_owned()+" => https://tirupatibalaji.ap.gov.in/"+link_url;
                            
                        }
                    
                }
                }

                final_result_vec.push(indiv_result.clone()+"\n");
            }
            final_result_vec.dedup();
            let final_result = "<ol><li>".to_owned()+&final_result_vec.join("</li><br><br><li>")+"</ol>";

             

            let recipient = email_addr;
            let recipient = EmailAddress::address(&recipient);
            let message = Message {
            to: vec![recipient],
            subject: String::from("TTD Latest Updates"),
            html: String::from(r#"<h1>Greetings from TTD Updates Bot!</h1>
            Tirumala Tirupati Devasthanams' latest updates on the website have been updated. These are the updates:<br>
            "#.to_owned()+&final_result.to_owned()),
            ..Default::default()
            };
            let client = Mailgun{api_key: String::from(key), domain: String::from(domain), message: message};
            let sender = EmailAddress::name_address("no-reply", "no-reply@TTDUpdatesBot.com");
    
            match client.send(&sender) {
                Ok(_) => {
                  println!("Successfully sent email");
                }
                Err(err) => {
                  println!("{}", err.to_string());
                }
              }


        }
    



    
}
