//! This example shows how to publish [ResourceRecord]s directly to the DHT.
//!
//! Change the `Keypair::random()` to your own keypair to publish your own records.
//! Change the `packet.answers` to your own records.
//!
//! run this example from the project root:
//!     $ cargo run --example publish

use tracing::Level;
use tracing_subscriber;

use std::time::Instant;

use pkarr::{dns, Keypair, PkarrClient, Result, SignedPacket};

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .init();

    let client = PkarrClient::builder().build().unwrap();

    let keypair = Keypair::random();

    let mut packet = dns::Packet::new_reply(0);
    packet.answers.push(dns::ResourceRecord::new(
        dns::Name::new("_foo").unwrap(),
        dns::CLASS::IN,
        30,
        dns::rdata::RData::TXT("bar".try_into()?),
    ));

    let signed_packet = SignedPacket::from_packet(&keypair, &packet)?;

    let instant = Instant::now();

    println!("\nPublishing {} ...", keypair.public_key());

    match client.publish(&signed_packet) {
        Ok(()) => {
            println!(
                "\nSuccessfully published {} in {:?}",
                keypair.public_key(),
                instant.elapsed(),
            );
        }
        Err(err) => {
            println!("\nFailed to publish {} \n {}", keypair.public_key(), err);
        }
    };

    Ok(())
}
