#[macro_use]
extern crate serde_derive;
extern crate structopt;
extern crate reqwest;
extern crate serde;

use std::fs::File;
use std::env;
use std::collections::HashMap;
use structopt::StructOpt;
use reqwest::Url;
use reqwest::header::CONTENT_TYPE;

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

    #[structopt(subcommand)]
    cmd: Cmd,
}

#[derive(StructOpt, Debug)]
enum Cmd {
    #[structopt(name = "list")]
    /// List releases for a repo
    List {},

    #[structopt(name = "get")]
    /// Get a particular release
    Get {
        /// Semver for this release
        #[structopt(short, long)]
        version: String,
    },

    #[structopt(name = "create")]
    /// Create a new release
    Create {
        /// Semver for this release
        #[structopt(short, long)]
        version: String,

        /// Git commit or branch associated with this release
        #[structopt(short, long)]
        target: Option<String>,
    },

    #[structopt(name = "upload")]
    /// Upload an asset to a release
    Upload {
        /// Semver for this release
        #[structopt(short, long)]
        version: String,

        /// Name of the asset
        #[structopt(short, long)]
        file: String,
    },
}

#[derive(Deserialize, Debug)]
struct Release {
    id: usize,
    url: String,
    upload_url: String,
    tag_name: String,
    created_at: String,
    published_at: String,
}

#[derive(Deserialize, Debug)]
struct Asset {
    id: usize,
    url: String,
    browser_download_url: String,
    name: String,
    label: String,
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

    match &opt.cmd {
        Cmd::List {} => {
            match list_releases(&repo) {
                Ok(res) => println!("{:?}", res),
                Err(err) => panic!("{:?}", err),
            }
        },
        Cmd::Get { version, } => {
            match get_release(&repo, &version) {
                Ok(res) => println!("{:?}", res),
                Err(err) => panic!("{:?}", err),
            }
        },
        Cmd::Create { version, target } => {
            println!("{:?}", version);
            println!("{:?}", target);

            match create_release(&repo, &version, &target) {
                Ok(res) => println!("{:?}", res),
                Err(err) => panic!("{:?}", err),
            }
        },
        Cmd::Upload { version, file } => {
            match upload(&repo, &version, &file) {
                Ok(res) => println!("{:?}", res),
                Err(err) => panic!("{:?}", err),
            }
        }
    }

    println!("Hello, world!");
}

fn get_release(repo: &Repo, version: &String) -> Result<Release, reqwest::Error> {
    let client = reqwest::Client::new();
    let s = format!("https://api.github.com/repos/{}/{}/releases/tags/{}", repo.org.name, repo.name, version);
    match Url::parse(&s) {
        Ok(url) => {
            let req = client.get(url);
            let out: Release = req.send()?.json()?;
            return Ok(out);
        }
        Err(err) => {
            panic!("{:?}", err);
        }
    }
}

fn create_release(repo: & Repo, version: &String, target: &Option<String>) -> Result<Release, reqwest::Error> {
    let client = reqwest::Client::new();
    let s = format!("https://api.github.com/repos/{}/{}/releases", repo.org.name, repo.name);
    let token = env::var("GITHUB_TOKEN").unwrap();
    println!("{:?}", token);

    let mut map = HashMap::new();
    map.insert("tag_name", version);

    if let Some(tgt) = target {
        map.insert("target_commitish", tgt);
    }

    match Url::parse(&s) {
        Ok(url) => {
            let req = client.post(url).bearer_auth(token).json(&map);
            let out: Release = req.send()?.json()?;
            return Ok(out);
        }
        Err(err) => {
            panic!("{:?}", err);
        }
    }
}

fn upload(repo: & Repo, version: &String, file: &String) -> Result<Asset, reqwest::Error> {
    let client = reqwest::Client::new();
    let id = get_release(repo, version)?.id;
    let s = format!("https://uploads.github.com/repos/{}/{}/releases/{}/assets", repo.org.name, repo.name, id);
    println!("{:?}", s);
    let token = env::var("GITHUB_TOKEN").unwrap();

    let f = match File::open(file) {
        Ok(f) => f,
        Err(err) => panic!("{:?}", err),
    };

    match Url::parse(&s) {
        Ok(url) => {
            let req = client.post(url).bearer_auth(token).header(CONTENT_TYPE, "multipart/form-data").query(&[("name", file)]).body(f);
            println!("{:?}", req);
            let out: Asset = req.send()?.json()?;
            return Ok(out);
        }
        Err(err) => {
            panic!("{:?}", err);
        }
    }
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
}
