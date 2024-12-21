#[cfg(test)]
mod tests {
    use mmap_sync::synchronizer::Synchronizer;
    use rkyv::{Archive, Serialize};
    use std::sync::{Arc, Mutex};
    use std::thread;
    use std::time::Duration;

    #[derive(Debug, Clone, Archive, Serialize)]
    #[archive(check_bytes)]
    struct HelloWorld {
        version: u32,
        messages: Vec<String>,
    }

    impl HelloWorld {
        fn from_archived(archived: &<HelloWorld as Archive>::Archived) -> Self {
            HelloWorld {
                version: archived.version,
                messages: archived
                    .messages
                    .iter()
                    .map(|msg| msg.to_string())
                    .collect(),
            }
        }
    }

    #[test]
    fn test_single_writer_multiple_readers() {
        let shared_path = "/tmp/hello_world";
        let synchronizer = Arc::new(Mutex::new(Synchronizer::new(shared_path.as_ref())));

        let data = HelloWorld {
            version: 1,
            messages: vec!["Single Writer Message".to_string()],
        };

        let synchronizer_writer = Arc::clone(&synchronizer);
        let writer = thread::spawn(move || {
            for _ in 0..5 {
                let mut sync = synchronizer_writer.lock().unwrap();
                sync.write(&data, Duration::from_secs(1)).expect("Writer failed to write");
                thread::sleep(Duration::from_millis(100));
            }
        });

        let mut readers = vec![];
        for _ in 0..3 {
            let synchronizer_reader = Arc::clone(&synchronizer);
            let reader = thread::spawn(move || {
                for _ in 0..5 {
                    let mut sync = synchronizer_reader.lock().unwrap();
                    let archived_data = unsafe { sync.read::<HelloWorld>(false) }.expect("Reader failed to read");
                    let data = HelloWorld::from_archived(&archived_data);
                    println!("Reader: version: {} | messages: {:?}", data.version, data.messages);
                    thread::sleep(Duration::from_millis(150));
                }
            });
            readers.push(reader);
        }

        writer.join().unwrap();
        for reader in readers {
            reader.join().unwrap();
        }

        println!("Test completed: Single Writer Multiple Readers.");
    }

    #[test]
    fn test_concurrent_writers_and_readers() {
        let shared_path = "/tmp/hello_world";
        let synchronizer = Arc::new(Mutex::new(Synchronizer::new(shared_path.as_ref())));

        let data1 = HelloWorld {
            version: 1,
            messages: vec!["Writer1".to_string()],
        };

        let data2 = HelloWorld {
            version: 2,
            messages: vec!["Writer2".to_string()],
        };

        let synchronizer_writer1 = Arc::clone(&synchronizer);
        let writer1 = thread::spawn(move || {
            for _ in 0..5 {
                let mut sync = synchronizer_writer1.lock().unwrap();
                sync.write(&data1, Duration::from_secs(1)).expect("Writer1 failed to write");
                thread::sleep(Duration::from_millis(100));
            }
        });

        let synchronizer_writer2 = Arc::clone(&synchronizer);
        let writer2 = thread::spawn(move || {
            for _ in 0..5 {
                let mut sync = synchronizer_writer2.lock().unwrap();
                sync.write(&data2, Duration::from_secs(1)).expect("Writer2 failed to write");
                thread::sleep(Duration::from_millis(150));
            }
        });

        let synchronizer_reader = Arc::clone(&synchronizer);
        let reader = thread::spawn(move || {
            for _ in 0..10 {
                let mut sync = synchronizer_reader.lock().unwrap();
                let archived_data = unsafe { sync.read::<HelloWorld>(false) }.expect("Reader failed to read");
                let data = HelloWorld::from_archived(&archived_data);
                println!("Reader: version: {} | messages: {:?}", data.version, data.messages);
                thread::sleep(Duration::from_millis(200));
            }
        });

        writer1.join().unwrap();
        writer2.join().unwrap();
        reader.join().unwrap();

        println!("Test completed: Concurrent Writers and Readers.");
    }

    #[test]
    fn test_large_data() {
        let shared_path = "/tmp/hello_world";
        let synchronizer = Arc::new(Mutex::new(Synchronizer::new(shared_path.as_ref())));

        let large_messages: Vec<String> = (0..1000).map(|i| format!("Message {}", i)).collect();
        let data = HelloWorld {
            version: 1,
            messages: large_messages.clone(),
        };

        let synchronizer_writer = Arc::clone(&synchronizer);
        let writer = thread::spawn(move || {
            let mut sync = synchronizer_writer.lock().unwrap();
            sync.write(&data, Duration::from_secs(1)).expect("Writer failed to write");
        });

        let synchronizer_reader = Arc::clone(&synchronizer);
        let reader = thread::spawn(move || {
            let mut sync = synchronizer_reader.lock().unwrap();
            let archived_data = unsafe { sync.read::<HelloWorld>(false) }.expect("Reader failed to read");
            let data = HelloWorld::from_archived(&archived_data);
            println!("Reader: version: {} | messages: {:?}", data.version, data.messages.len());
        });

        writer.join().unwrap();
        reader.join().unwrap();

        println!("Test completed: Large Data.");
    }

    #[test]
    fn test_frequent_state_reset() {
        let shared_path = "/tmp/hello_world";
        let synchronizer = Arc::new(Mutex::new(Synchronizer::new(shared_path.as_ref())));
    
        let data = HelloWorld {
            version: 1,
            messages: vec!["Reset Test".to_string()],
        };
    
        let synchronizer_writer = Arc::clone(&synchronizer);
        let writer = thread::spawn(move || {
            for _ in 0..5 {
                let mut sync = synchronizer_writer.lock().unwrap();
                sync.write(&data, Duration::from_secs(1)).expect("Writer failed to write");
                // Simulate a reset by writing an empty structure or clearing manually
                let reset_data = HelloWorld {
                    version: 0,
                    messages: vec![],
                };
                sync.write(&reset_data, Duration::from_secs(1))
                    .expect("Writer failed to reset state");
                thread::sleep(Duration::from_millis(100));
            }
        });
    
        let synchronizer_reader = Arc::clone(&synchronizer);
        let reader = thread::spawn(move || {
            for _ in 0..5 {
                let mut sync = synchronizer_reader.lock().unwrap();
                let archived_data = unsafe { sync.read::<HelloWorld>(false) }.expect("Reader failed to read");
                let data = HelloWorld::from_archived(&archived_data);
                println!("Reader: version: {} | messages: {:?}", data.version, data.messages);
                thread::sleep(Duration::from_millis(150));
            }
        });
    
        writer.join().unwrap();
        reader.join().unwrap();
    
        println!("Test completed: Frequent State Reset.");
    }
    
}
