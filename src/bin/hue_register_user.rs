extern crate hueclient;
use hueclient::HueError;
use std::env;

#[allow(while_true)]
#[allow(dead_code)]
fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("usage : {:?} <devicetype>", args[0]);
    } else {
        let bridge = hueclient::Bridge::discover_required();
        println!("posting user {:?} in {:?}", args[1], bridge);
        loop {
            let r = bridge.clone().register_user(&args[1]);
            match r {
                Ok(r) => {
                    eprint!("done: ");
                    println!("{}", r.username.as_ref().unwrap());
                    break;
                }
                Err(HueError::BridgeError { code, .. }) if code == 101 => {
                    println!("Push the bridge button");
                    std::thread::sleep(::std::time::Duration::from_secs(5));
                }
                Err(e) => panic!("error {:?}", e),
            }
        }
    }
}
