use std::sync::Arc;
use std::pin::Pin;
use std::future::Future;

fn main() {
    let useful_thing = Arc::new(vec![1, 2, 3]);
	
	let useful_thing_callback = useful_thing.clone();
	register_handler(Box::new(move || {
		let useful_thing = useful_thing_callback.clone();
		Box::pin(async move {
			for _ in useful_thing.iter() {
				// Do something useful.
			}
		})
	}));

	// Also need to use useful thing here:
	for _ in useful_thing.iter() {}
}

fn register_handler(_callback: Box<dyn FnMut() -> Pin<Box<dyn Future<Output = ()>>>>) {}