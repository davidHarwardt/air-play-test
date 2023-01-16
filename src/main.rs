#![allow(unused)]

use std::{net::IpAddr, str::FromStr, time::Duration};

use serde::{Serialize, Deserialize};

#[repr(transparent)]
struct AirplayFeatures(u32);
impl AirplayFeatures {
    const VIDEO: u32 = 1 << 0;
    const PHOTO: u32 = 1 << 1;
    const VIDEO_HTTP_LIVE_STREAMS: u32 = 1 << 4;
    const SCREEN: u32 = 1 << 7;
    const SCREEN_ROTATE: u32 = 1 << 8;
    const AUDIO: u32 = 1 << 9;
    const AUDIO_REDUNDANT: u32 = 1 << 11;
}
impl std::fmt::Display for AirplayFeatures {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "0x{:x}", self.0) }
}

fn main() {
    let mut builder = env_logger::Builder::new();
    // builder.parse_filters("libmdns=debug");
    builder.init();

    let mac_address = mac_address::get_mac_address().unwrap().unwrap();
    println!("mac_address: {mac_address}");
    // let mac_address = "01:02:03:04:05:06";

    let features = AirplayFeatures::VIDEO
                 | AirplayFeatures::PHOTO
                 | AirplayFeatures::VIDEO_HTTP_LIVE_STREAMS
                 | AirplayFeatures::SCREEN
                 | AirplayFeatures::SCREEN_ROTATE
                 | AirplayFeatures::AUDIO | AirplayFeatures::AUDIO_REDUNDANT;
    let features = AirplayFeatures(0x39f7);

    let responder = libmdns::Responder::new().unwrap();
    let address = mac_address.to_string().replace(":", "");

    let server_name = "test";
    let air_tunes_server_name = format!("{address}@{server_name}");
    let air_tunes_service_name = "_raop._tcp";
    let air_tunes_port = 5001;

    let _air_tunes = responder.register(
        format!("{air_tunes_service_name}"),
        format!("{air_tunes_server_name}"),
        air_tunes_port,
        &[
            "ch=2",
            "chn=0,1,2,3",
            "da=true",
            "et=0,3,5",
            "vv=2",
            "ft=0x5A7FFFF7,0x1E",
            "am=AppleTV2,1",
            "md=0,1,2",
            "rhd=5.6.0.0",
            "pw=false",
            "sr=44100",
            "ss=16",
            "sv=false",
            "tp=UDP",
            "txtvers=1",
            "sf=0x4",
            "vs=220.68",
            "vn=66637",
            "pk=b07727d6f6cd6e08b58ede525ec3cdeaa252ad9f683feb212ef8a205246554e7",
        ],
    );
    println!("registered airtunes with name: {air_tunes_server_name} -> {air_tunes_service_name} on port {air_tunes_port}");

    let air_play_server_name = format!("{server_name}");
    let air_play_service_name = "_airplay._tcp";
    let air_play_port = 7001;

    let _air_play = responder.register(
        format!("{air_play_service_name}"),
        format!("{air_play_server_name}"),
        air_play_port,
        &[
            &format!("deviceid={mac_address}"),
            "features=0x5A7FFFF7,0x1E",
            "srcvers=220.68",
            "flags=0x4",
            "vv=2",
            "model=AppleTV2,1",
            "rhd=5.6.0.0",
            "pw=false",
            "pk=b07727d6f6cd6e08b58ede525ec3cdeaa252ad9f683feb212ef8a205246554e7",
            "pi=2e388006-13ba-4041-9a67-25dd4a43d536",
        ],
    );
    println!("registered airplay with name: {air_play_server_name} -> {air_play_service_name} on port {air_play_port}");



    use std::net::{TcpListener, Shutdown};
    use std::io::{Write, Read};
    let listener = TcpListener::bind(("0.0.0.0", air_tunes_port)).expect("could not create server");
    for conn in listener.incoming() {
        let mut conn = conn.unwrap();
        let mut data = Vec::new();

        let mut msg = [0u8; 1024];
        while let Ok(len) = conn.read(&mut msg) {
            data.extend_from_slice(&msg[0..len]);
            let body_idx = data.windows(4).position(|v| match v {
                b"\r\n\r\n" => true,
                _ => false,
            });

            if let Some(body_idx) = body_idx {
                let head = &data[0..body_idx];
                let body = &data[(body_idx + 4)..];

                #[derive(Serialize, Deserialize, Debug)]
                struct Test {
                    qualifier: Vec<String>,
                }

                println!("head: {}", std::str::from_utf8(head).unwrap_or("<invalid>"));
                let body: Test = plist::from_bytes(body).expect("could not parse plist");
                println!("body: {:?}", body);

                break;
            }

            // match rtsp_types::Message::<Vec<u8>>::parse(&data) {
            //     Ok((msg, len)) => {
            //         data = data.split_off(len);
            //         println!("{msg:?}");
            //     },
            //     Err(rtsp_types::ParseError::Incomplete) => { println!("incomplete") },
            //     Err(rtsp_types::ParseError::Error) => {
            //         let res = match std::str::from_utf8(&data).map_err(|err| std::str::from_utf8(&data[0..err.valid_up_to()]).unwrap()) {
            //             Ok(v) | Err(v) => v,
            //         };
            //         println!("data: {res}");
            //         println!("raw: {data:?}");
            //         println!("error");
            //         break;
            //     },
            // }
        }
        conn.shutdown(Shutdown::Both).expect("could not shutdown connection");
        println!("shut down");
    }
}

