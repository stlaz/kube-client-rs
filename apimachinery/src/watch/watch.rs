use std::result::Result;

use k8s_openapi::apimachinery::pkg::apis::meta::v1::WatchEvent;
use k8s_openapi::serde;

// WatchEvent associated object is:
//  * If Added or Modified: the new state of the object.
//  * If Deleted: the state of the object immediately before deletion.
//  * If Bookmark: the object (instance of a type being watched) where
//    only ResourceVersion field is set. On successful restart of watch from a
//    bookmark resourceVersion, client is guaranteed to not get repeat event
//    nor miss any events.
//  * If Error: *api.Status is recommended; other types may make sense
//    depending on context.
pub trait ResourceWatcher<T>
where
    T: serde::de::DeserializeOwned,
{
    fn stop(&self);
    fn next(&mut self) -> Result<WatchEvent<T>, String>;
}
