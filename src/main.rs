#[macro_use]
extern crate serde_derive;
extern crate structopt;
extern crate reqwest;
extern crate serde;

use structopt::StructOpt;
use reqwest::Url;

#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
/// List and create GitHub releases
struct Opt {
    /// Github org name
    #[structopt(short, long)]
    org: String,
    /// Github repo name
    #[structopt(short, long)]
    repo: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Release {
    url: String,
    tag_name: String,
    created_at: String,
    published_at: String,
}

struct Org {
    name: String,
}

struct Repo<'a> {
    org: &'a Org,
    name: String,
}

fn main() {
    let opt = Opt::from_args();

    let org = Org {
        name: opt.org,
    };

    let repo = Repo {
        org: &org,
        name: opt.repo,
    };

    match list_releases(&repo) {
        Ok(res) => println!("{:?}", res),
        Err(err) => println!("{:?}", err),
    }

    println!("Hello, world!");
}

fn list_releases(repo: & Repo) -> Result<Vec<Release>, reqwest::Error> {
    let client = reqwest::Client::new();
    let s = format!("https://api.github.com/repos/{}/{}/releases", repo.org.name, repo.name);
    match Url::parse(&s) {
        Ok(url) => {
            let req = client.get(url);
            let out: Vec<Release> = req.send()?.json()?;
            return Ok(out);
        }
        Err(err) => {
            panic!("{:?}", err);
        }
    }



        /*
    if let Ok(mut res) = req.send() {
        println!("{:?}", res);
        match res.json() {
            Ok(json) => {
                let out: Vec<Response> = json;
                return out;
            },
            Err(err) => {
                println!("{:?}", err);
                return err;
            }
        }
    }
    */
}
