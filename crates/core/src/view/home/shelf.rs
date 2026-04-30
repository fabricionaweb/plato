use std::thread;
use std::sync::Mutex;
use std::path::PathBuf;
use lazy_static::lazy_static;
use super::book::Book;
use crate::device::CURRENT_DEVICE;
use crate::view::{View, Event, Hub, Bus, Id, ID_FEEDER, RenderQueue, RenderData};
use crate::view::{BIG_BAR_HEIGHT, THICKNESS_MEDIUM};
use crate::view::filler::Filler;
use crate::document::open;
use crate::framebuffer::{Framebuffer, UpdateMode};
use crate::settings::{FirstColumn, SecondColumn};
use crate::geom::{Rectangle, Dir, CycleDir, halves};
use crate::color::{WHITE, SEPARATOR_NORMAL};
use crate::gesture::GestureEvent;
use crate::unit::scale_by_dpi;
use crate::metadata::Info;
use crate::geom::divide;
use crate::font::Fonts;
use crate::context::Context;

lazy_static! {
    static ref EXCLUSIVE_ACCESS: Mutex<u8> = Mutex::new(0);
}

pub struct Shelf {
    id: Id,
    pub rect: Rectangle,
    children: Vec<Box<dyn View>>,
    pub max_lines: usize,
    first_column: FirstColumn,
    second_column: SecondColumn,
    thumbnail_previews: bool,
    cover_view: bool,
    large_thumbnails: bool,
}

impl Shelf {
    pub fn new(rect: Rectangle, first_column: FirstColumn, second_column: SecondColumn, thumbnail_previews: bool, large_thumbnails: bool, cover_view: bool) -> Shelf {
        let mut shelf = Shelf {
            id: ID_FEEDER.next(),
            rect,
            children: Vec::new(),
            max_lines: 0,
            first_column,
            second_column,
            thumbnail_previews,
            cover_view,
            large_thumbnails,
        };
        shelf.max_lines = shelf.compute_max_lines();
        shelf
    }

    pub fn set_first_column(&mut self, first_column: FirstColumn) {
        self.first_column = first_column;
    }

    pub fn set_second_column(&mut self, second_column: SecondColumn) {
        self.second_column = second_column;
    }

    pub fn set_thumbnail_previews(&mut self, thumbnail_previews: bool) {
        self.thumbnail_previews = thumbnail_previews;
    }

    pub fn set_cover_view(&mut self, cover_view: bool) {
        self.cover_view = cover_view;
    }

    pub fn set_large_thumbnails(&mut self, large_thumbnails: bool) {
        self.large_thumbnails = large_thumbnails;
    }

    pub fn compute_max_lines(&self) -> usize {
        let dpi = CURRENT_DEVICE.dpi;
        let big_height = scale_by_dpi(BIG_BAR_HEIGHT, dpi) as i32;
        if self.cover_view {
            let cover_height = 3 * big_height;
            let cover_width = 3 * cover_height / 4;
            let cover_rows = (self.rect.height() as i32 / (3 * big_height)).max(1) as usize;
            let cover_cols = (self.rect.width() as i32 / cover_width).max(1) as usize;
            cover_rows * cover_cols
        } else {
            let row_height = if self.thumbnail_previews && self.large_thumbnails { 2 * big_height } else { big_height };
            let thickness = scale_by_dpi(THICKNESS_MEDIUM, dpi) as i32;
            ((self.rect.height() as i32 + thickness) / row_height) as usize
        }
    }

