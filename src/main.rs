use std::hash::Hash;
use std::collections::HashMap;
use std::process::Command;
use serde_json::Value;
use std::thread;
use std::time::Duration;


// ADD your token as variable TOKEN and code should work!


//split method
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


// GET and POST request to telegram api
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
    let mut pattern: HashMap<u64, u64> = HashMap::new();

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

    //long polling
    loop {
        let res = req("getUpdates".to_string(), "GET", "".to_string(), offset);
        if res[0] == "Success" {
            let body: Value = serde_json::from_str(&res[2])
                .expect("Failed parse to JSON");

            if body["ok"].to_string() == "true" {
                let arr = body["result"].as_array().unwrap();
                for line in arr {
                    if !line["message"].is_null() {


                        //work with message response
                        let chat_id = line["message"]["chat"]["id"].as_u64().unwrap();
                        let text = line["message"]["text"].as_str().unwrap();
                        let update_id = line["update_id"].as_u64().unwrap();

                        println!("type: message, update_id: {:?}, chat_id: {:?}, text: {:?}", update_id, chat_id, text);

                        //update update_id
                        if update_id >= offset {
                            offset = update_id + 1;
                        }

                        //create handler for message
                        let data = handle_message(text, chat_id, line, &mut pattern);

                        if data != "".to_string() {
                            let res = req("sendMessage".to_string(),
                                          "POST",
                                          data,
                                          0);
                        }

                    } else if !line["callback_query"].is_null() {


                        //work with callback response
                        let chat_id = line["callback_query"]["message"]["chat"]["id"].as_u64().unwrap();
                        let callback_data = line["callback_query"]["data"].as_str().unwrap();
                        let update_id = line["update_id"].as_u64().unwrap();
                        let text_message = line["callback_query"]["message"]["text"].as_str().unwrap();
                        let callback_id = line["callback_query"]["id"].as_str().unwrap();

                        println!("type: callback, update_id: {:?}, chat_id: {:?}, callback_data: {:?}", update_id, chat_id, callback_data);
                        println!("{:?}\n", line);

                        if update_id >= offset {
                            offset = update_id + 1;
                        }

                        //create handler for callback
                        let (data, mode) = handle_callback(callback_data, chat_id, callback_id,line);

                        //change mode for uniq chat_id
                        if mode != 0 {
                            pattern.insert(chat_id, mode);
                        }

                        if data != "".to_string() {
                            let res = req("sendMessage".to_string(),
                                          "POST",
                                          data,
                                          0);
                        }
                    }
                }
            }
        }



        //timesllep on 3 sec
        thread::sleep(Duration::from_secs(2));
    }
}


fn handle_callback(callback: &str, chat_id: u64, callback_id: &str, line: &Value) -> (String, u64) {
    if callback.to_string() == "btn_start_next".to_string() {

        //move that disabled button
        let callback_data = format!(r#"{{
                "callback_query_id": "{}"
                }}"#, callback_id);
        let res = req("answerCallbackQuery".to_string(),
                                          "POST",
                                          callback_data,
                                          0);


        //response to callback
        let response_text: String = "В следующем сообщении введи номер своей карты. \
                                     \nДля теста введите число длиной в 3 символа".to_string();


        let data = format!(r#"
                    {{
                        "chat_id": {},
                        "text": "{}",
                        "disable_notification": true
                    }}"#, chat_id, response_text);
        (data, 1)
    } else {
        ("".to_string(), 0)
    }
}


//create handler for text
fn handle_message(text: &str, chat_id: u64, line: &Value, pattern: &mut HashMap<u64, u64>) -> String {
    if text.to_string() == "/start".to_string() {

        let response_text: String = "Привет, с помощью данного бота можно пополнить различные карты \
                                     питербургского метрополитена. \nРаботает по лицензии: та-та-та".to_string();

        let data = format!(r#"
                    {{
                        "chat_id": {},
                        "text": "{}",
                        "reply_markup": {{
                            "inline_keyboard": [[
                                {{
                                    "text": "Продолжить",
                                    "callback_data": "btn_start_next"
                                }}
                            ]]
                            }}
                    }}"#, chat_id, response_text);
        data
    }
    else {
        if let Some(mode) = pattern.get(&chat_id) {
            if *mode == 1 {
                //validate text for request card number
                if text.parse::<u64>().is_ok() && text.chars().count() == 3 {

                    //if validation is okey
                    pattern.insert(chat_id, 2);

                    let response_text: String = "Тип карты: {тут то что я получу от сервера} \
                                                 \nБаланс: {тут то что я получу от сервера} \
                                                 \nДля пополнения счет введите желаемую сумму пополнения".to_string();

                    let data = format!(r#"
                                {{
                                    "chat_id": {},
                                    "text": "{}",
                                    "reply_markup": {{
                                        "inline_keyboard": [[
                                            {{
                                                "text": "Назад",
                                                "callback_data": "btn_start_next"
                                            }}
                                        ]]
                                        }}
                                }}"#, chat_id, response_text);
                    data

                }
                else {

                    //if not next cross validation
                    let response_text: String = "Некорректный номер карты. \
                                                 Примечание: в сообщении должны быть только числа и тра-та-та".to_string();

                    let data = format!(r#"
                                {{
                                    "chat_id": {},
                                    "text": "{}"
                                }}"#, chat_id, response_text);
                    data

                }
            } else if *mode == 2 {

                //validation text for refill
                if text.parse::<u64>().is_ok() {

                    if text.parse::<u64>().unwrap() < 14_000 {
                        //if validation is okey
                        pattern.remove(&chat_id);

                        let response_text: String = "Сумма пополнения. \
                                                     \n{Здесь будет ссылка на пополнение}".to_string();

                        let data = format!(r#"
                                {{
                                    "chat_id": {},
                                    "text": "{}",
                                    "reply_markup": {{
                                        "inline_keyboard": [[
                                            {{
                                                "text": "Назад",
                                                "callback_data": "btn_start_next"
                                            }}
                                        ]]
                                        }}
                                }}"#, chat_id, response_text);
                        data
                    } else {
                        // send message that your message is number, but more than 14_000
                        let response_text: String = "Число должно быть в интервале от 0 до 14_000".to_string();

                        let data = format!(r#"
                                    {{
                                        "chat_id": {},
                                        "text": "{}"
                                    }}"#, chat_id, response_text);
                        data
                    }
                }
                else {

                    //send message that your message is not number or number < 0
                    let response_text: String = "Отправленное сообщение должно быть число в интервале от 0 до 14_000".to_string();

                    let data = format!(r#"
                                {{
                                    "chat_id": {},
                                    "text": "{}"
                                }}"#, chat_id, response_text);
                    data
                }
            }
            else {
                "".to_string()
            }
        } else {
            "".to_string()
        }
    }
}