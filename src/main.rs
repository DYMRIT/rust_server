use std::hash::Hash;
use std::process::Command;
use serde_json::Value;
use std::thread;
use std::time::Duration;




fn split_str(string: &str, sep: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut current_string = String::new();

    for word in string.split(sep) {
        current_string.push_str(word.trim());
        result.push(current_string.clone());
        current_string.clear();
    }

    result
}


fn req(method: String, http_method: &str, data: String, offset: u64) -> Vec<String> {
    if http_method == "GET" {
        let url = format!("{}/bot{}/{}?offset={}", "https://api.telegram.org", TOKEN, method, offset);

        let result = Command::new("curl")
            .arg("-i")
            .arg(&url)
            .output();

        match result {
            Ok(res) => {
                let mut result = vec!["Success".to_string()];
                result.extend(split_str(&String::from_utf8_lossy(&res.stdout), "\r\n\r\n"));
                return result;
            },
            Err(e) => {
                let mut result = vec!["Failed".to_string()];
                result.extend(vec![e.to_string()]);
                return result;
            }
        }
    } else {
        let url = format!("{}/bot{}/{}", "https://api.telegram.org", TOKEN, method);

        let result = Command::new("curl")
            .args(&[
                "-H",
                "Content-Type: application/json",
                "-i",
                "-d",
                &data,
                "-X",
                "POST",
                &url
            ])
            .output();

        match result {
            Ok(res) => {
                let mut result = vec!["Success".to_string()];
                result.extend(split_str(&String::from_utf8_lossy(&res.stdout), "\r\n\r\n"));
                return result;
            },
            Err(e) => {
                let mut result = vec!["Failed".to_string()];
                result.extend(vec![e.to_string()]);
                return result;
            }
        }
    }
}


fn main() {

    let mut offset = 0; // getUpdates is empty ? 0 : max(update_id) + 1

    //first request to telegram bot api
    //get max update_id that use it in next iteration
    let mut arr_updates_id: Vec<u64> = Vec::new();
    let res = req("getUpdates".to_string(), "GET", "".to_string(), 693852679);
        if res[0] == "Success" {
            let body: Value = serde_json::from_str(&res[2])
                .expect("Failed parse to JSON");

            if body["ok"].to_string() == "true" {
                let arr = body["result"].as_array().unwrap();
                for line in arr {
                   arr_updates_id.push(line["update_id"].as_u64().unwrap());
                }
            }

            if arr_updates_id.iter().len() != 0 {
                offset = arr_updates_id.iter().max().unwrap() + 1;
            }
        }


    loop {
        let res = req("getUpdates".to_string(), "GET", "".to_string(), offset);
        if res[0] == "Success" {
            let body: Value = serde_json::from_str(&res[2])
                .expect("Failed parse to JSON");

            if body["ok"].to_string() == "true" {
                let arr = body["result"].as_array().unwrap();
                for line in arr {

                    let chat_id = line["message"]["chat"]["id"].as_u64().unwrap();
                    let text = line["message"]["text"].as_str().unwrap();
                    let update_id = line["update_id"].as_u64().unwrap();

                    println!("update_id: {:?}, chat_id: {:?}, text: {:?}", update_id, chat_id, text);

                    //update update_id
                    if update_id >= offset {
                        offset = update_id+1;
                    }

                    //create handler for message
                    let data = handle(text, chat_id, line);

                    if data != "".to_string() {
                        let res = req("sendMessage".to_string(),
                                      "POST",
                                      data,
                                      0);
                    }
                }
            }
        }



        //timesllep on 3 sec
        thread::sleep(Duration::from_secs(2));
    }
}


fn handle(text: &str, chat_id: u64, line: &Value) -> String {
    if text.to_string() == "/start".to_string() {
        let response_text: String = "hello".to_string();
        let data = format!(r#"
                    {{
                        "chat_id": {},
                        "text": "{}",
                    }}"#, chat_id, response_text);
        data
    }
    else {
        "".to_string()
    }
}