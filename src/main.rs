use std::collections::HashMap;
use std::sync::Arc;
use std::thread;

use rand::Rng;

const MAPS_COUNT: usize = 1024;
const MAP_SIZE: usize = 1024;

const THREADS_COUNT: usize = 2;

fn main() {
    let mut rng = rand::thread_rng();

    let mut maps = vec![];

    eprint!(
        "Populating {} maps with {} elements each .. ",
        MAPS_COUNT, MAP_SIZE
    );

    for _i in 0..MAPS_COUNT {
        let mut map = HashMap::<usize, usize>::new();

        for _j in 0..MAP_SIZE {
            map.insert(rng.gen(), rng.gen());
        }

        maps.push(map);
    }

    eprintln!("done");

    let maps = Arc::new(maps);

    let mut handles = vec![];

    eprintln!("Launching {} threads to churn through maps", THREADS_COUNT);

    for _i in 0..THREADS_COUNT {
        let maps = Arc::clone(&maps);
        handles.push(thread::spawn(move || {
            let mut rng = rand::thread_rng();
            for _i in 0..500 {
                let mut sum = 0;

                let map = maps.get(rng.gen::<usize>() % MAPS_COUNT).unwrap();

                if map.len() != MAP_SIZE {
                    panic!("uh oh");
                }

                for (k, v) in map.iter() {
                    sum += (*k % 8) + (*v % 8);
                }

                if rng.gen::<usize>() % 100 == 0 {
                    eprintln!("sum = {} from thread {:?}", sum, thread::current().id());
                }
            }
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }
}
