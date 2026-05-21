use crate::{
    application::ports::{
        ActivationChecker, AppScanner, KnowledgeBase, OnlineLicenseProvider, PackageLicenseProvider,
    },
    domain::{
        check_result::CheckResult,
        error::ScanError,
        license::{is_oss_spdx, ActivationStatus, LicenseModel},
    },
};

pub struct ScanAndCheckUseCase<'a> {
    scanner: &'a dyn AppScanner,
    knowledge_base: &'a dyn KnowledgeBase,
    activation_checker: &'a dyn ActivationChecker,
    pkg_license_provider: &'a dyn PackageLicenseProvider,
    online_provider: &'a dyn OnlineLicenseProvider,
}

impl<'a> ScanAndCheckUseCase<'a> {
    pub fn new(
        scanner: &'a dyn AppScanner,
        knowledge_base: &'a dyn KnowledgeBase,
        activation_checker: &'a dyn ActivationChecker,
        pkg_license_provider: &'a dyn PackageLicenseProvider,
        online_provider: &'a dyn OnlineLicenseProvider,
    ) -> Self {
        Self { scanner, knowledge_base, activation_checker, pkg_license_provider, online_provider }
    }

    pub fn execute(&self) -> Result<Vec<CheckResult>, ScanError> {
        let entries = self.scanner.scan()?;
        let mut results = Vec::with_capacity(entries.len());

        for entry in entries {
            let kb_record = self.knowledge_base.lookup(entry.bundle_id.as_deref(), &entry.name);

            // Resolution order: KB → package manager SPDX → online
            let (license_model, work_allowed, spdx, mut notes, crack_indicators) =
                if let Some(r) = &kb_record {
                    let mut n = Vec::new();
                    if let Some(note) = &r.notes {
                        n.push(note.clone());
                    }
                    (
                        r.license_model.clone(),
                        r.work_allowed,
                        r.spdx.clone(),
                        n,
                        r.crack_indicators.clone(),
                    )
                } else if let Some(pkg_spdx) = self
                    .pkg_license_provider
                    .lookup_spdx(&entry.name, entry.bundle_id.as_deref())
                {
                    if is_oss_spdx(&pkg_spdx) {
                        (LicenseModel::OpenSource, true, Some(pkg_spdx), Vec::new(), None)
                    } else {
                        self.resolve_online(&entry, Some(pkg_spdx))
                    }
                } else {
                    self.resolve_online(&entry, None)
                };

            let activation_status = self.activation_checker.check(&entry, &license_model);

            let crack_suspected = self.is_crack_suspected(
                &license_model,
                &activation_status,
                &crack_indicators,
                &mut notes,
            );

            results.push(CheckResult {
                entry,
                license_model,
                spdx,
                activation_status,
                work_allowed,
                crack_suspected,
                notes,
            });
        }

        results.sort_by(|a, b| a.entry.name.to_lowercase().cmp(&b.entry.name.to_lowercase()));
        Ok(results)
    }

    /// Try the online provider; if it returns nothing fall through to Unknown.
    /// `fallback_spdx` is a non-OSS SPDX from the package manager that we pass
    /// along rather than discarding.
    fn resolve_online(
        &self,
        entry: &crate::domain::app_entry::AppEntry,
        fallback_spdx: Option<String>,
    ) -> (LicenseModel, bool, Option<String>, Vec<String>, Option<crate::application::ports::CrackIndicators>) {
        match self.online_provider.lookup(entry) {
            Some(info) => (
                info.license_model,
                info.work_allowed,
                info.spdx.or(fallback_spdx),
                vec![info.source],
                None,
            ),
            None => (LicenseModel::Unknown, true, fallback_spdx, Vec::new(), None),
        }
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
        let Ok(content) = std::fs::read_to_string(hosts_file_path()) else {
            return false;
        };
        domains.iter().any(|domain| content.contains(domain.as_str()))
    }

    fn check_crack_apps(&self, bundle_ids: &[String]) -> bool {
        if bundle_ids.is_empty() {
            return false;
        }
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

fn hosts_file_path() -> std::path::PathBuf {
    #[cfg(target_os = "windows")]
    {
        let root = std::env::var("SYSTEMROOT").unwrap_or_else(|_| r"C:\Windows".to_string());
        std::path::PathBuf::from(root)
            .join("System32")
            .join("drivers")
            .join("etc")
            .join("hosts")
    }
    #[cfg(not(target_os = "windows"))]
    std::path::PathBuf::from("/etc/hosts")
}
