use std::env;
use std::io::Result;
use std::path::Path;
use std::sync::Arc;

use tokio::fs::File;
use tokio::io;
use tokio::net::{TcpListener, TcpStream};
use tokio::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    let mut args = env::args();
    args.next();

    let mode = args.next().unwrap();
    let param = args.next().unwrap();
    let param2 = args.next().unwrap();

    match mode.as_str() {
        // host, file path
        "-c" => client(param, param2).await?,
        // host, dic path
        _ => server(param, param2).await?
    }
    Ok(())
}

async fn client(host: String, file: String) -> Result<()> {
    let file = Path::new(&file);
    let file_name = file.file_name().unwrap().to_str().unwrap();

    let socket = TcpStream::connect(host);
    let file = File::open(file);

    let (mut socket, mut file) = tokio::try_join!(socket, file)?;

    socket.write_u64(file_name.len() as u64).await?;
    socket.write_all(file_name.as_bytes()).await?;
    let size = io::copy(&mut file, &mut socket).await?;

    println!("success {} length", size);
    Ok(())
}

async fn server(host: String, dic: String) -> Result<()> {
    let dic = Arc::new(dic);
    let mut listener = TcpListener::bind(host).await?;

    loop {
        let (socket, _) = listener.accept().await?;

        let dic = dic.clone();
        tokio::spawn(async {
            match process(socket, dic).await {
                Ok(size) => println!("success {} length", size),
                Err(e) => eprintln!("{:?}", e)
            }
        });
    }
}

async fn process(mut socket: TcpStream, dic: Arc<String>) -> Result<u64> {
    let len = socket.read_u64().await?;
    let mut file_name: Vec<u8> = vec![0u8; len as usize];
    socket.read_exact(&mut file_name).await?;
    let file_name = String::from_utf8(file_name).unwrap();

    let mut file_dic = String::new();
    file_dic.push_str(&dic);
    file_dic.push('/');
    file_dic.push_str(&file_name);

    let size = io::copy(&mut socket, &mut File::create(file_dic).await?).await?;
    Ok(size)
}