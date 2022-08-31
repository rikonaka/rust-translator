use clap::Parser;
#[cfg(target_os = "windows")]
use cli_clipboard;
use colored::Colorize;
use reqwest;
use serde_json;
#[cfg(target_os = "linux")]
use std::process::Command;
use std::time::SystemTime;
use std::{thread, time::Duration};

/// Simple program to translate text
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// source language
    #[clap(
        short,
        long,
        value_parser,
        default_value = "english",
        default_missing_value = "english"
    )]
    sl: String,

    /// target translation language
    #[clap(
        short,
        long,
        value_parser,
        default_value = "chinese",
        default_missing_value = "chinese"
    )]
    tl: String,
    /// fast mode or slow mode
    #[clap(
        short,
        long,
        value_parser,
        default_value = "slow",
        default_missing_value = "slow"
    )]
    mode: String,
}

#[tokio::main]
async fn google_translate_longstring(
    sl: &str,
    tl: &str,
    translate_string: &str,
) -> Result<Vec<Vec<String>>, Box<dyn std::error::Error>> {
    let fliter_char = |x: &str| -> String { x.replace("al.", "al") };
    let max_loop = 100;
    let translate_url = format!(
        "https://translate.googleapis.com/translate_a/single?client=gtx&sl={}&tl={}&dt=t&q={}",
        sl,
        tl,
        fliter_char(translate_string)
    );
    let request_result = reqwest::get(translate_url)
        .await?
        .json::<serde_json::Value>()
        .await?;

    // println!("{:#?}", request_result);
    // [[["翻译","translate",null,null,10]],null,"en",null,null,null,null,[]]
    let mut i = 0;
    let mut result_vec: Vec<Vec<String>> = Vec::new();
    loop {
        let result_string_0 = format!("{}", request_result[0][i][0]);
        let result_string_1 = format!("{}", request_result[0][i][1]);
        match result_string_0.as_str() {
            "null" => break,
            _ => {
                let string_0 = result_string_0.replace("\"", "");
                let string_1 = result_string_1.replace("\"", "");
                if string_0.len() == 1 && string_0 == "." {
                    // there is no possible for length of result is 1
                } else {
                    let mut tmp_vec: Vec<String> = Vec::new();
                    tmp_vec.push(string_0);
                    tmp_vec.push(string_1);
                    result_vec.push(tmp_vec);
                }
            }
        }
        i += 1;
        if i > max_loop {
            break;
        }
    }
    Ok(result_vec)
}

#[tokio::main]
async fn google_translate_shortword(
    sl: &str,
    tl: &str,
    translate_string: &str,
) -> Result<Vec<Vec<String>>, Box<dyn std::error::Error>> {
    let fliter_char = |x: &str| -> String {
        x.replace(".", "")
            .replace(",", "")
            .replace("?", "")
            .replace("!", "")
            .replace(":", "")
            .replace("\"", "")
            .replace("(", "")
            .replace(")", "")
            .replace("<", "")
            .replace(">", "")
            // UTF-8 char
            .replace("“", "")
            .replace("”", "")
            .replace("。", "")
            .replace("，", "")
            .replace("：", "")
            .replace("（", "")
            .replace("）", "")
            .replace("《", "")
            .replace("》", "")
    };
    let translate_url = format!(
        "https://translate.googleapis.com/translate_a/single?client=gtx&sl={}&tl={}&dj=1&dt=t&dt=bd&dt=qc&dt=rm&dt=ex&dt=at&dt=ss&dt=rw&dt=ld&q={}&button&tk=233819.233819",
        sl, tl, fliter_char(translate_string)
    );
    let request_result = reqwest::get(translate_url)
        .await?
        .json::<serde_json::Value>()
        .await?;

    // println!("{:#?}", request_result);
    // {"sentences":[{"trans":"这","orig":"The","backend":10},{"translit":"Zhè"}],"src":"en","alternative_translations":[{"src_phrase":"The","alternative":[{"word_postproc":"这","score":1000,"has_preceding_space":true,"attach_to_next_token":false,"backends":[10]},{"word_postproc":"该","score":0,"has_preceding_space":true,"attach_to_next_token":false,"backends":[3],"backend_infos":[{"backend":3}]},{"word_postproc":"那个","score":0,"has_preceding_space":true,"attach_to_next_token":false,"backends":[8]}],"srcunicodeoffsets":[{"begin":0,"end":3}],"raw_src_segment":"The","start_pos":0,"end_pos":0}],"confidence":1.0,"spell":{},"ld_result":{"srclangs":["en"],"srclangs_confidences":[1.0],"extended_srclangs":["en"]}}
    let mut result_vec: Vec<Vec<String>> = Vec::new();
    let mut tmp_vec: Vec<String> = Vec::new();
    let trans_string = format!(
        "{}",
        request_result.get("sentences").unwrap()[0]
            .get("trans")
            .unwrap()
    );
    let orig_string = format!(
        "{}",
        request_result.get("sentences").unwrap()[0]
            .get("orig")
            .unwrap()
    );
    tmp_vec.push(trans_string.replace("\"", ""));
    tmp_vec.push(orig_string.replace("\"", ""));
    let alter_vec = request_result.get("alternative_translations").unwrap()[0]
        .get("alternative")
        .unwrap();
    let mut i = 0;
    loop {
        let av = match alter_vec[i].get("word_postproc") {
            Some(a) => a,
            _ => break,
        };
        let alter_string = format!("{}", av);
        // jump the first word
        if i != 0 {
            tmp_vec.push(alter_string.replace("\"", ""));
        }
        i += 1;
    }
    result_vec.push(tmp_vec);
    Ok(result_vec)
}

