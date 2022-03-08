use std::{hash::{Hasher, Hash}, time::Duration};
use futures::{Future, StreamExt};
use iced::Subscription;
use iced_native::subscription;
use tokio::{self, time};
use crate::services::bookmark_api::{BookmarkApi, BookmarkCategory, MarkMeta};

pub struct MarksRecipe{
    category: Option<BookmarkCategory>,
    search: Option<String>,
    last_index: usize

}

impl<H, I> subscription::Recipe<H, I> for MarksRecipe
where H: Hasher,
{
    type Output = MarkMeta;
    fn hash(&self, state: &mut H) {
        std::any::TypeId::of::<Self>().hash(state);
    }
    fn stream(
        self: Box<Self>,
        _input: futures::stream::BoxStream<'static, I>,
    ) -> futures::stream::BoxStream<'static, Self::Output>
     {
         let (has_more, marks) = BookmarkApi::load_marks_sync(self.last_index,self.category, self.search).unwrap();
        let marks_length: usize = marks.len().clone();
        let last_index = self.last_index.clone();
        if marks.len() > 0 {
            let stream = futures::stream::iter(marks.into_iter());
            let stream = stream.enumerate();
            stream.then(move |(i, item)| async move {
                    MarkMeta{
                    mark: Some(item.clone()),
                    index: i + last_index,
                    all_len: last_index + marks_length,
                    has_more
                }
            }).boxed()
        } else {
            futures::stream::iter(0..1).map(|_| MarkMeta{mark: None, index: 0, all_len: 0, has_more: false}).boxed()
        }
    }
}

pub fn load_marks(last_index: &usize, cat: Option<BookmarkCategory>, search: Option<String>) -> Subscription<MarkMeta> {
    Subscription::from_recipe(MarksRecipe{
        category: cat,
        search,
        last_index: *last_index
    })
}
pub fn set_interval<F, Fut>(mut f: F, dur: Duration)
where
    F: Send + 'static + FnMut() -> Fut,
    Fut: Future<Output = ()> + Send + 'static,
{
    // Create stream of intervals.
    let mut interval = time::interval(dur);
    tokio::spawn(async move {
        // Skip the first tick at 0ms.
        interval.tick().await;
        loop {
            // Wait until next tick.
            interval.tick().await;
            // Spawn a task for this tick.
            tokio::spawn(f());
        }
    });
}