use std::iter::{Enumerate, Peekable};
use std::ops::RangeInclusive;
use thiserror;

pub use pulldown_cmark::HeadingLevel;
use pulldown_cmark::{CodeBlockKind, Event, MetadataBlockKind, Options, Tag, TagEnd};
use pulldown_cmark_to_cmark::cmark;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("End of stream reached")]
    EOF,
    #[error("Expected token")]
    Expected(String),
}

pub struct MarkdownParser<'input> {
    events: Vec<Event<'input>>,
}
impl<'input> MarkdownParser<'input> {
    pub fn new(markdown_input: &'input str) -> Self {
        let mut options = Options::empty();
        options.insert(Options::ENABLE_YAML_STYLE_METADATA_BLOCKS);
        Self {
            events: pulldown_cmark::Parser::new_ext(markdown_input, options).collect(),
        }
    }

    pub fn to_string(&self, range: Option<RangeInclusive<usize>>) -> String {
        range.map_or_else(
            || String::new(),
            |r| {
                let mut s = String::new();
                cmark((self.events[r]).iter(), &mut s).unwrap();
                s
            },
        )
    }

    pub fn get_text(&self, range: RangeInclusive<usize>) -> String {
        let mut x = self.events[range].iter();
        x.next(); // skip the start tag
        let Some(Event::Text(text)) = x.next() else {
            return "".to_string();
        };
        text.trim_end().to_string()
    }
    pub fn iter(&self) -> MarkdownIter {
        MarkdownIter::new(&self.events)
    }
}

pub struct MarkdownIter<'inner> {
    inner: Peekable<Enumerate<std::slice::Iter<'inner, Event<'inner>>>>,
}

impl<'inner> Iterator for MarkdownIter<'inner> {
    type Item = (usize, &'inner Event<'inner>);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

impl<'inner> MarkdownIter<'inner> {
    fn new(events: &'inner [Event<'inner>]) -> Self {
        Self {
            inner: events.iter().enumerate().peekable(),
        }
    }

    fn take<'a, F>(&mut self, f: F) -> Result<(&'a Event<'inner>, usize), Error>
    where
        F: Fn(&Event) -> bool,
    {
        let i = self.next();
        let Some((pos, e)) = i else {
            return Err(Error::EOF);
        };
        if !f(e) {
            return Err(Error::Expected(format!("{:?}", e)));
        }
        Ok((e, pos))
    }
    fn take_map<'a, 'b, F, T>(&mut self, f: F) -> Result<T, Error>
    where
        F: Fn(&Event) -> Option<T>,
    {
        let i = self.next();
        let Some((_, e)) = i else {
            return Err(Error::EOF);
        };
        match f(e) {
            Some(value) => Ok(value),
            None => {
                return Err(Error::Expected(format!("{:?}", e)));
            }
        }
    }

    fn take_text<'a, 'b>(&mut self) -> Result<String, Error> {
        let metadata = self.take_map(|e| match e {
            Event::Text(text) => Some(text.to_string()),
            _ => None,
        })?;
        Ok(metadata)
    }

    fn check<'a, F>(&mut self, f: F) -> Result<(&'a Event<'inner>, usize), Error>
    where
        F: Fn(&Event) -> bool,
    {
        let i = self.inner.peek();
        let Some((pos, e)) = i else {
            return Err(Error::EOF);
        };
        if !f(e) {
            return Err(Error::Expected(format!("{:?}", e)));
        }
        Ok((e, *pos))
    }
    pub fn metadata(&mut self) -> Result<String, Error> {
        self.take(|e| e == &Event::Start(Tag::MetadataBlock(MetadataBlockKind::YamlStyle)))?;
        let metadata = self.take_text()?;
        self.take(|e| e == &Event::End(TagEnd::MetadataBlock(MetadataBlockKind::YamlStyle)))?;
        Ok(metadata)
    }

    pub fn heading(&mut self, level: HeadingLevel) -> Result<String, Error> {
        self.take(|e| {
            let &Event::Start(Tag::Heading { level: l, .. }) = e else {
                return false;
            };
            l == level
        })?;
        let heading = self.take_text()?;
        self.take(|e| {
            let &Event::End(TagEnd::Heading(l)) = e else {
                return false;
            };
            l == level
        })?;
        Ok(heading)
    }

