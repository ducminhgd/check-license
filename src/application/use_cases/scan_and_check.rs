use crate::{
    application::ports::{ActivationChecker, AppScanner, KnowledgeBase},
    domain::{
        check_result::CheckResult,
        error::ScanError,
        license::{ActivationStatus, LicenseModel},
    },
};

pub struct ScanAndCheckUseCase<'a> {
    scanner: &'a dyn AppScanner,
    knowledge_base: &'a dyn KnowledgeBase,
    activation_checker: &'a dyn ActivationChecker,
}

impl<'a> ScanAndCheckUseCase<'a> {
    pub fn new(
        scanner: &'a dyn AppScanner,
        knowledge_base: &'a dyn KnowledgeBase,
        activation_checker: &'a dyn ActivationChecker,
    ) -> Self {
        Self { scanner, knowledge_base, activation_checker }
    }

    pub fn execute(&self) -> Result<Vec<CheckResult>, ScanError> {
        let entries = self.scanner.scan()?;
        let mut results = Vec::with_capacity(entries.len());

        for entry in entries {
            let record = self.knowledge_base.lookup(entry.bundle_id.as_deref(), &entry.name);

            let (license_model, work_allowed, mut notes, crack_indicators) = match &record {
                Some(r) => {
                    let mut n = Vec::new();
                    if let Some(note) = &r.notes {
                        n.push(note.clone());
                    }
                    (r.license_model.clone(), r.work_allowed, n, r.crack_indicators.clone())
                }
                None => (LicenseModel::Unknown, true, Vec::new(), None),
            };

            let activation_status = match &record {
                Some(r) => self.activation_checker.check(&entry, r),
                None => ActivationStatus::Unknown,
            };

            let crack_suspected = self.is_crack_suspected(
                &license_model,
                &activation_status,
                &crack_indicators,
                &mut notes,
            );

            results.push(CheckResult {
                entry,
                license_model,
                activation_status,
                work_allowed,
                crack_suspected,
                notes,
            });
        }

        results.sort_by(|a, b| a.entry.name.to_lowercase().cmp(&b.entry.name.to_lowercase()));
        Ok(results)
    }

    fn is_crack_suspected(
        &self,
        license: &LicenseModel,
        activation: &ActivationStatus,
        indicators: &Option<crate::application::ports::CrackIndicators>,
        notes: &mut Vec<String>,
    ) -> bool {
        if *license != LicenseModel::Paid {
            return false;
        }
        // Need both: unactivated state AND at least one positive crack indicator.
        if *activation != ActivationStatus::Unactivated {
            return false;
        }
        let Some(ind) = indicators else {
            return false;
        };
        let hosts = self.check_hosts_entries(&ind.hosts_entries);
        let crack_app = self.check_crack_apps(&ind.known_crack_app_bundle_ids);

        if hosts {
            notes.push("Activation domain(s) blocked in /etc/hosts".to_string());
        }
        if crack_app {
            notes.push("Known crack/patcher application detected".to_string());
        }
        hosts || crack_app
    }

    fn check_hosts_entries(&self, domains: &[String]) -> bool {
        let Ok(content) = std::fs::read_to_string("/etc/hosts") else {
            return false;
        };
        domains.iter().any(|domain| content.contains(domain.as_str()))
    }

    fn check_crack_apps(&self, bundle_ids: &[String]) -> bool {
        if bundle_ids.is_empty() {
            return false;
        }
        // Check common app directories for crack tools by scanning Info.plist
        let search_dirs: Vec<std::path::PathBuf> = [
            std::path::PathBuf::from("/Applications"),
            dirs::home_dir().map(|h| h.join("Applications")).unwrap_or_default(),
        ]
        .into_iter()
        .filter(|p| p.exists())
        .collect();

        for dir in search_dirs {
            let Ok(entries) = std::fs::read_dir(&dir) else { continue };
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) != Some("app") {
                    continue;
                }
                let plist_path = path.join("Contents/Info.plist");
                if self.bundle_id_matches(&plist_path, bundle_ids) {
                    return true;
                }
            }
        }
        false
    }

    #[cfg(target_os = "macos")]
    fn bundle_id_matches(&self, plist_path: &std::path::Path, bundle_ids: &[String]) -> bool {
        let Ok(val) = plist::Value::from_file(plist_path) else { return false };
        let Some(dict) = val.as_dictionary() else { return false };
        let Some(bid) = dict.get("CFBundleIdentifier").and_then(|v| v.as_string()) else {
            return false;
        };
        bundle_ids.iter().any(|id| id == bid)
    }

    #[cfg(not(target_os = "macos"))]
    fn bundle_id_matches(&self, _plist_path: &std::path::Path, _bundle_ids: &[String]) -> bool {
        false
    }
}
