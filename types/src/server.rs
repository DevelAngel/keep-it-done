use crate::Task;
use crate::Uuid;

use ahash::{HashMap, RandomState};
use indexmap::{IndexMap, IndexSet};
use miette::Diagnostic;
use thiserror::Error;
use tokio::fs;

use std::collections::VecDeque;
use std::env;
use std::fs::File;
use std::io;
use std::io::BufReader;
use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct TaskCache {
    dir: PathBuf,
    tasks: DataMap,
    dirty: ChangeSet,
}

#[derive(Debug)]
pub struct TaskMutGuard<'a> {
    id: Uuid,
    task: &'a mut Task,
    dirty: &'a mut ChangeSet,
}

pub type StorageResult<T> = Result<T, StorageError>;

#[derive(Error, Diagnostic, Debug)]
pub enum StorageError {
    #[cfg(any(feature = "ssr-test-storagefail", feature = "ssr-test-rand"))]
    #[error("task {0}: failed to flush")]
    Test(Uuid),
    #[error("task {0}: failed to convert from/to JSON")]
    Json(Uuid, #[source] serde_json::Error),
    #[error("task {0}: failed to remove task file")]
    IoRemove(Uuid, #[source] io::Error),
    #[error("task {0}: failed to read directory {0}")]
    IoReadDir(PathBuf, #[source] io::Error),
    #[error("task {0}: failed to read entry of directory {0}")]
    IoReadDirEntry(PathBuf, #[source] io::Error),
    #[error("task {0}: failed to read entry of directory {0}")]
    IoOpen(Uuid, PathBuf, #[source] io::Error),
    #[error("task {0}: failed to write temporary task file")]
    IoWriteTemp(Uuid, #[source] io::Error),
    #[error("task {0}: failed to save task file (by renaming the temporary task file)")]
    IoRename(Uuid, #[source] io::Error),
    #[error("loading of {0}/{1} tasks failed")]
    LoadErrors(usize, usize, #[related] VecDeque<StorageError>),
    #[error("flushing of {0}/{1} tasks failed")]
    FlushErrors(usize, usize, #[related] Vec<StorageError>),
}

type DataMap = IndexMap<Uuid, Task, RandomState>;
type ChangeSet = IndexSet<Uuid, RandomState>;

impl TaskCache {
    fn has_valid_id(id: &Uuid) -> bool {
        id.get_version_num() == 7
    }

    fn create_id() -> Uuid {
        let id = Uuid::now_v7();
        assert_eq!(id.get_version_num(), 7, "invalid UUID");
        id
    }

    fn with_capacity(capacity: usize) -> Self {
        let path = env::current_dir().expect("CWD available");
        Self {
            dir: path.to_path_buf(),
            tasks: DataMap::with_capacity_and_hasher(capacity, RandomState::new()),
            dirty: ChangeSet::with_hasher(RandomState::new()),
        }
    }

    #[allow(dead_code)]
    fn with_dir<P: Into<PathBuf>>(mut self, dir: P) -> Self {
        self.dir = dir.into();
        assert!(self.dir.is_dir(), "invalid directory");
        self
    }

    pub fn remove(&mut self, id: &Uuid) -> bool {
        assert_eq!(id.get_version_num(), 7, "invalid UUID");
        self.dirty.insert(*id);
        self.tasks.shift_remove(id).is_some()
    }

    pub fn add(&mut self, task: Task) -> bool {
        let id = TaskCache::create_id();
        self.dirty.insert(id);
        self.tasks.insert(id, task).is_some()
    }

    /*
     * Note: get() is accessable via Deref trait
     */

    pub fn get_mut(&mut self, id: &Uuid) -> Option<TaskMutGuard<'_>> {
        let dirty = &mut self.dirty;
        self.tasks.get_mut(id).map(|task| TaskMutGuard {
            id: *id,
            dirty,
            task,
        })
    }

    pub async fn load(&mut self) -> StorageResult<usize> {
        let mut num = 0;
        let mut errors = VecDeque::new();
        for entry in std::fs::read_dir(self.dir.as_path())
            .map_err(|e| StorageError::IoReadDir(self.dir.clone(), e))?
        {
            match entry.map_err(|e| StorageError::IoReadDirEntry(self.dir.clone(), e)) {
                Ok(entry) => {
                    let entry = entry.path();
                    let Some(id) = Self::is_task_file(&entry) else {
                        // no task file
                        continue;
                    };
                    match Self::read_task_file(&id, &entry).await {
                        Ok(task) => {
                            self.tasks.insert(id, task);
                            num += 1;
                        }
                        Err(e) => {
                            errors.push_back(e);
                        }
                    }
                }
                Err(e) => {
                    errors.push_back(e);
                }
            }
        }

        if errors.is_empty() {
            Ok(num)
        } else {
            Err(StorageError::LoadErrors(errors.len(), num, errors))
        }
    }

    pub async fn flush(&mut self) -> StorageResult<usize> {
        let num = self.dirty.len();
        let mut errors = HashMap::default();

        for id in &self.dirty {
            if let Some(task) = self.tasks.get(id) {
                // Task found -> write file
                if let Err(e) = self.write_task_file(id, task).await {
                    errors.insert(*id, e);
                }
            } else {
                // Task deleted -> delete file
                if let Err(e) = self.delete_task_file(id).await {
                    errors.insert(*id, e);
                }
            }
        }

        if errors.is_empty() {
            self.dirty.clear();
            Ok(num)
        } else {
            // remove ids for successfully written files
            for id in self.dirty.clone().iter() {
                if !errors.contains_key(id) {
                    self.dirty.swap_remove(id);
                }
            }
            Err(StorageError::FlushErrors(
                self.dirty.len(),
                num,
                errors.into_values().collect(),
            ))
        }
    }

    fn is_task_file<P: AsRef<Path>>(file: P) -> Option<Uuid> {
        let file = file.as_ref();
        if !file.is_file() {
            return None;
        }

        let filename = file.file_prefix().unwrap().display().to_string();
        let Some(id) = filename.strip_prefix("task-") else {
            return None;
        };

        // we have found a task file
        let Ok(id) = Uuid::try_parse(id) else {
            tracing::warn!("file with invalid UUID detected: {}", file.display());
            return None;
        };
        if !Self::has_valid_id(&id) {
            tracing::warn!("file with invalid UUIDv7 detected: {}", file.display());
            return None;
        }
        Some(id)
    }

    async fn read_task_file<P: AsRef<Path>>(id: &Uuid, file: P) -> StorageResult<Task> {
        assert!(Self::has_valid_id(&id));
        let file = file.as_ref();
        assert!(file.is_file());
        let file =
            File::open(file).map_err(|e| StorageError::IoOpen(*id, file.to_path_buf(), e))?;
        let reader = BufReader::new(file);
        let task = serde_json::from_reader(reader).map_err(|e| StorageError::Json(*id, e))?;
        Ok(task)
    }

    async fn write_task_file(&self, id: &Uuid, task: &Task) -> StorageResult<()> {
        let path = Self::filename(&self.dir, id);
        let temp_path = path.with_extension("json.tmp");

        let task = serde_json::to_string_pretty(task).map_err(|e| StorageError::Json(*id, e))?;
        fs::write(&temp_path, task)
            .await
            .map_err(|e| StorageError::IoWriteTemp(*id, e))?;
        fs::rename(&temp_path, path)
            .await
            .map_err(|e| StorageError::IoRename(*id, e))?;

        cfg_if::cfg_if! {
            if #[cfg(feature = "ssr-test-storagefail")] {
                Self::return_error(id)
            } else if #[cfg(feature = "ssr-test-rand")] {
                Self::should_return_error(id)
            } else {
                Ok(())
            }
        }
    }

    #[cfg(feature = "ssr-test-storagefail")]
    #[allow(dead_code)]
    fn return_error(id: &Uuid) -> StorageResult<()> {
        Err(StorageError::Test(*id))
    }

    #[cfg(feature = "ssr-test-rand")]
    #[allow(dead_code)]
    fn should_return_error(id: &Uuid) -> StorageResult<()> {
        use rand::Rng;
        let mut rng = rand::rng();
        if rng.random::<bool>() {
            Ok(())
        } else {
            Err(StorageError::Test(*id))
        }
    }

    async fn delete_task_file(&self, id: &Uuid) -> StorageResult<()> {
        fs::remove_file(Self::filename(&self.dir, id))
            .await
            .map_err(|e| StorageError::IoRemove(*id, e))?;
        Ok(())
    }

    fn filename<P: AsRef<Path>>(dir: P, id: &Uuid) -> PathBuf {
        let id = id.as_simple();
        dir.as_ref().join(format!("task-{id}.json"))
    }
}

impl Default for TaskCache {
    fn default() -> Self {
        Self::with_capacity(10)
    }
}

impl Deref for TaskCache {
    type Target = DataMap;
    fn deref(&self) -> &Self::Target {
        &self.tasks
    }
}

impl<'a> Deref for TaskMutGuard<'a> {
    type Target = Task;
    fn deref(&self) -> &Self::Target {
        self.task
    }
}

impl<'a> DerefMut for TaskMutGuard<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.task
    }
}

impl<'a> Drop for TaskMutGuard<'a> {
    fn drop(&mut self) {
        self.dirty.insert(self.id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use assert_fs::TempDir;
    use miette::{IntoDiagnostic, MietteHandlerOpts, Result};
    use std::path::PathBuf;
    use tracing_test::traced_test;
    use uuid::uuid;

    #[test]
    fn filenames() {
        // empty dir
        assert_eq!(
            TaskCache::filename(
                &PathBuf::new(),
                &uuid!("019c500f-e598-75a1-9bd9-286e3d82cd04")
            ),
            PathBuf::from("task-019c500fe59875a19bd9286e3d82cd04.json")
        );
        // absolute dir
        assert_eq!(
            TaskCache::filename(
                &PathBuf::from("/tasks"),
                &uuid!("019c500f-e598-75a1-9bd9-286e3d82cd04")
            ),
            PathBuf::from("/tasks/task-019c500fe59875a19bd9286e3d82cd04.json")
        );
        // relative dir
        assert_eq!(
            TaskCache::filename(
                &PathBuf::from("tasks"),
                &uuid!("019c500f-e598-75a1-9bd9-286e3d82cd04")
            ),
            PathBuf::from("tasks/task-019c500fe59875a19bd9286e3d82cd04.json")
        );
    }

    #[tokio::test]
    #[traced_test]
    async fn flush() -> Result<()> {
        init_miette_report();
        let dir = TempDir::with_prefix("kid-").into_diagnostic()?;
        tracing::debug!("dir: {}", dir.path().display());
        assert_files(&dir, 0)?;
        let mut cache = TaskCache::default().with_dir(dir.path());
        assert_files(&dir, 0)?;
        cache.flush().await?;
        assert_files(&dir, cache.len())?;
        dir.close().into_diagnostic()?;
        Ok(())
    }

    fn assert_files(dir: &Path, num: usize) -> Result<()> {
        let entries = std::fs::read_dir(dir).into_diagnostic()?;
        assert_eq!(entries.count(), num);
        Ok(())
    }

    fn init_miette_report() {
        let _ = miette::set_hook(Box::new(|_| {
            Box::new(
                MietteHandlerOpts::new()
                    .color(false)
                    .without_syntax_highlighting()
                    .terminal_links(true)
                    .unicode(true)
                    .build(),
            )
        }));
    }
}
