#[macro_use]
extern crate serde_derive;
extern crate reqwest;
extern crate serde;
extern crate structopt;

use reqwest::header::CONTENT_TYPE;
use reqwest::Url;
use std::collections::HashMap;
use std::fs::File;
use std::{env, error, fmt};
use structopt::StructOpt;

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
    assets: Vec<Asset>,
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

#[derive(Debug)]
enum Error {
    Reqwest(reqwest::Error),
    Request(u16),
    Url(reqwest::UrlError),
    Io(std::io::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Reqwest(ref err) => write!(f, "Reqwest error: {}", err),
            Error::Request(ref err) => write!(f, "Request error: {}", err),
            Error::Url(ref err) => write!(f, "Url error: {}", err),
            Error::Io(ref err) => write!(f, "IO error: {}", err),
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Reqwest(ref err) => err.description(),
            Error::Request(ref err) => "foobar",
            Error::Url(ref err) => err.description(),
            Error::Io(ref err) => err.description(),
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            Error::Reqwest(ref err) => Some(err),
            Error::Request(ref err) => None,
            Error::Url(ref err) => Some(err),
            Error::Io(ref err) => Some(err),
        }
    }
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Error {
        Error::Reqwest(err)
    }
}

impl From<reqwest::UrlError> for Error {
    fn from(err: reqwest::UrlError) -> Error {
        Error::Url(err)
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Error {
        Error::Io(err)
    }
}

fn main() {
    let opt = Opt::from_args();

    let org = Org { name: opt.org };

    let repo = Repo {
        org: &org,
        name: opt.repo,
    };

    match &opt.cmd {
        Cmd::List {} => run(list_releases(&repo)),
        Cmd::Get { version } => run(get_release(&repo, &version)),
        Cmd::Create { version, target } => run(create_release(&repo, &version, &target)),
        Cmd::Upload { version, file } => run(upload(&repo, &version, &file)),
    };
}

fn run<T: std::fmt::Debug>(res: Result<T, Error>) {
    std::process::exit(match res {
        Ok(out) => {
            eprintln!("{:?}", out);
            0
        }
        Err(err) => {
            eprintln!("{}", err);
            1
        }
    });
}

fn get_release(repo: &Repo, version: &String) -> Result<Release, Error> {
    let s = format!(
        "https://api.github.com/repos/{}/{}/releases/tags/{}",
        repo.org.name, repo.name, version
    );

    let res = get(&s)?.send()?;
    let mut succ = success_handler(res)?;
    let json = succ.json()?;
    let out: Release = json;
    Ok(out)
}

fn authenticated_request(
    method: reqwest::Method,
    url: &str,
) -> Result<reqwest::RequestBuilder, Error> {
    let client = reqwest::Client::new();
    let token = env::var("GITHUB_TOKEN").expect("missing env var GITHUB_TOKEN");
    println!("{:?}", &url);

    let u = Url::parse(&url)?;
    Ok(client.request(method, u).bearer_auth(token))
}

fn success_handler(res: reqwest::Response) -> Result<reqwest::Response, Error> {
    if !res.status().is_success() {
        return Err(Error::Request(res.status().as_u16()));
    }

    Ok(res)
}

fn get(url: &str) -> Result<reqwest::RequestBuilder, Error> {
    authenticated_request(reqwest::Method::GET, url)
}

fn post(url: &str) -> Result<reqwest::RequestBuilder, Error> {
    authenticated_request(reqwest::Method::POST, url)
}

fn create_release(
    repo: &Repo,
    version: &String,
    target: &Option<String>,
) -> Result<Release, Error> {
    let s = format!(
        "https://api.github.com/repos/{}/{}/releases",
        repo.org.name, repo.name
    );

    let mut map = HashMap::new();
    map.insert("tag_name", version);

    if let Some(tgt) = target {
        map.insert("target_commitish", tgt);
    }

    let mut res = post(&s)?.json(&map).send()?;
    let json = res.json()?;
    let out: Release = json;
    Ok(out)
}

fn upload(repo: &Repo, version: &String, file: &String) -> Result<Asset, Error> {
    let id = get_release(repo, version)?.id;
    let s = format!(
        "https://uploads.github.com/repos/{}/{}/releases/{}/assets",
        repo.org.name, repo.name, id
    );

    let f = File::open(file)?;

    let res = post(&s)?
        .header(CONTENT_TYPE, "multipart/form-data")
        .query(&[("name", file)])
        .body(f)
        .send()?;

    let mut succ = success_handler(res)?;
    let out: Asset = succ.json()?;
    Ok(out)
}

fn list_releases(repo: &Repo) -> Result<Vec<Release>, Error> {
    let s = format!(
        "https://api.github.com/repos/{}/{}/releases",
        repo.org.name, repo.name
    );

    let mut res = get(&s)?.send()?;
    println!("{:?}", res);
    let out: Vec<Release> = res.json()?;

    Ok(out)
}
