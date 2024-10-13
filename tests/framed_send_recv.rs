use futures::SinkExt;
use rand::{prelude::StdRng, SeedableRng};
use tokio_stream::StreamExt;
use tokio_util::codec::Framed;

use dev_utils::{create_random_v1_raw_message, create_random_v2_raw_message};
use mavlink_codec::{codec::MavlinkCodec, Packet};

#[tokio::test]
async fn send_recv_v1() {
    let seed = 42;
    println!("Using seed {seed:?}");
    let mut rng: StdRng = SeedableRng::seed_from_u64(seed);

    let packets_count = 100000;
    let mut packets = Vec::with_capacity(packets_count);
    for _ in 0..packets_count {
        let mavlink_v1_message_raw = create_random_v1_raw_message(&mut rng);
        let packet = Packet::from(mavlink_v1_message_raw);
        packets.push(packet);
    }

    let codec = MavlinkCodec::<true, false, false, false, false, false>::default();
    let simplex = tokio::io::SimplexStream::new_unsplit(4096);
    let framed = Framed::new(simplex, codec);
    let (mut writer, mut reader) = futures::StreamExt::split(framed);

    // Send and receive each packet sequentially
    for (idx, packet) in packets.iter().enumerate() {
        println!("Sending packet {idx}");
        // Send the packet
        writer.send(packet.clone()).await.unwrap();

        // Wait for the response
        if let Some(Ok(Ok(received_packet))) = reader.next().await {
            println!("Received packet {idx}");
            assert_eq!(received_packet, *packet);
        } else {
            panic!("Failed to receive packet {idx}");
        }
    }

    println!("All packets sent and received successfully!");
}

#[tokio::test]
async fn send_recv_v1_concurrent() {
    let seed = 42;
    println!("Using seed {seed:?}");
    let mut rng: StdRng = SeedableRng::seed_from_u64(seed);

    let packets_count = 100000;
    let mut packets = Vec::with_capacity(packets_count);
    for _ in 0..packets_count {
        let mavlink_v1_message_raw = create_random_v1_raw_message(&mut rng);
        let packet = Packet::from(mavlink_v1_message_raw);
        packets.push(packet);
    }

    let codec = MavlinkCodec::<true, false, false, false, false, false>::default();
    let simplex = tokio::io::SimplexStream::new_unsplit(4096);
    let framed = Framed::new(simplex, codec);
    let (mut writer, mut reader) = futures::StreamExt::split(framed);

    let packets_cloned = packets.clone();
    let writer_task = tokio::spawn(async move {
        for (idx, packet) in packets_cloned.iter().enumerate() {
            println!("Sending packet {idx}");
            writer.send(packet.clone()).await.unwrap();
        }
    });

    let packets_cloned = packets.clone();
    let reader_task = tokio::spawn(async move {
        let mut received_count = 0;
        while received_count < packets_count {
            if let Some(Ok(Ok(received_packet))) = reader.next().await {
                println!("Received packet {received_count}");
                assert_eq!(received_packet, packets_cloned[received_count]);
                received_count += 1;
            }
        }
    });

    let _ = tokio::join!(writer_task, reader_task);
    println!("All packets sent and received successfully!");
}

#[tokio::test]
async fn send_recv_v2() {
    let seed = 42;
    println!("Using seed {seed:?}");
    let mut rng: StdRng = SeedableRng::seed_from_u64(seed);

    let packets_count = 100000;
    let mut packets = Vec::with_capacity(packets_count);
    for _ in 0..packets_count {
        let mavlink_v2_message_raw = create_random_v2_raw_message(&mut rng);
        let packet = Packet::from(mavlink_v2_message_raw);
        packets.push(packet);
    }

    let codec = MavlinkCodec::<false, true, false, false, false, false>::default();
    let simplex = tokio::io::SimplexStream::new_unsplit(4096);
    let framed = Framed::new(simplex, codec);
    let (mut writer, mut reader) = futures::StreamExt::split(framed);

    // Send and receive each packet sequentially
    for (idx, packet) in packets.iter().enumerate() {
        println!("Sending packet {idx}");
        // Send the packet
        writer.send(packet.clone()).await.unwrap();

        // Wait for the response
        if let Some(Ok(Ok(received_packet))) = reader.next().await {
            println!("Received packet {idx}");
            assert_eq!(received_packet, *packet);
        } else {
            panic!("Failed to receive packet {idx}");
        }
    }

    println!("All packets sent and received successfully!");
}

#[tokio::test]
async fn send_recv_v2_concurrent() {
    let seed = 42;
    println!("Using seed {seed:?}");
    let mut rng: StdRng = SeedableRng::seed_from_u64(seed);

    let packets_count = 100000;
    let mut packets = Vec::with_capacity(packets_count);
    for _ in 0..packets_count {
        let mavlink_v2_message_raw = create_random_v2_raw_message(&mut rng);
        let packet = Packet::from(mavlink_v2_message_raw);
        packets.push(packet);
    }

    let codec = MavlinkCodec::<false, true, false, false, false, false>::default();
    let simplex = tokio::io::SimplexStream::new_unsplit(4096);
    let framed = Framed::new(simplex, codec);
    let (mut writer, mut reader) = futures::StreamExt::split(framed);

    let packets_cloned = packets.clone();
    let writer_task = tokio::spawn(async move {
        for (idx, packet) in packets_cloned.iter().enumerate() {
            println!("Sending packet {idx}");
            writer.send(packet.clone()).await.unwrap();
        }
    });

    let packets_cloned = packets.clone();
    let reader_task = tokio::spawn(async move {
        let mut received_count = 0;
        while received_count < packets_count {
            if let Some(Ok(Ok(received_packet))) = reader.next().await {
                println!("Received packet {received_count}");
                assert_eq!(received_packet, packets_cloned[received_count]);
                received_count += 1;
            }
        }
    });

    let _ = tokio::join!(writer_task, reader_task);
    println!("All packets sent and received successfully!");
}
