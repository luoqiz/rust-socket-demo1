use std::thread;
use std::time::Duration;

use futures_util::future::BoxFuture;
use futures_util::FutureExt;
use rust_socketio::{
    asynchronous::{Client, ClientBuilder},
    Payload,
};
use serde_json::json;
use tokio::io;
use tokio::task::JoinHandle;

// https://socket.io/zh-CN/docs/v4/namespaces/
#[tokio::main]
async fn main() {
    let user_id = 123;
    let to_user_id = 789;

    let callback = |payload: Payload, socket: Client| {
        async move {
            match payload {
                Payload::Text(values) => println!("Received: {:#?}", values),
                Payload::Binary(bin_data) => println!("Received bytes: {:#?}", bin_data),
                _ => {}
            }
        }
            .boxed()
    };
    // 发起连接
    let mut socket = ClientBuilder::new("http://127.0.0.1:9001/?userId=".to_owned() + &*user_id.to_string())
        // .namespace("/")
        .on("SINGLE_CHAT", callback)
        .on("error", |err, _| {
            async move { eprintln!("Error: {:#?}", err) }.boxed()
        })
        .connect()
        .await
        .expect("Connection failed");

    // 定期推送ping，如果想改这里自己建立信息通讯mpsc::channel(0) 发送不同的数据;
    let handle: JoinHandle<io::Result<()>> = tokio::spawn(async move {
        loop {
            // 这里因为是async自己包裹一下不然跑步起来
            // tokio::runtime::Builder::new_multi_thread()
            //     .enable_all()
            //     .build()
            //     .unwrap()
            //     .block_on(async {
            // 睡眠定期推送数据
            thread::sleep(Duration::new(20, 0));

            let json_payload = json!({"type":23,"messageId":34,"content":"ping","userId":user_id,"fromUserId":user_id,"toUserId":to_user_id});
            socket
                .emit("SINGLE_CHAT", json_payload)
                .await
                .expect("Server unreachable");
            // });
        }
    });

    handle.await.expect("TODO: panic message");

    socket.disconnect().await.expect("Disconnect failed");
}