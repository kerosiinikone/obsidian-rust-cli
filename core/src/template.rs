use std::io::Read;
use std::{fs::File, path::PathBuf};

use anyhow::Result;

#[derive(Debug, Clone, Default)]
pub struct TemplArgs {
    pub date: String,
    pub body: String, // ...
}

#[derive(Debug, Clone, Default)]
pub struct Template {
    pub path: PathBuf,
    pub template: String,
}

// Own parser (struct?) to support more intricate
// templates later?
impl Template {
    pub fn parse_string(&mut self) -> Result<()> {
        let mut file = File::open(self.path.as_path())?;
        file.read_to_string(&mut self.template)?;
        Ok(())
    }

    // Refactor when necessary
    pub fn render(&self, context: &TemplArgs) -> Result<String> {
        let mut new_note_templ = self.template.clone();
        new_note_templ = new_note_templ.replace("?time", &context.date);
        new_note_templ = new_note_templ.replace("?body", &context.body);
        Ok(new_note_templ)
    }
}

#[cfg(test)]
mod tests {
    use crate::template::{TemplArgs, Template};

    #[test]
    fn render() {
        let mut templ = Template {
            ..Default::default()
        };
        let args = &TemplArgs {
            date: "Hello,".to_string(),
            body: "World".to_string(),
        };
        templ.template = "?time ?body".to_string();
        assert_eq!(templ.render(args).unwrap(), "Hello, World".to_string())
    }
}
