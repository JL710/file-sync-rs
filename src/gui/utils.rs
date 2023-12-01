pub fn error_popup(prompt: &str) {
    rfd::MessageDialog::new()
        .set_title("Error")
        .set_buttons(rfd::MessageButtons::Ok)
        .set_description(prompt)
        .show();
} 