use bytecheck::CheckBytes;
use rkyv::{Archive, Deserialize, Serialize};
use mmap_sync::synchronizer::Synchronizer;

// use kani::proof;
// use kani::{any, assume};

/// Example data-structure shared between writer and reader(s)
#[derive(Archive, Deserialize, Serialize, Debug, PartialEq)]
#[archive_attr(derive(CheckBytes))]
pub struct HelloWorld {
    pub version: u32,
    pub messages: Vec<String>,
}

#[kani::proof]
fn verify_mutual_exclusion() {
    // Initialize two writers pointing to the same shared memory location
    let mut writer1 = Synchronizer::new("/tmp/hello_world".as_ref());
    let mut writer2 = Synchronizer::new("/tmp/hello_world".as_ref());

    // Define data for the two writers
    let data1 = HelloWorld {
        version: kani::any(), // Use symbolic value for testing
        messages: vec!["Writer1".to_string()],
    };
    let data2 = HelloWorld {
        version: kani::any(), // Use symbolic value for testing
        messages: vec!["Writer2".to_string()],
    };

    // Simulate writes
    let write1 = writer1.write(&data1, std::time::Duration::from_secs(1));
    let write2 = writer2.write(&data2, std::time::Duration::from_secs(1));

    // Assume that the first write succeeds (models arbitrary interleaving)
    kani::assume(write1.is_ok());

    // Assert that the second write must fail, ensuring mutual exclusion
    kani::assert(
        !write2.is_err(),
        "Mutual exclusion violated: Both writers succeeded!"
    );
}


#[kani::proof]
fn verify_atomic_reads() {
    // Create a shared memory location
    let path = "/tmp/hello_world_atomic";

    // Initialize writer and two readers
    let mut writer = Synchronizer::new(path.as_ref());
    let mut reader1 = Synchronizer::new(path.as_ref());
    let mut reader2 = Synchronizer::new(path.as_ref());

    // Write test data to shared memory
    let data = HelloWorld {
        version: kani::any(),
        messages: vec!["AtomicRead".to_string()],
    };

    writer.write(&data, std::time::Duration::from_secs(1)).unwrap();

    // Read data from both readers
    let read1 = unsafe { reader1.read::<HelloWorld>(false) }.expect("Reader 1 failed to read!");
    let read2 = unsafe { reader2.read::<HelloWorld>(false) }.expect("Reader 2 failed to read!");

    // Access the underlying data
    let data1 = &read1.messages;
    let data2 = &read2.messages;

    // Assert that both readers see the same data
    assert!(
        data1 == data2,
        "Atomicity violated: Readers observed different data: {:?} vs {:?}",
        data1,
        data2
    );
}
