use rand::{prelude::StdRng, Rng, SeedableRng};
use tokio::io::AsyncWriteExt;
use tokio_stream::StreamExt;
use tokio_util::codec::FramedRead;

use dev_utils::{add_random_v1_message, add_random_v2_message, chunk_buffer_randomly};
use mavlink_codec::{codec::MavlinkCodec, v1::V1Packet, v2::V2Packet};

#[tokio::test]
async fn chuncked_decode_v1() {
    let seed = 42;
    println!("Using seed {seed:?}");
    let mut rng: StdRng = SeedableRng::seed_from_u64(seed);

    let messages_count = 1000;
    let mut messages = Vec::with_capacity(messages_count);
    for _ in 0..messages_count {
        let mut message = Vec::with_capacity(V1Packet::MAX_PACKET_SIZE);
        add_random_v1_message(&mut message, &mut rng);
        messages.push(message);
    }
    println!("Generated {} messages", messages.len());

    let messages_cloned = messages.clone();
    let mut messages = messages.concat();
    println!("Total concatenated message size: {}", messages.len());

    // Add some trash in the beginning
    for _ in 0..100 {
        messages.insert(0, rng.gen_range(0..255));
    }
    println!("Added trash to the beginning of the message");

    let (reader, mut writer) = tokio::io::simplex(4096);
    let chunked_buf = chunk_buffer_randomly(&messages, &mut rng, 1, 128);
    println!("Chunked buffer into {} chunks", chunked_buf.len());

    // Inject the chunked data into the writer asynchronously
    let writer_task = tokio::spawn(async move {
        for (idx, chunk) in chunked_buf.iter().enumerate() {
            writer.write_all(chunk).await.unwrap();
            println!("Sent chunk {idx}, size: {:?}", chunk.len());
            // Simulate network delay
            tokio::time::sleep(tokio::time::Duration::from_micros(1)).await;
        }
        println!("All messages sent");
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        writer.shutdown().await.unwrap();
    });

    let codec = MavlinkCodec::<true, false, false, false, false, false>::default();
    let mut framed = FramedRead::new(reader, codec);

    let mut i = 0;
    while let Some(Ok(result)) = framed.next().await {
        match result {
            Ok(packet) => {
                let buffer = match &packet {
                    mavlink_codec::Packet::V1(v1_packet) => v1_packet.as_slice(),
                    mavlink_codec::Packet::V2(_v2_packet) => panic!("Got a wrong package"),
                };

                assert_eq!(buffer, messages_cloned[i].as_slice());
                // println!("Successfully decoded message {i}");

                i += 1;
                if i == messages_count {
                    break;
                }
            }
            Err(error) => {
                eprintln!(
                    "Error while decoding packet at message {i}: {error:?}. framed buffer: {:?}",
                    framed.read_buffer().to_vec()
                );
            }
        }
    }

    assert_eq!(i, messages_count);
    println!("All {i} messages successfully decoded!");

    writer_task.abort();
}

#[tokio::test]
async fn chuncked_decode_v2() {
    let seed = 42;
    println!("Using seed {seed:?}");
    let mut rng: StdRng = SeedableRng::seed_from_u64(seed);

    let messages_count = 1000;
    let mut messages = Vec::with_capacity(messages_count);
    for _ in 0..messages_count {
        let mut message = Vec::with_capacity(V2Packet::MAX_PACKET_SIZE);
        add_random_v2_message(&mut message, &mut rng);
        messages.push(message);
    }
    println!("Generated {} messages", messages.len());

    let messages_cloned = messages.clone();
    let mut messages = messages.concat();
    println!("Total concatenated message size: {}", messages.len());

    // Add some trash in the beginning
    for _ in 0..100 {
        messages.insert(0, rng.gen_range(0..255));
    }
    println!("Added trash to the beginning of the message");

    let (reader, mut writer) = tokio::io::simplex(4096);
    let chunked_buf = chunk_buffer_randomly(&messages, &mut rng, 1, 128);
    println!("Chunked buffer into {} chunks", chunked_buf.len());

    // Inject the chunked data into the writer asynchronously
    let writer_task = tokio::spawn(async move {
        for (idx, chunk) in chunked_buf.iter().enumerate() {
            writer.write_all(chunk).await.unwrap();
            println!("Sent chunk {idx}, size: {:?}", chunk.len());
            // Simulate network delay
            tokio::time::sleep(tokio::time::Duration::from_micros(1)).await;
        }
        println!("All messages sent");
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        writer.shutdown().await.unwrap();
    });

    let codec = MavlinkCodec::<false, true, false, false, false, false>::default();
    let mut framed = FramedRead::new(reader, codec);

    let mut i = 0;
    while let Some(Ok(result)) = framed.next().await {
        match result {
            Ok(packet) => {
                let buffer = match &packet {
                    mavlink_codec::Packet::V1(_v1_packet) => panic!("Got a wrong package"),
                    mavlink_codec::Packet::V2(v2_packet) => v2_packet.as_slice(),
                };

                assert_eq!(buffer, messages_cloned[i]);
                // println!("Successfully decoded message {i}");

                i += 1;
                if i == messages_count {
                    break;
                }
            }
            Err(error) => {
                eprintln!(
                    "Error while decoding packet at message {i}: {error:?}. framed buffer: {:?}",
                    framed.read_buffer().to_vec()
                );
            }
        }
    }

    assert_eq!(i, messages_count);
    println!("All {i} messages successfully decoded!");

    writer_task.abort();
}
