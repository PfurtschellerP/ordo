//! The Typst world of one document: the template as main file, the data, the FHIR bundle
//! and the photo as files, the profile's fonts, and today's date. Nothing is read from the
//! disk or the network while compiling.

use std::collections::HashMap;

use typst::diag::{FileError, FileResult};
use typst::foundations::{Bytes, Datetime};
use typst::syntax::{FileId, RootedPath, Source, VirtualPath, VirtualRoot};
use typst::text::{Font, FontBook};
use typst::utils::LazyHash;
use typst::{Library, LibraryExt, World};

pub(crate) struct DocumentWorld {
    library: LazyHash<Library>,
    book: LazyHash<FontBook>,
    fonts: Vec<Font>,
    main: Source,
    files: HashMap<FileId, Bytes>,
    today: Datetime,
}

/// The ID of a file at `path` in the project.
pub(crate) fn file_id(path: &str) -> FileId {
    RootedPath::new(
        VirtualRoot::Project,
        VirtualPath::new(path).expect("a valid virtual path"),
    )
    .intern()
}

impl DocumentWorld {
    pub(crate) fn new(
        main: String,
        files: Vec<(&str, Vec<u8>)>,
        fonts: &[Vec<u8>],
        today: (i32, u8, u8),
    ) -> Self {
        let fonts: Vec<Font> = fonts
            .iter()
            .flat_map(|data| Font::iter(Bytes::new(data.clone())))
            .collect();
        let book = FontBook::from_fonts(&fonts);
        let main = Source::new(file_id("/main.typ"), main);
        Self {
            library: LazyHash::new(Library::builder().build()),
            book: LazyHash::new(book),
            fonts,
            main,
            files: files
                .into_iter()
                .map(|(path, data)| (file_id(path), Bytes::new(data)))
                .collect(),
            today: Datetime::from_ymd(today.0, today.1, today.2)
                .unwrap_or(Datetime::from_ymd(1970, 1, 1).expect("a valid date")),
        }
    }
}

impl World for DocumentWorld {
    fn library(&self) -> &LazyHash<Library> {
        &self.library
    }

    fn book(&self) -> &LazyHash<FontBook> {
        &self.book
    }

    fn main(&self) -> FileId {
        self.main.id()
    }

    fn source(&self, id: FileId) -> FileResult<Source> {
        if id == self.main.id() {
            return Ok(self.main.clone());
        }
        let bytes = self.file(id)?;
        let text = std::str::from_utf8(&bytes).map_err(|_| FileError::InvalidUtf8)?;
        Ok(Source::new(id, text.to_owned()))
    }

    fn file(&self, id: FileId) -> FileResult<Bytes> {
        if id == self.main.id() {
            return Ok(Bytes::from_string(self.main.clone()));
        }
        self.files
            .get(&id)
            .cloned()
            .ok_or_else(|| FileError::NotFound(id.vpath().get_without_slash().into()))
    }

    fn font(&self, index: usize) -> Option<Font> {
        self.fonts.get(index).cloned()
    }

    fn today(&self, _offset: Option<typst::foundations::Duration>) -> Option<Datetime> {
        Some(self.today)
    }
}
