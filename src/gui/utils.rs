pub fn error_popup(prompt: &str) {
    sync::InvalidSyncerParameters::TargetInSource(source) => {
        rfd::MessageDialog::new()
            .set_title("Error")
            .set_buttons(rfd::MessageButtons::Ok)
            .set_description(lang::target_in_source_error(lang, &source))
            .show();
    }
} 