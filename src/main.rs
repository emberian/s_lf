#![allow(non_camel_case_types)] // don't cramp my style >:|

use serde_derive::{Serialize, Deserialize};
use argh::FromArgs;
use std::collections::{HashMap};
use std::cell::Cell;

type DateTime = chrono::DateTime<chrono::Utc>;

/*

s_lf is notes to self crossed with tiddlywiki: a way to spurt out bits and bobs of text during the course of your life, or converse with yourself, or keep tracking of things, etc.  periodically one
desires reflection or analysis; s_lf provides a built-in wiki-like bag of documents. you can rope
some messages together into a commentary, or embed in the middle of a daily journal, or build a graph, or drop in images or etc. s_lf lives with you, wherever you want to be with it digitally.
all of your data is always there, to the extent possible with networking.

set up different channels for different purposes.

eventually: multiuser. share "servers" w/others?

git repos?

what API  to expose to dynamic content, also? definitely cap based somehow :)

*/

#[derive(Serialize, Deserialize)]
struct DynamicContent {
  // TODO: some wasm type? literally just a module?
  program_blob: (),
  // frozen heap etc?
  program_state: (),
  // TODO: rest? cap table or sth?
}

// copy some stuff from web that we won't bother to interpose w caps

#[derive(Serialize, Deserialize)]
enum Authority {
  MediaDevices,
  Network,
  Storage,
  /// access to the s_lf api is itself an authority. gets you into capland
  S_lfApi,
}

#[derive(Serialize, Deserialize)]
struct ContentPolicy {
  authorities: Vec<Authority>,
}

#[derive(Serialize, Deserialize)]
struct CachedImgRef {
  url: url::Url,
}

// i feel like im going to regret this document model

#[derive(Serialize, Deserialize)]
struct Person {
  id: u64,
  #[serde(skip)]
  card: Option<vcard::VCard>,
  profile: Option<MsgId>,
  // is this just something from their vcard or do we need to fetch? (probably fetch)
  cached_profile_image: Vec<u8>,
}

type MsgId = u64;

type DMId = u64;

#[derive(Serialize, Deserialize)]
enum MsgContent {
  #[serde(skip)]
  /// a hyperlink, but with a loaded preview (prompt like keybase to emit an edit action to replace link with embed)
  Embed(webpage::Webpage),
  /// reference another message
  Ref(MsgId),
  /// plain ole text. "paragraph", one might say.
  Txt(String),
  /// completely unstructured data
  Data(Vec<u8>),
  /// an image! splat that shit in there!
  Img(Vec<u8>),
  /// sometimes ya just gotta link externally, ya know?
  /// i think s_lf is going to fetch and archive mentioned sites.
  /// maybe even try and get a few pages deep, heuristically somehow? idk.
  Hyperlink {
    url: url::Url,
    text: Option<String>,
  },
  /// look ma! i'm actually just a list of other messages!
  Composite(Vec<MsgId>),
  /// basically a plugin, a wasm blob plus some default permissions (render visibly! editable!)
  Dynamic(DynamicContent, ContentPolicy),
  /// literally just plop an html literal in the output why not. what could go wrong.
  Html(Vec<u8>),
  /// attach a file. keep the mime for icon? idk.
  #[serde(skip)]
  File(mime::Mime, Vec<u8>), // todo: dataref? reference into some independent Filesystem?
  /// a person :o did you know you are a person? it took me some time to figure that out. this app helps manage people.
  Person(Person),
  /// reference some academic work. in the background we'll scrape scihub for the content, and possibly run some analysis and at least generate a thumbnail
  Doi(String),
  /// shitty dropbox, basically?
  Filesystem(Vec<u8>), // todo: dataref. stored as a tarball???? idfk.
  /// sometimes one refers to dates
  Date(/* as input */ String, chrono::DateTime<chrono::Utc>),
  /// sometimes one has temporal obligations (appointments etc), dunno how to represent? ical sth?
  TemporalObligation(),
  /// link to a dm explcitly
  Dm(DMId),
  //Transclude(MsgId, MsgId), // maybe?
}

struct Msg {
  id: MsgId,
  content: MsgContent,
  authored_date: DateTime,
}

trait Storage {
  fn save(&self, state: &dyn erased_serde::Serialize) -> anyhow::Result<()>;
}

#[derive(Serialize, Deserialize)]
struct DM {
  name: String,
  scroll_state: (MsgId, u64), // wtf should this be. a msg id for the top, plus a pixel (???) offset?
  msgs_in_sequence: Vec<MsgId>,
}

#[derive(Serialize, Deserialize)]
struct JournalPage {
  title: String,
  msgs_in_sequence: Vec<MsgId>,
}

/// all s_lf state
#[derive(Serialize, Deserialize)]
struct S_lf {
  // for counting things. anything nonce'd is unique. includes all id's.
  nonce: Cell<u64>,
  // some id'd objects get exported as uuids. track them here.
  #[serde(skip)]
  export_nonce_map: HashMap<uuid::Uuid, u64>, 
  // for greeting / rendering
  username: String,
  pronouns: (String, String, String),
  me: Person, // i'm a human being goddammit, i have value!
  dm_storage: Vec<DM>,
  journal_storage: Vec<JournalPage>,
  doi_storage: HashMap<String, Vec<u8>>, // todo: actual type here for the pdf n metadata etc
  #[serde(skip)]
  storage_backend: Option<Box<dyn Storage>>, // we use this for persisting.
}

impl S_lf {
  fn gensym(&self) -> u64 {
    // ლ(ಠ益ಠლ) y u unstable Cell::update
    // self.nonce.update(|x| x + 1)
    let old = self.nonce.get();
    let new = old + 1;
    self.nonce.set(new);
    new
  }

  fn save(&self) -> anyhow::Result<()> {
    self.storage_backend.as_ref().map_or(Ok(()), |x| x.save(self))
  }
}

#[derive(FromArgs)]
/// an exocortex for one.
struct S_lfParams {
  #[argh(option)]
  /// database location for the state store (default g)
  db_location: Option<String>,
  /// where to load from at startup, and persist edits
  #[argh(option)]
  sync_address: String,
}

fn main() {
  // todo: hella cool transition animation filling up the whole terminal
  let args: S_lfParams = argh::from_env();

  // open the state
  println!("welcome to s_lf.")

  // todo: tui w s_lf with blinking _ in the bottom left? idk peripiheral movement bad?
}