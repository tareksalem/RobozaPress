use crate::utils::Error;
use futures::future;
use jfs::Store;
use link_preview;
use once_cell::sync::Lazy;
use regex::RegexBuilder;
use reqwest;
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::fs;
use std::fs::OpenOptions;
use std::io::BufReader;
use std::io::Write;
use std::path::Path;
use std::sync::Mutex;
use std::sync::MutexGuard;
use std::time::Duration;
use tokio::time::timeout;
use crate::config;

static BOOKMARK_API: Lazy<Mutex<BookmarkApi>> = Lazy::new(|| Mutex::new(BookmarkApi::new()));

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BookmarkFile {
    checksum: String,
    roots: BookmarksRoot,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BookmarksRoot {
    bookmark_bar: BookmarkBar,
    other: BookmarkBar,
    synced: BookmarkBar,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BookmarkBar {
    children: Option<Vec<BookmarksItem>>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum BookmarkType {
    folder,
    url,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BookmarksItem {
    pub date_added: String,
    pub guid: String,
    pub id: String,
    pub name: String,
    pub r#type: BookmarkType,
    pub url: Option<String>,
    pub children: Option<Vec<BookmarksItem>>,
}
#[derive(PartialEq)]
pub enum ChildLevel {
    Outer,
    Inner,
}
impl BookmarksItem {
    pub fn flatten(
        nested: Vec<BookmarksItem>,
        acc_arr: &mut Vec<BookmarksItem>,
        level: ChildLevel,
    ) -> Vec<BookmarksItem> {
        // loop throw the parent array
        nested.into_iter().for_each(|item| {
            // check if the item has children
            if item.r#type == BookmarkType::folder && item.children.is_some() {
                // check if the child has sub children
                let found_sub_child = item
                    .children
                    .as_ref()
                    .unwrap()
                    .into_iter()
                    .find(|child| child.children.is_some());
                if found_sub_child.is_some() {
                    // in this case do recursive to get the inner one
                    BookmarksItem::flatten(
                        item.children.as_ref().unwrap().to_vec(),
                        acc_arr.as_mut(),
                        ChildLevel::Inner,
                    );
                    let mut cloned_item = item.clone();
                    // reset the children to get children with url type only not folders to avoid children duplication
                    cloned_item.children = Some(
                        cloned_item
                            .children
                            .unwrap()
                            .into_iter()
                            .filter(|child_item| child_item.r#type == BookmarkType::url)
                            .collect(),
                    );
                    // push the cloned item into the accumulated array
                    acc_arr.push(cloned_item);
                } else {
                    // push the cloned item into the accumulated array
                    acc_arr.push(item.clone());
                }
            }
            // check if the item in the top level, no sub children and with url type
            else if item.r#type == BookmarkType::url
                && item.children.is_none()
                && level == ChildLevel::Outer
            {
                // push the cloned item into the accumulated array
                acc_arr.push(item.clone());
            }
        });
        acc_arr.to_vec()
    }
}
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BookmarkCategory {
    pub name: String,
    pub id: String,
}

impl BookmarkCategory{
    pub fn default() -> Self{
        BookmarkCategory{
            id: String::from("non categorized"),
            name: String::from("non categorized"),
        }
    }
}
// #[derive(Debug)]
pub struct BookmarkApi {
    bookmark_file: Option<BookmarkFile>,
    bookmarks: Option<Vec<BookmarksItem>>,
    categories: Vec<BookmarkCategory>,
    // storage: Option<File>,
    db: Store,
}
impl BookmarkApi {
    fn bootstrap() -> Store {
        fs::create_dir_all(
            Path::new(
                dirs::cache_dir()
                    .unwrap_or(Path::new("").to_path_buf())
                    .as_path(),
            )
            .join(config::CACHE_MAIN_DIR),
        )
        .ok();
        let dir_path = Path::new(dirs::cache_dir().unwrap().as_path()).join(config::CACHE_IMG_PATH);
        fs::create_dir_all(&dir_path).ok();
        let mut cfg = jfs::Config::default();
        cfg.single = true;
        cfg.pretty = true;
        Store::new_with_cfg(
            Path::new(dirs::cache_dir().unwrap().as_path()).join(config::CACHE_FILE_PATH),
            cfg,
        )
        .unwrap()
    }
    pub fn new() -> Self {
        BookmarkApi {
            bookmark_file: None,
            bookmarks: None,
            categories: Vec::new(),
            db: Self::bootstrap(),
        }
    }
    pub fn init<'a>() -> MutexGuard<'a, Self> {
        BOOKMARK_API.lock().unwrap()
    }
    pub fn read_bookmarks_from_file(&mut self) -> &Self {
        let file = fs::File::open(config::get_bookmarks_path()).unwrap();
        let reader = BufReader::new(file);
        let book_marks_file: BookmarkFile = serde_json::from_reader(reader).unwrap();
        self.bookmark_file = Some(book_marks_file);
        self
    }
    pub fn get_bookmark_file(&mut self) -> &mut Option<BookmarkFile> {
        if self.bookmark_file.is_none() {
            self.read_bookmarks_from_file();
        }
        &mut self.bookmark_file
    }
    pub fn get_raw_bookmarks(&mut self) -> &Vec<BookmarksItem> {
        match self.bookmarks {
            None => {
                let mut new_bookmarks: Vec<BookmarksItem> = Vec::new();
                new_bookmarks.append(
                    self.get_bookmark_file()
                        .as_mut()
                        .unwrap()
                        .roots
                        .bookmark_bar
                        .children
                        .as_mut()
                        .unwrap(),
                );
                new_bookmarks.append(
                    self.get_bookmark_file()
                        .as_mut()
                        .unwrap()
                        .roots
                        .other
                        .children
                        .as_mut()
                        .unwrap(),
                );
                new_bookmarks.append(
                    self.get_bookmark_file()
                        .as_mut()
                        .unwrap()
                        .roots
                        .synced
                        .children
                        .as_mut()
                        .unwrap(),
                );
                self.bookmarks = Some(new_bookmarks);
            }
            Some(_) => (),
        };
        self.bookmarks = Some(BookmarksItem::flatten(
            self.bookmarks.as_ref().unwrap_or(&Vec::new()).to_vec(),
            Vec::new().as_mut(),
            ChildLevel::Outer,
        ));
        self.bookmarks.as_ref().unwrap()
    }
    pub fn get_categories(&mut self) -> &Vec<BookmarkCategory> {
        if self.categories.len() == 0 {
            let bookmarks = self.get_raw_bookmarks();
            let mut categories: Vec<BookmarkCategory> = bookmarks
                .into_iter()
                .filter_map(|bookmark| {
                    if bookmark.r#type == BookmarkType::folder {
                        Some(BookmarkCategory {
                            name: bookmark.name.clone(),
                            id: bookmark.id.clone(),
                        })
                    } else {
                        None
                    }
                })
                .collect();
            self.categories.push(BookmarkCategory::default());
            self.categories.append(categories.as_mut())
        }
        self.categories.as_ref()
    }
    pub fn get_category(&mut self, cat_id: &str) -> Option<&BookmarkCategory> {
        if self.categories.len() == 0 {
            let bookmarks = self.get_raw_bookmarks();
            let mut categories: Vec<BookmarkCategory> = bookmarks
                .into_iter()
                .filter_map(|bookmark| {
                    if bookmark.r#type == BookmarkType::folder {
                        Some(BookmarkCategory {
                            name: bookmark.name.clone(),
                            id: bookmark.id.clone(),
                        })
                    } else {
                        None
                    }
                })
                .collect();
            self.categories.append(categories.as_mut())
        }
        let cat = self.get_categories().into_iter().find(|item| item.id == cat_id);
        cat
    }
    pub fn filter_marks_by_category(&mut self, cat_id: Option<&str>) -> Vec<BookmarksItem> {
        let default_list = &Vec::new();
        let category = if cat_id.is_some() {
            Some(self
                .bookmarks
                .as_ref()
                .unwrap_or(default_list)
                .into_iter()
                .find(|bookmark| bookmark.id.eq(&cat_id.unwrap())))
        } else {
            None
        };
        if category.is_some() {
            if category.unwrap().is_some() {
                category
                    .unwrap()
                    .unwrap()
                    .children
                    .as_ref()
                    .unwrap_or(&Vec::new())
                    .to_vec()
            } else {
                Vec::new()
            }
        } else {
            let res: Vec<BookmarksItem> = self.bookmarks.as_ref().unwrap_or(&Vec::new()).into_iter()
            .map(|item| item.clone())
            .filter(|item| {
                &item.r#type == &BookmarkType::url
            }).collect();
            res.to_vec()
        }
    }
    pub async fn sync_all() {
        let cats = {
            Self::reset_bookmarks();
            let mut bookmark_api = Self::init();
            let cats = bookmark_api.get_categories().clone();
            cats
        };
        future::try_join_all(cats.into_iter().map(Self::sync_category_bookmarks)).await.ok();
        Self::sync_root_bookmarks().await.ok();
    }
    fn reset_bookmarks() {
        let mut bookmark_api = Self::init();
        bookmark_api.categories = Vec::new();
        bookmark_api.bookmarks = None;
        bookmark_api.bookmark_file = None;
        bookmark_api.get_categories();
    }
    pub async fn sync_root_bookmarks() -> Result<(), Error> {
        let bookmark_items: Vec<BookmarksItem> = {
            let mut bookmark_api: MutexGuard<BookmarkApi> = Self::init();
            bookmark_api.filter_marks_by_category(None)
        };
        let cat = BookmarkCategory::default();
        future::try_join_all(bookmark_items.into_iter().map(|item| Self::sync_bookmark(item, cat.clone())))
            .await
            .ok();
        Ok(())
    }
    pub async fn sync_category_bookmarks(cat: BookmarkCategory) -> Result<(), Error> {
        let bookmark_items: Vec<BookmarksItem> = {
            let mut bookmark_api: MutexGuard<BookmarkApi> = Self::init();
            bookmark_api.filter_marks_by_category(Some(&cat.id))
        };
        future::try_join_all(bookmark_items.into_iter().map(|item| Self::sync_bookmark(item, cat.clone())))
            .await
            .ok();
        Ok(())
    }
    async fn sync_bookmark(bookmark: BookmarksItem, cat: BookmarkCategory) -> Result<(), Error> {
        let mut item_exists: bool = false;
        {
            let bookmark_api = Self::init();
            let db = &bookmark_api.db;
            let item = db.get::<MarkData>(&bookmark.id);
            if item.is_ok() {
                item_exists = true;
            }
        }
        if !item_exists {
            Self::perform_scrape(&bookmark, cat).await;
        }
        Ok(())
    }
    async fn perform_scrape(bookmark: &BookmarksItem, cat: BookmarkCategory) {
        let res = timeout(Duration::from_secs(30), Self::scrap_bookmark(&bookmark, &cat)).await;
        if res.is_ok() {
            if res.as_ref().unwrap().is_ok() {
                let mark_data = res.as_ref().unwrap().as_ref().unwrap();
                let bookmark_api = Self::init();
                let db = &bookmark_api.db;
                db.save_with_id(mark_data, &bookmark.id).ok();
            }
        } else {
            let res = timeout(Duration::from_secs(30), Self::scrap_bookmark(&bookmark, &cat)).await;
            if res.is_ok() {
                if res.as_ref().unwrap().is_ok() {
                    let mark_data = res.as_ref().unwrap().as_ref().unwrap();
                    let bookmark_api = Self::init();
                    let db = &bookmark_api.db;
                    db.save_with_id(mark_data, &bookmark.id).ok();
                }
            }
        }
    }
    fn cache_bookmark_img(img_data: &Vec<u8>, img_path: &str) -> Result<String, Error> {
        let dir_path = Path::new(dirs::cache_dir().unwrap().as_path()).join(config::CACHE_IMG_PATH);
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .open(&dir_path.join(img_path));
        if file.is_ok() {
            let cache_result = file.unwrap().write_all(img_data);
            if cache_result.is_ok() {
                Ok(img_path.to_string())
            } else {
                Ok(config::DEFAULT_IMG_PATH.to_string())
            }
        } else {
            Ok(config::DEFAULT_IMG_PATH.to_string())
        }
    }
    fn get_extension_from_filename(filename: &str) -> Option<&str> {
        Path::new(filename).extension().and_then(OsStr::to_str)
    }
    async fn scrap_bookmark(item: &BookmarksItem, cat: &BookmarkCategory) -> Result<MarkData, Error> {
        let link = item.url.clone();
        let img_url: String = if link.is_some() {
            let img_url = {
                let link_result = link_preview::fetch::fetch(link.unwrap().as_str()).await;
                if link_result.is_ok() {
                    let img_url =
                        link_preview::LinkPreview::find_first_image_url(&link_result.unwrap());
                    img_url
                } else {
                    None
                }
            };
            if img_url.is_some() {
                let img_data = Self::fetch_image(img_url.as_ref().unwrap().as_str()).await;
                let mut img_name = String::from(&item.id);
                img_name.push('.');
                img_name.push_str(
                    Self::get_extension_from_filename(img_url.as_ref().unwrap().as_str())
                        .unwrap_or("png"),
                );
                let min_length: usize = 0;
                if img_data.is_ok() && img_data.as_ref().unwrap().len() > min_length {
                    let saved_path =
                        Self::cache_bookmark_img(img_data.as_ref().unwrap(), &img_name).unwrap();
                    saved_path
                } else {
                    config::DEFAULT_IMG_PATH.to_string()
                }
            } else {
                config::DEFAULT_IMG_PATH.to_string()
            }
        } else {
            config::DEFAULT_IMG_PATH.to_string()
        };
        Ok(MarkData::new(MarkData {
            title: item.name.clone(),
            description: format!(""),
            image: img_url,
            content: item.id.clone(),
            image_data: None,
            category: cat.to_owned(),
            link: item.url.as_ref().unwrap().clone()
        }))
    }
    pub async fn fetch_image(image_url: &str) -> Result<Vec<u8>, Error> {
        let url = format!("{}", image_url);
        let bytes = {
            let client = reqwest::Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .ok()
                .unwrap();
            let result = client.get(&url).send().await;
            if result.is_ok() {
                result.unwrap().bytes().await?.as_ref().to_vec()
            } else {
                Vec::new()
            }
        };
        Ok(bytes)
    }
    pub async fn load_marks(cat: BookmarkCategory) -> Result<Vec<MarkData>, Error> {
        let mut items: Vec<MarkData> = Vec::new();
        let bookmark_api = Self::init();
        let db: &Store = &bookmark_api.db;
        let result = db.all::<MarkData>();
        if result.is_ok() {
            let all_data = result.unwrap();
            items = all_data
                .into_iter()
                .filter(|(_, item)| {
                    cat.id.as_str().eq(item.category.id.as_str())
                })
                .map(|(_, item)| item)
                .collect();
        } else {
        }
        Ok(items)
    }
    pub fn load_marks_sync(last_index: usize, category: Option<BookmarkCategory>, search: Option<String>) -> Result<(bool, Vec<MarkData>), Error> {
        let bookmark_api = Self::init();
        let db: &Store = &bookmark_api.db;
        let result = db.all::<MarkData>();
        let (has_more, items): (bool, Vec<MarkData>) = QueryBuilder::new(result)
        .filter_by_category(category)
        .search(search)
        .paginate(last_index)
        .result();
        Ok((has_more, items))
    }
    pub async fn perform_load() -> Result<(), Error> {
        Ok(())
    }
    pub async fn flush_all_resync() -> Result<(), Error> {
        {
            Self::reset_bookmarks();
            let mut bookmark_api = Self::init();
            fs::remove_dir_all(Path::new(dirs::cache_dir().unwrap().as_path()).join(config::CACHE_MAIN_DIR)).ok();
            bookmark_api.db = Self::bootstrap();
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MarkData {
    pub title: String,
    pub description: String,
    pub content: String,
    pub image: String,
    pub image_data: Option<Vec<u8>>,
    pub category: BookmarkCategory,
    pub link: String,
}

impl MarkData {
    pub fn new(data: MarkData) -> Self {
        Self { ..data }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MarkMeta{
    pub mark: Option<MarkData>,
    pub index: usize,
    pub all_len: usize,
    pub has_more: bool
}

pub trait IQueryBuilder{
    fn new(items: Result<BTreeMap<String, MarkData>, std::io::Error>) -> Self;
    fn paginate(&mut self, last_index: usize) -> &mut Self;
    fn filter_by_category(&mut self, cat: Option<BookmarkCategory>) -> &mut Self;
    fn search(&mut self, txt: Option<String>) -> &mut Self;
    // fn append_cat_to_list(&mut self) -> &mut Self;
    fn result (&self) -> (bool, Vec<MarkData>);
}

pub struct QueryBuilder{
    items: Vec<MarkData>,
    has_more: bool,
}
impl IQueryBuilder for QueryBuilder {
    fn new(items: Result<BTreeMap<String, MarkData>, std::io::Error>) -> Self{

        Self {
            items: items.unwrap_or(BTreeMap::<String, MarkData>::new()).into_iter().map(|i| i.1).collect(),
            has_more: false
        }
    }

    fn paginate(&mut self, last_index: usize) -> &mut Self{
        let offset: usize = 4;
        let next_slice = offset + last_index;
            if self.items.len() >= last_index {
                if self.items.len() >= next_slice {
                    if self.items.len() >= next_slice + offset {
                        self.has_more = true;
                    } else {
                        self.has_more = false;
                    }
                    self.items = self.items.as_slice()[last_index..next_slice].to_vec();
                } else {
                    self.has_more = false;
                }
            } else {
                self.items = Vec::new();
                self.has_more = false;
            }
        self
    }

    fn filter_by_category(&mut self, cat: Option<BookmarkCategory>) -> &mut Self {
        match cat {
            Some(cat) => self.items.retain(|item| {
                cat.id.as_str().eq(item.category.id.as_str())
            }),
            None => ()
        };
        self
    }

    fn search(&mut self, txt: Option<String>) -> &mut Self {
        match txt {
            Some(val) => {
                let re = RegexBuilder::new(format!("({})", &val).as_str())
                .multi_line(true).case_insensitive(true).ignore_whitespace(true).build().unwrap();
                self.items.retain(|item| re.is_match(&item.title) || re.is_match(&item.description));
            },
            None => ()
        };
        self
    }
    // fn append_cat_to_list(&mut self) -> &Self {
    //     self.items.iter_mut().for_each(|item| {

    //     })
    // }
    fn result(&self) -> (bool, Vec<MarkData>) {
        (self.has_more, self.items.to_vec())
    }
}

