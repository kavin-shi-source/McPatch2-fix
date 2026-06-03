//! 运行内置服务端，使用私有协议
use std::future::Future;
use std::io::ErrorKind;
use std::ops::Range;
use std::path::Path;
use std::time::SystemTime;

use chrono::Local;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncSeekExt;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;
use tokio::net::TcpStream;

use crate::app_path::AppPath;
use crate::config::Config;
use crate::utility::partial_read::PartialAsyncRead;
use crate::utility::traffic_control::AsyncTrafficControl;

pub async fn start_builtin_server(config: Config, app_path: AppPath) {
    if !config.builtin_server.enabled {
        return;
    }

    let capacity = config.builtin_server.capacity;
    let regain = config.builtin_server.regain;

    if capacity > 0 && regain > 0 {
        println!("私有协议服务端已经启动。capacity: {}, regain: {}", capacity, regain);
    } else {
        println!("私有协议服务端已经启动。");
    }

    let host = config.builtin_server.listen_addr.to_owned();
    let port = format!("{}", config.builtin_server.listen_port);

    println!("private protocol is now listening on {}:{}", host, port);

    let listener = TcpListener::bind(format!("{}:{}", host, port)).await.unwrap();

    loop {
        let (stream, _peer_addr) = listener.accept().await.unwrap();

        let config = config.clone();
        let app_path = app_path.clone();

        tokio::spawn(async move { serve_loop(stream, config, app_path).await });
    }
}

async fn serve_loop(mut stream: TcpStream, config: Config, app_path: AppPath) {
    let tbf_burst = config.builtin_server.capacity as u64;
    let tbf_rate = config.builtin_server.regain as u64;
    let public_dir = app_path.public_dir;

    async fn inner(
        mut stream: &mut TcpStream,
        tbf_burst: u64,
        tbf_rate: u64,
        public_dir: &Path,
        info: &mut Option<(String, Range<u64>)>,
    ) -> std::io::Result<()> {
        let mut path = String::with_capacity(1024);
        receive_data(&mut stream).await?.read_to_string(&mut path).await?;

        let start = timeout(stream.read_u64_le()).await?;
        let mut end = timeout(stream.read_u64_le()).await?;

        *info = Some((path.to_owned(), start..end));

        let path = public_dir.join(path);

        assert!(start <= end, "the end is {} and the start is {}", end, start);

        let len = match tokio::fs::metadata(&path).await {
            Ok(meta) => {
                if end > meta.len() {
                    stream.write_all(&(-2i64).to_le_bytes()).await?;
                    return Ok(());
                }
                meta.len()
            }
            Err(_) => {
                stream.write_all(&(-1i64).to_le_bytes()).await?;
                return Ok(());
            }
        };

        if start == 0 && end == 0 {
            end = len as u64;
        }

        let mut remains = end - start;

        stream.write_all(&(remains as i64).to_le_bytes()).await?;

        let mut file = tokio::fs::File::open(path).await?;
        file.seek(std::io::SeekFrom::Start(start)).await?;

        let mut file = AsyncTrafficControl::new(&mut file, tbf_burst, tbf_rate);

        while remains > 0 {
            let mut buf = [0u8; 32 * 1024];
            let limit = buf.len().min(remains as usize);
            let buf = &mut buf[0..limit];

            let read = file.read(buf).await?;

            stream.write_all(&buf[0..read]).await?;

            remains -= read as u64;
        }

        Ok(())
    }

    loop {
        let mut info = Option::<(String, Range<u64>)>::None;

        let start = SystemTime::now();
        let result = inner(&mut stream, tbf_burst, tbf_rate, &public_dir, &mut info).await;
        let time = SystemTime::now().duration_since(start).unwrap();

        match result {
            Ok(_) => {
                let info = info.unwrap();
                let ts = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
                println!(
                    "[{}] {} - {} {}+{} ({}ms)",
                    ts,
                    stream.peer_addr().unwrap(),
                    info.0,
                    info.1.start,
                    info.1.end - info.1.start,
                    time.as_millis()
                );
            }
            Err(e) => {
                match e.kind() {
                    ErrorKind::UnexpectedEof => {}
                    ErrorKind::ConnectionAborted => {}
                    ErrorKind::ConnectionReset => {}
                    _ => println!("{} - {:?}", stream.peer_addr().unwrap(), e.kind()),
                }

                break;
            }
        }
    }
}

async fn _send_data(stream: &mut TcpStream, data: &[u8]) -> std::io::Result<()> {
    stream.write_u64_le(data.len() as u64).await?;
    stream.write_all(data).await?;

    Ok(())
}

async fn receive_data<'a>(stream: &'a mut TcpStream) -> std::io::Result<PartialAsyncRead<&'a mut TcpStream>> {
    let len = timeout(stream.read_u64_le()).await?;

    Ok(PartialAsyncRead::new(stream, len))
}

async fn timeout<T, F: Future<Output = std::io::Result<T>>>(f: F) -> std::io::Result<T> {
    tokio::select! {
        _ = tokio::time::sleep(std::time::Duration::from_secs(30)) => {
            Err(std::io::Error::new(ErrorKind::UnexpectedEof, "timeout"))
        },
        d = f => {
            d
        }
    }
}