    pub fn update(&mut self, metadata: &[Info], hub: &Hub, rq: &mut RenderQueue, context: &Context) {
        self.children.clear();
        let dpi = CURRENT_DEVICE.dpi;
        let big_height = scale_by_dpi(BIG_BAR_HEIGHT, dpi) as i32;
        let thickness = scale_by_dpi(THICKNESS_MEDIUM, dpi) as i32;

        if self.cover_view {
            let cover_height = 3 * big_height;
            let cover_width = 3 * cover_height / 4;
            let cover_rows = (self.rect.height() as i32 / (3 * big_height)).max(1) as usize;
            let cover_cols = (self.rect.width() as i32 / cover_width).max(1) as usize;
            let max_items = cover_rows * cover_cols;

            if metadata.len() < max_items {
                let filler = Filler::new(rect![self.rect.min.x, self.rect.min.y,
                                               self.rect.max.x, self.rect.max.y],
                                         WHITE);
                self.children.push(Box::new(filler) as Box<dyn View>);
            }

            let row_height = self.rect.height() as i32 / cover_rows as i32;
            let col_width = self.rect.width() as i32 / cover_cols as i32;
            let cover_th = 3 * big_height;
            let cover_tw = 3 * cover_th / 4;

            for (index, info) in metadata.iter().enumerate() {
                let row = index / cover_cols;
                let col = index % cover_cols;

                let x_min = self.rect.min.x + (col as i32) * col_width;
                let y_min = self.rect.min.y + (row as i32) * row_height;

                let preview_path: Option<PathBuf> = {
                    let thumb_path = context.library.cover_preview(&info.file.path);
                    if !thumb_path.exists() {
                        let hub2 = hub.clone();
                        let thumb_path2 = thumb_path.to_string_lossy().into_owned();
                        let path2 = info.file.path.clone();
                        let full_path = context.library.home.join(&path2);
                        thread::spawn(move || {
                            let _guard = EXCLUSIVE_ACCESS.lock().unwrap();
                            open(full_path).and_then(|mut doc| {
                                doc.preview_pixmap(cover_tw as f32, cover_th as f32, CURRENT_DEVICE.color_samples())
                            }).map(|pixmap| {
                                if pixmap.save(&thumb_path2).is_ok() {
                                    hub2.send(Event::RefreshBookPreview(path2, Some(PathBuf::from(thumb_path2)))).ok();
                                }
                            })
                        });
                        Some(PathBuf::default())
                    } else {
                        Some(thumb_path)
                    }
                };
                let book = Book::new(rect![x_min, y_min,
                                           x_min + col_width, y_min + row_height],
                                     info.clone(),
                                     index,
                                     self.first_column,
                                     self.second_column,
                                     true,
                                     preview_path);
                self.children.push(Box::new(book) as Box<dyn View>);
            }
        } else {
        let row_height = if self.thumbnail_previews && self.large_thumbnails { 2 * big_height } else { big_height };
        let (small_thickness, big_thickness) = halves(thickness);
        self.max_lines = self.compute_max_lines();
        let max_lines = self.max_lines;
        let book_heights = divide(self.rect.height() as i32, max_lines as i32);
        let mut y_pos = self.rect.min.y;
        let th = row_height;
        let tw = 3 * th / 4;

        for (index, info) in metadata.iter().enumerate() {
            let y_min = y_pos + if index > 0 { big_thickness } else { 0 };
            let y_max = y_pos + book_heights[index] - if index < max_lines - 1 { small_thickness } else { 0 };

            let preview_path: Option<PathBuf> = if self.thumbnail_previews {
                let thumb_path = if self.large_thumbnails {
                    context.library.thumbnail_preview_large(&info.file.path)
                } else {
                    context.library.thumbnail_preview(&info.file.path)
                };
                if !thumb_path.exists() {
                    let hub2 = hub.clone();
                    let thumb_path2 = thumb_path.to_string_lossy().into_owned();
                    let path = info.file.path.clone();
                    let full_path = context.library.home.join(&info.file.path);
                    thread::spawn(move || {
                        // This is a hack to circumvent a segfault (EXC_BAD_ACCESS)
                        // triggered by loading multiple jp2 pixmaps in parallel.
                        let _guard = EXCLUSIVE_ACCESS.lock().unwrap();
                        open(full_path).and_then(|mut doc| {
                            doc.preview_pixmap(tw as f32, th as f32, CURRENT_DEVICE.color_samples())
                        }).map(|pixmap| {
                            if pixmap.save(&thumb_path2).is_ok() {
                                hub2.send(Event::RefreshBookPreview(path, Some(PathBuf::from(thumb_path2)))).ok();
                            }
                        })
                    });
                    Some(PathBuf::default())
                } else {
                    Some(thumb_path)
                }
            } else {
                None
            };

            let book = Book::new(rect![self.rect.min.x, y_min,
                                       self.rect.max.x, y_max],
                                 info.clone(),
                                 index,
                                 self.first_column,
                                 self.second_column,
                                 false,
                                 preview_path);
            self.children.push(Box::new(book) as Box<dyn View>);

            if index < max_lines - 1 {
                let separator = Filler::new(rect![self.rect.min.x, y_max,
                                                  self.rect.max.x, y_max + thickness],
                                            SEPARATOR_NORMAL);
                self.children.push(Box::new(separator) as Box<dyn View>);
            }

            y_pos += book_heights[index];
        }

        if metadata.len() < max_lines {
            let y_start = y_pos + if metadata.is_empty() { 0 } else { thickness };
            let filler = Filler::new(rect![self.rect.min.x, y_start,
                                           self.rect.max.x, self.rect.max.y],
                                     WHITE);
            self.children.push(Box::new(filler) as Box<dyn View>);
        }

        }
        rq.add(RenderData::new(self.id, self.rect, UpdateMode::Partial));
    }
}

impl View for Shelf {
    fn handle_event(&mut self, evt: &Event, _hub: &Hub, bus: &mut Bus, _rq: &mut RenderQueue, _context: &mut Context) -> bool {
        match *evt {
            Event::Gesture(GestureEvent::Swipe { dir, start, .. }) if self.rect.includes(start) => {
                match dir {
                    Dir::West => {
                        bus.push_back(Event::Page(CycleDir::Next));
                        true
                    },
                    Dir::East => {
                        bus.push_back(Event::Page(CycleDir::Previous));
                        true
                    },
                    _ => false,
                }
            },
            _ => false,
        }
    }

    fn render(&self, _fb: &mut dyn Framebuffer, _rect: Rectangle, _fonts: &mut Fonts) {
    }

    fn rect(&self) -> &Rectangle {
        &self.rect
    }

    fn rect_mut(&mut self) -> &mut Rectangle {
        &mut self.rect
    }

    fn children(&self) -> &Vec<Box<dyn View>> {
        &self.children
    }

    fn children_mut(&mut self) -> &mut Vec<Box<dyn View>> {
        &mut self.children
    }

    fn id(&self) -> Id {
        self.id
    }
}
