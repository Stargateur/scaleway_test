use scaleway_test;
use snafu::{
  ResultExt,
  Snafu,
};

#[derive(Snafu, Debug)]
enum Error {
  #[snafu(display("Failed to create DHCP server: {}", source))]
  DhcpServerCreation {
    source: scaleway_test::dhcp::Error,
  },
  // #[snafu(display("Failed to resolve DNS: {}", source))]
  // DnsResolution { source: scaleway_dhcp_dns::dns::Error },
}

#[tokio::main]
async fn main() -> Result<(), Error> {
  let vpc_api = scaleway_test::vpc::VpcApiMock::new();
  let mut ipam_api = scaleway_test::ipam::IpamApiMock::new();

  let mac_address = "aa:bb:cc:dd:ee:ff";
  let ip = scaleway_test::dhcp::handle_dhcp_request(mac_address, &vpc_api, &mut ipam_api);
  println!("IP assignée: {:?}", ip);

  let resolved = scaleway_test::dns::resolve_dns(
    "machine-aa:bb:cc:dd:ee:ff",
    "backend",
    &vpc_api,
    &ipam_api,
  );
  println!("Résolution DNS: {:?}", resolved);

  Ok(())
}
