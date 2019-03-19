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

fn get_release(repo: &Repo, version: &String) -> Result<Release, Error> {
    let s = format!("https://api.github.com/repos/{}/{}/releases/tags/{}", repo.org.name, repo.name, version);

    match authenticated_request(reqwest::Method::GET, &s)?.send() {
        Ok(mut res) => match res.json() {
            Ok(json) => {
                let out: Release = json;
                return Ok(out);
            },
            Err(err) => return Err(Error { message: format!("get: Error extracting json: {}", err.to_string()) }),
        },
        Err(err) => {
            panic!("{:?}", err);
        }
    }
}

#[derive(Debug)]
struct Error {
    message: String,
}

fn authenticated_request(method: reqwest::Method, url: &str) -> Result<reqwest::RequestBuilder, Error> {
    let client = reqwest::Client::new();
    let token = env::var("GITHUB_TOKEN").unwrap();
    println!("{:?}", token);

    match Url::parse(&url) {
        Ok(u) => {
            return Ok(client.request(method, u).bearer_auth(token));
        },
        Err(err) => return Err(Error { message: err.to_string() }),
    }
}

fn create_release(repo: & Repo, version: &String, target: &Option<String>) -> Result<Release, Error> {
    let s = format!("https://api.github.com/repos/{}/{}/releases", repo.org.name, repo.name);

    let mut map = HashMap::new();
    map.insert("tag_name", version);

    if let Some(tgt) = target {
        map.insert("target_commitish", tgt);
    }

    let client = authenticated_request(reqwest::Method::POST, &s)?;

    let req = client.json(&map);
    match req.send() {
        Ok(mut res) => {
            match res.json() {
                Ok(json) => {
                    let out: Release = json;
                    return Ok(out);
                },
                Err(err) => return Err(Error { message: format!("create: Error parsing json in response: {}", err.to_string()) }),
            }
        },
        Err(err) => return Err(Error { message: format!("create: Error making request: {}", err.to_string()) }),
    }
}

fn upload(repo: & Repo, version: &String, file: &String) -> Result<Asset, Error> {
    let id = get_release(repo, version)?.id;
    let s = format!("https://uploads.github.com/repos/{}/{}/releases/{}/assets", repo.org.name, repo.name, id);

    let f = match File::open(file) {
        Ok(f) => f,
        Err(err) => panic!("{:?}", err),
    };

    match authenticated_request(reqwest::Method::POST, &s)?.header(CONTENT_TYPE, "multipart/form-data").query(&[("name", file)]).body(f).send() {
        Ok(mut res) => {
            match res.json() {
                Ok(json) => {
                    let out: Asset = json;
                    return Ok(out);
                }
                Err(err) => return Err(Error { message: format!("upload: Error parsing json in response: {}", err.to_string()) }),
            }
        },
        Err(err) => return Err(Error { message: format!("upload: Error making request: {}", err.to_string()) }),
    }
}

fn list_releases(repo: & Repo) -> Result<Vec<Release>, Error> {
    let s = format!("https://api.github.com/repos/{}/{}/releases", repo.org.name, repo.name);

    match authenticated_request(reqwest::Method::GET, &s)?.send() {
        Ok(mut res) => match res.json() {
            Ok(json) => {
                let out: Vec<Release> = json;
                return Ok(out);
            },
            Err(err) => return Err(Error { message: format!("list_releases: Error parsing json in response: {}", err.to_string()) }),
        },
        Err(err) => return Err(Error { message: format!("list_releases: Error making request: {}", err.to_string()) }),

    }
}
