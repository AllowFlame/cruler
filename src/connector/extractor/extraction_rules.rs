use std::vec::Vec;
use std::str::FromStr;

use super::super::super::toml;
use super::super::super::configure::*;

#[derive(Deserialize)]
pub struct ExtractionRules {
    extraction: Vec<UnitExtractionRule>,
}

impl ExtractionRules {
    pub fn new(file_path: &str) -> ExtractionRules {
        let content = RuleUtils::read_file_content(file_path);
        let content = content.as_str();

        let rules: ExtractionRules = toml::from_str(content).unwrap();
        rules
    }

    pub fn extraction(&self) -> &Vec<UnitExtractionRule> {
        &self.extraction
    }
}

impl FromStr for ExtractionRules {
    type Err = ConfigureError;
    fn from_str(content: &str) -> Result<Self, Self::Err> {
        let rules = match toml::from_str(content) {
            Result::Ok(rules) => Result::Ok(rules),
            Result::Err(_err) => Result::Err(ConfigureError::new("ExtractionRules::from_str error")),
        };
        rules
    }
}

impl Default for ExtractionRules {
    fn default() -> ExtractionRules {
        ExtractionRules::new("pack/extraction_rules.toml")
    }
}

#[derive(Deserialize)]
pub struct UnitExtractionRule {
    name: String,
    links: Option<Vec<String>>,
    local_path: Option<String>,
    parts: Option<Vec<String>>,
    extract: Option<String>,
    procedure: Option<ProcedureRule>,
}

impl Clone for UnitExtractionRule {
    fn clone(&self) -> Self {
        let name = self.name.clone();
        let links = match self.links() {
            Some(ref_links) => {
                let links = ref_links.clone();
                Some(links)
            },
            None => None,
        };
        let local_path = match self.local_path() {
            Some(ref_path) => {
                let path = ref_path.clone();
                Some(path)
            },
            None => None,
        };
        let parts = match self.parts() {
            Some(ref_parts) => {
                let parts = ref_parts.clone();
                Some(parts)
            },
            None => None,
        };
        let extract = match self.extract() {
            Some(ref_extract) => {
                let extract = ref_extract.clone();
                Some(extract)
            },
            None => None,
        };
        let procedure = match self.procedure() {
            Some(ref_procedure) => {
                let procedure = ref_procedure.clone();
                Some(procedure)
            },
            None => None,
        };

        UnitExtractionRule {
            name: name,
            links: links,
            local_path: local_path,
            parts: parts,
            extract: extract,
            procedure: procedure,
        }
    }
}

impl RuleConfigure for UnitExtractionRule {
    fn parts(&self) -> Option<&Vec<String>> {
        self.parts.as_ref()
    }

    fn extract(&self) -> Option<&String> {
        self.extract.as_ref()
    }
}

impl UnitExtractionRule {
    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn links(&self) -> Option<&Vec<String>> {
        self.links.as_ref()
    }

    pub fn local_path(&self) -> Option<&String> {
        self.local_path.as_ref()
    }

    pub fn procedure(&self) -> Option<&ProcedureRule> {
        self.procedure.as_ref()
    }
}

pub enum ProcedureName {
    NaverWebtoon,
    None,
}

#[derive(Deserialize)]
pub struct ProcedureRule {
    parts: Option<Vec<String>>,
    extract: Option<String>,
    pattern: Option<String>,
    //NOTE: pre_process might be useless, but for in case make this as a reserved field
    pre_procedure: Option<String>,
    post_procedure: Option<String>,
}

impl Clone for ProcedureRule {
    fn clone(&self) -> Self {
        let parts = match self.parts() {
            Some(ref_parts) => {
                let parts = ref_parts.clone();
                Some(parts)
            },
            None => None,
        };
        let extract = match self.extract() {
            Some(ref_extract) => {
                let extract = ref_extract.clone();
                Some(extract)
            },
            None => None,
        };
        let pattern = match self.pattern() {
            Some(ref_pattern) => {
                let pattern = ref_pattern.clone();
                Some(pattern)
            },
            None => None,
        };
        let pre_procedure = match self.pre_procedure() {
            Some(ref_pre_procedure) => {
                let pre_procedure = ref_pre_procedure.clone();
                Some(pre_procedure)
            },
            None => None,
        };
        let post_procedure = match self.post_procedure() {
            Some(ref_post_procedure) => {
                let post_procedure = ref_post_procedure.clone();
                Some(post_procedure)
            },
            None => None,
        };

        ProcedureRule {
            parts: parts,
            extract: extract,
            pattern: pattern,
            pre_procedure: pre_procedure,
            post_procedure: post_procedure,
        }
    }
}

impl RuleConfigure for ProcedureRule {
    fn parts(&self) -> Option<&Vec<String>> {
        self.parts.as_ref()
    }

    fn extract(&self) -> Option<&String> {
        self.extract.as_ref()
    }
}

impl ProcedureRule {
    pub fn pattern(&self) -> Option<&String> {
        self.pattern.as_ref()
    }

    pub fn pre_procedure(&self) -> Option<&String> {
        self.pre_procedure.as_ref()
    }

    pub fn post_procedure(&self) -> Option<&String> {
        self.post_procedure.as_ref()
    }

    pub fn procedure_name(name: &str) -> ProcedureName {
        let procedure_name = match name {
            "naver-webtoon" => ProcedureName::NaverWebtoon,
            _ => ProcedureName::None,
        };
        procedure_name
    }
}