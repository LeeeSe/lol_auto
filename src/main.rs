use std::{process::Command, sync::mpsc};
use regex::Regex;
use std::str;
use reqwest::{self, Client};
use std::thread;

slint::include_modules!();
fn main() {
    let base_url = get_url();
    let client = reqwest::Client::builder().danger_accept_invalid_certs(true).build().unwrap();

    let app = App::new();
    let app_handle = app.as_weak();
    let app_handle_2 = app.as_weak();
    let (tx, rx): (mpsc::Sender<bool>, mpsc::Receiver<bool>) = mpsc::channel();
    let (tx_2, rx_2): (mpsc::Sender<bool>, mpsc::Receiver<bool>) = mpsc::channel();
    app.on_dianwo(move || {
        let app = app_handle.unwrap();
        let status = app.get_flag();
        tx.send(status).expect("accept消息发送失败");
    });
    app.on_dianfind(move || {
        let app = app_handle_2.unwrap();
        let find = app.get_find();
        tx_2.send(find).expect("find消息发送失败");
    });

    thread::spawn(move ||{
        let mut now_status = false;
        let mut find_or_not = false;
        loop
        {
            let status =  rx.try_recv();
            let find =  rx_2.try_recv();
            if status.is_ok() {
                now_status = status.unwrap();
            }
            if  find.is_ok() {
                find_or_not = find.unwrap();
            }
            if now_status{
                if find_or_not{find_game(&base_url, &client)}
                accept_game(&base_url, &client);
                println!("当前状态：开启");
            } else {
                println!("当前状态：关闭");
            }
            thread::sleep(std::time::Duration::from_secs(1));
        }
    });
    app.run();
}

fn get_url() -> String {
    let output = Command::new("sh")
                                .arg("-c")
                                .arg("ps -A | grep LeagueClientUx")
                                .output()
                                .expect("ps命令运行失败");
    let info = str::from_utf8(&output.stdout).unwrap();
    println!("{:?}", info);
    let port_re = Regex::new(r"--app-port=(\d*)").unwrap();
    let token_re = Regex::new(r"--remoting-auth-token=(.*?) ").unwrap();
    let port = port_re.captures(info).expect("获取url失败，可能是游戏未打开").get(1).unwrap().as_str();
    let token = token_re.captures(info).unwrap().get(1).unwrap().as_str();
    // println!("{:?}", port);
    // println!("{:?}", token);
    let mut url: String = String::from("https://riot:");
    url.push_str(token);
    url.push_str("@");
    url.push_str("127.0.0.1");
    url.push_str(":");
    url.push_str(port);
    
    return url;
}

#[tokio::main]
async fn accept_game(base_url: &String, client: &Client) {
    let mut url = String::new();
    url.push_str(base_url.as_str());
    url.push_str("/lol-matchmaking/v1/ready-check/accept");
    client.post(url).send().await.expect("发送接受对局post失败");
}

#[tokio::main]
async fn find_game(base_url: &String, client: &Client) {
    let mut url = String::new();
    url.push_str(base_url.as_str());
    url.push_str("/lol-lobby/v2/lobby/matchmaking/search");
    // println!("{}", url);
    client.post(url).send().await.expect("发送寻找游戏post失败");
}
