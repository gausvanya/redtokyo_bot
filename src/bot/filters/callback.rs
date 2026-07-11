use crate::bot::filters::command::ParsedCommand;
use regex::Regex;
use smallvec::SmallVec;
use telers::errors::FilterError;
use telers::{Filter, FilterResult, Request};

#[derive(Clone)]
pub struct CallbackFilter {
    regex: &'static Regex,
}

impl CallbackFilter {
    #[inline]
    pub fn new(regex: &'static Regex) -> Self {
        Self { regex }
    }
}

impl<Client> Filter<Client> for CallbackFilter
where
    Client: Send + Sync + 'static,
{
    type Error = FilterError;

    fn check(
        &mut self,
        request: &mut Request<Client>,
    ) -> impl Future<Output = FilterResult<Self::Error>> + Send {
        let regex = self.regex;

        async move {
            let query = match request.update.callback_query() {
                Some(q) => q,
                None => return Ok(false),
            };

            let data = match &query.data {
                Some(d) => d,
                None => return Ok(false),
            };

            let caps = match regex.captures(data) {
                Some(c) => c,
                None => return Ok(false),
            };

            let mut groups: SmallVec<[(&'static str, Box<str>); 4]> = SmallVec::new();
            for name in regex.capture_names().flatten() {
                if let Some(m) = caps.name(name) {
                    groups.push((name, m.as_str().to_owned().into()));
                }
            }

            request.extensions.insert(ParsedCommand { groups });

            Ok(true)
        }
    }
}