    pub fn named_heading(
        &mut self,
        level: HeadingLevel,
        content: &'static str,
    ) -> Result<(), Error> {
        if self.heading(level)? != content {
            return Err(Error::Expected(format!(
                "A heading with the content: {}",
                content
            )));
        }
        Ok(())
    }

    pub fn paragraph(&mut self) -> Result<RangeInclusive<usize>, Error> {
        let (_, start) = self.check(|e| e == &Event::Start(Tag::Paragraph))?;
        let Some((end, _)) = self
            .take_while(|(_i, x)| **x != Event::End(TagEnd::Paragraph))
            .last()
        else {
            return Err(Error::Expected("expected a end of a Paragraph".to_string()));
        };

        Ok(start..=end)
    }

    pub fn alt<Fun>(&mut self, funs: &[Fun]) -> Result<RangeInclusive<usize>, Error>
    where
        Fun: Fn(&mut Self) -> Result<RangeInclusive<usize>, Error>,
    {
        for f in funs {
            if let Ok(a) = f(self) {
                return Ok(a);
            }
        }
        Err(Error::Expected(
            "Expected either one of the functions to return".to_string(),
        ))
    }

    pub fn multi<Fun>(&mut self, f: Fun) -> Option<RangeInclusive<usize>>
    where
        Fun: Fn(&mut Self) -> Result<RangeInclusive<usize>, Error>,
    {
        let content = self.paragraph().ok()?;
        let start = *(content.start());
        let mut end = *(content.end());

        while let Ok(next) = f(self) {
            end = *next.end();
        }
        Some(start..=end)
    }

    fn code_block_fn<F>(&mut self, f: F) -> Result<RangeInclusive<usize>, Error>
    where
        F: Fn(&Event) -> bool,
    {
        let (_, start) = self.check(f)?;
        let Some((end, _)) = self
            .take_while(|(_i, x)| **x != Event::End(TagEnd::CodeBlock))
            .last()
        else {
            return Err(Error::Expected("expected a end of a CodeBlock".to_string()));
        };

        Ok(start..=end)
    }
    pub fn code_block(&mut self) -> Result<RangeInclusive<usize>, Error> {
        self.code_block_fn(|e| {
            if let Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(ref _code_lang))) = e {
                true
            } else {
                false
            }
        })
    }
    pub fn lang_block(&mut self, lang: &str) -> Result<RangeInclusive<usize>, Error> {
        self.code_block_fn(|e| {
            if let Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(ref code_lang))) = e {
                *lang == **code_lang
            } else {
                false
            }
        })
    }
    fn item(&mut self) -> Result<Option<String>, Error> {
        match self.inner.peek() {
            Some((_i, Event::Start(Tag::Item))) => {
                self.inner.next();
            }
            _ => {
                return Ok(None);
            }
        }
        let item = match self.next() {
            Some((_i, Event::Text(s))) => {
                //Trim empty line from output
                s.to_string()
            }
            _ => {
                return Err(Error::Expected("expected a Text block".to_string()));
            }
        };
        let Some((_i, &Event::End(TagEnd::Item))) = self.next() else {
            return Err(Error::Expected("expected a code block".to_string()));
        };
        Ok(Some(item))
    }

    pub fn item_list(&mut self) -> Result<Vec<String>, Error> {
        match self.next() {
            Some((_i, Event::Start(Tag::List(None)))) => {}
            _ => {
                return Err(Error::Expected("expected a list block".to_string()));
            }
        }

        let mut items = vec![];

        while let Some(next) = self.item()? {
            items.push(next);
        }
        let Some((_i, &Event::End(TagEnd::List(_b)))) = self.next() else {
            return Err(Error::Expected("expected a end of list".to_string()));
        };

        Ok(items)
    }
    pub fn text(&mut self) -> Result<RangeInclusive<usize>, Error> {
        self.alt(&vec![MarkdownIter::paragraph, MarkdownIter::code_block])
    }
}