fn contains_symbol(input_string: &str) -> bool {
    input_string.contains(" ")
}

fn translate(sl: &str, tl: &str, translate_string: &str, index: usize) {
    let start = SystemTime::now();
    let result_vec = match contains_symbol(translate_string) {
        true => match google_translate_longstring(sl, tl, translate_string) {
            Ok(r) => r,
            Err(e) => {
                println!("translate failed: {}", e);
                vec![]
            }
        },
        false => match google_translate_shortword(sl, tl, translate_string) {
            Ok(r) => r,
            Err(e) => {
                println!("translate failed: {}", e);
                vec![]
            }
        },
    };
    // println!("{:?}", result_vec);
    let end = SystemTime::now();
    let duration = end.duration_since(start).unwrap();
    let translate_title = format!("Translate[{}] => {:.3}s", index, duration.as_secs_f64());
    if result_vec.len() > 0 {
        if cfg!(target_os = "linux") {
            println!(">>> {}", translate_title.bold().red());
            for v in result_vec {
                println!("[{}] {}", "O".bright_blue().bold(), v[1]);
                println!("[{}] {}", "T".green().bold(), v[0]);
                if v.len() > 2 {
                    for i in 2..v.len() {
                        println!("[{}] {}", "A".cyan().bold(), v[i]);
                    }
                }
            }
        } else if cfg!(target_os = "windows") {
            println!(">>> {}", translate_title);
            for v in result_vec {
                println!("[{}] {}", "O", v[1]);
                println!("[{}] {}", "T", v[0]);
                if v.len() > 2 {
                    for i in 2..v.len() {
                        println!("[{}] {}", "A", v[i]);
                    }
                }
            }
        }
    }
}

#[cfg(target_os = "windows")]
fn get_clipboard_text_windows() -> Option<String> {
    let output = match cli_clipboard::get_contents() {
        Ok(o) => o.trim().to_string(),
        Err(_) => String::from(""),
    };

    if output.len() > 0 {
        return Some(output);
    }
    None
}

#[cfg(target_os = "linux")]
fn get_select_text_linux() -> Option<String> {
    // return "" at least
    let output = Command::new("xsel")
        .output()
        .expect("Please install xsel first!");
    let output = String::from_utf8_lossy(&output.stdout).to_string();
    if output.len() > 0 {
        return Some(output);
    }
    None
}

fn get_text() -> Option<String> {
    let filter = |x: &str| -> String {
        let x = match x.strip_prefix(".") {
            Some(x) => x,
            _ => x,
        };
        let x = match x.strip_prefix(",") {
            Some(x) => x,
            _ => x,
        };
        x.replace("-\n", "")
            .replace("%", "%25")
            .replace("&", "%26")
            .replace("\n", " ")
            .trim()
            .to_string()
    };
    if cfg!(target_os = "linux") {
        #[cfg(target_os = "linux")]
        match get_select_text_linux() {
            Some(t) => return Some(filter(&t)),
            _ => return None,
        }
    } else if cfg!(target_os = "windows") {
        #[cfg(target_os = "windows")]
        match get_clipboard_text_windows() {
            Some(t) => return Some(filter(&t)),
            _ => return None,
        }
    }
    None
}

fn convert_args<'a>(source_language: &'a str, target_language: &'a str) -> (&'a str, &'a str) {
    let convert_language = |x: &str| -> &str {
        let result = match x {
            "english" => "en",
            "chinese" => "zh-CN",
            "japanese" => "ja",
            "french" => "fr",
            "german" => "de",
            _ => "en",
        };
        result
    };
    let sl_result = convert_language(source_language);
    let tl_result = convert_language(target_language);
    (sl_result, tl_result)
}

fn main() {
    let mut index: usize = 1;
    let args = Args::parse();
    let (sl, tl) = convert_args(&args.sl, &args.tl);
    let sleep_time = match args.mode.as_str() {
        "fast" => Duration::from_secs_f32(0.3),
        _ => Duration::from_secs(1),
    };
    if cfg!(target_os = "linux") {
        #[cfg(target_os = "linux")]
        loop {
            thread::sleep(sleep_time);
            let selected_text = match get_text() {
                Some(t) => t,
                _ => continue,
            };
            if selected_text.trim().len() > 0 {
                // println!("{}", &selected_text);
                // let test_string = String::from("translate");
                translate(&sl, &tl, &selected_text, index);
                index += 1;
            }
        }
    } else if cfg!(target_os = "windows") {
        #[cfg(target_os = "windows")]
        let mut last_clipboard_text = String::from("");
        #[cfg(target_os = "windows")]
        loop {
            thread::sleep(sleep_time);
            let clipboard_text = match get_text() {
                Some(t) => t,
                _ => continue,
            };
            if clipboard_text != last_clipboard_text {
                last_clipboard_text = clipboard_text.clone();
                if clipboard_text.trim().len() > 0 {
                    translate(&sl, &tl, &clipboard_text, index);
                    index += 1;
                }
            }
        }
    } else {
        panic!("Not support running at the other system!");
    }
}
