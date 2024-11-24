//use mmap_sync::*;
use std::time::Duration;
use crate::synchronizer::Synchronizer;
use rkyv::{Archive, Deserialize, Serialize};
use bytecheck::CheckBytes;

//#[derive(Debug, PartialEq, Clone)]
#[derive(Archive, Deserialize, Serialize, Debug, PartialEq)]
#[archive_attr(derive(CheckBytes))]
struct TestData {
    version: u64,
    value: String
}

#[kani::proof]
fn verify_single_writer() {
    let path = "/tmp/test_single_writer";
    let mut sync1 = Synchronizer::new(path.as_ref());
    let mut sync2 = Synchronizer::new(path.as_ref());

    let data = TestData {
        version: kani::any(),
        value: "test".to_string()
    };

    let write1 = sync1.write(&data, Duration::from_secs(1));
    let write2 = sync2.write(&data, Duration::from_secs(1));
    
    kani::assume(write1.is_ok());
    assert!(write2.is_err());
}

#[kani::proof]
fn verify_atomic_reads() {
    let path = "/tmp/test_atomic_reads";
    let mut writer = Synchronizer::new(path.as_ref());
    let reader1 = Synchronizer::new(path.as_ref());
    let reader2 = Synchronizer::new(path.as_ref());

    let data = TestData {
        version: kani::any(),
        value: "test".to_string()
    };

    writer.write(&data, Duration::from_secs(1)).unwrap();

    let read1 = unsafe { reader1.read::<TestData>(false) }.unwrap();
    let read2 = unsafe { reader2.read::<TestData>(false) }.unwrap();
    
    assert!(read1 == read2);
}