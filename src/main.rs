use anyhow::Result;
use mdbook::book::{Book, Chapter, SectionNumber};
use mdbook::preprocess::{Preprocessor, PreprocessorContext};
use mdbook::BookItem;

fn main() {
    mdbook_preprocessor_boilerplate::run(
        AddFolderPreprocessor,
        "An mdbook preprocessor that adds whole folders", // CLI description
    );
}

struct AddFolderPreprocessor;

impl Preprocessor for AddFolderPreprocessor {
    fn name(&self) -> &str {
        "addall"
    }

    fn run(&self, ctx: &PreprocessorContext, mut book: Book) -> Result<Book> {
        let root = ctx.config.book.src.to_string_lossy().to_string();

        let mut section_number = SectionNumber(vec![]);

        for number in book.sections.iter().filter_map(|s| match s {
            BookItem::Chapter(chapter) => chapter.number.clone(),
            _ => None,
        }) {
            section_number = number;
        }
        if let Some(last) = section_number.0.last_mut() {
            *last = *last + 1
        } else {
            section_number.0 = vec![1];
        }

        let mut extras_chapter = Chapter::new("Extras", "".to_string(), "Extras", vec![]);
        extras_chapter.number = Some(section_number.clone());

        if let Some(config) = ctx.config.get_preprocessor(self.name()) {
            if let Some(folders) = config["folders"].as_array() {
                for (index, folder_name) in folders.iter().filter_map(|f| f.as_str()).enumerate() {
                    let mut folder_chapter =
                        Chapter::new(folder_name, "".to_string(), folder_name, vec![]);
                    let mut section_number = section_number.clone();
                    section_number.push((index + 1) as u32);
                    folder_chapter.number = Some(section_number.clone());

                    let mut folder_path = ctx.config.book.src.clone();
                    folder_path.push(folder_name);
                    for (index, md) in
                        glob::glob(format!("{}/*.md", folder_path.to_string_lossy()).as_str())?
                            .enumerate()
                    {
                        let mut chapter_number = section_number.clone();
                        chapter_number.0.push((index + 1) as u32);
                        let md = md?;
                        let title = md.file_stem().unwrap().to_string_lossy().to_string();
                        let path = md
                            .with_extension("")
                            .to_string_lossy()
                            .to_string()
                            .strip_prefix(&format!("{root}/"))
                            .unwrap()
                            .to_string();
                        let content = std::fs::read_to_string(md)?;
                        let mut chapter =
                            Chapter::new(&title, content, &path, vec![folder_name.to_string()]);
                        chapter.number = Some(chapter_number);
                        folder_chapter.sub_items.push(BookItem::Chapter(chapter));
                    }
                    extras_chapter
                        .sub_items
                        .push(BookItem::Chapter(folder_chapter));
                }
            }
        }

        book.push_item(BookItem::Chapter(extras_chapter));

        Ok(book)
    }
}
