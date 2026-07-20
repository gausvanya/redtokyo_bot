use regex::Regex;
use smallvec::SmallVec;
use telers::errors::FilterError;
use telers::{Filter, FilterResult, Request};
use tokio::time::Instant;

#[derive(Debug, Clone)]
pub struct ParsedCommand {
    pub(crate) groups: SmallVec<(&'static str, Box<str>), 4>,
}

impl ParsedCommand {
    #[inline]
    pub fn get(&self, name: &str) -> Option<&str> {
        self.groups
            .iter()
            .find(|(k, _)| *k == name)
            .map(|(_, v)| v.as_ref())
    }

    #[inline]
    pub fn require(&self, name: &str) -> &str {
        self.get(name)
            .unwrap_or_else(|| panic!("Missing capture group: {name}"))
    }
}

#[derive(Clone)]
pub struct CommandFilter {
    regex: &'static Regex,
}

impl CommandFilter {
    #[inline]
    pub fn new(regex: &'static Regex) -> Self {
        Self { regex }
    }
}

impl<Client> Filter<Client> for CommandFilter
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
            let start = Instant::now();

            let text = match request
                .update
                .message()
                .and_then(|m| m.html_text().or_else(|| m.html_caption()))
            {
                Some(t) => t,
                None => return Ok(false),
            };

            let caps = match regex.captures(&text) {
                Some(c) => c,
                None => {
                    let duration = start.elapsed();
                    tracing::debug!("CommandFilter: mismatch handled in {:?}", duration);
                    return Ok(false);
                }
            };

            let mut groups: SmallVec<(&'static str, Box<str>), 4> = SmallVec::new();

            for name in regex.capture_names().flatten() {
                if let Some(m) = caps.name(name) {
                    tracing::debug!("Match found: {} = {}", name, m.as_str());
                    groups.push((name, m.as_str().to_owned().into()));
                }
            }

            request.extensions.insert(ParsedCommand { groups });

            let duration = start.elapsed();

            tracing::debug!(
                target: "bot::filters",
                "CommandFilter successfully matched in {:?}",
                duration
            );
            Ok(true)
        }
    }
}
