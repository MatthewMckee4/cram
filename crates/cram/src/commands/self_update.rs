pub fn self_update(token: Option<String>, prerelease: bool) -> anyhow::Result<()> {
    let mut updater = axoupdater::AxoUpdater::new_for("cram");

    if let Some(ref token) = token {
        updater.set_github_token(token);
    }

    if prerelease {
        updater.configure_version_specifier(axoupdater::UpdateRequest::LatestMaybePrerelease);
    }

    if let Err(e) = updater.load_receipt() {
        if matches!(
            e,
            axoupdater::errors::AxoupdateError::NoReceipt { .. }
                | axoupdater::errors::AxoupdateError::ReceiptLoadFailed { .. }
        ) {
            anyhow::bail!(
                "cram was not installed via a standalone installer, \
                 so self-update is not available.\n\
                 Update cram via the method you used to install it."
            );
        }
        return Err(e.into());
    }

    updater
        .set_current_version(env!("CARGO_PKG_VERSION").parse()?)
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    if !updater
        .check_receipt_is_for_this_executable()
        .unwrap_or(false)
    {
        let current_exe = std::env::current_exe()?;
        eprintln!(
            "warning: the install receipt does not match this executable ({}).\n\
             You may have multiple cram installations.",
            current_exe.display()
        );
    }

    match updater.run_sync() {
        Ok(Some(result)) => {
            let tag = &result.new_version_tag;
            println!(
                "Upgraded cram from {} to {}.\n\
                 Release notes: https://github.com/MatthewMckee4/cram/releases/tag/{tag}",
                env!("CARGO_PKG_VERSION"),
                result.new_version,
            );
        }
        Ok(None) => {
            println!(
                "cram is already up to date ({}).",
                env!("CARGO_PKG_VERSION")
            );
        }
        Err(e) => {
            if is_rate_limited(&e) {
                anyhow::bail!(
                    "GitHub API rate limit exceeded. \
                     Use `cram self update --token <GITHUB_TOKEN>` to authenticate."
                );
            }
            return Err(e.into());
        }
    }

    Ok(())
}

/// Check if the error is a GitHub API rate limit (HTTP 403).
fn is_rate_limited(err: &axoupdater::errors::AxoupdateError) -> bool {
    if let axoupdater::errors::AxoupdateError::Reqwest(reqwest_err) = err
        && let Some(status) = reqwest_err.status()
    {
        return status == 403;
    }
    false
}
