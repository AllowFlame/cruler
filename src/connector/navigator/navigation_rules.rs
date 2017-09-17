use super::super::super::toml;
use super::super::super::configure::*;

#[derive(Deserialize)]
pub struct NavigationRules {
    navigation: Vec<UnitNavigationRule>,
}

impl NavigationRules {
    pub fn new(file_path: &str) -> NavigationRules {
        let content = RuleUtils::read_file_content(file_path);
        let content = content.as_str();

        let rules: NavigationRules = toml::from_str(content).unwrap();
        rules
    }

    pub fn navigation(&self) -> &Vec<UnitNavigationRule> {
        &self.navigation
    }
}

impl Default for NavigationRules {
    fn default() -> NavigationRules {
        NavigationRules::new("pack/navigation_rules.toml")
    }
}

#[derive(Deserialize)]
pub struct UnitNavigationRule {
    name: String,
    entry: String,
    parts: Option<Vec<String>>,
    extract: Option<String>,
    procedure: Option<ProcedureRule>,
    pager: Option<PagerRule>,
}

impl Clone for UnitNavigationRule {
    fn clone(&self) -> Self {
        let name = self.name.clone();
        let entry = self.entry.clone();
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
        let pager = match self.pager() {
            Some(ref_pager) => {
                let pager = ref_pager.clone();
                Some(pager)
            },
            None => None,
        };

        UnitNavigationRule {
            name: name,
            entry: entry,
            parts: parts,
            extract: extract,
            procedure: procedure,
            pager: pager,
        }
    }
}

impl RuleConfigure for UnitNavigationRule {
    fn parts(&self) -> Option<&Vec<String>> {
        self.parts.as_ref()
    }

    fn extract(&self) -> Option<&String> {
        self.extract.as_ref()
    }
}

impl UnitNavigationRule {
    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn entry(&self) -> &String {
        &self.entry
    }

    pub fn procedure(&self) -> Option<&ProcedureRule> {
        self.procedure.as_ref()
    }

    pub fn pager(&self) -> Option<&PagerRule> {
        self.pager.as_ref()
    }
}

#[derive(Deserialize)]
pub struct ProcedureRule {
    pre_procedure: Option<String>,
    post_procedure: Option<String>,
}

impl Clone for ProcedureRule {
    fn clone(&self) -> Self {
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
            pre_procedure: pre_procedure,
            post_procedure: post_procedure,
        }
    }
}

impl ProcedureRule {
    pub fn pre_procedure(&self) -> Option<&String> {
        self.pre_procedure.as_ref()
    }

    pub fn post_procedure(&self) -> Option<&String> {
        self.post_procedure.as_ref()
    }

}

#[derive(Deserialize)]
pub struct PagerRule {
    pager: String,
    parts: Option<Vec<String>>,
    extract: Option<String>,
}

impl Clone for PagerRule {
    fn clone(&self) -> Self {
        let pager = self.pager.clone();
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

        PagerRule {
            pager: pager,
            parts: parts,
            extract: extract,
        }
    }
}

impl PagerRule {
    pub fn pager(&self) -> &String {
        &self.pager
    }
}

impl RuleConfigure for PagerRule {
    fn parts(&self) -> Option<&Vec<String>> {
        self.parts.as_ref()
    }

    fn extract(&self) -> Option<&String> {
        self.extract.as_ref()
    }
}