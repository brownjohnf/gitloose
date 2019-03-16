#[macro_use]
extern crate serde_derive;
extern crate structopt;
extern crate reqwest;

use structopt::StructOpt;
use std::collections::HashMap;

/// A basic example
#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
struct Opt {
    // A flag, true if used in the command line. Note doc comment will
    // be used for the help message of the flag.
    /// Activate debug mode
    #[structopt(short = "d", long = "debug")]
    debug: bool,
}

#[derive(Deserialize, Debug)]
struct Release {
    name: String
}

impl Release {
    fn new() -> Release {
        Release {
            name: String::from("foobar")
        }
    }
}

fn main() {
    let opt = Opt::from_args();
    match make_request() {
        Ok(res) => println!("{:?}", res),
        Err(err) => println!("{:?}", err),
    }
    println!("Hello, world!");
}

fn make_request() -> Result<reqwest::Response, reqwest::Error> {
    let client = reqwest::Client::new();
    let req = client.get("https://api.github.com/repos/brownjohnf/slit/releases");

    let res = req.send()?;
    println!("{:?}", res);

    return Ok(res);
            /*
    match &res {
        Ok(d) => {
            match d {
                Response(r) => {
                    return Some(releases[0]);
                }
            }
            println!("{:?}", d);
        }
        Err(err) => ()
    }
            */
}
