use std::collections::VecDeque;
use std::fs::{self, create_dir_all, File};
use std::io::{self, Write};
use std::path::Path;
use indexmap::IndexMap;
use serde_json::Value;
use serde_json::to_writer_pretty;

/// 起動時に必要なディレクトリが存在するか確認し、なければ作成する
fn ensure_directories() {
    let input_dir = "./input";
    let output_dir = "./output";

    if !Path::new(input_dir).exists() {
        create_dir_all(input_dir).expect("inputディレクトリの作成に失敗しました。");
        println!("inputディレクトリを作成しました。");
    }
    if !Path::new(output_dir).exists() {
        create_dir_all(output_dir).expect("outputディレクトリの作成に失敗しました。");
        println!("outputディレクトリを作成しました。");
    }
}

/// .langファイルを読み込んで順序を保持するIndexMapに格納する関数
fn load_lang_file(file_path: &str) -> Result<IndexMap<String, String>, String> {
    let contents = fs::read_to_string(file_path).map_err(|_| format!("{} の読み込みに失敗しました。", file_path))?;
    let mut lang_map = IndexMap::new();
    for line in contents.lines() {
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((key, value)) = line.split_once('=') {
            lang_map.insert(key.trim().to_string(), value.trim().to_string());
        }
    }
    Ok(lang_map)
}

/// JSONファイルを読み込んでIndexMapに変換する関数
fn load_json_file(file_path: &str) -> Result<IndexMap<String, String>, String> {
    let file_content = fs::read_to_string(file_path).map_err(|_| format!("{} の読み込みに失敗しました。", file_path))?;
    let json_value: Value = serde_json::from_str(&file_content).map_err(|_| format!("{} のJSON解析に失敗しました。", file_path))?;
    let mut lang_map = IndexMap::new();
    if let Value::Object(map) = json_value {
        for (key, value) in map {
            if let Value::String(val) = value {
                lang_map.insert(key, val);
            }
        }
    }
    Ok(lang_map)
}

/// JSONファイルに整形して出力する関数
fn save_as_pretty_json(output_path: &str, lang_map: &IndexMap<String, String>) -> Result<(), String> {
    if let Some(parent_dir) = Path::new(output_path).parent() {
        create_dir_all(parent_dir).map_err(|_| format!("出力先ディレクトリの作成に失敗しました: {}", output_path))?;
    }
    let file = File::create(output_path).map_err(|_| format!("{} のJSONファイル作成に失敗しました。", output_path))?;
    to_writer_pretty(file, lang_map).map_err(|_| format!("{} へのJSONデータ書き込みに失敗しました。", output_path))?;
    Ok(())
}

/// .langファイルとして保存する関数
fn save_as_lang(output_path: &str, lang_map: &IndexMap<String, String>) -> Result<(), String> {
    if let Some(parent_dir) = Path::new(output_path).parent() {
        create_dir_all(parent_dir).map_err(|_| format!("出力先ディレクトリの作成に失敗しました: {}", output_path))?;
    }
    let mut file = File::create(output_path).map_err(|_| format!("{} の.langファイル作成に失敗しました。", output_path))?;
    for (key, value) in lang_map {
        writeln!(file, "{}={}", key, value).map_err(|_| format!("{} へのデータ書き込みに失敗しました。", output_path))?;
    }
    Ok(())
}

/// 特定の変換を実行する関数
fn process_files(mode: u8) {
    let input_dir = "./input";
    let output_dir = "./output";

    let mut failed_reads = VecDeque::new();    // 読み込み失敗の記録
    let mut failed_writes = VecDeque::new();   // 書き込み失敗の記録

    for entry in fs::read_dir(input_dir).expect("inputディレクトリが存在しません。") {
        if let Ok(entry) = entry {
            let path = entry.path();
            let file_name = path.file_stem().unwrap().to_str().unwrap().to_string();

            if mode == 1 && path.extension().map_or(false, |e| e == "lang") {
                // .lang => JSON
                let input_path = path.to_str().unwrap();
                let output_path = format!("{}/{}.json", output_dir, file_name);
                if let Ok(lang_map) = load_lang_file(input_path) {
                    println!("{} => {}", input_path, output_path);
                    if let Err(e) = save_as_pretty_json(&output_path, &lang_map) {
                        failed_writes.push_back(format!("{}: {}", file_name, e));
                    }
                } else if let Err(e) = load_lang_file(input_path) {
                    failed_reads.push_back(format!("{}: {}", file_name, e))
                }
            } else if mode == 2 && path.extension().map_or(false, |e| e == "json") {
                // JSON => .lang
                let input_path = path.to_str().unwrap();
                let output_path = format!("{}/{}.lang", output_dir, file_name);
                if let Ok(lang_map) = load_json_file(input_path) {
                    println!("{} => {}", input_path, output_path);
                    if let Err(e) = save_as_lang(&output_path, &lang_map) {
                        failed_writes.push_back(format!("{}: {}", file_name, e));
                    }
                } else if let Err(e) = load_json_file(input_path) {
                    failed_reads.push_back(format!("{}: {}", file_name, e))
                }
            } else if mode == 3 {
                // 両方の変換
                if path.extension().map_or(false, |e| e == "lang") {
                    let input_path = path.to_str().unwrap();
                    let output_path = format!("{}/{}.json", output_dir, file_name);
                    match load_lang_file(input_path) {
                        Ok(lang_map) => {
                            println!("{} => {}", input_path, output_path);
                            if let Err(e) = save_as_pretty_json(&output_path, &lang_map) {
                                failed_writes.push_back(format!("{}: {}", file_name, e));
                            }
                        }
                        Err(e) => failed_reads.push_back(format!("{}: {}", file_name, e)),
                    }
                } else if path.extension().map_or(false, |e| e == "json") {
                    let input_path = path.to_str().unwrap();
                    let output_path = format!("{}/{}.lang", output_dir, file_name);
                    match load_json_file(input_path) {
                        Ok(lang_map) => {
                            println!("{} => {}", input_path, output_path);
                            if let Err(e) = save_as_lang(&output_path, &lang_map) {
                                failed_writes.push_back(format!("{}: {}", file_name, e));
                            }
                        }
                        Err(e) => failed_reads.push_back(format!("{}: {}", file_name, e)),
                    }
                }
            }
        }
    }

    // 結果表示
    println!("\n処理完了:");
    if failed_reads.is_empty() && failed_writes.is_empty() {
        println!("すべてのファイルが正常に処理されました。");
    } else {
        if !failed_reads.is_empty() {
            println!("\n読み込みに失敗したファイル:");
            for error in &failed_reads {
                println!("- {}", error);
            }
        }
        if !failed_writes.is_empty() {
            println!("\n出力に失敗したファイル:");
            for error in &failed_writes {
                println!("- {}", error);
            }
        }
    }
}

/// メニュー表示と選択を繰り返す関数
fn prompt_for_mode() -> u8 {
    loop {
        println!("\n1: lang=>json\n2: json=>lang\n3: すべて変換\n0: アプリを終了");
        print!("選択してください: ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("入力の読み取りに失敗しました。");

        match input.trim().parse::<u8>() {
            Ok(0) => {
                println!("アプリを終了します。");
                std::process::exit(0);
            }
            Ok(1) | Ok(2) | Ok(3) => return input.trim().parse().unwrap(),
            _ => println!("無効な選択です。0(終了)、1(変換:lang=>json)、2(変換:json=>lang)、3(全て変換)を選択してください。\n"),
        }
    }
}

fn main() {
    ensure_directories();
    loop {
        let mode = prompt_for_mode();
        process_files(mode);
    }
}
