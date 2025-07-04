use std::{
  net::{
    Ipv4Addr,
    SocketAddrV4,
  },
  time::Duration,
};

use dhcproto::{
  Decodable,
  Decoder,
  Encodable,
  Encoder,
};
use snafu::{
  ResultExt,
  Snafu,
};
use tokio::net::UdpSocket;
use tracing::info;

use crate::{
  ipam::{
    self,
    Ipam,
    IpamApiMock,
    Resource,
  },
  vpc::{
    Vpc,
    VpcApiMock,
  },
};

#[derive(Debug, Snafu)]
pub enum Error {
  #[snafu(display("Socket error: {}", source))]
  Socket { source: std::io::Error },
  #[snafu(display("Decoding error: {}", source))]
  Decode {
    source: dhcproto::error::DecodeError,
  },
  #[snafu(display("Encoding error: {}", source))]
  Encode {
    source: dhcproto::error::EncodeError,
  },
}

pub struct DhcpServer {
  socket: UdpSocket,
}

impl DhcpServer {
  pub async fn new() -> Result<DhcpServer, Error> {
    let socket = UdpSocket::bind("0.0.0.0:67").await.context(SocketSnafu)?;

    info!("DHCP server listening on UDP/67");

    Ok(DhcpServer { socket })
  }

  async fn run(&self) -> Result<(), Error> {
    let mut buf = [0; 1024];
    loop {
      let (n, addr) = self.socket.recv_from(&mut buf).await.context(SocketSnafu)?;
      info!("Received {} bytes from {}", n, addr);
      let msg = dhcproto::v4::Message::decode(&mut Decoder::new(&buf[..n])).context(DecodeSnafu)?;
    }
  }
}

pub fn handle_dhcp_request(
  mac: &str, vpc_api: &VpcApiMock, ipam_api: &mut IpamApiMock,
) -> Option<Ipv4Addr> {
  ipam_api.assign_ip(mac, vpc_api)
}
